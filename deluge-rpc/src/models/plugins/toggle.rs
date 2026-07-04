//! Models for the Toggle plugin.
//!
//! The Toggle plugin has no configuration structs. Its two methods return `bool`:
//! - `toggle.get_status` → `bool` (True if session is paused)
//! - `toggle.toggle` → `bool` (new paused state after toggling)
