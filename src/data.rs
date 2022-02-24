use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::slice;

use crate::bindings::*;
use crate::bits::*;
use crate::content::Content;
use crate::entry::Entry;
use crate::error::ExifError;
use crate::internal::*;
use crate::loader::Loader;
use crate::value::Value;

/// Container for all EXIF data found in an image.
pub struct Data {
    inner: &'static mut ExifData,
}

impl FromLibExif<*mut ExifData> for Data {
    fn from_libexif(ptr: *mut ExifData) -> Data {
        Data {
            inner: unsafe { &mut *ptr },
        }
    }
}

impl ToLibExif<ExifData> for Data {
    fn to_libexif(&self) -> ExifData {
        *self.inner
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            exif_data_unref(self.inner);
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::new()
    }
}

impl Data {
    /// Create an empty  EXIF data
    pub fn new() -> Self {
        let inner = unsafe { &mut *exif_data_new() };
        Self { inner }
    }

    /// Construct a new EXIF data container with EXIF data from a JPEG file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Data> {
        let mut file = File::open(path)?;
        let mut loader = Loader::new();
        let mut buffer = Vec::<u8>::with_capacity(1024);

        loop {
            let read_buf =
                unsafe { slice::from_raw_parts_mut(buffer.as_mut_ptr(), buffer.capacity()) };

            let len = file.read(read_buf)?;

            unsafe {
                buffer.set_len(len);
            }

            if !loader.write_data(&mut buffer) {
                break;
            }
        }

        loader
            .data()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid EXIF data"))
    }

    /// Return the byte order in use by this EXIF data.
    pub fn byte_order(&self) -> ByteOrder {
        ByteOrder::from_libexif(unsafe {
            exif_data_get_byte_order(self.inner as *const _ as *mut _)
        })
    }

    /// Set the byte order used for this EXIF data.
    pub fn set_byte_order(&mut self, byte_order: ByteOrder) {
        unsafe {
            exif_data_set_byte_order(self.inner, byte_order.to_libexif());
        }
    }

    /// Return the encoding in use by this EXIF data.
    pub fn encoding(&self) -> DataEncoding {
        DataEncoding::from_libexif(unsafe {
            exif_data_get_data_type(self.inner as *const _ as *mut _)
        })
    }

    /// Set the encoding used for this EXIF data.
    pub fn set_encoding(&mut self, encoding: DataEncoding) {
        unsafe {
            exif_data_set_data_type(self.inner, encoding.to_libexif());
        }
    }

    /// Enable a data processing option.
    pub fn set_option(&mut self, option: DataOption) {
        unsafe {
            exif_data_set_option(self.inner, option.to_libexif());
        }
    }

    /// Disable a data processing option.
    pub fn unset_option(&mut self, option: DataOption) {
        unsafe {
            exif_data_unset_option(self.inner, option.to_libexif());
        }
    }

    /// Get a Entry
    pub fn get_entry(
        &self,
        ifd: impl ToLibExif<ExifIfd>,
        tag: ExifTag,
    ) -> Result<Entry, ExifError> {
        // The C call to this function
        // exif_content_get_entry(exif->ifd[ifd], tag)
        let entry_ptr =
            unsafe { exif_content_get_entry(self.inner.ifd[ifd.to_libexif() as usize], tag) };
        if entry_ptr.is_null() {
            Err(ExifError::EntryNotFound)
        } else {
            Ok(Entry::from_libexif(unsafe { &mut *entry_ptr }))
        }
    }

    /// Set an entry
    pub fn set_entry<T>(
        &mut self,
        ifd: impl ToLibExif<ExifIfd> + Clone,
        // tag: impl ToLibExif<ExifTag>,
        tag: ExifTag,
        value: Value,
    ) -> Result<(), ExifError> {
        // First check if the entry exists
        if self.get_entry(ifd.clone(), tag).is_ok() {
            todo!()
        } else {
            // Allocate a new entry
            let entry = unsafe { exif_entry_new() };
            // If OOM it may return null
            if entry.is_null() {
                return Err(ExifError::EntryNewFail);
            }

            // tag must be set before calling exif_content_add_entry
            unsafe { *entry }.tag = tag;

            // Attach the ExifEntry to IFD
            unsafe { exif_content_add_entry(self.inner.ifd[ifd.to_libexif() as usize], entry) };
            // Allocate memory for the entry and fill with default data
            unsafe { exif_entry_initialize(entry, tag) };

            /* Ownership of the ExifEntry has now been passed to the IFD.
             * One must be very careful in accessing a structure after
             * unref'ing it; in this case, we know "entry" won't be freed
             * because the reference count was bumped when it was added to
             * the IFD.
             */
            unsafe { exif_entry_unref(entry) };
        }

        Ok(())
    }

    /// Iterate over the contents of the EXIF data.
    pub fn contents(&self) -> impl ExactSizeIterator<Item = Content> {
        Contents {
            contents: &self.inner.ifd[..],
            index: 0,
        }
    }

    /// Return the raw binary data for the ExifData
    pub fn raw_data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.inner.data, self.inner.size as usize) }
    }

    /// Fix the EXIF data to make it compatible with the EXIF specification.
    pub fn fix(&mut self) {
        unsafe {
            exif_data_fix(self.inner);
        }
    }

    /// Dump all EXIF data to stdout.
    pub fn dump(&self) {
        unsafe {
            exif_data_dump(self.inner as *const _ as *mut _);
        }
    }
}

struct Contents<'a> {
    contents: &'a [*mut ExifContent],
    index: usize,
}

impl<'a> Iterator for Contents<'a> {
    type Item = Content<'a>;

    fn next(&mut self) -> Option<Content<'a>> {
        if self.index < self.contents.len() {
            let content = self.contents[self.index];
            self.index += 1;

            Some(Content::from_libexif(unsafe { &mut *content }))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.contents.len() - self.index;

        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for Contents<'a> {
    fn len(&self) -> usize {
        self.contents.len()
    }
}
