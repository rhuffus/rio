//! Managed block markers and extraction.
//!
//! A rio-managed file contains a delimited block whose contents are owned by
//! the plugin. Everything outside the block belongs to the user. The block is
//! marked with the two constants below.

pub const BLOCK_OPEN: &str = "# >>> RhuffusIO Managed Block >>>";
pub const BLOCK_CLOSE: &str = "# <<< RhuffusIO Managed Block <<<";
