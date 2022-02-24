use crate::bindings::*;
use libc::c_uint;

use crate::data::Data;
use crate::internal::*;

pub struct Loader {
    inner: *mut ExifLoader,
}

impl Drop for Loader {
    fn drop(&mut self) {
        unsafe {
            exif_loader_unref(self.inner);
        }
    }
}

impl Loader {
    pub fn new() -> Self {
        let ptr = unsafe { exif_loader_new() };

        assert!(!ptr.is_null());

        Loader { inner: ptr }
    }

    pub fn data(&self) -> Option<Data> {
        let ptr = unsafe { exif_loader_get_data(self.inner) };

        if !ptr.is_null() {
            Some(Data::from_libexif(ptr))
        } else {
            None
        }
    }

    pub fn write_data(&mut self, data: &mut [u8]) -> bool {
        unsafe { exif_loader_write(self.inner, data.as_mut_ptr(), data.len() as c_uint) != 0 }
    }
}
