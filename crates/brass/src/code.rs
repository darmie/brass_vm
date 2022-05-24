use std::io::Read;

use scanner_rust::generic_array::typenum::UInt;

use crate::decoder::{Decode, Decoder};
use crate::errors::{DecodeError, DecodeErrorKind};
use crate::types::{
    EnumConstruct, EnumType, FuncType, ObjField, ObjProto, ObjType, TypeKind, ValueType,
    ValueTypeU, VirtualType,
};

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

fn UINDEX(decoder: &mut Decoder) -> Result<usize, DecodeError> {
    Ok(u32::decode(decoder)?.try_into().unwrap())
}

fn INDEX(decoder: &mut Decoder) -> Result<i32, DecodeError> {
    Ok(i32::decode(decoder)?)
}

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
    pub nnatives: usize,
    pub hasdebug: u32,
    pub version: i8,
    pub bytes: Vec<u8>,
    pub bytes_pos: Vec<u32>,
    pub ndebugfiles: usize,
    pub debugfiles: Vec<String>,
    pub debugfiles_lens: Vec<usize>,
}

impl Code {
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
            nnatives: 0,
            hasdebug: 0,
            bytes: Vec::new(),
            bytes_pos: Vec::new(),
            ndebugfiles: 0,
            debugfiles: Vec::new(),
            debugfiles_lens: Vec::new(),
        }
    }

    pub fn read_string(decoder: &mut crate::decoder::Decoder) -> Result<String, DecodeError> {
        let index = i32::decode(decoder)?;
        if index < 0 || index >= decoder.code.nstrings.try_into().unwrap() {
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

    pub fn read_ustring(decoder: &mut crate::decoder::Decoder) -> Result<String, DecodeError> {
        let index = i32::decode(decoder)?;
        if index < 0 || index >= decoder.code.nstrings.try_into().unwrap() {
            Err(DecodeError::with_info(
                DecodeErrorKind::InvalidStringIndex,
                decoder.file_position,
            ))
        } else {
            let string = decoder.code.get_ustring(index.try_into().unwrap());
            Ok(string)
        }
    }

    pub fn get_type(decoder: &mut crate::decoder::Decoder) -> Result<ValueType, DecodeError> {
        let index = i32::decode(decoder)?;
        if index < 0 || index >= decoder.code.ntypes.try_into().unwrap() {
            Err(DecodeError::with_info(
                DecodeErrorKind::InvalidTypeIndex,
                decoder.file_position,
            ))
        } else {
            Ok(decoder
                .code
                .types
                .clone()
                .into_iter()
                .nth(index.try_into().unwrap())
                .unwrap())
        }
    }

    pub fn read_type(
        decoder: &mut crate::decoder::Decoder,
        t: &mut ValueType,
    ) -> Result<(), DecodeError> {
        t.kind = TypeKind::try_from(u32::decode(decoder)?).expect("invalid type");

        match t.kind {
            TypeKind::HFUN | TypeKind::HMETHOD => {
                let nargs: usize = u8::decode(decoder)?.try_into().unwrap();
                let mut fun = FuncType {
                    nargs,
                    args: Vec::new(),
                    ret: Box::new(ValueType::default()),
                };

                for i in 0..nargs {
                    fun.args.insert(i, Code::get_type(decoder)?);
                }

                fun.ret = Box::new(Code::get_type(decoder)?);

                t.union = ValueTypeU::FuncType(fun);
            }
            TypeKind::HDYNOBJ | TypeKind::HSTRUCT => {
                // Todo: decode dynamic obj and structs
                let name = Code::read_ustring(decoder)?;
                let superIndex = INDEX(decoder)?;
                let mut obj = ObjType {
                    name,
                    super_type: if superIndex < 0 {
                        Box::new(ValueType::default())
                    } else {
                        Box::new(
                            decoder
                                .code
                                .types
                                .clone()
                                .into_iter()
                                .nth(superIndex.try_into().unwrap())
                                .unwrap(),
                        )
                    },
                    global_value: &(UINDEX(decoder)?),
                    nfields: UINDEX(decoder)?,
                    nproto: UINDEX(decoder)?,
                    nbindings: UINDEX(decoder)?,
                    fields: Vec::new(),
                    proto: Vec::new(),
                    bindings: Vec::new(),
                    rt: None,
                };

                for i in 0..obj.nfields {
                    let field = ObjField {
                        name: Code::read_ustring(decoder)?,
                        hashed_name: -1, // Todo: implement hash generator
                        t: Code::get_type(decoder)?,
                    };

                    obj.fields.insert(i, field);
                }
                for i in 0..obj.nproto {
                    let obj_proto = ObjProto {
                        name: Code::read_ustring(decoder)?,
                        hashed_name: -1, // Todo: implement hash generator
                        findex: UINDEX(decoder)?,
                        pindex: INDEX(decoder)?,
                    };

                    obj.proto.insert(i, obj_proto);
                }
                for i in 0..obj.nbindings {
                    obj.bindings
                        .insert(i << 1, UINDEX(decoder)?.try_into().unwrap());
                    obj.bindings
                        .insert((i << 1) | 1, UINDEX(decoder)?.try_into().unwrap());
                }

                t.union = ValueTypeU::ObjType(obj);
            }
            TypeKind::HREF => {
                t.tparam = Box::new(Code::get_type(decoder)?);
            }
            TypeKind::HVIRTUAL => {
                let nfields = UINDEX(decoder)?;
                let mut virt = VirtualType {
                    nfields,
                    fields: Vec::new(),
                };

                for i in 0..nfields {
                    let field = ObjField {
                        name: Code::read_ustring(decoder)?,
                        hashed_name: -1, // Todo: implement hash generator
                        t: Code::get_type(decoder)?,
                    };

                    virt.fields.insert(i, field);
                }

                t.union = ValueTypeU::VirtualType(virt);
            }
            TypeKind::HABSTRACT => {
                t.abs_name = Some(Code::read_ustring(decoder)?);
            }
            TypeKind::HENUM => {
                let mut tenum = EnumType {
                    name: Code::read_ustring(decoder)?,
                    global_value: &(UINDEX(decoder)?), // Todo
                    nconstructs: UINDEX(decoder)?,
                    constructs: Vec::new(),
                };

                for i in 0..tenum.nconstructs {
                    let name = Code::read_ustring(decoder)?;
                    let nparams = UINDEX(decoder)?;
                    let mut con = EnumConstruct {
                        name,
                        nparams,
                        params: Vec::new(),
                        offsets: Vec::new(),
                        hasptr: false,
                        size: 0,
                    };
                    for j in 0..nparams {
                        con.params.insert(j, Code::get_type(decoder)?);
                    }

                    tenum.constructs.insert(i, con);
                }

                t.union = ValueTypeU::EnumType(tenum);
            }
            TypeKind::HNULL | TypeKind::HPACKED => {
                t.tparam = Box::new(Code::get_type(decoder)?);
            }
            _ => {
                if u32::try_from(t.kind) >= u32::try_from(TypeKind::HLAST) {
                    return Err(DecodeError::with_info(
                        DecodeErrorKind::InvalidType,
                        decoder.file_position,
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn read_strings(
        decoder: &mut crate::decoder::Decoder,
        nstrings: usize,
        out_lens: &mut Vec<usize>,
    ) -> Result<Vec<String>, DecodeError> {
        let size = INDEX(decoder)?;
        let mut sdata: Vec<u8> = vec![0; size.try_into().unwrap()];
        decoder.read_bytes_vec(&mut sdata)?;
        let mut strings: Vec<String> = Vec::new();
        let mut cur = 0;
        for i in 0..nstrings {
            let sz: usize = UINDEX(decoder)?;
            strings.insert(
                i,
                String::from_utf8(sdata[cur..sz].to_vec()).expect("invalid string"),
            );
            out_lens.insert(i, sz);

            cur = sz + 1;
        }

        Ok(strings)
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

        c.nints = UINDEX(&mut decoder)?;
        c.nfloats = UINDEX(&mut decoder)?;
        c.nstrings = UINDEX(&mut decoder)?;

        if c.version >= 5 {
            c.nbytes = UINDEX(&mut decoder)?;
        }
        c.ntypes = UINDEX(&mut decoder)?;
        c.nglobals = UINDEX(&mut decoder)?;
        c.nnatives = UINDEX(&mut decoder)?;
        c.nfunctions = UINDEX(&mut decoder)?;

        if c.version >= 4 {
            c.nconstants = UINDEX(&mut decoder)?;
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

        let mut lens = Vec::new();

        c.strings = Code::read_strings(&mut decoder, c.nstrings, &mut lens)?;
        if c.version >= 5 {
            let size: usize = i32::decode(&mut decoder)?.try_into().unwrap();
            c.bytes = vec![0; size];
            decoder.read_bytes_vec(&mut c.bytes)?;
            for i in 0..c.nbytes {
                c.bytes_pos.insert(i, u32::decode(&mut decoder)?);
            }
        }

        if c.hasdebug == 1 {
            c.ndebugfiles = UINDEX(&mut decoder)?;
            c.debugfiles = Code::read_strings(&mut decoder, c.ndebugfiles, &mut c.debugfiles_lens)?;
        }

        for i in 0..c.ntypes {
            let mut t = ValueType::default();
            Code::read_type(&mut decoder, &mut t)?;
        }

        decoder.code = c;
        Ok(decoder.code)
    }
}
