use crate::bindings::*;

use crate::internal::*;

/// Defines the byte order of binary values.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
            ExifByteOrder::EXIF_BYTE_ORDER_MOTOROLA => ByteOrder::BigEndian,
            ExifByteOrder::EXIF_BYTE_ORDER_INTEL => ByteOrder::LittleEndian,
        }
    }
}

impl ToLibExif<ExifByteOrder> for ByteOrder {
    fn to_libexif(&self) -> ExifByteOrder {
        match *self {
            ByteOrder::BigEndian => ExifByteOrder::EXIF_BYTE_ORDER_MOTOROLA,
            ByteOrder::LittleEndian => ExifByteOrder::EXIF_BYTE_ORDER_INTEL,
        }
    }
}

/// Defines the encoding used to represent EXIF data.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
            ExifDataType::EXIF_DATA_TYPE_UNCOMPRESSED_CHUNKY => DataEncoding::Chunky,
            ExifDataType::EXIF_DATA_TYPE_UNCOMPRESSED_PLANAR => DataEncoding::Planar,
            ExifDataType::EXIF_DATA_TYPE_UNCOMPRESSED_YCC => DataEncoding::Ycc,
            ExifDataType::EXIF_DATA_TYPE_COMPRESSED => DataEncoding::Compressed,
            ExifDataType::EXIF_DATA_TYPE_UNKNOWN => DataEncoding::Unknown,
        }
    }
}

impl ToLibExif<ExifDataType> for DataEncoding {
    fn to_libexif(&self) -> ExifDataType {
        match *self {
            DataEncoding::Chunky => ExifDataType::EXIF_DATA_TYPE_UNCOMPRESSED_CHUNKY,
            DataEncoding::Planar => ExifDataType::EXIF_DATA_TYPE_UNCOMPRESSED_PLANAR,
            DataEncoding::Ycc => ExifDataType::EXIF_DATA_TYPE_UNCOMPRESSED_YCC,
            DataEncoding::Compressed => ExifDataType::EXIF_DATA_TYPE_COMPRESSED,
            DataEncoding::Unknown => ExifDataType::EXIF_DATA_TYPE_UNKNOWN,
        }
    }
}

/// Options that affect the behavior of [`Data`](struct.Data.html).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
            ExifDataOption::EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS => DataOption::IgnoreUnknownTags,
            ExifDataOption::EXIF_DATA_OPTION_FOLLOW_SPECIFICATION => {
                DataOption::FollowSpecification
            }
            ExifDataOption::EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE => {
                DataOption::DontChangeMakerNote
            }
        }
    }
}

impl ToLibExif<ExifDataOption> for DataOption {
    fn to_libexif(&self) -> ExifDataOption {
        match *self {
            DataOption::IgnoreUnknownTags => ExifDataOption::EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS,
            DataOption::FollowSpecification => {
                ExifDataOption::EXIF_DATA_OPTION_FOLLOW_SPECIFICATION
            }
            DataOption::DontChangeMakerNote => {
                ExifDataOption::EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE
            }
        }
    }
}

/// EXIF tag data formats.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
        unsafe { exif_format_get_size(self.to_libexif()) as usize }
    }
}

impl FromLibExif<ExifFormat> for DataType {
    fn from_libexif(format: ExifFormat) -> Self {
        match format {
            ExifFormat::EXIF_FORMAT_ASCII => DataType::Text,
            ExifFormat::EXIF_FORMAT_BYTE => DataType::U8,
            ExifFormat::EXIF_FORMAT_SBYTE => DataType::I8,
            ExifFormat::EXIF_FORMAT_SHORT => DataType::U16,
            ExifFormat::EXIF_FORMAT_SSHORT => DataType::I16,
            ExifFormat::EXIF_FORMAT_LONG => DataType::U32,
            ExifFormat::EXIF_FORMAT_SLONG => DataType::I32,
            ExifFormat::EXIF_FORMAT_RATIONAL => DataType::URational,
            ExifFormat::EXIF_FORMAT_SRATIONAL => DataType::IRational,
            _ => DataType::Undefined,
        }
    }
}

impl ToLibExif<ExifFormat> for DataType {
    fn to_libexif(&self) -> ExifFormat {
        match *self {
            DataType::Text => ExifFormat::EXIF_FORMAT_ASCII,
            DataType::U8 => ExifFormat::EXIF_FORMAT_BYTE,
            DataType::I8 => ExifFormat::EXIF_FORMAT_SBYTE,
            DataType::U16 => ExifFormat::EXIF_FORMAT_SHORT,
            DataType::I16 => ExifFormat::EXIF_FORMAT_SSHORT,
            DataType::U32 => ExifFormat::EXIF_FORMAT_LONG,
            DataType::I32 => ExifFormat::EXIF_FORMAT_SLONG,
            DataType::URational => ExifFormat::EXIF_FORMAT_RATIONAL,
            DataType::IRational => ExifFormat::EXIF_FORMAT_SRATIONAL,
            DataType::Undefined => ExifFormat::EXIF_FORMAT_UNDEFINED,
        }
    }
}

/// Image file directory types.
///
/// An image file directory (IFD) is a group of related EXIF tags.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
    Count,
}

impl FromLibExif<ExifIfd> for IFD {
    fn from_libexif(ifd: ExifIfd) -> Self {
        match ifd {
            ExifIfd::EXIF_IFD_0 => IFD::Image,
            ExifIfd::EXIF_IFD_1 => IFD::Thumbnail,
            ExifIfd::EXIF_IFD_EXIF => IFD::EXIF,
            ExifIfd::EXIF_IFD_GPS => IFD::GPS,
            ExifIfd::EXIF_IFD_INTEROPERABILITY => IFD::Interoperability,
            ExifIfd::EXIF_IFD_COUNT => IFD::Count,
        }
    }
}

impl ToLibExif<ExifIfd> for IFD {
    fn to_libexif(&self) -> ExifIfd {
        match *self {
            IFD::Image => ExifIfd::EXIF_IFD_0,
            IFD::Thumbnail => ExifIfd::EXIF_IFD_1,
            IFD::EXIF => ExifIfd::EXIF_IFD_EXIF,
            IFD::GPS => ExifIfd::EXIF_IFD_GPS,
            IFD::Interoperability => ExifIfd::EXIF_IFD_INTEROPERABILITY,
            IFD::Count => ExifIfd::EXIF_IFD_COUNT,
        }
    }
}

/// Requirement specificatoins for standard EXIF tags.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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
            ExifSupportLevel::EXIF_SUPPORT_LEVEL_MANDATORY => SupportLevel::Required,
            ExifSupportLevel::EXIF_SUPPORT_LEVEL_OPTIONAL => SupportLevel::Optional,
            ExifSupportLevel::EXIF_SUPPORT_LEVEL_NOT_RECORDED => SupportLevel::NotAllowed,
            ExifSupportLevel::EXIF_SUPPORT_LEVEL_UNKNOWN => SupportLevel::Unknown,
        }
    }
}

impl ToLibExif<ExifSupportLevel> for SupportLevel {
    fn to_libexif(&self) -> ExifSupportLevel {
        match *self {
            SupportLevel::Required => ExifSupportLevel::EXIF_SUPPORT_LEVEL_MANDATORY,
            SupportLevel::Optional => ExifSupportLevel::EXIF_SUPPORT_LEVEL_OPTIONAL,
            SupportLevel::NotAllowed => ExifSupportLevel::EXIF_SUPPORT_LEVEL_NOT_RECORDED,
            SupportLevel::Unknown => ExifSupportLevel::EXIF_SUPPORT_LEVEL_UNKNOWN,
        }
    }
}
