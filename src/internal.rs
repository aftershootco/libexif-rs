pub trait FromLibExif<T> {
    fn from_libexif(inner: T) -> Self;
}

pub trait ToLibExif<T> {
    fn to_libexif(&self) -> T;
}
