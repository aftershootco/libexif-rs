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

pub const EXIF_HEADER: [u8; 4] = [0xff, 0xd8, 0xff, 0xe1];
pub const JPEG_HEADER: [u8; 4] = [0xff, 0xd8, 0xff, 0xe0];

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
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Data, ExifError> {
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
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid EXIF data").into())
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

    pub fn get_entry_raw(
        &self,
        ifd: impl ToLibExif<ExifIfd>,
        tag: ExifTag,
    ) -> Result<&mut ExifEntry, ExifError> {
        let entry_ptr =
            unsafe { exif_content_get_entry(self.inner.ifd[ifd.to_libexif() as usize], tag) };
        if entry_ptr.is_null() {
            Err(ExifError::EntryNotFound)
        } else {
            Ok(unsafe { &mut *entry_ptr })
        }
    }

    /// Set an entry
    pub fn set_entry(
        &mut self,
        ifd: IFD,
        // tag: impl ToLibExif<ExifTag>,
        tag: ExifTag,
        value: Value,
        order: ByteOrder,
    ) -> Result<(), ExifError> {
        // First calculate the components, size, and format of the value
        let (components, size, format) = value.get_components_size_format()?;
        let tag_name_ptr = unsafe { exif_tag_get_title_in_ifd(tag, ifd.to_libexif()) };


        // Check if the tag is unknown
        if tag_name_ptr.is_null() {
            return Err(ExifError::TagNotInIfd(tag, ifd));
        }

        // First check if the entry exists
        if let Ok(entry) = self.get_entry_raw(ifd, tag) {
            // Check if the format matches the entry
            if entry.format != format {
                return Err(ExifError::FormatMismatch(entry.format, format));
            }

            // Check if the memory need reallocation
            if entry.size != (components * size) as u32 {
                let mem = unsafe { exif_mem_new_default() };
                if mem.is_null() {
                    return Err(ExifError::MemNewFail);
                }

                unsafe {
                    entry.size = (components * size) as u32;
                    entry.components = components as u64;
                    exif_mem_realloc(
                        mem,
                        entry.data as *mut libc::c_void,
                        (components * size) as u32,
                    );
                    // exif_content_add_entry(self.to_libexif().ifd[ifd.to_libexif() as usize], entry);
                    exif_mem_unref(mem);
                };
            }

            value.insert(*entry, components, order)?;
        } else {
            let entry = crate::tag::create_tag(
                self.to_libexif(),
                ifd,
                tag,
                components as u64,
                (components * size) as u32,
                format,
            )?;

            value.insert(entry, components, order)?;
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

    pub fn write_from_file(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<(), ExifError> {
        let old_buffer = std::fs::read(from)?;
        self.write(old_buffer, to)
    }

    pub fn write(
        &mut self,
        old_buffer: impl AsRef<[u8]>,
        to: impl AsRef<Path>,
    ) -> Result<(), ExifError> {
        let mut exif_data: *mut u8 = std::ptr::null_mut();
        let mut exif_data_len: u32 = 0;
        unsafe {
            exif_data_fix(self.inner);
            exif_data_save_data(self.inner, &mut exif_data, &mut exif_data_len);
        }
        if exif_data.is_null() {
            return Err(ExifError::ExifDataNull);
        }
        if exif_data_len == 0 {
            return Err(ExifError::ExifDataLenZero);
        }
        let exif_data: &[u8] =
            unsafe { std::slice::from_raw_parts_mut(exif_data, exif_data_len as usize) };

        // let old_jpeg = std::fs::read(from)?;
        let old_buffer = old_buffer.as_ref();

        let skip = if old_buffer.starts_with(&EXIF_HEADER) {
            // Skip the size of the exif header which is a 2 byte big-endian number
            //
            // NOTE: In the original library they take a u32 and the bitshift it by 8
            // and then and it by 0xff to get the big-endian bytes for a u16 so I just calculated
            // it as a u16
            let skip_data: &[u8; 2] = old_buffer[4..=5].try_into().unwrap();
            u16::from_be_bytes(*skip_data) as usize + 4
        } else if old_buffer.starts_with(&JPEG_HEADER) {
            // Skip 4 bytes if the buffer contains a normal jpeg file
            4
        } else {
            // If the user already skipped the headers themselves
            0
        };

        // Size of the exif header is 4 bytes
        // and u16::MAX = 65536 so that's 8KiB of data for a single ExifData block
        // FIXME handle exif size with greater than 8KiB of data
        // let skip =

        let jpeg_data_old = &old_buffer[skip..];
        let exif_data_len = exif_data_len as u16 + 2;

        let mut jpeg_buffer = Vec::new();
        jpeg_buffer.write_all(&EXIF_HEADER)?;
        jpeg_buffer.write_all(&exif_data_len.to_be_bytes())?;
        jpeg_buffer.write_all(exif_data)?;
        jpeg_buffer.write_all(jpeg_data_old)?;

        let mut file = std::fs::File::create(to)?;
        file.write_all(&jpeg_buffer)?;
        Ok(())
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
