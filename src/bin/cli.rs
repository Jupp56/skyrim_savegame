use std::env;
use skyrim_savegame::parse_save_file;
use std::fs::File;
use std::io::Read;

fn main() {
    let args : Vec<String> = env::args().collect();
    let save_file = args.get(1).expect("Please provide the file to parse as arg 1!");
    let mut fh = File::open(save_file).expect("Could not open file.");
    let mut buf: Vec<u8> = Vec::new();
    fh.read_to_end(&mut buf).expect("Could not read file!");
    let parsed_file = parse_save_file(buf);
    dbg!(parsed_file);
}