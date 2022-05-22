use crate::decoder::{Decode, Decoder};
use crate::errors::{DecodeError, DecodeErrorKind};
use crate::types::ValueType;

// Copyright 2022 Zenturi Software Co.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

const HLVERSION: u32 = 0x010C00;

#[derive(Clone)]
pub struct Code {
    pub types: Vec<ValueType>,
    pub ntypes: usize,
    pub strings: Vec<String>,
    pub ustrings: Vec<Option<String>>,
    pub nstrings: usize,
    pub nints: usize,
    pub ints: Vec<i32>,
    pub nfloats: usize,
    pub floats: Vec<f64>,
    pub nbytes: usize,
    pub nfunctions: usize,
    pub nconstants: usize,
    pub entrypoint: u32,
    pub nglobals: usize,
    pub nnatives:usize,
    pub hasdebug: u32,
    pub version: i8,
}

impl<'input> Code {
    pub fn new() -> Self {
        Code {
            types: Vec::new(),
            ntypes: 0,
            ustrings: Vec::new(),
            strings: Vec::new(),
            nfloats: 0,
            nints: 0,
            floats: Vec::new(),
            nstrings: 0,
            version: -1,
            ints: Vec::new(),
            nbytes: 0,
            nfunctions: 0,
            nconstants: 0,
            entrypoint: 0,
            nglobals: 0,
            nnatives:0,
            hasdebug: 0,
        }
    }

    pub fn read_string(decoder: &mut crate::decoder::Decoder,
    ) -> Result<String, DecodeError> {
        let index = i32::decode(decoder)?;
        if index < 0 || index >=  decoder.code.nstrings.try_into().unwrap() {
            Err(DecodeError::with_info(
                DecodeErrorKind::InvalidStringIndex,
                decoder.file_position,
            ))
        } else {
            let strings = decoder.code.strings.clone();
            Ok(strings.into_iter().nth(index.try_into().unwrap()).unwrap())
        }
    }

    pub fn get_ustring(&mut self, index: usize) -> String {
        let ustr = self.ustrings.clone().into_iter().nth(index).unwrap();

        match ustr {
            None => {
                let s = self.strings.clone().into_iter().nth(index);
                match s {
                    None => {
                        self.ustrings.insert(index, None);
                        ustr.unwrap()
                    }
                    Some(res) => {
                        self.ustrings.insert(index, Some(res));
                        ustr.unwrap()
                    }
                }
            }
            Some(s) => s,
        }
    }

    pub fn read_ustring(decoder: &mut crate::decoder::Decoder,
    ) -> Result<String, DecodeError> {
        let index = i32::decode(decoder)?;
        if index < 0 || index >=  decoder.code.nstrings.try_into().unwrap() {
            Err(DecodeError::with_info(
                DecodeErrorKind::InvalidStringIndex,
                decoder.file_position,
            ))
        } else {
            let string = decoder.code.get_ustring(index.try_into().unwrap());
            Ok(string)
        }
    }

    pub fn get_type(
        &self,
        decoder: &mut crate::decoder::Decoder,
    ) -> Result<ValueType, DecodeError> {
        let index = i32::decode(decoder)?;
        if index < 0 || index >= self.ntypes.try_into().unwrap() {
            Err(DecodeError::with_info(
                DecodeErrorKind::InvalidTypeIndex,
                decoder.file_position,
            ))
        } else {
            Ok(self
                .types
                .clone()
                .into_iter()
                .nth(index.try_into().unwrap())
                .unwrap())
        }
    }

    pub fn read(buf: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(buf);
        let mut c = Code::new();
        let max_version = 5;

        if u8::decode(&mut decoder)? as char != 'H'
            || u8::decode(&mut decoder)? as char != 'L'
            || u8::decode(&mut decoder)? as char != 'B'
        {
            return Err(DecodeError::with_info(
                DecodeErrorKind::InvalidBytecodeHeader,
                decoder.file_position,
            ));
        }
        c.version = u8::decode(&mut decoder)? as i8;
        if c.version <= 1 || c.version > max_version {
            println!(
                "Found version {} while HL {}.{} supports up to {}",
                c.version,
                HLVERSION >> 16,
                (HLVERSION >> 8) & 0xFF,
                max_version
            );
            return Err(DecodeError::with_info(
                DecodeErrorKind::UnsupportedBytecodeVersion,
                decoder.file_position,
            ));
        }

        let flags = u32::decode(&mut decoder)?;

        c.nints = u32::decode(&mut decoder)?.try_into().unwrap();
        c.nfloats = u32::decode(&mut decoder)?.try_into().unwrap();
        c.nstrings = u32::decode(&mut decoder)?.try_into().unwrap();

        if c.version >= 5 {
            c.nbytes = u32::decode(&mut decoder)?.try_into().unwrap();
        }
        c.ntypes = u32::decode(&mut decoder)?.try_into().unwrap();
        c.nglobals = u32::decode(&mut decoder)?.try_into().unwrap();
        c.nnatives = u32::decode(&mut decoder)?.try_into().unwrap();
        c.nfunctions = u32::decode(&mut decoder)?.try_into().unwrap();

        if c.version >= 4 {
            c.nconstants = u32::decode(&mut decoder)?.try_into().unwrap();
        } else {
            c.nconstants = 0;
        }

        c.entrypoint = u32::decode(&mut decoder)?;
        c.hasdebug = flags & 1;

        for i in 0..c.nints {
            c.ints.insert(i, i32::decode(&mut decoder)?)
        }

        for i in 0..c.nfloats {
            c.floats.insert(i, f64::decode(&mut decoder)?)
        }

        // Todo: read strings

        decoder.code = c;
        Ok(decoder.code)
    }
}
