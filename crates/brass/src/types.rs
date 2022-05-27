use std::any::Any;


use crc32fast::Hasher as Crc32Hasher;
use std::hash::Hasher;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use strum_macros::Display;

use crate::op::Opcode;
use strum_macros::IntoStaticStr;

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

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, IntoStaticStr)]
#[derive(PartialEq)]
#[repr(u8)]
pub enum TypeKind {
    HVOID = 0,
    HUI8 = 1,
    HUI16 = 2,
    HI32 = 3,
    HI64 = 4,
    HF32 = 5,
    HF64 = 6,
    HBOOL = 7,
    HBYTES = 8,
    HDYN = 9,
    HFUN = 10,
    HOBJ = 11,
    HARRAY = 12,
    HTYPE = 13,
    HREF = 14,
    HVIRTUAL = 15,
    HDYNOBJ = 16,
    HABSTRACT = 17,
    HENUM = 18,
    HNULL = 19,
    HMETHOD = 20,
    HSTRUCT = 21,
    HPACKED = 22,
    // ---------
    HLAST = 23,
    // HForceInt = 0x7FFFFFFF,
}

#[derive(PartialEq)]
#[derive(Debug, Clone, Display)]
pub enum ValueTypeU {
    FuncType {
        args: Vec<ValueType>,
        nargs: usize,
        ret: Box<ValueType>,
    },
    ObjType {
        name: String,
        super_type: Box<ValueType>,
        fields: Vec<ObjField>,
        nfields: usize,
        nproto: usize,
        nbindings: usize,
        proto: Vec<ObjProto>,
        bindings: Vec<u32>,
        global_value: Vec<isize>,
        rt: Option<RuntimeObj>,
    },
    VirtualType {
        nfields: usize,
        fields: Vec<ObjField>,
    },
    EnumType {
        name: String,
        nconstructs: usize,
        constructs: Vec<EnumConstruct>,
        global_value: Vec<isize>,
    },
    Ref(Box<ValueType>),
    Abstract(String),
    Null,
    Void,
}

#[derive(PartialEq)]
#[derive(Debug, Clone)]
pub struct ValueType {
    pub union: ValueTypeU,
    pub abs_name: Option<String>,
    pub tparam: Option<Box<ValueType>>,
    pub kind: TypeKind,
}

impl ValueType {
    pub fn hash(self, isrec: bool) -> u32 {
        let mut hasher = Crc32Hasher::new();
        let kind = self.kind;
        match kind {
            TypeKind::HFUN | TypeKind::HMETHOD => {
                if let ValueTypeU::FuncType { args, nargs, ret } = self.union {
                    hasher.write_usize(nargs);
                    for i in 0..nargs {
                        if !isrec {
                            let a = args.get(i).unwrap().clone();
                            hasher.write_u32(a.hash(true));
                        }
                    }
                    if !isrec {
                        let r = ret.as_ref().clone();
                        hasher.write_u32(r.hash(true));
                    }
                }
            }
            TypeKind::HOBJ | TypeKind::HSTRUCT => {
                if let ValueTypeU::ObjType {
                    name,
                    super_type: _,
                    fields,
                    nfields,
                    nproto,
                    nbindings: _,
                    proto: _,
                    bindings: _,
                    global_value: _,
                    rt: _,
                } = self.union
                {
                    hasher.write(name.as_bytes());
                    hasher.write_usize(nfields);
                    hasher.write_usize(nproto);

                    for i in 0..nfields {
                        hasher.write_u32(fields.get(i).unwrap().clone().hashed_name);
                        if !isrec {
                            hasher.write_u32(fields.get(i).unwrap().clone().t.hash(true));
                        }
                    }
                }
            }
            TypeKind::HREF | TypeKind::HNULL => {
                if !isrec {
                    hasher.write_u32(self.tparam.unwrap().hash(true));
                }
            }
            TypeKind::HVIRTUAL => {
                if let ValueTypeU::VirtualType { nfields, fields } = self.union {
                    hasher.write_usize(nfields);
                    for i in 0..nfields {
                        hasher.write_u32(fields.get(i).unwrap().clone().hashed_name);
                        if !isrec {
                            hasher.write_u32(fields.get(i).unwrap().clone().t.hash(true));
                        }
                    }
                }
            }
            TypeKind::HENUM => {
                if let ValueTypeU::EnumType {
                    name,
                    nconstructs,
                    constructs,
                    global_value: _,
                } = self.union
                {
                    hasher.write(name.as_bytes());
                    for i in 0..nconstructs {
                        let con = constructs.get(i).unwrap().clone();
                        hasher.write_usize(con.nparams);
                        hasher.write(con.name.as_bytes());
                        for k in 0..con.nparams {
                            let p = con.params.get(k).unwrap().clone();
                            if !isrec {
                                hasher.write_u32(p.hash(true));
                            }
                        }
                    }
                }
            }
            TypeKind::HABSTRACT => {
                hasher.write(self.abs_name.unwrap().as_bytes());
            }
            _ => {}
        }

        hasher.finalize()
    }
}

impl ValueType {
    pub fn default() -> Self {
        ValueType {
            union: ValueTypeU::Null,
            abs_name: None,
            tparam: None,
            kind: TypeKind::HNULL,
        }
    }
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct ObjField {
    pub name: String,
    pub hashed_name: u32,
    pub t: ValueType,
}
#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct ObjProto {
    pub name: String,
    pub hashed_name: u32,
    pub findex: usize,
    pub pindex: i32,
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct EnumConstruct {
    pub name: String,
    pub nparams: usize,
    pub params: Vec<ValueType>,
    pub size: usize,
    pub hasptr: bool,
    pub offsets: Vec<i32>,
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct RuntimeObj {}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct HLFunction {
    pub t: ValueType,
    pub findex: usize,
    pub nregs: usize,
    pub nops: usize,
    pub rf:u32,
    pub regs: Vec<ValueType>,
    pub ops: Vec<Opcode>,
    pub debug: Vec<i32>,
    pub obj:Option<ValueTypeU>,
    pub field: Option<FuncField>,
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub struct FuncField {
    pub name:String,
    pub rf: Option<Box<HLFunction>>,
}

#[derive(Clone, Debug)]
pub struct Constant {
    pub global: u32,
    pub nfields: usize,
    pub fields: Vec<u32>,
}
