use std::any::Any;

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
        global_value: Vec<*mut dyn Any>,
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
        global_value: Vec<*mut dyn Any>,
    },
    Ref(Box<ValueType>),
    Abstract(String),
    Null,
    Void,
}

#[derive(Debug, Clone)]
pub struct ValueType {
    pub union: ValueTypeU,
    pub abs_name: Option<String>,
    pub tparam: Option<Box<ValueType>>,
    pub kind: TypeKind,
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
pub struct ObjField {
    pub name: String,
    pub hashed_name: i32,
    pub t: ValueType,
}
#[derive(Clone, Debug)]
pub struct ObjProto {
    pub name: String,
    pub hashed_name: i32,
    pub findex: usize,
    pub pindex: i32,
}

#[derive(Clone, Debug)]
pub struct EnumConstruct {
    pub name: String,
    pub nparams: usize,
    pub params: Vec<ValueType>,
    pub size: usize,
    pub hasptr: bool,
    pub offsets: Vec<i32>,
}

#[derive(Clone, Debug)]
pub struct RuntimeObj {}

#[derive(Clone, Debug)]
pub struct HLFunction {
    pub t: ValueType,
    pub findex: usize,
    pub nregs: usize,
    pub nops: usize,
    pub regs: Vec<ValueType>,
    pub ops: Vec<Opcode>,
    pub debug: Vec<i32>,
}

#[derive(Clone, Debug)]
pub struct Constant {
    pub global: u32,
    pub nfields: usize,
    pub fields: Vec<u32>,
}
