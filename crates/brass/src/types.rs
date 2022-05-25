use std::any::Any;

use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use crate::op::Op;
use crate::op::Opcode;

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

#[derive(Clone, Copy)]
#[derive(Debug)]
#[derive(IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug)]
#[derive(Clone)]
pub enum ValueTypeU {
    FuncType(FuncType),
    ObjType(ObjType),
    VirtualType(VirtualType),
    EnumType(EnumType),
    Ref(Box<ValueType>),
    Abstract(String),
    Null(()),
    Void(()),
}

#[derive(Clone)]
#[derive(Debug)]
pub struct ValueType {
    pub union: ValueTypeU,
    pub abs_name: Option<String>,
    pub tparam: Option<Box<ValueType>>,
    pub kind:TypeKind,
}

impl ValueType {
    pub fn default() -> Self {
        return ValueType { union: ValueTypeU::Null(()), abs_name: None, tparam: None, kind: TypeKind::HNULL }
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub struct FuncType {
    pub args: Vec<ValueType>,
    pub nargs: usize,
    pub ret: Box<ValueType>,
}



#[derive(Clone)]
#[derive(Debug)]
pub struct ObjType {
    pub name: String,
    pub super_type: Box<ValueType>,
    pub fields: Vec<ObjField>,
    pub nfields:usize,
    pub nproto:usize,
    pub nbindings:usize,
    pub proto: Vec<ObjProto>,
    pub bindings: Vec<u32>,
    pub global_value: Vec<*mut dyn Any>,
    pub rt:Option<RuntimeObj>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct ObjField {
    pub name:String,
    pub hashed_name:i32,
    pub t:ValueType
}
#[derive(Clone)]
#[derive(Debug)]
pub struct ObjProto {
    pub name:String,
    pub hashed_name:i32,
    pub findex:usize,
    pub pindex:i32
}

#[derive(Clone)]
#[derive(Debug)]
pub struct VirtualType {
    pub nfields:usize,
    pub fields:Vec<ObjField>,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct EnumType {
    pub name:String,
    pub nconstructs:usize,
    pub constructs:Vec<EnumConstruct>,
    pub global_value: Vec<*mut dyn Any>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct EnumConstruct {
    pub name:String,
    pub nparams:usize,
    pub params:Vec<ValueType>,
    pub size:usize,
    pub hasptr:bool,
    pub offsets:Vec<i32>
}


#[derive(Clone)]
#[derive(Debug)]
pub struct RuntimeObj {}

#[derive(Clone)]
#[derive(Debug)]
pub struct HLFunction {
    pub t:ValueType,
    pub findex:usize,
    pub nregs:usize,
    pub nops:usize,
    pub regs:Vec<ValueType>,
    pub ops:Vec<Opcode>,
    pub debug:Vec<i32>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Constant {
    pub global:u32,
    pub nfields:usize,
    pub fields:Vec::<u32>,
}