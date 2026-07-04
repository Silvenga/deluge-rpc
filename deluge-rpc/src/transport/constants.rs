/// Wire protocol version (Deluge 2.x).
pub const PROTOCOL_VERSION: u8 = 1;
/// Fixed header size: 1 byte version + 4 bytes big-endian body length.
pub const HEADER_LEN: usize = 5;
/// Maximum allowed frame body size to prevent OOM from malicious daemons.
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MiB
