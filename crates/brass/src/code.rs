#![allow(arithmetic_overflow)]


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

static HLVERSION: u32 = 0x010C00;
static mut ERRORS:Option<DecodeError> = None;

fn UINDEX(decoder: &mut Decoder) -> Result<usize, DecodeError> {
    let i = INDEX(decoder).map_err(Code::map_err).unwrap();
    if i < 0 {
        // println!("Negative Index {}", i);
        unsafe {
            ERRORS = Some(DecodeError::with_info(
                DecodeErrorKind::NegativeIndex,
                decoder.file_position,
            ));
        }
        return Ok(0);
    }
    Ok(i.try_into().unwrap())
}

fn INDEX(decoder: &mut Decoder) -> Result<i32, DecodeError> {
    let i = decoder.read_index().unwrap();
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
        let index = INDEX(decoder).map_err(Code::map_err).unwrap();
        if index < 0 || index >= decoder.code.nstrings.try_into().unwrap() {
            unsafe {
                ERRORS = Some(DecodeError::with_info(
                    DecodeErrorKind::InvalidStringIndex,
                    decoder.file_position,
                ));
            }
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
        let index = INDEX(decoder).unwrap();
        if index < 0 || index >= decoder.code.nstrings.try_into().unwrap() {
            unsafe {
                ERRORS = Some(DecodeError::with_info(
                    DecodeErrorKind::InvalidStringIndex,
                    decoder.file_position,
                ));
            }
            Ok(String::new())
        } else {
            let string = decoder.code.get_ustring(index.try_into().unwrap());
            Ok(string)
        }
    }

    pub fn get_type(decoder: &mut crate::decoder::Decoder) -> Result<ValueType, DecodeError> {
        let mut index = INDEX(decoder).unwrap();
        if index < 0 || index >= decoder.code.ntypes.try_into().unwrap() {
            unsafe {
                ERRORS = Some(DecodeError::with_info(
                    DecodeErrorKind::InvalidTypeIndex,
                    decoder.file_position,
                ));
            }
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
        let v = u8::decode(decoder).unwrap();
        
        if v >= u8::try_from(TypeKind::HLAST).unwrap() {
            Code::map_err(DecodeError::with_info(
                DecodeErrorKind::InvalidType,
                decoder.file_position,
            ));
            return Ok(());
        }

        let k = TypeKind::try_from(v).unwrap();

        t.kind = k;

        match t.kind {
            TypeKind::HFUN | TypeKind::HMETHOD => {
                let nargs: usize = u8::decode(decoder).unwrap().try_into().unwrap();
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
                        (*args).insert(i, Code::get_type(decoder).unwrap());
                    }

                    *ret = Box::new(Code::get_type(decoder).unwrap());
                }

                t.union = fun;
            }
            TypeKind::HOBJ | TypeKind::HSTRUCT => {
                let name = Code::read_ustring(decoder).unwrap();
                let super_index = INDEX(decoder).unwrap();
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
                    global_value: vec![(UINDEX(decoder).unwrap()) as isize],
                    nfields: UINDEX(decoder).unwrap(),
                    nproto: UINDEX(decoder).unwrap(),
                    nbindings: UINDEX(decoder).unwrap(),
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
                        let name = Code::read_ustring(decoder).unwrap();
                        let hashed_name = hash(name.as_bytes());
                        let field = ObjField {
                            name,
                            hashed_name,
                            t: Code::get_type(decoder).unwrap(),
                        };

                        (*fields).insert(i, field);
                    }
                    for i in 0..nproto {
                        let name = Code::read_ustring(decoder).unwrap();
                        let hashed_name = hash(name.as_bytes());
                        let obj_proto = ObjProto {
                            name,
                            hashed_name,
                            findex: UINDEX(decoder).unwrap(),
                            pindex: INDEX(decoder).unwrap(),
                        };

                        (*proto).insert(i, obj_proto);
                    }
                    for i in 0..nbindings {
                        (*bindings).insert(i << 1, UINDEX(decoder).unwrap().try_into().unwrap());
                        (*bindings).insert((i << 1) | 1, UINDEX(decoder).unwrap().try_into().unwrap());
                    }
                }

                t.union = obj;
            }
            TypeKind::HREF => {
                t.tparam = Some(Box::new(Code::get_type(decoder).unwrap()));
            }
            TypeKind::HVIRTUAL => {
                let nfields = UINDEX(decoder).unwrap();
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
                        let name = Code::read_ustring(decoder).unwrap();
                        let hashed_name = hash(name.as_bytes());
                        let field = ObjField {
                            name,
                            hashed_name,
                            t: Code::get_type(decoder).unwrap(),
                        };

                        (*fields).insert(i, field);
                    }
                }

                t.union = virt;
            }
            TypeKind::HABSTRACT => {
                t.abs_name = Some(Code::read_ustring(decoder).unwrap());
            }
            TypeKind::HENUM => {
                let mut tenum = ValueTypeU::EnumType {
                    name: Code::read_ustring(decoder).unwrap(),
                    global_value: vec![(UINDEX(decoder).unwrap()) as isize], // Todo
                    nconstructs: UINDEX(decoder).unwrap(),
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
                        let name = Code::read_ustring(decoder).unwrap();
                        let nparams = UINDEX(decoder).unwrap();
                        let mut con = EnumConstruct {
                            name,
                            nparams,
                            params: Vec::new(),
                            offsets: Vec::new(),
                            hasptr: false,
                            size: 0,
                        };
                        for j in 0..nparams {
                            con.params.insert(j, Code::get_type(decoder).unwrap());
                        }

                        (*constructs).insert(i, con);
                    }
                }

                t.union = tenum;
            }
            TypeKind::HNULL | TypeKind::HPACKED => {
                t.tparam = Some(Box::new(Code::get_type(decoder).unwrap()));
            }
            _ => {
               
                if u8::try_from(t.kind) >= u8::try_from(TypeKind::HLAST) {
                    Code::map_err(DecodeError::with_info(
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
        let size = i32::decode(decoder).unwrap();

        let st = String::from_utf8_lossy(&decoder.buf[..size as usize]).to_string();
     
        decoder.advance(size as usize)?;

        let mut strings:Vec<String> = Vec::new();
        let mut cursor = 0;
        for _i in 0..nstrings {
            let sz: usize = UINDEX(decoder).unwrap();
            let s = st[cursor..][0..sz].to_string();
            strings.push(s.clone());
            out_lens.push(s.len());
            cursor += sz;

            if cursor >= st.len() {
                break;
            }

            cursor += 1;
        }

        Ok(strings)
    }

    pub fn read_function(decoder: &mut Decoder) -> Result<HLFunction, DecodeError> {
        let mut f = HLFunction {
            t: Code::get_type(decoder).unwrap(),
            findex: UINDEX(decoder).unwrap(),
            nregs: UINDEX(decoder).unwrap(),
            nops: UINDEX(decoder).unwrap(),
            regs: Vec::new(),
            ops: Vec::new(),
            debug: Vec::new(),
            rf: 0,
            obj: None,
            field: None,
        };

        for i in 0..f.nregs {
            f.regs.insert(i, Code::get_type(decoder).map_err(Code::map_err).unwrap());
        }
        if let Err(e) = Code::check_error() {
            return Ok(f);
        }
        for i in 0..f.nops {
            let op = Code::read_opcode(decoder).unwrap();
            f.ops.insert(i, op);
        }

        Ok(f)
    }

    pub fn read_opcode(decoder: &mut Decoder) -> Result<Opcode, DecodeError> {
        let n = u8::decode(decoder).unwrap();
        let mut res = Opcode::default();
        if n >= 99 {
            unsafe {
                ERRORS = Some(DecodeError::with_info(
                    DecodeErrorKind::InvalidOpcode,
                    decoder.file_position,
                ));
            }
            return Ok(res);
        }

        let op: Op = n.try_into().unwrap();
        res.op = op;
        let i = OP_NARGS.into_iter().nth(n.try_into().unwrap()).unwrap();
        match i {
            0 => {}
            1 => {
                res.p1 = Some(INDEX(decoder).unwrap());
            }
            2 => {
                res.p1 = Some(INDEX(decoder).unwrap());
                res.p2 = Some(INDEX(decoder).unwrap());
            }
            3 => {
                res.p1 = Some(INDEX(decoder).unwrap());
                res.p2 = Some(INDEX(decoder).unwrap());
                res.p3 = Some(INDEX(decoder).unwrap());
            }
            4 => {
                res.p1 = Some(INDEX(decoder).unwrap());
                res.p2 = Some(INDEX(decoder).unwrap());
                res.p3 = Some(INDEX(decoder).unwrap());
                let i: isize = INDEX(decoder).unwrap().try_into().unwrap(); // not sure if this is necessary
                res.extra.insert(0, i);
            }
            -1 => match res.op {
                Op::OCallN | Op::OCallClosure | Op::OCallMethod | Op::OCallThis | Op::OMakeEnum => {
                    res.p1 = Some(INDEX(decoder).unwrap());
                    res.p2 = Some(INDEX(decoder).unwrap());
                    let p3 = u8::decode(decoder).unwrap().into();
                    res.p3 = Some(p3);
                    let extra = vec![0; p3.try_into().unwrap()];
                    res.extra = extra;

                    for i in 0..p3 {
                        res.extra
                            .insert(i.try_into().unwrap(), INDEX(decoder).unwrap().try_into().unwrap());
                    }
                }
                Op::OSwitch => {
                    res.p1 = Some(UINDEX(decoder).unwrap().try_into().unwrap());
                    let p2 = UINDEX(decoder).unwrap().try_into().unwrap();
                    res.p2 = Some(p2);

                    res.extra = vec![0; p2.try_into().unwrap()];

                    for i in 0..p2 {
                        res.extra
                            .insert(i.try_into().unwrap(), UINDEX(decoder).unwrap().try_into().unwrap());
                    }
                    res.p3 = Some(UINDEX(decoder).unwrap().try_into().unwrap());
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
                res.p1 = Some(INDEX(decoder).unwrap());
                res.p2 = Some(INDEX(decoder).unwrap());
                res.p3 = Some(INDEX(decoder).unwrap());

                res.extra = vec![0; size.try_into().unwrap()];

                for i in 0..size {
                    res.extra
                        .insert(i.try_into().unwrap(), INDEX(decoder).unwrap().try_into().unwrap());
                }
            }
        }

        Ok(res)
    }

    pub fn debug_infos(decoder: &mut Decoder, nops: usize) -> Result<Vec<i32>, DecodeError> {
        let mut curfile: i32 = -1;
        let mut curline: i32 = 0;

        let mut debug: Vec<i32> = vec![0; nops * 2];
        // println!("debug_len {}", debug.len());
        let mut i: usize = 0;

        while i < nops {
            let mut c:i32 = u8::decode(decoder).unwrap().into();
            if (c & 1) != 0 {
                c >>= 1;
                curfile = (c << 8)  | u8::decode(decoder).unwrap() as i32;
                if curfile >= decoder.code.ndebugfiles.try_into().unwrap() {
                    unsafe {
                        ERRORS = Some(DecodeError::with_info(DecodeErrorKind::InvalidDebugFile,  decoder.file_position));
                    }
                }
            } else if (c & 2) != 0 {
                let delta: i32 = (c >> 6) as i32;
                let count = ((c >> 2) & 15) as i32;
                if i + count as usize > nops {
                    unsafe {
                        ERRORS = Some(DecodeError::with_info(DecodeErrorKind::OutsideRange,  decoder.file_position));
                    }
                }

                for _j in count..0 {
                    debug[i << 1] = curfile as i32;
                    debug[(i << 1) | 1] = curline as i32;
                    i += 1;
                }
                curline += delta;
            } else if (c & 4) != 0 {
                curline += (c >> 3);
                debug[i << 1] = curfile as i32;
                debug[(i << 1) | 1] = curline as i32;
                i += 1;
            } else {
                let b2 = u8::decode(decoder).unwrap();
                let b3 = u8::decode(decoder).unwrap();
                let a = c >> 3;
                let b = (b2 << 5) as i32;
                let c = b3 as i32;
                curline = a | b | c;
                debug[i << 1] = curfile as i32;
                debug[(i << 1) | 1] = curline as i32;
                i += 1;
            }
        }
        Ok(debug)
    }

    fn map_err(e: DecodeError) {
        unsafe {
            if ERRORS == None {
                ERRORS = Some(e);
            }
        }
    }

    fn check_error() -> Result<(), DecodeError> {
        unsafe {
            if let Some(e) = ERRORS {
                return Err(e);
            }
            Ok(())
        }
    }

    pub fn read(buf: &[u8]) -> Result<Self, DecodeError> {
        let mut decoder = Decoder::new(buf);
        let mut c = Code::new();
        let max_version = 5;

        if u8::decode(&mut decoder).unwrap() as char != 'H'
            || u8::decode(&mut decoder).unwrap() as char != 'L'
            || u8::decode(&mut decoder).unwrap() as char != 'B'
        {
            return Err(DecodeError::with_info(
                DecodeErrorKind::InvalidBytecodeHeader,
                decoder.file_position,
            ));
        }
        c.version = u8::decode(&mut decoder).map_err(Code::map_err).unwrap();
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

        let mut flags = 0;
        if let Ok(_flags) = UINDEX(&mut decoder) {
            flags = _flags;
        }
        if let Ok(nints) = UINDEX(&mut decoder) {
            c.nints = nints;
        }
        if let Ok(nfloats) = UINDEX(&mut decoder) {
            c.nfloats = nfloats;
        }
        if let Ok(nstrings) = UINDEX(&mut decoder) {
            c.nstrings = nstrings;
        }

        if c.version >= 5 {
            if let Ok(nbytes) = UINDEX(&mut decoder) {
                c.nbytes = nbytes;
            }
        }
        if let Ok(ntypes) = UINDEX(&mut decoder) {
            c.ntypes = ntypes;
        }
        if let Ok(nglobals) = UINDEX(&mut decoder) {
            c.nglobals = nglobals;
        }
        if let Ok(nnatives) = UINDEX(&mut decoder) {
            c.nnatives = nnatives;
        }
        if let Ok(nfunctions) = UINDEX(&mut decoder) {
            c.nfunctions = nfunctions;
        }

        c.nconstants = 0;
        if c.version >= 4 {
            if let Ok(nconstants) = UINDEX(&mut decoder) {
                c.nconstants = nconstants;
            }
        }

        if let Ok(_entrypoint) = UINDEX(&mut decoder) {
            c.entrypoint = _entrypoint as u32;
        }

        c.hasdebug = flags as u32 & 1;

        let _ = Code::check_error()?;

        for i in 0..c.nints {
            if let Ok(_int) = i32::decode(&mut decoder).map_err(Code::map_err) {
                c.ints.insert(i, _int)
            }
        }
        let _ = Code::check_error()?;

        for i in 0..c.nfloats {
            if let Ok(float) = f64::decode(&mut decoder).map_err(Code::map_err) {
                c.floats.insert(i, float);
            }
        }
        let _ = Code::check_error()?;

        if let Ok(strings) =
            Code::read_strings(&mut decoder, c.nstrings, &mut c.strings_lens).map_err(Code::map_err)
        {
            c.strings = strings;
        }
        let _ = Code::check_error()?;

        c.ustrings = vec![None; c.nstrings];
        if c.version >= 5 {
            let size = i32::decode(&mut decoder)?;
            c.bytes = vec![0; size as usize];
            decoder.read_bytes(&mut c.bytes).map_err(Code::map_err).unwrap();
            for i in 0..c.nbytes {
                if let Ok(byte_pos) = UINDEX(&mut decoder).map_err(Code::map_err){
                    c.bytes_pos.insert(i, byte_pos);
                }
            }
            let _ = Code::check_error()?;
        }

        if c.hasdebug != 0 {
            if let Ok(ndebugfiles) =  UINDEX(&mut decoder) {
                c.ndebugfiles = ndebugfiles;
            }
            decoder.code = c.clone();
            
            if let Ok(debugfiles) = Code::read_strings(&mut decoder, c.ndebugfiles, &mut c.debugfiles_lens).map_err(Code::map_err) {
                decoder.code.debugfiles = debugfiles;
            }
            c = decoder.code.clone();
            let _ = Code::check_error()?;
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
            if let Ok(t) = Code::get_type(&mut decoder).map_err(Code::map_err) {
                decoder.code.globals.insert(i, t);
            }
            c = decoder.code.clone();
        }

        let _ = Code::check_error()?;

        c.natives = Vec::new();
        for _i in 0..c.nnatives {
            c.natives.push(Native {
                lib: Code::read_string(&mut decoder).map_err(Code::map_err).unwrap(),
                name: Code::read_string(&mut decoder).map_err(Code::map_err).unwrap(),
                t: Code::get_type(&mut decoder).map_err(Code::map_err).unwrap(),
                findex: UINDEX(&mut decoder).map_err(Code::map_err).unwrap(),
            });
            decoder.code = c.clone();
        }
        let _ = Code::check_error()?;

       

        for i in 0..c.nfunctions {
            decoder.code = c.clone();
            let f = Code::read_function(&mut decoder).map_err(Code::map_err).unwrap();
            println!("{:?}", f);
            decoder.code.functions.push(f);
            if decoder.code.hasdebug != 0 {
                let nops = decoder.code.functions[i].nops;
                if let Ok(debug) = Code::debug_infos(&mut decoder, nops).map_err(Code::map_err) {
                    decoder.code.functions[i].debug = debug;
                }
                if decoder.code.version >= 3 {
                    // skip assigns (no need here)
                    let nassigns = UINDEX(&mut decoder).map_err(Code::map_err).unwrap();
                    for _j in 0..nassigns {
                        let _ = UINDEX(&mut decoder);
                        let _ = INDEX(&mut decoder);
                    }
                }
            }
            c = decoder.code.clone();
        }

        let _ = Code::check_error()?;

        for i in 0..c.nconstants {
            let mut k = Constant {
                global: UINDEX(&mut decoder).map_err(Code::map_err).unwrap().try_into().unwrap(),
                nfields: UINDEX(&mut decoder).map_err(Code::map_err).unwrap(),
                fields: Vec::new(),
            };

            for j in 0..k.nfields {
                k.fields
                    .insert(j, UINDEX(&mut decoder).map_err(Code::map_err).unwrap().try_into().unwrap());
            }
            let _ = Code::check_error()?;
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

        assert!(code.is_ok(), "Code is not okay! = {:?}", code.err().unwrap())
    }
}
