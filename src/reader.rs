use crate::fundamental_types::*;
use std::convert::{TryFrom, TryInto};

pub struct SaveFileReader {
    index: usize,
    buffer: Vec<u8>,
}

impl SaveFileReader {
    pub fn new(buffer: Vec<u8>) -> Self {
        SaveFileReader {
            index: 0,
            buffer,
        }
    }

    pub fn read_f32(&mut self) -> f32 {
        // should not panic as try_from cannot fail as long as read_bytes actually returns 4 bytes.
        let bytes: [u8; 4] = <[u8; 4]>::try_from(self.read_bytes_to_vec(4).as_slice()).unwrap();
        f32::from_le_bytes(bytes)
    }

    pub fn read_i32(&mut self) -> i32 {
        // should not panic as try_from cannot fail as long as read_bytes actually returns 4 bytes.
        let bytes: [u8; 4] = <[u8; 4]>::try_from(self.read_bytes_to_vec(4).as_slice()).unwrap();
        i32::from_le_bytes(bytes)
    }


    pub fn read_u32(&mut self) -> u32 {
        // should not panic as try_from cannot fail as long as read_bytes actually returns 4 bytes.
        let bytes: [u8; 4] = <[u8; 4]>::try_from(self.read_bytes(4)).unwrap();
        u32::from_le_bytes(bytes)
    }

    pub fn read_u16(&mut self) -> u16 {
        // should not panic as try_from cannot fail as long as read_bytes actually returns 4 bytes.
        let bytes: [u8; 2] = <[u8; 2]>::try_from(self.read_bytes(2)).unwrap();
        u16::from_le_bytes(bytes)
    }

    pub fn read_u8(&mut self) -> u8 {
        let result = self.buffer[self.index];
        self.index += 1;
        result
    }

    /// Reads a vsval. If it has an invalid size indicator, returns U8(0)
    pub fn read_vsval(&mut self) -> VSVal {
        let first_byte = self.read_u8();
        let val_type_enc = first_byte & 0b00000011;
        match val_type_enc {
            0 => VSVal::U8((first_byte & 0b11111100) >> 2),
            1 => {
                let first_byte = first_byte as u16;
                let second_byte = self.read_u8();
                VSVal::U16(((second_byte as u16) << 8 ^ first_byte) >> 2)
            }
            2 => {
                let first_byte = first_byte as u32;
                let second_byte = self.read_u8() as u32;
                let third_byte = self.read_u8() as u32;
                VSVal::U32((third_byte << 16 ^ second_byte << 8 ^ first_byte) >> 2)
            }
            _ => {
                println!("Found invalid vsval!");
                VSVal::U8(0)
            }
        }
    }

    pub fn read_w_string(&mut self) -> WString {
        let length: u16 = self.read_u16();
        let string_part = &self.buffer[self.index..self.index + length as usize];
        self.index += length as usize;
        let content = match std::str::from_utf8(string_part) {
            Ok(str) => str.to_string(),
            Err(e) => {
                println!("String parse error: {:?}", e);
                "Error while parsing string!".to_string()
            }
        };
        WString {
            length,
            content,
        }
    }

    pub fn read_string(&mut self, length: usize) -> String {
        let string_part = &self.buffer[self.index..self.index + length];
        self.index += length;
        std::str::from_utf8(string_part).expect("Could not parse string.").to_string()
    }

    fn read_bytes(&mut self, bytes: usize) -> &[u8] {
        let res = &self.buffer[self.index..self.index + bytes];
        self.index += bytes;
        res
    }

    pub fn read_bytes_to_vec(&mut self, bytes: usize) -> Vec<u8> {
        let res = &self.buffer[self.index..self.index + bytes];
        self.index += bytes;
        res.to_vec()
    }

    pub fn get_buffer(self) -> Vec<u8> {
        self.buffer
    }

    pub fn get_buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_index(&self) -> usize {
        self.index
    }
}

pub fn read_filetime(r: &mut SaveFileReader) -> FileTime {
    FileTime {
        dw_low_date_time: r.read_u32(),
        dw_high_date_time: r.read_u32(),
    }
}

pub fn read_strings_into_vec(save_file_reader: &mut SaveFileReader, count: u32) -> Vec<String> {
    read_into_vec(save_file_reader, count, |r| r.read_w_string().content)
}

pub fn read_u32s_into_vec(save_file_reader: &mut SaveFileReader, count: u32) -> Vec<u32> {
    read_into_vec(save_file_reader, count, |r| r.read_u32())
}

pub fn read_ref_ids_into_vec(r: &mut SaveFileReader, count: u32) -> Vec<RefIdType> {
    read_into_vec(r, count, |r| read_ref_id(r))
}

/// Calls ```func``` with the argument ```arg``` ```count``` times and stores the result of those calls in a ```Vec```.
///
/// This function is normally used to read loads of elements from an array.
pub fn read_into_vec<S, T>(arg: &mut S, count: u32, func: fn(&mut S) -> T) -> Vec<T> {
    let arr_count: usize = match count.try_into() {
        Ok(c) => c,
        Err(_) => usize::max_value()
    };
    let mut vec: Vec<T> = Vec::with_capacity(arr_count);
    for _i in 0..count {
        vec.push(func(arg));
    }
    vec
}

pub fn read_ref_id(sfr: &mut SaveFileReader) -> RefIdType {
    RefId {
        byte0: sfr.read_u8(),
        byte1: sfr.read_u8(),
        byte2: sfr.read_u8(),
    }.get_form_id()
}

/// Convenience function for when vsvals are used as array size indicators for usage in loops.
/// This function returns a u32 that can be used directly instead of a vsval enum variant that first
/// has to be matched
pub fn read_vsval_to_u32(sfr: &mut SaveFileReader) -> u32 {
    match sfr.read_vsval() {
        VSVal::U8(x) => x as u32,
        VSVal::U16(x) => x as u32,
        VSVal::U32(x) => x
    }
}