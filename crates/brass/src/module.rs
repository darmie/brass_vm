use crate::{valuetypes::{ValueType, ObjType}, op::Op};
use comet::{mutator::MutatorRef, gc_base::GcBase, immix::instantiate_immix, immix::ImmixOptions};
use comet_extra::alloc::{array, string::{self, Str}, vector::Vector};

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


pub struct Native<T:GcBase + 'static> {
    pub lib:&Str,
    pub name:&Str,
    pub t:ValueType<T>,
    pub findex:i32
}

pub struct Constant<T:GcBase + 'static> {
    pub global:i32,
    pub  nfields:usize,
    pub fields:Vector<i32, T>
}

pub struct OpCode {
    pub op:Op,
    pub p1:i32,
    pub p2:i32,
    pub p3:i32,
    pub extra:Vector<&dyn Any, GcBase>
}


pub union FunctionFieldUnion<T:GcBase + 'static> {
    pub name:&string::Str,
    pub Ref: Function<T>,
}

pub struct Function<T:GcBase + 'static>  {
    pub findex:i32,
    pub Type:&mut ValueType<T>, 
    pub regs:Vector<ValueType<T>>,
    pub debug: Box<&mut i32>,
    pub obj: Option<ObjType<T>>,
    pub field:Option<FunctionFieldUnion<T>>,
    pub ops: Vector<OpCode>,
}


pub struct Code<T:GcBase + 'static> {
    pub version:i32,
    pub entrypoint:&mut i32,
    pub ints:&mut Vector<i32, T>,
    pub floats:&mut Vector<f32, T>,
    pub strings:&mut Vector<&string::Str, T>,
    pub bytes: &mut Vector<u8, T>,
    pub bytes_pos:&mut Vector<i32, T>,
    pub debug_files:&mut Vector<&string::Str, T>,
    pub ustrings: Vector<&string::Str, T>,
    pub types:&mut Vector<ValueType<T>, T>,
    pub globals:&mut Vector<ValueType<T>, T>,
    pub natives:&mut Vector<Native<T>, T>,
    pub functions:&mut Vector<Function<T>, T>,
    pub constants: &mut Vector<Constant, T>,
    pub alloc:&mut MutatorRef<T>,
    pub falloc:&mut MutatorRef<T>,
    pub hasdebug: &mut bool,
}

impl<T:GcBase + 'static> Code<T> {
    pub fn default() -> Self {
        let mut alloc = instantiate_immix(
            ImmixOptions::default()
            .with_verbose(1)
            .with_heap_size(1024 * 1024 * 1024)
            .with_max_heap_size(1024 * 1024 * 1024)
            .with_min_heap_size(512 * 1024 * 1024),
        );
        let mut falloc = instantiate_immix(
            ImmixOptions::default()
            .with_verbose(1)
            .with_heap_size(1024 * 1024 * 1024)
            .with_max_heap_size(1024 * 1024 * 1024)
            .with_min_heap_size(512 * 1024 * 1024),
        );
        Self{
            version: 5, // max version
            entrypoint: {},
            ints: Vector::new(alloc),
            floats: Vector::new(alloc),
            strings: Vector::new(alloc),
            bytes: Vector::new(alloc),
            bytes_pos: Vector::new(alloc),
            debug_files: Vector::new(alloc),
            ustrings: Vector::new(alloc),
            types: Vector::new(alloc),
            globals: Vector::new(alloc),
            natives: Vector::new(alloc),
            functions: Vector::new(alloc),
            constants: Vector::new(alloc),
            alloc,
            falloc,
            hasdebug: &mut false
        }
    }
}

pub struct DebugInfos {
    offsets:Vec<&mut dyn Any>,
    start:i32,
    large:bool
}

pub struct CodeHash<T:GcBase + 'static> {
    code:&mut Code<T>,
    types_hashes:Vec<i32>,
    global_signs:Vec<i32>,
    functions_signs:Vec<i32>,
    functions_hashes:Vec<i32>,
    functions_indexes:Vec<i32>
}

pub struct Module<T:GcBase + 'static> {
    code:&mut Code<T>,
    globals_indexes:Vec<i32>,
    globals_data:Vec<u8>,
    functions_ptrs:Vec<Gc<&mut dyn Any, T>>,
    functions_indexes:Vec<i32>,
    hash:&mut CodeHash<T>,
    ctx:&mut ModuleContext
}

pub struct ModuleContext<T:GcBase + 'static>{
    alloc:MutatorRef<T>,
    functions_ptrs:Vec<Gc<&mut dyn Any, T>>,
    functions_types:Vec<&mut ValueType<T>>
}