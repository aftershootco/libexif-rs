use std::env;
use std::path::Path;

fn main() {
    for arg in env::args_os().skip(1) {
        println!("arg {:?}", &arg);
        dump_exif(&arg).unwrap();
    }
}

fn dump_exif<P: AsRef<Path>>(file_name: P) -> Result<(), libexif::ExifError> {
    let data = libexif::Data::open(file_name.as_ref())?;

    println!("EXIF data for {:?}", file_name.as_ref());
    println!("  Encoding:   {:?}", data.encoding());
    println!("  Byte Order: {:?}", data.byte_order());

    for content in data.contents() {
        if content.len() > 0 {
            println!("[{:=>31}{:=>46}]", format!(" {:?} ", content.ifd()), "");

            for entry in content.entries() {
                println!(
                    " {:<30} = {}",
                    entry.tag().title(content.ifd()),
                    entry.text_value()?
                );
            }
        }
    }

    Ok(())
}
