use std::ffi::CString;
use std::fmt::{self, Display, Formatter};
use std::mem;

use crate::bindings::*;
use crate::Data;
use libc::{self, c_char};

use crate::bits::*;
use crate::internal::*;

/// A rational number consisting of a numerator and denominator.
///
/// A rational number is any number that can be represented as a fraction of two whole numbers,
/// e.g., 42/100. A `Rational` is a tuple struct containing the fraction's numerator as its first
/// element and the fraction's denominator as its second element.
///
/// # Example
///
/// The fraction 42/100 is represented by `Rational(42, 100)`:
///
/// ```
/// let ratio = exif::Rational(42, 100);
/// assert_eq!(42, ratio.numerator());
/// assert_eq!(100, ratio.denominator());
/// ```
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Rational<T>(pub T, pub T);

impl<T: Copy> Rational<T> {
    /// Returns the numerator.
    pub fn numerator(&self) -> T {
        self.0
    }

    /// Returns the denominator.
    pub fn denominator(&self) -> T {
        self.1
    }
}

impl<T: Display + Copy> Display for Rational<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("{}/{}", self.numerator(), self.denominator()))
    }
}

/// Dynamic value for an EXIF tag.
///
/// Each variant of `Value` corresponds to a variant of [`DataType`](enum.DataType.html). Each
/// variant (except for `Text`) contains a `Vec` with a length equal to the number of components in
/// the corresponding [`Entry`](struct.Entry.html).
#[derive(Debug, Clone)]
pub enum Value {
    /// Value interpreted as a string.
    Text(String),

    /// Value interpreted as unsigned bytes.
    U8(Vec<u8>),

    /// Value interpreted as signed bytes.
    I8(Vec<i8>),

    /// Value interpreted as unsigned 16-bit integers.
    U16(Vec<u16>),

    /// Value interpreted as signed 16-bit integers.
    I16(Vec<i16>),

    /// Value interpreted as unsigned 32-bit integers.
    U32(Vec<u32>),

    /// Value interpreted as signed 32-bit integers.
    I32(Vec<i32>),

    /// Value interpreted as unsigned [`Rational`](struct.Rational.html) numbers.
    URational(Vec<Rational<u32>>),

    /// Value interpreted as signed [`Rational`](struct.Rational.html) numbers.
    IRational(Vec<Rational<i32>>),

    /// Value is uninterpreted sequence of bytes.
    Undefined(Vec<u8>),
}

impl Value {
    pub(crate) fn extract(
        raw_data: &[u8],
        data_type: DataType,
        components: usize,
        byte_order: ByteOrder,
    ) -> Self {
        assert_eq!(raw_data.len(), data_type.size() * components);

        match data_type {
            DataType::Text => Value::Text(extract_text(raw_data, components, byte_order)),
            DataType::U8 => Value::U8(extract_vec::<u8>(raw_data, components, byte_order, get_u8)),
            DataType::I8 => Value::I8(extract_vec::<i8>(raw_data, components, byte_order, get_i8)),
            DataType::U16 => Value::U16(extract_vec::<u16>(
                raw_data,
                components,
                byte_order,
                exif_get_short,
            )),
            DataType::I16 => Value::I16(extract_vec::<i16>(
                raw_data,
                components,
                byte_order,
                exif_get_sshort,
            )),
            DataType::U32 => Value::U32(extract_vec::<u32>(
                raw_data,
                components,
                byte_order,
                exif_get_long,
            )),
            DataType::I32 => Value::I32(extract_vec::<i32>(
                raw_data,
                components,
                byte_order,
                exif_get_slong,
            )),
            DataType::URational => Value::URational(extract_vec::<Rational<u32>>(
                raw_data,
                components,
                byte_order,
                get_urational,
            )),
            DataType::IRational => Value::IRational(extract_vec::<Rational<i32>>(
                raw_data,
                components,
                byte_order,
                get_irational,
            )),
            DataType::Undefined => {
                Value::Undefined(extract_vec::<u8>(raw_data, components, byte_order, get_u8))
            }
        }
    }
    pub(crate) fn set(data: Data, order: ByteOrder, value: Value) -> Result<(), crate::ExifError> {
        use Value::*;
        match value {
            // Text(val) =>
            // U8(val) =>
            // I8(val) =>
            U16(val) => set_vec::<u16>(data.to_libexif(), val.len(), order, val, exif_set_short),
            I16(val) => set_vec::<i16>(data.to_libexif(), val.len(), order, val, exif_set_sshort),
            U32(val) => set_vec::<u32>(data.to_libexif(), val.len(), order, val, exif_set_long),
            I32(val) => set_vec::<i32>(data.to_libexif(), val.len(), order, val, exif_set_slong),
            URational(val) => {
                set_vec::<Rational<u32>>(data.to_libexif(), val.len(), order, val, set_urational)
            }
            IRational(val) => {
                set_vec::<Rational<i32>>(data.to_libexif(), val.len(), order, val, set_irational)
            }
            _ => todo!(),
        }

        Ok(())
    }
}
/// Usually the components is 1 but in case of data like EXIF_TAG_SUBJECT_AREA it is 4
///
/// Set is a generic trait for exif_set_<T> functions
fn set_vec<T>(
    exif_data: ExifData,
    components: usize,
    byte_order: ByteOrder,
    values: Vec<T>,
    set: unsafe extern "C" fn(*mut u8, ExifByteOrder, T),
) {
    // FIXME
    // Cannot assert will need to allocate memory in the future
    // And since it will still be a raw pointer there's probably no good way to check it.

    // First lets convert the raw pointer to a slice
    let raw_data: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(exif_data.data, mem::size_of::<T>() * components) };

    assert_eq!(raw_data.len(), mem::size_of::<T>() * components);

    let data_value_iter = raw_data.chunks_exact_mut(mem::size_of::<T>()).zip(values);

    for data_value in data_value_iter {
        let (d, v) = data_value;
        unsafe { set(d.as_mut_ptr(), byte_order.to_libexif(), v) }
    }
}

fn extract_text(raw_data: &[u8], components: usize, byte_order: ByteOrder) -> String {
    let mut vec = extract_vec::<u8>(raw_data, components, byte_order, get_u8);

    let cstring = unsafe {
        let len = libc::strlen(vec.as_ptr() as *const c_char);
        vec.set_len(len);

        CString::from_vec_unchecked(vec)
    };

    cstring.to_string_lossy().into_owned()
}

fn extract_vec<T>(
    raw_data: &[u8],
    components: usize,
    byte_order: ByteOrder,
    get: unsafe extern "C" fn(*const u8, ExifByteOrder) -> T,
) -> Vec<T> {
    assert_eq!(raw_data.len(), mem::size_of::<T>() * components);

    let mut values = Vec::with_capacity(components);

    values.extend(
        raw_data
            .chunks(mem::size_of::<T>())
            .map(|chunk| unsafe { get(chunk.as_ptr(), byte_order.to_libexif()) }),
    );

    values
}

unsafe extern "C" fn get_u8(buf: *const u8, _byte_order: ExifByteOrder) -> u8 {
    *buf
}

unsafe extern "C" fn get_i8(buf: *const u8, _byte_order: ExifByteOrder) -> i8 {
    *buf as i8
}

unsafe extern "C" fn get_urational(buf: *const u8, byte_order: ExifByteOrder) -> Rational<u32> {
    let rational = exif_get_rational(buf, byte_order);

    Rational(rational.numerator, rational.denominator)
}

unsafe extern "C" fn get_irational(buf: *const u8, byte_order: ExifByteOrder) -> Rational<i32> {
    let rational = exif_get_srational(buf, byte_order);

    Rational(rational.numerator, rational.denominator)
}
unsafe extern "C" fn set_urational(
    buf: *mut u8,
    byte_order: ExifByteOrder,
    urational: Rational<u32>,
) {
    let exif_rational = ExifRational {
        numerator: urational.0,
        denominator: urational.1,
    };
    exif_set_rational(buf, byte_order, exif_rational);
}

unsafe extern "C" fn set_irational(
    buf: *mut u8,
    byte_order: ExifByteOrder,
    irational: Rational<i32>,
) {
    let exif_srational = ExifSRational {
        numerator: irational.0,
        denominator: irational.1,
    };
    exif_set_srational(buf, byte_order, exif_srational);
}
