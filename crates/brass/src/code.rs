

use std::num::Wrapping;

use crate::decoder::{Decode, Decoder};
use crate::errors::{DecodeError, DecodeErrorKind};
use crate::native::Native;
use crate::op::{Op, Opcode, OP_NARGS};
use crate::types::{
    Constant, EnumConstruct, EnumType, FuncType, HLFunction, ObjField, ObjProto, ObjType, TypeKind,
    ValueType, ValueTypeU, VirtualType,
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

        // println!("{:?}", t.kind);

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
            TypeKind::HOBJ | TypeKind::HSTRUCT => {
                let name = Code::read_ustring(decoder)?;
                let super_index = INDEX(decoder)?;
                let mut obj = ObjType {
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
                    global_value: vec![&mut (UINDEX(decoder)?)],
                    nfields: UINDEX(decoder)?,
                    nproto: UINDEX(decoder)?,
                    nbindings: UINDEX(decoder)?,
                    fields: Vec::new(),
                    proto: Vec::new(),
                    bindings: Vec::new(),
                    rt: None,
                };

                for i in 0..obj.nfields {
                    let name = Code::read_ustring(decoder)?;
                    let field = ObjField {
                        name,
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
                t.tparam = Some(Box::new(Code::get_type(decoder)?));
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
                    global_value: vec![&mut (UINDEX(decoder)?)], // Todo
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
            // println!("size {:?}", sz);
            let s = String::from_utf8_lossy(&sdata[..sz]).to_string();
            // println!("string {}", s);
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

        let op = Op::try_from(n).unwrap();

        let i = OP_NARGS.into_iter().nth(n.try_into().unwrap()).unwrap();
        match i {
            0 => {}
            1 => {
                res.p1 = INDEX(decoder)?;
            }
            2 => {
                res.p1 = INDEX(decoder)?;
                res.p2 = INDEX(decoder)?;
            }
            3 => {
                res.p1 = INDEX(decoder)?;
                res.p2 = INDEX(decoder)?;
                res.p3 = INDEX(decoder)?;
            }
            4 => {
                res.p1 = INDEX(decoder)?;
                res.p2 = INDEX(decoder)?;
                res.p3 = INDEX(decoder)?;
                let i: isize = INDEX(decoder)?.try_into().unwrap(); // not sure if this is necessary
                res.extra.insert(0, i.try_into().unwrap());
            }
            -1 => match op {
                Op::OCallN | Op::OCallClosure | Op::OCallMethod | Op::OCallThis | Op::OMakeEnum => {
                    res.p1 = INDEX(decoder)?;
                    res.p2 = INDEX(decoder)?;
                    res.p3 = u8::decode(decoder)?.into();
                    let extra = vec![0; res.p3.try_into().unwrap()];
                    res.extra = extra;
                   
                    for i in 0..res.p3 {
                        res.extra.insert(i.try_into().unwrap(), INDEX(decoder)?);
                    }
                }
                Op::OSwitch => {
                    res.p1 = UINDEX(decoder)?.try_into().unwrap();
                    res.p2 = UINDEX(decoder)?.try_into().unwrap();

                    res.extra = vec![0; res.p2.try_into().unwrap()];

                    for i in 0..res.p2 {
                        res.extra.insert(i.try_into().unwrap(), INDEX(decoder)?);
                    }
                    res.p3 = UINDEX(decoder)?.try_into().unwrap();
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
                res.p1 = INDEX(decoder)?;
                res.p2 = INDEX(decoder)?;
                res.p3 = INDEX(decoder)?;

                res.extra = vec![0; size.try_into().unwrap()];

                for i in 0..size {
                    res.extra.insert(i.try_into().unwrap(), INDEX(decoder)?);
                }
            }
        }

        Ok(res)
    }
    
    pub fn debu_infos(decoder: &mut Decoder, nops: usize) -> Result<Vec<i32>, DecodeError> {
        let mut curfile: i8 = -1;
        let mut curline: usize = 0;

        let mut debug: Vec<i32> = vec![0; 4 * nops * 2];

        let mut i:usize = 0;

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
                        
                        if count >= 0 {break;}
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
                    let b3:u32 = u8::decode(decoder)?.into();
                    curline = ((c >> 3) as u32 | (b2 << 5) as u32 | (b3 << 13)) as usize;
                    debug[(i << 1) as usize] = curfile as i32;
                    debug[((i << 1) | 1) as usize]  = curline as i32;
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

        // println!("{:?}", c.strings);
        // println!("{:?}", decoder.buf.len());

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
        for i in 0..c.nnatives {
            decoder.code = c.clone();
            c.natives.push(Native {
                lib: Code::read_string(&mut decoder)?,
                name: Code::read_string(&mut decoder)?,
                t: Code::get_type(&mut decoder)?,
                findex: UINDEX(&mut decoder)?,
            });

            c = decoder.code.clone();
        }

        for i in 0..c.nfunctions {
            decoder.code = c.clone();
            let f = Code::read_function(&mut decoder)?;
            decoder.code.functions.push(f);
            if decoder.code.hasdebug == 1 {
                let nops = decoder.code.functions[i].nops;
                decoder.code.functions[i].debug = Code::debu_infos(&mut decoder, nops)?;
                if decoder.code.version >= 3 {
                    // skip assigns (no need here)
                    let nassigns = UINDEX(&mut decoder)?;
                    for j in 0..nassigns {
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
