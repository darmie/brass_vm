// use std::{
//     borrow::Borrow,
//     fs::File,
//     io::{self, BufReader, Read},
// };

// use scanner_rust::ScannerU8Slice;

// use scanner_rust::Scanner;

// use crate::{code::Code, types::ValueType};

// // Copyright 2022 Zenturi Software Co.
// //
// // Licensed under the Apache License, Version 2.0 (the "License");
// // you may not use this file except in compliance with the License.
// // You may obtain a copy of the License at
// //
// //     http://www.apache.org/licenses/LICENSE-2.0
// //
// // Unless required by applicable law or agreed to in writing, software
// // distributed under the License is distributed on an "AS IS" BASIS,
// // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// // See the License for the specific language governing permissions and
// // limitations under the License.

// pub struct ByteCodeReader {
//     // pub b:  BufReader<File>,
//     pub err: Option<String>,
//     // pub pos: &'static mut usize,
//     pub code: Code,
//     pub scanner: Scanner<BufReader<File>>,
// }

// impl ByteCodeReader {
//     pub fn new(path: String) -> io::Result<Self> {
//         let f = File::open(path.as_str())?;

//         let mut reader = BufReader::new(f);

//         let mut buffer = Vec::new();
//         reader.read_to_end(&mut buffer)?;

//         let sc = Scanner::new(reader);

//         // let sc = ScannerU8Slice::new(buffer.as_slice());

//         Ok(Self {
//             // b: reader,
//             err: None,
//             // pos: &mut 0,
//             code: Code::new(),
//             scanner: sc,
//         })
//     }


// }

// pub fn read_b(reader: &mut ByteCodeReader ) -> u8 {
//     let r = reader.scanner.next_u8();
//     if r.is_err() {
//         reader.err = Some(String::from("No more data"));
//         return 0;
//     }

//     r.unwrap().unwrap_or(0)
// }

// pub fn read_index(reader: &mut ByteCodeReader ) -> i32 {
//     let r = reader.scanner.next_i32();
//     if r.is_err() {
//         reader.err = Some(String::from("Could not read index"));
//         return 0;
//     }
//     r.unwrap().unwrap_or(0)
// }

// pub fn get_ustring(reader: &mut ByteCodeReader, i: usize) -> &str {
//     match reader.code.ustrings[i] {
//         None => {
//             let s = &reader.code.strings[i];
//             s.as_str()
//         }
//         Some(s) => s,
//     }
// }

// pub fn read_ustring(reader: &mut ByteCodeReader ) -> &str {
//     let mut i = read_index(reader) as usize;
//     if i >= reader.code.nstrings {
//         reader.err = Some(String::from("Invalid string index"));
//         i = 0;
//     }

//     get_ustring(reader,i)
// }

// pub fn read_uindex(reader:&mut ByteCodeReader ) -> u32 {
//     let r = reader.scanner.next_u32();
//     if r.is_err() {
//         reader.err = Some(String::from("Could not read index"));
//         return 0;
//     }
//     r.unwrap().unwrap_or(0)
// }

// pub fn get_type(reader: &mut ByteCodeReader ) -> ValueType {
//     let mut i: usize = read_index(reader).try_into().unwrap();
//     if i >= reader.code.ntypes {
//         reader.err = Some(String::from("Invalid type index"));
//         i = 0;
//     }

//     let v = &reader.code.types[i];
//     v.clone()
// }



