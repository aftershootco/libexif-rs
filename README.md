# LIBEXIF

```rust
    let mut data = Data::open("somefile.jpg").unwrap();
    data.set_entry(
        IFD::Image,
        ExifTag::EXIF_TAG_ORIENTATION,
        Value::U16(vec![100u16]),
        ByteOrder::LittleEndian,
    )
    .unwrap();
    let o = data
        .get_entry(IFD::Image, ExifTag::EXIF_TAG_ORIENTATION)
        .unwrap()
        .value(ByteOrder::LittleEndian)
        .unwrap_u16();
```

You need a working C compiler to build this ( clang/gcc ) along with make.

