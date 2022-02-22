use exif_sys::*;

use crate::internal::*;

/// Defines the byte order of binary values.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum ByteOrder {
    /// Most significant bytes come first.
    ///
    /// An integer value of `0x1234ABCD` will be represented in memory as `12 34 AB CD`.
    BigEndian,

    /// Least significant bytes come first.
    ///
    /// An integer value of `0x1234ABCD` will be represented in memory as `CD AB 34 12`.
    LittleEndian,
}

impl FromLibExif<ExifByteOrder> for ByteOrder {
    fn from_libexif(byte_order: ExifByteOrder) -> Self {
        match byte_order {
            EXIF_BYTE_ORDER_MOTOROLA => ByteOrder::BigEndian,
            EXIF_BYTE_ORDER_INTEL => ByteOrder::LittleEndian,
        }
    }
}

impl ToLibExif<ExifByteOrder> for ByteOrder {
    fn to_libexif(&self) -> ExifByteOrder {
        match *self {
            ByteOrder::BigEndian => EXIF_BYTE_ORDER_MOTOROLA,
            ByteOrder::LittleEndian => EXIF_BYTE_ORDER_INTEL,
        }
    }
}

/// Defines the encoding used to represent EXIF data.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum DataEncoding {
    Chunky,
    Planar,
    Ycc,
    Compressed,
    Unknown,
}

impl FromLibExif<ExifDataType> for DataEncoding {
    fn from_libexif(data_type: ExifDataType) -> Self {
        match data_type {
            EXIF_DATA_TYPE_UNCOMPRESSED_CHUNKY => DataEncoding::Chunky,
            EXIF_DATA_TYPE_UNCOMPRESSED_PLANAR => DataEncoding::Planar,
            EXIF_DATA_TYPE_UNCOMPRESSED_YCC => DataEncoding::Ycc,
            EXIF_DATA_TYPE_COMPRESSED => DataEncoding::Compressed,
            EXIF_DATA_TYPE_UNKNOWN => DataEncoding::Unknown,
        }
    }
}

impl ToLibExif<ExifDataType> for DataEncoding {
    fn to_libexif(&self) -> ExifDataType {
        match *self {
            DataEncoding::Chunky => EXIF_DATA_TYPE_UNCOMPRESSED_CHUNKY,
            DataEncoding::Planar => EXIF_DATA_TYPE_UNCOMPRESSED_PLANAR,
            DataEncoding::Ycc => EXIF_DATA_TYPE_UNCOMPRESSED_YCC,
            DataEncoding::Compressed => EXIF_DATA_TYPE_COMPRESSED,
            DataEncoding::Unknown => EXIF_DATA_TYPE_UNKNOWN,
        }
    }
}

/// Options that affect the behavior of [`Data`](struct.Data.html).
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum DataOption {
    /// Act as though unknown tags don't exist.
    IgnoreUnknownTags,

    /// Automatically fix discrepencies in EXIF tags to follow the spec.
    FollowSpecification,

    /// Leave the maker note alone.
    DontChangeMakerNote,
}

impl FromLibExif<ExifDataOption> for DataOption {
    fn from_libexif(data_option: ExifDataOption) -> Self {
        match data_option {
            EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS => DataOption::IgnoreUnknownTags,
            EXIF_DATA_OPTION_FOLLOW_SPECIFICATION => DataOption::FollowSpecification,
            EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE => DataOption::DontChangeMakerNote,
        }
    }
}

impl ToLibExif<ExifDataOption> for DataOption {
    fn to_libexif(&self) -> ExifDataOption {
        match *self {
            DataOption::IgnoreUnknownTags => EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS,
            DataOption::FollowSpecification => EXIF_DATA_OPTION_FOLLOW_SPECIFICATION,
            DataOption::DontChangeMakerNote => EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE,
        }
    }
}

/// EXIF tag data formats.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum DataType {
    /// Tag contains text.
    Text,

    /// Tag contains unsigned bytes.
    U8,

    /// Tag contains signed bytes.
    I8,

    /// Tag contains unsigned 16-bit integers.
    U16,

    /// Tag contains signed 16-bit integers.
    I16,

    /// Tag contains unsigned 32-bit integers.
    U32,

    /// Tag contains signed 32-bit integers.
    I32,

    /// Tag contains unsigned rational numbers.
    URational,

    /// Tag contains signed rational numbers.
    IRational,

    /// Tag contains undefined data type.
    Undefined,
}

impl DataType {
    pub(crate) fn size(&self) -> usize {
        unsafe {
            exif_format_get_size(self.to_libexif()) as usize
        }
    }
}

impl FromLibExif<ExifFormat> for DataType {
    fn from_libexif(format: ExifFormat) -> Self {
        match format {
            EXIF_FORMAT_ASCII => DataType::Text,
            EXIF_FORMAT_BYTE => DataType::U8,
            EXIF_FORMAT_SBYTE => DataType::I8,
            EXIF_FORMAT_SHORT => DataType::U16,
            EXIF_FORMAT_SSHORT => DataType::I16,
            EXIF_FORMAT_LONG => DataType::U32,
            EXIF_FORMAT_SLONG => DataType::I32,
            EXIF_FORMAT_RATIONAL => DataType::URational,
            EXIF_FORMAT_SRATIONAL => DataType::IRational,
            _ => DataType::Undefined,
        }
    }
}

impl ToLibExif<ExifFormat> for DataType {
    fn to_libexif(&self) -> ExifFormat {
        match *self {
            DataType::Text => EXIF_FORMAT_ASCII,
            DataType::U8 => EXIF_FORMAT_BYTE,
            DataType::I8 => EXIF_FORMAT_SBYTE,
            DataType::U16 => EXIF_FORMAT_SHORT,
            DataType::I16 => EXIF_FORMAT_SSHORT,
            DataType::U32 => EXIF_FORMAT_LONG,
            DataType::I32 => EXIF_FORMAT_SLONG,
            DataType::URational => EXIF_FORMAT_RATIONAL,
            DataType::IRational => EXIF_FORMAT_SRATIONAL,
            DataType::Undefined => EXIF_FORMAT_UNDEFINED,
        }
    }
}

/// Image file directory types.
///
/// An image file directory (IFD) is a group of related EXIF tags.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum IFD {
    /// IFD contents describe the primary image.
    Image,

    /// IFD contents describe the thumbnail image.
    Thumbnail,

    /// IFD contents contain EXIF-specific attributes.
    EXIF,

    /// IFD contents contain GPS data.
    GPS,

    /// IFD contents contain tags used for interoperability.
    Interoperability,

    /// Unknown IFD type.
    Unknown,
}

impl FromLibExif<ExifIfd> for IFD {
    fn from_libexif(ifd: ExifIfd) -> Self {
        match ifd {
            EXIF_IFD_0 => IFD::Image,
            EXIF_IFD_1 => IFD::Thumbnail,
            EXIF_IFD_EXIF => IFD::EXIF,
            EXIF_IFD_GPS => IFD::GPS,
            EXIF_IFD_INTEROPERABILITY => IFD::Interoperability,
            EXIF_IFD_UNKNOWN => IFD::Unknown,
        }
    }
}

impl ToLibExif<ExifIfd> for IFD {
    fn to_libexif(&self) -> ExifIfd {
        match *self {
            IFD::Image => EXIF_IFD_0,
            IFD::Thumbnail => EXIF_IFD_1,
            IFD::EXIF => EXIF_IFD_EXIF,
            IFD::GPS => EXIF_IFD_GPS,
            IFD::Interoperability => EXIF_IFD_INTEROPERABILITY,
            IFD::Unknown => EXIF_IFD_UNKNOWN,
        }
    }
}

/// Requirement specificatoins for standard EXIF tags.
#[derive(Debug,PartialEq,Eq,Clone,Copy,Hash)]
pub enum SupportLevel {
    /// EXIF tag is mandatory for the given IFD.
    Required,

    /// EXIF tag is optional for the given IFD.
    Optional,

    /// EXIF tag is not allowed for the given IFD.
    NotAllowed,

    /// EXIF tag is not known.
    Unknown,
}

impl FromLibExif<ExifSupportLevel> for SupportLevel {
    fn from_libexif(support_level: ExifSupportLevel) -> Self {
        match support_level {
            EXIF_SUPPORT_LEVEL_MANDATORY => SupportLevel::Required,
            EXIF_SUPPORT_LEVEL_OPTIONAL => SupportLevel::Optional,
            EXIF_SUPPORT_LEVEL_NOT_RECORDED => SupportLevel::NotAllowed,
            EXIF_SUPPORT_LEVEL_UNKNOWN => SupportLevel::Unknown,
        }
    }
}

impl ToLibExif<ExifSupportLevel> for SupportLevel {
    fn to_libexif(&self) -> ExifSupportLevel {
        match *self {
            SupportLevel::Required => EXIF_SUPPORT_LEVEL_MANDATORY,
            SupportLevel::Optional => EXIF_SUPPORT_LEVEL_OPTIONAL,
            SupportLevel::NotAllowed => EXIF_SUPPORT_LEVEL_NOT_RECORDED,
            SupportLevel::Unknown => EXIF_SUPPORT_LEVEL_UNKNOWN,
        }
    }
}
