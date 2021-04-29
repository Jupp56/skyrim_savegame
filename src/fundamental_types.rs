#[derive(Clone, Debug)]
pub enum VSVal {
    U8(u8),
    U16(u16),
    U32(u32),
}

#[derive(Clone, Debug)]
pub struct FileTime {
    pub dw_low_date_time: u32,
    pub dw_high_date_time: u32,
}

#[derive(Clone, Debug)]
pub struct WString {
    pub length: u16,
    pub content: String,
}

#[derive(Clone, Copy, Debug)]
/// The actual RefId data. Use ```get_form_id()``` to get a RefIdType that actually represents the data.
pub struct RefId {
    pub byte0: u8,
    pub byte1: u8,
    pub byte2: u8,
}

impl RefId {
    /// The type of form id is determined by the first two bits of the first bytes.
    /// This function returns the respective type containing the value, already parsed.
    /// formIDArray indexes have already 1 subtracted to be used directly.
    pub fn get_form_id(&self) -> RefIdType {
        match &self.byte0 & 0b11000000 {
            0 => {
                let parsed_id = self.get_parsed_id();
                if parsed_id == 0 {
                    RefIdType::Default(0)
                } else {
                    RefIdType::Index(parsed_id - 1)
                }
            },
            64 => {
                RefIdType::Default(self.get_parsed_id())
            },
            128 => {
                RefIdType::Created(self.get_parsed_id())
            },
            _ => RefIdType::Unknown(self.get_parsed_id())
        }
    }

    pub fn get_parsed_id(&self) -> u32 {
        (self.byte0 as u32) << 16 ^ (self.byte1 as u32) << 8 ^ (self.byte1 as u32)
    }
}

/// The different types of formId that can be stored in a RefID.
#[derive(Clone, Copy, Debug)]
pub enum RefIdType {
    /// An index into the File.formIDArray.
    /// If the index value of 0 is given, the formID is 0x00000000, else, index into the array using value - 1.
    /// get_form_id() already takes care of subtracting 1!
    Index(u32),
    /// Default (ie, came from Skyrim.esm)
    Default(u32),
    /// Created (ie, plugin index of 0xFF)
    Created(u32),
    /// ???
    Unknown(u32),
}