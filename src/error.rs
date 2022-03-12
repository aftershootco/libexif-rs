use crate::bindings::{ExifFormat, ExifTag};
use crate::bits::IFD;
#[derive(thiserror::Error, Debug)]
pub enum ExifError {
    #[error("Entry was not found")]
    EntryNotFound,
    #[error("Failed to crate a new entry")]
    EntryNewFail,
    #[error("Failed to create a new exif memory allocation")]
    MemNewFail,
    #[error("Failed to allocalte a new exif buffer")]
    BufNewFail,
    #[error("Tried to set entry before initialising the entry")]
    EntryUninitialized,
    #[error("Error parsing UTF-8 : {0:?}")]
    IntoStringError(#[from] std::ffi::IntoStringError),
    #[error("Couln't generate an CString because: {0:?}")]
    NulError(#[from] std::ffi::NulError),
    #[error("Utf-8 is limit to 0xffff")]
    Utf8Limit,
    #[error("Format mistmatch: expected {0:?} found {1:?}")]
    FormatMismatch(ExifFormat, ExifFormat),
    #[error("Tag {0:?} is not present in IFD {1:?}")]
    TagNotInIfd(ExifTag, IFD),
    #[error("Exif Data length was zero")]
    ExifDataLenZero,
    #[error("Exif Data was null")]
    ExifDataNull,
    #[error("IOError {0}")]
    IOError(#[from] std::io::Error),
}
