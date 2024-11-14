/// APPNOTE 6.3
pub const VERSION_MADE_BY              : u16 = 63_u16;

/// APPNOTE 4.5, as it is the first version which supports zip64
pub const VERSION_NEEDED_TO_EXTRACT    : u16 = 45_u16;

/// "utf8" + "data descriptor" for files
pub const GENERAL_PURPOSE_BIG_FLAG     : u16 = 0b0000_1000_0000_1000_u16;

/// "utf8" for folders
pub const GENERAL_PURPOSE_BIG_FLAG_DIR : u16 = 0b0000_1000_0000_0000_u16;

/// "deflate" for files
pub const COMPRESSION_METHOD           : u16 = 8_u16;

/// "store" for folders
pub const COMPRESSION_METHOD_DIR       : u16 = 0_u16;
