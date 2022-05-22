use std::io;



use enum_dispatch::enum_dispatch;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

use crate::code::Code;
use crate::errors::DecodeError;


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

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum TypeKind {
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
    // ---------
    HLAST = 22,
    // HForceInt = 0x7FFFFFFF,
}

#[derive(Clone)]
#[enum_dispatch]
pub enum ValueType {
    FuncType(FuncType),
    ObjType(ObjType),
    VirtualType(VirtualType),
    EnumType(EnumType),
    Ref(Box<ValueType>),
    Abstract(String),
    Null(()),
    Void(())
}

// #[enum_dispatch(ValueType)]
// trait ValueTypeIntoCode<'input>: Sized + 'input {
//     fn read(&self, code: &mut Code, decoder: &mut crate::decoder::Decoder<'input>) -> Result<(), DecodeError>;
// }


#[derive(Clone)]
pub struct FuncType {
    pub args:Vec<ValueType>,
    pub ret: Box<ValueType>
}



#[derive(Clone)]
pub struct ObjType {
    pub name:String,
    pub super_type:Option<Box<ValueType>>,
    pub fields: Vec<ObjField>,
    pub proto: Vec<ObjProto>,
}

#[derive(Clone)]
pub struct ObjField {
    
}
#[derive(Clone)]
pub struct ObjProto {}

#[derive(Clone)]
pub struct VirtualType {}

#[derive(Clone)]
pub struct EnumType {}
