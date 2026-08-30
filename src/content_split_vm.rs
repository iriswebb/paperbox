pub type FpU8 = u8;
pub type FpU16 = u16;

use alloc::string::String;

pub enum Operation {
    Text(String, FpU16),
}
