use lz4_flex::decompress;
use std::fmt;

pub mod global_data;

pub use global_data::*;

pub mod fundamental_types;

pub use fundamental_types::*;

pub mod change_form;

pub use change_form::*;

pub mod reader;

pub use reader::*;

pub mod header;
use header::*;

#[derive(Clone)]
pub struct ScreenshotData {
    pub height: u32,
    pub width: u32,
    pub data: Vec<u8>,
}

impl std::fmt::Debug for ScreenshotData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screenshot Data")
            .field("height", &self.height)
            .field("width", &self.width)
            .field("Size in bytes", &self.data.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct SaveFile {
    pub magic: String,
    pub header: Header,
    pub screenshot_data: ScreenshotData,
    pub body_uncompressed_len: u32,
    pub body_compressed_len: u32,
    pub form_version: u8,
    pub plugin_info: Vec<String>,
    pub light_plugin_info: Vec<String>,
    pub file_location_table: FileLocationTable,
    pub global_data_table_1: Vec<GlobalDataType>,
    pub global_data_table_2: Vec<GlobalDataType>,
    pub change_forms: Vec<ChangeForm>,
    pub global_data_table_3: Vec<GlobalDataType>,
    pub form_id_array: Vec<u32>,
    pub visited_worldspace_array: Vec<u32>,
    pub unknown_3_table: Vec<String>,
}

impl fmt::Debug for SaveFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SaveFile")
            .field("magic", &self.magic)
            .field("header", &self.header)
            .field("screenshot_data", &self.screenshot_data)
            .field("form_version", &self.form_version)
            .field("plugin_info (length)", &self.plugin_info.len())
            .field("light_plugin_info (length)", &self.light_plugin_info.len())
            .field("file_location_table", &self.file_location_table)
            .field("global_data_table_1 (length)", &self.global_data_table_1.len())
            .field("global_data_table_2 (length)", &self.global_data_table_2.len())
            .field("change_forms (length)", &self.change_forms.len())
            .field("global_data_table_3 (length)", &self.global_data_table_3.len())
            .field("form_id_array (length)", &self.form_id_array.len())
            .field("visited_worldspace_array (length)", &self.visited_worldspace_array.len())
            .field("unknown_3_table (length)", &self.unknown_3_table.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FileLocationTable {
    pub form_id_array_count_offset: u32,
    pub unknown_table_3_offset: u32,
    pub global_data_table_1_offset: u32,
    pub global_data_table_2_offset: u32,
    pub change_forms_offset: u32,
    pub global_data_table_3_offset: u32,
    pub global_data_table_1_count: u32,
    pub global_data_table_2_count: u32,
    pub global_data_table_3_count: u32,
    pub change_form_count: u32,
}


pub fn parse_save_file(buf: Vec<u8>) -> SaveFile {
    let mut sfr = SaveFileReader::new(buf);
    let magic = sfr.read_string(13);
    if magic != "TESV_SAVEGAME".to_string() {
        panic!("File invalid or corrupted, could not read magic.")
    }

    let _header_size = sfr.read_u32();

    let header = read_header(&mut sfr);

    let screenshot_data = sfr.read_bytes_to_vec((4 * header.shot_width * header.shot_height) as usize);

    let uncompressed_len = sfr.read_u32();
    let compressed_len = sfr.read_u32();

    let body_buffer = read_body(sfr, &header, uncompressed_len);
    let mut sfr_body = SaveFileReader::new(body_buffer);

    let form_version = sfr_body.read_u8();

    let _plugin_info_size = sfr_body.read_u32();
    let plugin_count = sfr_body.read_u8();
    let plugin_info = read_strings_into_vec(&mut sfr_body, plugin_count as u32);
    let light_plugin_count = sfr_body.read_u16();
    let light_plugin_info = read_strings_into_vec(&mut sfr_body, light_plugin_count as u32);

    let file_location_table = read_file_location_table(&mut sfr_body);

    // file location table has some unused space at the end, we need to advance to the data afterwards
    sfr_body.read_bytes_to_vec(4 * 15);

    let global_data_table_1 = read_global_data(&mut sfr_body, file_location_table.global_data_table_1_count);


    let global_data_table_2 = read_global_data(&mut sfr_body, file_location_table.global_data_table_2_count);

    let change_forms = read_change_forms(&mut sfr_body, file_location_table.change_form_count);

    // We need to add 1 to the global data table 3 count as that is the actual value, known bug in Skyrim
    let global_data_table_3 = read_global_data(&mut sfr_body, file_location_table.global_data_table_3_count + 1);

    let form_id_array_count = sfr_body.read_u32();
    let form_id_array: Vec<u32> = read_u32s_into_vec(&mut sfr_body, form_id_array_count);

    let visited_worldspace_array_count = sfr_body.read_u32();
    let visited_worldspace_array = read_u32s_into_vec(&mut sfr_body, visited_worldspace_array_count);

    let _unknown_3_table_size = sfr_body.read_u32();
    let unknown_3_table_count = sfr_body.read_u32();
    let unknown_3_table = read_strings_into_vec(&mut sfr_body, unknown_3_table_count);

    let screenshot_height = header.shot_height;
    let screenshot_width = header.shot_width;


    SaveFile {
        magic,
        header,
        screenshot_data: ScreenshotData {
            height: screenshot_height,
            width: screenshot_width,
            data: screenshot_data,
        },
        body_uncompressed_len: uncompressed_len,
        body_compressed_len: compressed_len,
        form_version,
        plugin_info,
        light_plugin_info,
        file_location_table,
        global_data_table_1,
        global_data_table_2,
        change_forms,
        global_data_table_3,
        form_id_array,
        visited_worldspace_array,
        unknown_3_table,
    }
}


fn read_body(sfr: SaveFileReader, header: &Header, uncompressed_len: u32) -> Vec<u8> {
    let index = sfr.get_index();
    let buffer = sfr.get_buffer();
    let buffer_len = buffer.len();

    let range = std::ops::Range { start: index, end: buffer_len };
    match header.compression_type {
        0 => buffer[range].to_vec(),
        1 => panic!("zLib compression not supported"),
        2 => {
            decompress(&buffer[range], uncompressed_len as usize)
                .expect("Could not decompress body! File may be corrupted.")
        }
        _ => panic!("Encountered unspecified/unsupported compression type. Is the file corrupted?")
    }
}

fn read_file_location_table(sfr_body: &mut SaveFileReader) -> FileLocationTable {
    FileLocationTable {
        form_id_array_count_offset: sfr_body.read_u32(),
        unknown_table_3_offset: sfr_body.read_u32(),
        global_data_table_1_offset: sfr_body.read_u32(),
        global_data_table_2_offset: sfr_body.read_u32(),
        change_forms_offset: sfr_body.read_u32(),
        global_data_table_3_offset: sfr_body.read_u32(),
        global_data_table_1_count: sfr_body.read_u32(),
        global_data_table_2_count: sfr_body.read_u32(),
        global_data_table_3_count: sfr_body.read_u32(),
        change_form_count: sfr_body.read_u32(),
    }
}


