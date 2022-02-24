// use crate::bindings::ExifData;
// use std::fs::File;
// // use std::io::Seek;
// use std::path::Path;

// pub struct ExifWriter {}

// impl ExifWriter {
//     pub const EXIF_HEADER: [u8; 4] = [0xff, 0xd8, 0xff, 0xe1];
//     pub const EXIF_HEADER_LEN: usize = 4_usize;
//     pub fn write(path: impl AsRef<Path>) {
//         let path = path.as_ref();
//         let file = File::open(&path).unwrap();
//         // let exif_data = ExifData::new();
//         // First we write the exif header to a file
//         // file.write();
//         // file.seek(SeekFrom::Start(EXIF_HEADER_LEN);
//     }
// }
