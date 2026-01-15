pub const FORMAT_VERSION: u8 = 1;

pub fn is_supported_version(version: u8) -> bool {
    version == FORMAT_VERSION
}
pub const TYPE_NULL: u8 = 0x00;
pub const TYPE_BOOL_FALSE: u8 = 0x01;
pub const TYPE_BOOL_TRUE: u8 = 0x02;
pub const TYPE_INT64: u8 = 0x10;
pub const TYPE_F64: u8 = 0x11;
pub const TYPE_STRING: u8 = 0x20;
pub const TYPE_ARRAY: u8 = 0x30;
pub const TYPE_OBJECT: u8 = 0x31;
pub const TYPE_REF: u8 = 0x40;
