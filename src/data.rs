use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::slice;

use exif_sys::*;

use bits::*;
use content::Content;
use internal::*;
use loader::Loader;

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

impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            exif_data_unref(self.inner);
        }
    }
}

impl Data {
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

            if !loader.write_data(&buffer) {
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

    /// Iterate over the contents of the EXIF data.
    pub fn contents(&self) -> impl ExactSizeIterator<Item = Content> {
        Contents {
            contents: &self.inner.ifd[..],
            index: 0,
        }
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
