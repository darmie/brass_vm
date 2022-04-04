use comet::{gc_base::GcBase, api::Gc};
use comet_extra::alloc::{array, string, vector::Vector};
use std::any::{Any, TypeId};

use crate::module::ModuleContext;

use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

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
    // ---------
    HLAST = 22,
    HForceInt = 0x7FFFFFFF,
}


pub struct VBytes<T: GcBase + 'static> {
    vec:Vector<u8, T>
}

pub struct VArray<T: GcBase + 'static> {
    t:ValueType<T>,
    at:ValueType<T>,
    size:usize,
    __pad:i32 // force align on 16 bytes for double
}

pub struct Colsure<T: GcBase + 'static> {
    args:Vec<&mut ValueType<T>>,
    ret:&mut ValueType<T>,
    parent: Option<&ValueType<T>>
}
pub struct ClosureType<T: GcBase + 'static> {p:Gc<&mut dyn GcBase, T>,kind:TypeKind}
pub struct FuncType<T: GcBase + 'static> {
    pub args: Vector<&mut ValueType<T>, T>,
    pub ret: &mut ValueType<T>,
    pub parent: Option<&ValueType<T>>,
    pub closure_type:Option<ClosureType>,

}

pub struct ObjType<T:GcBase + 'static> {
    pub super_type: Option<&mut ValueType<T>>,
    pub name: &string::Str,
    pub fields: Vector<ObjField<T>, T>,
    pub proto: Vector<ObjProto<T>, T>,
    pub bindings: Vector<i32, T>,
    pub global_value: Gc<&dyn Any, T>,
    pub rt: Option<&RuntimeObj<T>>,
    pub module: Option<&mut ModuleContext>,
}

pub struct EnumType<T: GcBase + 'static>{
    pub name: &string::Str,
    pub constructs: Vector<EnumConstruct<T>, T>,
    pub global_value: Gc<&dyn GcBase, T>,
}

pub struct VirtualType<T: GcBase + 'static> {
    pub fields: Vector<ObjField<T>, T>,
    data_size: Option<usize>,
    indexes: Option<Vector<u32>>,
    lookup: Option<Vector<FieldLookup<T>>>,
}

pub union TypeUnion<T: GcBase + 'static> {
    pub func:Gc<FuncType<T>, T>,
    pub obj:Gc<ObjType<T>, T>,
    pub enum_type: Gc<EnumType<T>, T>,
    pub virtual: Gc<VirtualType<T>, T>,
    pub type_params: Gc<VirtualType<T>, T>,
    pub abs_name:&string::Str,
}
pub struct ValueType<T: GcBase + 'static> {
    pub kind:TypeKind,
    pub u:TypeUnion<T>,
}

pub struct ObjField<T: GcBase + 'static> {
    name: &string::Str,
    valtype: ValueType<T>,
    hashed_name: u32,
}

pub struct FieldLookup<T: GcBase + 'static> {
    t: &T,
    hashed_name: i32,
    field_index: i32,
}

pub struct ObjProto<T: GcBase + 'static> {
    name: &string::Str,
    hashed_name: u32,
    pindex: i32,
    findex: i32,
}

pub struct RuntimeObj<T: ValueType<H> + 'static> {
    t: &T,
    nfields: usize,
    nproto: usize,
    size: usize,
    nmethods: usize,
    hasptr: bool,
    methods: Vector<&dyn Any, T>,
    fields_indexes: Vector<i32, T>,
    parent: Option<RuntimeObj<T>>,
    bindings:Vec<RuntimeBinding<T>>,
    to_string_fun: fn(obj:&mut VDynamic<T>) -> string::Str,
    compare_fun:fn(a:&mut VDynamic<T>, b:&mut VDynamic<T>) -> i32,
    cast_fun: fn(obj: &mut VDynamic<T>, t:ValueType<T>) -> VDynamic<T>,
    getFieldFun: fn(obj: &mut VDynamic<T>, hfield:i32) -> VDynamic<T>,

    lookup: Vector<FieldLookup<T>>,
    interfaces: Vector<i32, T>
}

pub struct RuntimeBinding<T: ValueType<H> + 'static> {
    ptr:Gc<&mut dyn Any, T>,
    closure:&mut ValueType<T>,
    fid:i32
}

pub struct EnumConstruct<T: GcBase + 'static> {
    name: &string::Str,
    params: Vector<ValueType<T>>,
    size: Option<usize>,
    hasptr: Option<bool>,
    offsets: Vector<i32, T>,
}




pub union VDynamicUnion<T: GcBase + 'static> {
    b:bool,
    ui8:u8,
    ui16:u16,
    i:i32,
    f:f32,
    d:f64,
    bytes:VBytes<T>,
    ptr:Gc<&mut dyn Any,T>,
    i64:i64
}

pub struct VObj<T: GcBase + 'static> {
    t:ValueType<T>,
}

pub struct VDynamic<T: GcBase + 'static> {
    t:ValueType<T>,
    v:VDynamicUnion<T>
}


pub struct VVirtual<T: GcBase + 'static> {
    t:ValueType<T>,
    value:VDynamic<T>,
    next:Option<VVirtual<T>>
}

pub struct VEnum<T: GcBase + 'static> {
    t:ValueType<T>,
    index:i32
}

pub struct VDynObj<T: GcBase + 'static> {
    t:ValueType<T>,
    lookup: Vector<FieldLookup<T>>,
    raw_data:Vector<u8, T>,
    values:Vector<&mut dyn Any>,
    virtuals:Vector<VVirtual<T>>
}


pub struct VClosure<T: GcBase + 'static>{
    t:ValueType<T>,
    func: &mut dyn Any,
    hasValue:i32,
    stackCount:i32,
    value: &mut dyn Any
}

pub struct VClosureWrapper<T: GcBase + 'static>{
    cl:VClosure<T>,
    wrappedFun:&mut VClosure<T>
}

enum DynOp {
	OpAdd = 0,
	OpSub = 1,
	OpMul = 2,
	OpMod = 3,
	OpDiv = 4,
	OpShl = 5,
	OpShr = 6,
	OpUShr = 7,
	OpAnd = 8,
	OpOr = 9,
	OpXor = 10,
	OpLast = 11
}