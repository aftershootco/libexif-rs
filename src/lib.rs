//! The `exif` crate provides a safe wrapper around the `libexif` C library. It provides the
//! ability to read EXIF data from image files. The entry point for inspecting a file's EXIF data
//! is [`Data::open()`](struct.Data.html#method.open). EXIF data can be inspected by iterating over
//! the data's [`contents`](struct.Content.html) and [`entries`](struct.Entry.html):
//!
//! ```
//! # use std::io;
//! # use std::path::Path;
//! fn dump_exif<P: AsRef<Path>>(file_name: P) -> io::Result<()> {
//!     let data = try!(exif::Data::open("image.jpg"));
//!
//!     for content in data.contents() {
//!         println!("[{:=>32}{:=>46}]", format!(" {:?} ", content.ifd()), "");
//!
//!         for entry in content.entries() {
//!             println!("  {:<30} = {}",
//!                      entry.tag().title(content.ifd()),
//!                      entry.text_value());
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate paste;

pub use bits::*;
pub use content::*;
pub use data::*;
pub use entry::*;
pub use error::*;
pub use tag::*;
pub use value::*;

mod internal;

pub mod bindings; // Just in case someone wants access to the raw bindings
pub mod error;

mod bits;
mod content;
mod data;
mod entry;
mod loader;
mod tag;
mod value;
