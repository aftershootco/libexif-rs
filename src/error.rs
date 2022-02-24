#[derive(thiserror::Error, Debug)]
pub enum ExifError {
    #[error("Entry was not found")]
    EntryNotFound,
    #[error("Failed to crate a new entry")]
    EntryNewFail,
}
