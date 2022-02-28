use std::ffi::CStr;

use crate::bindings::*;
use crate::ExifError;

use crate::bits::*;
use crate::internal::*;

/// EXIF tag.
pub struct Tag {
    inner: ExifTag,
}

impl FromLibExif<ExifTag> for Tag {
    fn from_libexif(tag: ExifTag) -> Tag {
        Tag { inner: tag }
    }
}

impl ToLibExif<ExifTag> for Tag {
    fn to_libexif(&self) -> ExifTag {
        self.inner
    }
}

impl Tag {
    /// The name of the EXIF tag when found in the given IFD.
    pub fn name(&self, ifd: IFD) -> &'static str {
        let ptr = unsafe { exif_tag_get_name_in_ifd(self.inner, ifd.to_libexif()) };

        assert!(!ptr.is_null());

        let cstr = unsafe { CStr::from_ptr(ptr) };
        cstr.to_str().expect("invalid UTF-8")
    }

    /// The title of the EXIF tag when found in the given IFD.
    pub fn title(&self, ifd: IFD) -> &'static str {
        // error!("{:?}", self.inner as u64);
        let ptr = unsafe { exif_tag_get_title_in_ifd(self.inner, ifd.to_libexif()) };

        assert!(!ptr.is_null());

        let cstr = unsafe { CStr::from_ptr(ptr) };
        cstr.to_str().expect("invalid UTF-8")
    }

    /// A verbose description of the EXIF tag when found in the given IFD.
    pub fn description(&self, ifd: IFD) -> &'static str {
        let ptr = unsafe { exif_tag_get_description_in_ifd(self.inner, ifd.to_libexif()) };

        assert!(!ptr.is_null());

        let cstr = unsafe { CStr::from_ptr(ptr) };
        cstr.to_str().expect("invalid UTF-8")
    }

    /// The EXIF tag's support level with the given IFD and encoding.
    ///
    /// This method returns the tag's support level according to the EXIF specification.
    pub fn support_level(&self, ifd: IFD, encoding: DataEncoding) -> SupportLevel {
        let support_level = unsafe {
            exif_tag_get_support_level_in_ifd(self.inner, ifd.to_libexif(), encoding.to_libexif())
        };

        SupportLevel::from_libexif(support_level)
    }
}

pub fn create_tag(
    exif: ExifData,
    ifd: impl ToLibExif<ExifIfd>,
    tag: ExifTag,
    components: u64,
    length: u32,
    format: ExifFormat,
) -> Result<ExifEntry, ExifError> {
    let mem = unsafe { exif_mem_new_default() };
    if mem.is_null() {
        return Err(ExifError::MemNewFail);
    }
    let entry = unsafe { exif_entry_new_mem(mem) };
    if entry.is_null() {
        return Err(ExifError::EntryNewFail);
    }
    let buf = unsafe { exif_mem_alloc(mem, length) };
    if buf.is_null() {
        return Err(ExifError::BufNewFail);
    }

    let entry = unsafe { &mut *entry };
    let buf: *mut u8 = unsafe { std::mem::transmute(buf) };

    entry.data = buf;
    entry.size = length;
    entry.tag = tag;
    entry.components = components;
    // entry.format = ExifFormat::EXIF_FORMAT_UNDEFINED;
    entry.format = format;

    // Attach the exif entry to an ifd
    unsafe { exif_content_add_entry(exif.ifd[ifd.to_libexif() as usize], entry) };

    unsafe { exif_entry_initialize(entry, tag) };
    // The ExifMem and ExifEntry are now owned by the ExifData so unref them
    unsafe {
        exif_mem_unref(mem);
        exif_entry_unref(entry);
    }

    // let entry = unsafe { exif_content_get_entry(exif.ifd[ifd.to_libexif() as usize], tag) };

    // warn!("{:?}", entry.data);
    Ok(*entry)
}
