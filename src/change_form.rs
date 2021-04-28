use crate::reader::*;
use crate::fundamental_types::RefId;
use flate2::read::ZlibDecoder;
use std::io::Read;
use std::fmt;
use std::convert::TryInto;

const CHANGE_FORM_DECODE_ERROR: &str = "Failed to decode compressed change form!";


pub struct ChangeForm {
    pub form_id: RefId,
    pub change_flags: u32,
    pub data_type: u8,
    pub version: u8,
    pub length1: Vec<u8>,
    pub length2: Vec<u8>,
    pub data: Vec<u8>,
}

impl fmt::Debug for ChangeForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChangeForm")
            .field("form_id", &self.form_id)
            .field("change_flags", &self.change_flags)
            .field("data_type", &self.data_type)
            .field("version", &self.version)
            .field("length2 (is compressed)", &self.length2)
            .field("data (length)", &self.data.len())
            .finish()
    }
}

pub fn read_change_forms(sfr: &mut SaveFileReader, count: u32) -> Vec<ChangeForm> {
    let mut result: Vec<ChangeForm> = Vec::new();
    let mut compressed_count = 0;
    println!("processing {} change forms.", count);
    for _i in 0..count {
        //println!("handling change_form {}", i);
        let form_id = read_ref_id(sfr);
        let change_flags = sfr.read_u32();
        let data_type = sfr.read_u8();
        let data_length_val = data_type & 0b11000000;
        let version = sfr.read_u8();

        match data_length_val {
            0 => {
                let length1 = sfr.read_u8();
                let length2 = sfr.read_u8();
                match length2 == 0 {
                    true => {
                        result.push(ChangeForm {
                            form_id,
                            change_flags,
                            data_type,
                            version,
                            length1: vec!(length1),
                            length2: vec!(length2),
                            data: sfr.read_bytes_to_vec(length1.into()),
                        });
                    }
                    false => {
                        compressed_count += 1;

                        let compressed = sfr.read_bytes_to_vec(length1.into());
                        let mut decoder = ZlibDecoder::new(compressed.as_slice());
                        let mut data: Vec<u8> = Vec::new();
                        decoder.read_to_end(&mut data).expect(CHANGE_FORM_DECODE_ERROR);
                        assert_eq!(data.len(), length2 as usize);
                        result.push(ChangeForm {
                            form_id,
                            change_flags,
                            data_type,
                            version,
                            length1: vec!(length1),
                            length2: vec!(length2),
                            data,
                        });
                    }
                };
            }
            64 => {
                let length1 = sfr.read_u16();
                let length2 = sfr.read_u16();
                match length2 == 0 {
                    true => {
                        result.push(ChangeForm {
                            form_id,
                            change_flags,
                            data_type,
                            version,
                            length1: length1.to_le_bytes().to_vec(),
                            length2: length2.to_le_bytes().to_vec(),
                            data: sfr.read_bytes_to_vec(length1.into()),
                        });
                    }
                    false => {
                        compressed_count += 1;

                        let compressed = sfr.read_bytes_to_vec(length1.into());
                        let mut decoder = ZlibDecoder::new(compressed.as_slice());
                        let mut data: Vec<u8> = Vec::new();
                        decoder.read_to_end(&mut data).expect(CHANGE_FORM_DECODE_ERROR);
                        assert_eq!(data.len(), length2 as usize);
                        result.push(ChangeForm {
                            form_id,
                            change_flags,
                            data_type,
                            version,
                            length1: length1.to_le_bytes().to_vec(),
                            length2: length2.to_le_bytes().to_vec(),
                            data,
                        });
                    }
                };
            }
            128 => {
                let length1 = sfr.read_u32();
                let length2 = sfr.read_u32();
                match length2 == 0 {
                    true => {
                        result.push(ChangeForm {
                            form_id,
                            change_flags,
                            data_type,
                            version,
                            length1: length1.to_le_bytes().to_vec(),
                            length2: length2.to_le_bytes().to_vec(),
                            data: sfr.read_bytes_to_vec(length1.try_into().expect("length1 value on change form too large.")),
                        });
                    }
                    false => {
                        compressed_count += 1;
                        let ulength1: usize = length1.try_into().expect("length1 value on change form too large.");
                        let compressed = sfr.read_bytes_to_vec(ulength1);
                        let mut decoder = ZlibDecoder::new(compressed.as_slice());
                        let mut data: Vec<u8> = Vec::new();
                        decoder.read_to_end(&mut data).expect(CHANGE_FORM_DECODE_ERROR);
                        assert_eq!(data.len(), length2 as usize);
                        result.push(ChangeForm {
                            form_id,
                            change_flags,
                            data_type,
                            version,
                            length1: length1.to_le_bytes().to_vec(),
                            length2: length2.to_le_bytes().to_vec(),
                            data,
                        });
                    }
                };
            }
            _ => panic!("length value on change form invalid!")
        };
    }
    if compressed_count != 0 {
        println!("Found {} compressed change forms. Those can currently not be parsed and get ignored.", compressed_count);
    }
    result
}