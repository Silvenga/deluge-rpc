/// Variable-length list prefix (`0x3B`). Followed by elements and `CHR_TERM`.
pub const CHR_LIST: u8 = 59;
/// Variable-length dict prefix (`0x3C`). Followed by key-value pairs and `CHR_TERM`.
pub const CHR_DICT: u8 = 60;
/// Big-number ASCII prefix (`0x3D`). Followed by ASCII digits and `CHR_TERM`.
pub const CHR_INT: u8 = 61;
/// 1-byte signed int prefix (`0x3E`).
pub const CHR_INT1: u8 = 62;
/// 2-byte signed int prefix (`0x3F`).
pub const CHR_INT2: u8 = 63;
/// 4-byte signed int prefix (`0x40`).
pub const CHR_INT4: u8 = 64;
/// 8-byte signed int prefix (`0x41`).
pub const CHR_INT8: u8 = 65;
/// 32-bit float prefix (`0x42`).
pub const CHR_FLOAT32: u8 = 66;
/// 64-bit float prefix (`0x2C`).
pub const CHR_FLOAT64: u8 = 44;
/// `True` (`0x43`).
pub const CHR_TRUE: u8 = 67;
/// `False` (`0x44`).
pub const CHR_FALSE: u8 = 68;
/// `None` (`0x45`).
pub const CHR_NONE: u8 = 69;
/// Terminator for variable-length lists, dicts, and big numbers (`0x7F`).
pub const CHR_TERM: u8 = 127;
/// Length delimiter for long strings (`:`).
pub const LENGTH_DELIM: u8 = b':';

/// Start of embedded positive int range (`0x00`). Values `0..INT_POS_FIXED_COUNT`.
pub const INT_POS_FIXED_START: u8 = 0;
/// Count of embedded positive ints (44). So `0x00..=0x2B` encode `0..=43`.
pub const INT_POS_FIXED_COUNT: u8 = 44;
/// Start of embedded negative int range (`0x46`). Values `-1..-INT_NEG_FIXED_COUNT`.
pub const INT_NEG_FIXED_START: u8 = 70;
/// Count of embedded negative ints (32). So `0x46..=0x65` encode `-1..=-32`.
pub const INT_NEG_FIXED_COUNT: u8 = 32;

/// Start of fixed-length dict range (`0x66`). Dicts with `1..DICT_FIXED_COUNT` entries.
pub const DICT_FIXED_START: u8 = 102;
/// Count of fixed-length dicts (25). So `0x66..=0x7E` encode dicts of size `1..=25`.
pub const DICT_FIXED_COUNT: u8 = 25;

/// Start of fixed-length string range (`0x80`). Strings of length `0..STR_FIXED_COUNT`.
pub const STR_FIXED_START: u8 = 128;
/// Count of fixed-length strings (64). So `0x80..=0xBF` encode strings of length `0..=63`.
pub const STR_FIXED_COUNT: u8 = 64;

/// Start of fixed-length list range (`0xC0`). Lists with `0..LIST_FIXED_COUNT` entries.
pub const LIST_FIXED_START: u8 = STR_FIXED_START + STR_FIXED_COUNT;
/// Count of fixed-length lists (64). So `0xC0..=0xFF` encode lists of size `0..=63`.
pub const LIST_FIXED_COUNT: u8 = 64;

/// Maximum decode nesting depth before `DepthExceeded` is returned.
pub const MAX_DEPTH: usize = 100;
