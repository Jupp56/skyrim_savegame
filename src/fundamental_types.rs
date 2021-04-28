#[derive(Debug)]
pub enum VSVal {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Debug)]
pub struct FileTime {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
}

#[derive(Debug)]
pub struct WString {
    pub length: u16,
    pub content: String,
}

#[derive(Debug)]
pub struct RefId {
    pub byte0: u8,
    pub byte1: u8,
    pub byte2: u8,
}