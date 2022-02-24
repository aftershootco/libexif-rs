#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Entry was not found")]
    EntryNotFound,
}
