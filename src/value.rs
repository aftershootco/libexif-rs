use libc::{self, c_char};
use std::ffi::CString;
use std::fmt::{self, Display, Formatter};
use std::mem;

use crate::bindings::*;
use crate::ExifError;

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

    // /// Value interpreted as 64-bit floats.
    // F32(Vec<f32>),

    // /// Value interpreted as 64-bit floats.
    // F64(Vec<f64>),
    /// Value interpreted as unsigned [`Rational`](struct.Rational.html) numbers.
    URational(Vec<Rational<u32>>),

    /// Value interpreted as signed [`Rational`](struct.Rational.html) numbers.
    IRational(Vec<Rational<i32>>),

    /// Value is uninterpreted sequence of bytes.
    Undefined(Vec<u8>),
}

macro_rules! impl_vec {
    (
        $(
            $data_type: ident => $interal_type: ty,
        )*

    ) => {
        $(
            paste! {
                impl From<Vec<$interal_type>> for Value {
                    fn from(value: Vec<$interal_type>) -> Self {
                        Self::[<$data_type>](value)
                    }
                }
                impl From<$interal_type> for Value {
                    fn from(value: $interal_type) -> Self {
                        Self::[<$data_type>](vec![value])
                    }
                }

            }
        )*
    };
}

impl_vec! {
    U8 => u8,
    I8 => i8,
    U16 => u16,
    I16 => i16,
    U32 => u32,
    I32 => i32,
    URational => Rational<u32>,
    IRational => Rational<i32>,
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

macro_rules! unwrap_value {
    (
        $(
            $type_name: ident, $data_type: ident => $interal_type: ty,
        )*
    ) => {
        $(
            paste! {
                pub fn [<unwrap_$type_name>](&self) -> $interal_type {
                    match self {
                        Self::$data_type(val) => val.to_owned(),
                        _ => panic!("Value was not of type {}", stringify!($data_type) ),
                    }
                }
            }
        )*
    }

}

impl Value {
    unwrap_value! {
        u8, U8 => Vec<u8>,
        i8, I8 => Vec<i8>,
        u16, U16 => Vec<u16>,
        i16, I16 => Vec<i16>,
        u32, U32 => Vec<u32>,
        i32, I32 => Vec<i32>,
        undefined, Undefined => Vec<u8>,
        text, Text => String,
    }
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
            // DataType::F32 => Value::F32(extract_vec::<f32>(
            //     raw_data,
            //     components,
            //     byte_order,
            //     exif_get_float,
            // )),
            // DataType::F64 => Value::F64(extract_vec::<f64>(
            //     raw_data,
            //     components,
            //     byte_order,
            //     exif_get_double,
            // )),
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
    pub(crate) fn insert(
        self,
        exif_entry: ExifEntry,
        components: usize,
        order: ByteOrder,
    ) -> Result<(), crate::ExifError> {
        use Value::*;
        match self {
            Text(val) => insert_text(exif_entry, components, order, val)?,
            U8(val) => insert_vec::<u8>(exif_entry, components, order, val, insert_u8)?,
            I8(val) => insert_vec::<i8>(exif_entry, components, order, val, insert_i8)?,
            U16(val) => insert_vec::<u16>(exif_entry, components, order, val, exif_set_short)?,
            I16(val) => insert_vec::<i16>(exif_entry, components, order, val, exif_set_sshort)?,
            U32(val) => insert_vec::<u32>(exif_entry, components, order, val, exif_set_long)?,
            I32(val) => insert_vec::<i32>(exif_entry, components, order, val, exif_set_slong)?,
            URational(val) => {
                insert_vec::<Rational<u32>>(exif_entry, components, order, val, insert_urational)?
            }
            IRational(val) => {
                insert_vec::<Rational<i32>>(exif_entry, components, order, val, insert_irational)?
            }
            Undefined(val) => insert_vec::<u8>(exif_entry, components, order, val, insert_u8)?,
        };
        Ok(())
    }

    pub fn get_components_size_format(&self) -> Result<(usize, usize, ExifFormat), ExifError> {
        Ok(match self {
            // In case of u8 and i8 vectors the size is 1 * length
            Value::U8(ref data) => (data.len(), 1, ExifFormat::EXIF_FORMAT_BYTE),
            Value::I8(ref data) => (data.len(), 1, ExifFormat::EXIF_FORMAT_SBYTE),
            // In case of u16 and i16 vectors the size is 2 * length
            Value::U16(ref data) => (data.len(), 2, ExifFormat::EXIF_FORMAT_SHORT),
            Value::I16(ref data) => (data.len(), 2, ExifFormat::EXIF_FORMAT_SSHORT),
            // In case of u32 and i32 vectors the size is 4 * length
            Value::U32(ref data) => (data.len(), 4, ExifFormat::EXIF_FORMAT_LONG),
            Value::I32(ref data) => (data.len(), 4, ExifFormat::EXIF_FORMAT_SLONG),
            // In case if Rational<i32> and Rational<u32> length of array * size of the structs
            Value::URational(ref data) => (
                data.len(),
                std::mem::size_of::<Rational<u32>>(),
                ExifFormat::EXIF_FORMAT_RATIONAL,
            ),
            Value::IRational(ref data) => (
                data.len(),
                std::mem::size_of::<Rational<i32>>(),
                ExifFormat::EXIF_FORMAT_SRATIONAL,
            ),
            // Undefined data I'll consider as array of u8's
            Value::Undefined(ref data) => (data.len(), 1, ExifFormat::EXIF_FORMAT_UNDEFINED),
            // Text has to be converted to CString and then the length of the bytes have to be
            // measured
            // In any utf-8 character was sent return an Error
            Value::Text(ref data) => {
                // This checks if the text has any char greater than 0xffff or U+FFFF codepoint
                data.chars().try_for_each(|c| {
                    if (c as u32) >= 0xffff {
                        return Err(ExifError::Utf8Limit);
                    }
                    Ok(())
                })?;
                // (data.len() + 3, 1, ExifFormat::EXIF_FORMAT_ASCII)
                (data.len() + 3, 1, ExifFormat::EXIF_FORMAT_UNDEFINED)
            }
        })
    }
}
/// Usually the components is 1 but in case of data like EXIF_TAG_SUBJECT_AREA it is 4
///
/// insert is a generic trait for exif_set_<T> functions
fn insert_vec<T>(
    exif_entry: ExifEntry,
    components: usize,
    byte_order: ByteOrder,
    values: Vec<T>,
    insert: unsafe extern "C" fn(*mut u8, ExifByteOrder, T),
) -> Result<(), ExifError>
where
    T: std::fmt::Debug,
{
    // Check if the entry was initialized and wheter it points to null
    if exif_entry.data.is_null() {
        // debug!("raw_data points to {:?}", exif_entry);
        return Err(ExifError::EntryUninitialized);
    }

    // First lets convert the raw pointer to a slice
    let raw_data: &mut [u8] = unsafe {
        std::slice::from_raw_parts_mut(exif_entry.data, mem::size_of::<T>() * components)
    };
    assert_eq!(raw_data.len(), mem::size_of::<T>() * components);

    let data_value_iter = raw_data.chunks_exact_mut(mem::size_of::<T>()).zip(values);

    for data_value in data_value_iter {
        let (d, v) = data_value;
        unsafe { insert(d.as_mut_ptr(), byte_order.to_libexif(), v) }
    }

    // let mut buffer = Vec::with_capacity(256);
    // let len = libc::strlen(exif_entry_get_value(
    //     raw_data as *const _ as *mut _,
    //     buffer.as_mut_ptr() as *mut i8,
    //     buffer.capacity() as u32,
    // ));

    Ok(())
}
fn insert_text(
    entry: ExifEntry,
    components: usize,
    byte_order: ByteOrder,
    text: String,
) -> Result<(), ExifError> {
    // trace!("{}", text);
    let cstring = CString::new(text)?; // This should add the 0 byte

    insert_vec::<u8>(
        entry,
        components,
        byte_order,
        cstring.into_bytes_with_nul(),
        insert_u8,
    )
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

unsafe extern "C" fn insert_u8(buf: *mut u8, _byte_order: ExifByteOrder, val: u8) {
    *buf = val
}

unsafe extern "C" fn insert_i8(buf: *mut u8, _byte_order: ExifByteOrder, val: i8) {
    *buf = val as u8
}

unsafe extern "C" fn insert_urational(
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

unsafe extern "C" fn insert_irational(
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
