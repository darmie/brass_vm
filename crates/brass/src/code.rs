use std::num::Wrapping;

use crc32fast::hash;

use crate::decoder::{Decode, Decoder};
use crate::errors::{DecodeError, DecodeErrorKind};
use crate::native::Native;
use crate::op::{Op, Opcode, OP_NARGS};
use crate::types::{
    Constant, EnumConstruct, HLFunction, ObjField, ObjProto, TypeKind, ValueType, ValueTypeU,
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
    let i = INDEX(decoder)?;
    if i < 0 {
        // return Err(DecodeError::with_info(
        //     DecodeErrorKind::NegativeIndex,
        //     decoder.file_position,
        // ));
        return Ok(0);
    }
    Ok(i.try_into().unwrap())
}

fn INDEX(decoder: &mut Decoder) -> Result<i32, DecodeError> {
    let mut buf = <[u8; 4]>::default();
    let i = decoder.read_index(&mut buf)?;
    Ok(i)
}

#[derive(Clone, Debug)]
pub struct Code {
    pub types: Vec<ValueType>,
    pub ntypes: usize,
    pub strings: Vec<String>,
    pub strings_lens: Vec<usize>,
    pub ustrings: Vec<Option<String>>,
    pub nstrings: usize,
    pub nints: usize,
    pub ints: Vec<i32>,
    pub nfloats: usize,
    pub floats: Vec<f64>,
    pub nbytes: usize,
    pub nfunctions: usize,
    pub functions: Vec<HLFunction>,
    pub nconstants: usize,
    pub constants: Vec<Constant>,
    pub entrypoint: u32,
    pub nglobals: usize,
    pub globals: Vec<ValueType>,
    pub nnatives: usize,
    pub natives: Vec<Native>,
    pub hasdebug: u32,
    pub version: u8,
    pub bytes: Vec<u8>,
    pub bytes_pos: Vec<usize>,
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
            version: 0,
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
            globals: Vec::new(),
            natives: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            strings_lens: Vec::new(),
        }
    }

    pub fn read_string(decoder: &mut crate::decoder::Decoder) -> Result<String, DecodeError> {
        let index = INDEX(decoder)?;
        if index < 0 || index >= decoder.code.nstrings.try_into().unwrap() {
            // Err(DecodeError::with_info(
            //     DecodeErrorKind::InvalidStringIndex,
            //     decoder.file_position,
            // ))
            Ok(String::new())
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
                        String::new()
                    }
                    Some(res) => {
                        let ss = res.clone();
                        self.ustrings.insert(index, Some(res));
                        ss
                    }
                }
            }
            Some(s) => s,
        }
    }

    pub fn read_ustring(decoder: &mut crate::decoder::Decoder) -> Result<String, DecodeError> {
        let index = INDEX(decoder)?;
        if index < 0 || index >= decoder.code.nstrings.try_into().unwrap() {
            // Err(DecodeError::with_info(
            //     DecodeErrorKind::InvalidStringIndex,
            //     decoder.file_position,
            // ))
            Ok(String::new())
        } else {
            let string = decoder.code.get_ustring(index.try_into().unwrap());
            Ok(string)
        }
    }

    pub fn get_type(decoder: &mut crate::decoder::Decoder) -> Result<ValueType, DecodeError> {
        let mut index = INDEX(decoder)?;
        if index < 0 || index >= decoder.code.ntypes.try_into().unwrap() {
            // Err(DecodeError::with_info(
            //     DecodeErrorKind::InvalidTypeIndex,
            //     decoder.file_position,
            // ))
            index = 0;
        }

        Ok(decoder
            .code
            .types
            .clone()
            .into_iter()
            .nth(index.try_into().unwrap())
            .unwrap())
    }

    pub fn read_type(
        decoder: &mut crate::decoder::Decoder,
        t: &mut ValueType,
    ) -> Result<(), DecodeError> {
        let v = u8::decode(decoder)?;
        let k = TypeKind::try_from(v);

        if k.is_err() && v >= u8::try_from(TypeKind::HLAST).unwrap() {
            // return Err(DecodeError::with_info(
            //     DecodeErrorKind::InvalidType,
            //     decoder.file_position,
            // ));
            return Ok(());
        }

        t.kind = k.unwrap();

        match t.kind {
            TypeKind::HFUN | TypeKind::HMETHOD => {
                let nargs: usize = u8::decode(decoder)?.try_into().unwrap();
                let mut fun = ValueTypeU::FuncType {
                    nargs,
                    args: Vec::new(),
                    ret: Box::new(ValueType::default()),
                };

                if let ValueTypeU::FuncType {
                    ref mut args,
                    ref mut ret,
                    nargs,
                } = fun
                {
                    for i in 0..nargs {
                        (*args).insert(i, Code::get_type(decoder)?);
                    }

                    *ret = Box::new(Code::get_type(decoder)?);
                }

                t.union = fun;
            }
            TypeKind::HOBJ | TypeKind::HSTRUCT => {
                let name = Code::read_ustring(decoder)?;
                let super_index = INDEX(decoder)?;
                let mut obj = ValueTypeU::ObjType {
                    name,
                    super_type: if super_index < 0 {
                        Box::new(ValueType::default())
                    } else {
                        Box::new(
                            decoder
                                .code
                                .types
                                .clone()
                                .into_iter()
                                .nth(super_index.try_into().unwrap())
                                .unwrap(),
                        )
                    },
                    global_value: vec![(UINDEX(decoder)?) as isize],
                    nfields: UINDEX(decoder)?,
                    nproto: UINDEX(decoder)?,
                    nbindings: UINDEX(decoder)?,
                    fields: Vec::new(),
                    proto: Vec::new(),
                    bindings: Vec::new(),
                    rt: None,
                };

                if let ValueTypeU::ObjType {
                    name: _,
                    super_type: _,
                    ref mut fields,
                    nfields,
                    nproto,
                    nbindings,
                    ref mut proto,
                    ref mut bindings,
                    global_value: _,
                    rt: _,
                } = obj
                {
                    for i in 0..nfields {
                        let name = Code::read_ustring(decoder)?;
                        let hashed_name =  hash(name.as_bytes());
                        let field = ObjField {
                            name,
                            hashed_name,
                            t: Code::get_type(decoder)?,
                        };

                        (*fields).insert(i, field);
                    }
                    for i in 0..nproto {
                        let name = Code::read_ustring(decoder)?;
                        let hashed_name =  hash(name.as_bytes());
                        let obj_proto = ObjProto {
                            name,
                            hashed_name,
                            findex: UINDEX(decoder)?,
                            pindex: INDEX(decoder)?,
                        };

                        (*proto).insert(i, obj_proto);
                    }
                    for i in 0..nbindings {
                        (*bindings).insert(i << 1, UINDEX(decoder)?.try_into().unwrap());
                        (*bindings).insert((i << 1) | 1, UINDEX(decoder)?.try_into().unwrap());
                    }
                }
                
                t.union = obj;
            }
            TypeKind::HREF => {
                t.tparam = Some(Box::new(Code::get_type(decoder)?));
            }
            TypeKind::HVIRTUAL => {
                let nfields = UINDEX(decoder)?;
                let mut virt = ValueTypeU::VirtualType {
                    nfields,
                    fields: Vec::new(),
                };

                if let ValueTypeU::VirtualType {
                    nfields,
                    ref mut fields,
                } = virt
                {
                    for i in 0..nfields {
                        let name = Code::read_ustring(decoder)?;
                        let hashed_name =  hash(name.as_bytes());
                        let field = ObjField {
                            name,
                            hashed_name,
                            t: Code::get_type(decoder)?,
                        };

                        (*fields).insert(i, field);
                    }
                }

                t.union = virt;
            }
            TypeKind::HABSTRACT => {
                t.abs_name = Some(Code::read_ustring(decoder)?);
            }
            TypeKind::HENUM => {
                let mut tenum = ValueTypeU::EnumType {
                    name: Code::read_ustring(decoder)?,
                    global_value: vec![(UINDEX(decoder)?) as isize], // Todo
                    nconstructs: UINDEX(decoder)?,
                    constructs: Vec::new(),
                };

                if let ValueTypeU::EnumType {
                    name: _,
                    nconstructs,
                    ref mut constructs,
                    global_value: _,
                } = tenum
                {
                    for i in 0..nconstructs {
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

                        (*constructs).insert(i, con);
                    }
                }

                t.union = tenum;
            }
            TypeKind::HNULL | TypeKind::HPACKED => {
                t.tparam = Some(Box::new(Code::get_type(decoder)?));
            }
            _ => {
                if u8::try_from(t.kind) >= u8::try_from(TypeKind::HLAST) {
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
        let size = i32::decode(decoder)?;
        let mut sdata = vec![0; size.try_into().unwrap()];

        let d = decoder.read_bytes(&mut sdata);

        if d.is_err() {
            // invalid string
            return Ok([].to_vec());
        }

        let mut strings: Vec<String> = vec![String::from(""); nstrings];

        for i in 0..nstrings {
            let sz: usize = UINDEX(decoder)?;
            let s = String::from_utf8_lossy(&sdata[..sz]).to_string();
            strings.insert(i, s);

            out_lens.insert(i, sz);

            sdata = sdata[sz..].to_vec();

            if sdata.len() >= size.try_into().unwrap() {
                break;
            }
            sdata = sdata[1..].to_vec();
        }

        Ok(strings)
    }

    pub fn read_function(decoder: &mut Decoder) -> Result<HLFunction, DecodeError> {
        let mut f = HLFunction {
            t: Code::get_type(decoder)?,
            findex: UINDEX(decoder)?,
            nregs: UINDEX(decoder)?,
            nops: UINDEX(decoder)?,
            regs: Vec::new(),
            ops: Vec::new(),
            debug: Vec::new(),
            rf: 0,
            obj: None,
            field: None,
        };

        for i in 0..f.nregs {
            f.regs.insert(i, Code::get_type(decoder)?);
        }

        for i in 0..f.nops {
            let op = Code::read_opcode(decoder)?;
            f.ops.insert(i, op);
        }

        Ok(f)
    }

    pub fn read_opcode(decoder: &mut Decoder) -> Result<Opcode, DecodeError> {
        let n = u8::decode(decoder)?;
        let mut res = Opcode::default();
        if n >= 99 {
            // return Err(DecodeError::with_info(
            //     DecodeErrorKind::InvalidOpcode,
            //     decoder.file_position,
            // ));
            return Ok(res);
        }

        let op: Op = n.try_into().unwrap();
        res.op = op;
        let i = OP_NARGS.into_iter().nth(n.try_into().unwrap()).unwrap();
        match i {
            0 => {}
            1 => {
                res.p1 = Some(INDEX(decoder)?);
            }
            2 => {
                res.p1 = Some(INDEX(decoder)?);
                res.p2 = Some(INDEX(decoder)?);
            }
            3 => {
                res.p1 = Some(INDEX(decoder)?);
                res.p2 = Some(INDEX(decoder)?);
                res.p3 = Some(INDEX(decoder)?);
            }
            4 => {
                res.p1 = Some(INDEX(decoder)?);
                res.p2 = Some(INDEX(decoder)?);
                res.p3 = Some(INDEX(decoder)?);
                let i: isize = INDEX(decoder)?.try_into().unwrap(); // not sure if this is necessary
                res.extra.insert(0, i);
            }
            -1 => match res.op {
                Op::OCallN | Op::OCallClosure | Op::OCallMethod | Op::OCallThis | Op::OMakeEnum => {
                    res.p1 = Some(INDEX(decoder)?);
                    res.p2 = Some(INDEX(decoder)?);
                    let p3 = u8::decode(decoder)?.into();
                    res.p3 = Some(p3);
                    let extra = vec![0; p3.try_into().unwrap()];
                    res.extra = extra;

                    for i in 0..p3 {
                        res.extra.insert(i.try_into().unwrap(), INDEX(decoder)?.try_into().unwrap());
                    }
                }
                Op::OSwitch => {
                    res.p1 = Some(UINDEX(decoder)?.try_into().unwrap());
                    let p2 = UINDEX(decoder)?.try_into().unwrap();
                    res.p2 = Some(p2);

                    res.extra = vec![0; p2.try_into().unwrap()];

                    for i in 0..p2 {
                        res.extra
                            .insert(i.try_into().unwrap(), UINDEX(decoder)?.try_into().unwrap());
                    }
                    res.p3 = Some(UINDEX(decoder)?.try_into().unwrap());
                }
                _ => {
                    return Err(DecodeError::with_info(
                        DecodeErrorKind::CouldNotProcessOpcode,
                        decoder.file_position,
                    ));
                }
            },
            _ => {
                let size = OP_NARGS.into_iter().nth(n.try_into().unwrap()).unwrap() - 3;
                res.p1 = Some(INDEX(decoder)?);
                res.p2 = Some(INDEX(decoder)?);
                res.p3 = Some(INDEX(decoder)?);

                res.extra = vec![0; size.try_into().unwrap()];

                for i in 0..size {
                    res.extra.insert(i.try_into().unwrap(), INDEX(decoder)?.try_into().unwrap());
                }
            }
        }

        Ok(res)
    }

    pub fn debu_infos(decoder: &mut Decoder, nops: usize) -> Result<Vec<i32>, DecodeError> {
        let mut curfile: i8 = -1;
        let mut curline: usize = 0;

        let mut debug: Vec<i32> = vec![0; 4 * nops * 2];

        let mut i: usize = 0;

        loop {
            if (i as usize) < nops {
                let mut c = u8::decode(decoder)?;
                if (c & 1) != 0 {
                    c >>= 1;
                    curfile = (c | u8::decode(decoder)?) as i8;
                    if curfile >= decoder.code.ndebugfiles.try_into().unwrap() {
                        // ERROR("Invalid debug file");
                    }
                } else if (c & 2) != 0 {
                    let delta = c >> 6;
                    let mut count = ((c >> 2) & 15) as i32;
                    if i + count as usize > nops {
                        // ERROR("Outside range");
                    }
                    // count -= 1;
                    loop {
                        if count >= 0 {
                            break;
                        }
                        debug[(i << 1) as usize] = curfile as i32;
                        debug[((i << 1) | 1) as usize] = curline as i32;
                        i += 1;
                        count -= 1;
                    }
                    curline += delta as usize;
                } else if (c & 4) != 0 {
                    curline += (c >> 3) as usize;
                    debug[(i << 1) as usize] = curfile as i32;
                    debug[((i << 1) | 1) as usize] = curline as i32;
                    i += 1;
                } else {
                    let b2 = u8::decode(decoder)?;
                    let b3: u32 = u8::decode(decoder)?.into();
                    curline = ((c >> 3) as u32 | (b2 << 5) as u32 | (b3 << 13)) as usize;
                    debug[(i << 1) as usize] = curfile as i32;
                    debug[((i << 1) | 1) as usize] = curline as i32;
                    i += 1;
                }
            } else {
                break;
            }
        }

        Ok(debug)
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
        c.version = u8::decode(&mut decoder)?;
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

        let flags = UINDEX(&mut decoder)?;

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

        c.entrypoint = UINDEX(&mut decoder)?.try_into().unwrap();
        c.hasdebug = flags as u32 & 1;

        for i in 0..c.nints {
            c.ints.insert(i, i32::decode(&mut decoder)?)
        }

        for i in 0..c.nfloats {
            c.floats.insert(i, f64::decode(&mut decoder)?)
        }

        c.strings = Code::read_strings(&mut decoder, c.nstrings, &mut c.strings_lens)?;
        c.ustrings = vec![None; c.nstrings];
        if c.version >= 5 {
            let size: usize = i32::decode(&mut decoder)?.try_into().unwrap();
            c.bytes = vec![0; size];
            decoder.read_bytes(&mut c.bytes)?;
            for i in 0..c.nbytes {
                c.bytes_pos.insert(i, UINDEX(&mut decoder)?);
            }
        }

        if c.hasdebug != 0 {
            c.ndebugfiles = UINDEX(&mut decoder)?;
            decoder.code = c.clone();
            decoder.code.debugfiles =
                Code::read_strings(&mut decoder, c.ndebugfiles, &mut c.debugfiles_lens)?;
            c = decoder.code.clone();
        }

        c.types = vec![ValueType::default(); c.ntypes];

        for i in 0..c.ntypes {
            let mut t = ValueType::default();
            decoder.code = c.clone();
            Code::read_type(&mut decoder, &mut t)?;
            decoder.code.types.insert(i, t);
            c = decoder.code.clone();
        }

        c.globals = vec![ValueType::default(); c.nglobals];
        for i in 0..c.nglobals {
            decoder.code = c.clone();
            let t = Code::get_type(&mut decoder)?;
            decoder.code.globals.insert(i, t);
            c = decoder.code.clone();
        }

        c.natives = Vec::new();
        for _i in 0..c.nnatives {
            c.natives.push(Native {
                lib: Code::read_string(&mut decoder)?,
                name: Code::read_string(&mut decoder)?,
                t: Code::get_type(&mut decoder)?,
                findex: UINDEX(&mut decoder)?,
            });
            decoder.code = c.clone();
        }

        for i in 0..c.nfunctions {
            decoder.code = c.clone();
            let f = Code::read_function(&mut decoder)?;
            decoder.code.functions.push(f);
            if decoder.code.hasdebug != 0 {
                let nops = decoder.code.functions[i].nops;
                decoder.code.functions[i].debug = Code::debu_infos(&mut decoder, nops)?;
                if decoder.code.version >= 3 {
                    // skip assigns (no need here)
                    let nassigns = UINDEX(&mut decoder)?;
                    for _j in 0..nassigns {
                        UINDEX(&mut decoder)?;
                        INDEX(&mut decoder)?;
                    }
                }
            }
            c = decoder.code.clone();
        }

        for i in 0..c.nconstants {
            let mut k = Constant {
                global: UINDEX(&mut decoder)?.try_into().unwrap(),
                nfields: UINDEX(&mut decoder)?,
                fields: Vec::new(),
            };

            for j in 0..k.nfields {
                k.fields
                    .insert(j, UINDEX(&mut decoder)?.try_into().unwrap());
            }

            c.constants.insert(i, k);
            decoder.code = c.clone();
        }

        Ok(decoder.code)
    }
}


// pub const CRC32_TABLE:[u32;256] = [
//     0x00000000, 0x04c11db7, 0x09823b6e, 0x0d4326d9,
//     0x130476dc, 0x17c56b6b, 0x1a864db2, 0x1e475005,
//     0x2608edb8, 0x22c9f00f, 0x2f8ad6d6, 0x2b4bcb61,
//     0x350c9b64, 0x31cd86d3, 0x3c8ea00a, 0x384fbdbd,
//     0x4c11db70, 0x48d0c6c7, 0x4593e01e, 0x4152fda9,
//     0x5f15adac, 0x5bd4b01b, 0x569796c2, 0x52568b75,
//     0x6a1936c8, 0x6ed82b7f, 0x639b0da6, 0x675a1011,
//     0x791d4014, 0x7ddc5da3, 0x709f7b7a, 0x745e66cd,
//     0x9823b6e0, 0x9ce2ab57, 0x91a18d8e, 0x95609039,
//     0x8b27c03c, 0x8fe6dd8b, 0x82a5fb52, 0x8664e6e5,
//     0xbe2b5b58, 0xbaea46ef, 0xb7a96036, 0xb3687d81,
//     0xad2f2d84, 0xa9ee3033, 0xa4ad16ea, 0xa06c0b5d,
//     0xd4326d90, 0xd0f37027, 0xddb056fe, 0xd9714b49,
//     0xc7361b4c, 0xc3f706fb, 0xceb42022, 0xca753d95,
//     0xf23a8028, 0xf6fb9d9f, 0xfbb8bb46, 0xff79a6f1,
//     0xe13ef6f4, 0xe5ffeb43, 0xe8bccd9a, 0xec7dd02d,
//     0x34867077, 0x30476dc0, 0x3d044b19, 0x39c556ae,
//     0x278206ab, 0x23431b1c, 0x2e003dc5, 0x2ac12072,
//     0x128e9dcf, 0x164f8078, 0x1b0ca6a1, 0x1fcdbb16,
//     0x018aeb13, 0x054bf6a4, 0x0808d07d, 0x0cc9cdca,
//     0x7897ab07, 0x7c56b6b0, 0x71159069, 0x75d48dde,
//     0x6b93dddb, 0x6f52c06c, 0x6211e6b5, 0x66d0fb02,
//     0x5e9f46bf, 0x5a5e5b08, 0x571d7dd1, 0x53dc6066,
//     0x4d9b3063, 0x495a2dd4, 0x44190b0d, 0x40d816ba,
//     0xaca5c697, 0xa864db20, 0xa527fdf9, 0xa1e6e04e,
//     0xbfa1b04b, 0xbb60adfc, 0xb6238b25, 0xb2e29692,
//     0x8aad2b2f, 0x8e6c3698, 0x832f1041, 0x87ee0df6,
//     0x99a95df3, 0x9d684044, 0x902b669d, 0x94ea7b2a,
//     0xe0b41de7, 0xe4750050, 0xe9362689, 0xedf73b3e,
//     0xf3b06b3b, 0xf771768c, 0xfa325055, 0xfef34de2,
//     0xc6bcf05f, 0xc27dede8, 0xcf3ecb31, 0xcbffd686,
//     0xd5b88683, 0xd1799b34, 0xdc3abded, 0xd8fba05a,
//     0x690ce0ee, 0x6dcdfd59, 0x608edb80, 0x644fc637,
//     0x7a089632, 0x7ec98b85, 0x738aad5c, 0x774bb0eb,
//     0x4f040d56, 0x4bc510e1, 0x46863638, 0x42472b8f,
//     0x5c007b8a, 0x58c1663d, 0x558240e4, 0x51435d53,
//     0x251d3b9e, 0x21dc2629, 0x2c9f00f0, 0x285e1d47,
//     0x36194d42, 0x32d850f5, 0x3f9b762c, 0x3b5a6b9b,
//     0x0315d626, 0x07d4cb91, 0x0a97ed48, 0x0e56f0ff,
//     0x1011a0fa, 0x14d0bd4d, 0x19939b94, 0x1d528623,
//     0xf12f560e, 0xf5ee4bb9, 0xf8ad6d60, 0xfc6c70d7,
//     0xe22b20d2, 0xe6ea3d65, 0xeba91bbc, 0xef68060b,
//     0xd727bbb6, 0xd3e6a601, 0xdea580d8, 0xda649d6f,
//     0xc423cd6a, 0xc0e2d0dd, 0xcda1f604, 0xc960ebb3,
//     0xbd3e8d7e, 0xb9ff90c9, 0xb4bcb610, 0xb07daba7,
//     0xae3afba2, 0xaafbe615, 0xa7b8c0cc, 0xa379dd7b,
//     0x9b3660c6, 0x9ff77d71, 0x92b45ba8, 0x9675461f,
//     0x8832161a, 0x8cf30bad, 0x81b02d74, 0x857130c3,
//     0x5d8a9099, 0x594b8d2e, 0x5408abf7, 0x50c9b640,
//     0x4e8ee645, 0x4a4ffbf2, 0x470cdd2b, 0x43cdc09c,
//     0x7b827d21, 0x7f436096, 0x7200464f, 0x76c15bf8,
//     0x68860bfd, 0x6c47164a, 0x61043093, 0x65c52d24,
//     0x119b4be9, 0x155a565e, 0x18197087, 0x1cd86d30,
//     0x029f3d35, 0x065e2082, 0x0b1d065b, 0x0fdc1bec,
//     0x3793a651, 0x3352bbe6, 0x3e119d3f, 0x3ad08088,
//     0x2497d08d, 0x2056cd3a, 0x2d15ebe3, 0x29d4f654,
//     0xc5a92679, 0xc1683bce, 0xcc2b1d17, 0xc8ea00a0,
//     0xd6ad50a5, 0xd26c4d12, 0xdf2f6bcb, 0xdbee767c,
//     0xe3a1cbc1, 0xe760d676, 0xea23f0af, 0xeee2ed18,
//     0xf0a5bd1d, 0xf464a0aa, 0xf9278673, 0xfde69bc4,
//     0x89b8fd09, 0x8d79e0be, 0x803ac667, 0x84fbdbd0,
//     0x9abc8bd5, 0x9e7d9662, 0x933eb0bb, 0x97ffad0c,
//     0xafb010b1, 0xab710d06, 0xa6322bdf, 0xa2f33668,
//     0xbcb4666d, 0xb8757bda, 0xb5365d03, 0xb1f740b4
// ];



#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
        path::Path,
    };

    use super::Code;
    #[test]
    fn read() {
        let pwd = std::env::current_dir().expect("expect current dir");
        let binary = Path::new(&pwd.display().to_string())
            .join("..")
            .join("..")
            .join("example/bin/test.hl");
        let f = File::open(binary).expect("Could not read hashlink file");
        let mut reader = BufReader::new(f);
        let mut buf = Vec::new();

        // Read file into vector.
        reader
            .read_to_end(&mut buf)
            .expect("Could not read hashlink binary");

        let code = Code::read(&buf);

        assert!(code.is_ok(), "Code is not okay!")
    }
}
