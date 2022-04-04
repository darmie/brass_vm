use std::{
    borrow::Borrow,
    fs::File,
    io::{BufRead, BufReader, Error, Read},
    slice::SliceIndex,
};

use comet::gc_base::GcBase;
use comet_extra::alloc::{string::{self, Str}, vector::Vector};

use crate::{module::{Code, Function, OpCode, Native, Constant}, globals::GLOBAL_TABLE, valuetypes::{ValueType, TypeKind, FuncType, ObjType, ObjField, ObjProto, VirtualType, EnumType, EnumConstruct}, op::{Op, OP_NARGS}};

use num_enum::{TryFromPrimitive, IntoPrimitive};
extern crate strum;
#[macro_use]
extern crate strum_macros;
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


static HL_VERSION:i32 = 0x010C00;

pub struct CodeReader<T: GcBase + 'static> {
    b: &mut Vec<u8>,
    pos: &mut i32,
    code: &mut Code<T>,
    err: &mut str,
}

impl<T: GcBase + 'static> CodeReader<T> {
    pub fn new(path: &str) -> Self {
        let f = File::open(path);
        let mut reader = BufReader::new(f.unwrap());

        let mut buffer = Vec::new();
        // Read file into vector.
        reader.read_to_end(&mut buffer)?;
        Self {
            b: &mut buffer,
            pos: &mut 0,
            code: {},
            err: {},
        }
    }

    pub fn read_b(&self) -> u8 {
        if (self.pos >= self.b.len()) {
            self.err = "No more data";
            return 0;
        }

        self.pos += 1;
        self.b[self.pos]
    }

    pub fn read_bytes(&self, size: usize) -> Vector<u8> {
        if (size < 0) {
            self.err = "No more data";
            //Todo: debug break!
            return Vector::new(self.code.alloc);
        }

        if (self.pos + size > self.size) {
            self.err = "No more data";
            //Todo: debug break!
            return Vec::new();
        }

        let ret = Vector::new(self.code.alloc);
        ret.extend_from_slice(self.code.alloc, &mut self.b[self.pos..size]);
        self.pos += size;

        ret
    }

    pub fn read_double(&self) -> f64 {
        if (self.pos + 8 > self.b.len()) {
            self.err = "No more data";
            return 0.0;
        }
        let mut d = f64::from_le_bytes(self.b[self.pos..8]);
        self.pos += 8;
        d
    }

    pub fn read_i32(&self) -> i32 {
        if (self.pos + 4 > self.b.len()) {
            self.err = ("No more data");
            //Todo: debug break!
            return 0;
        }
        let mut i = i32::from_le_bytes(self.b[self.pos..4]);
        self.pos += 4;

        i
    }

    pub fn read_index(&self) -> i32 {
        let mut b = self.read_b();

        if ((b & 0x80) == 0) {
            return b & 0x7F;
        }

        if ((b & 0x40) == 0) {
            let mut v = self.read_b() | ((b & 31) << 8);
            return if (b & 0x20 == 0) { v } else { -v };
        }

        {
            let mut c = self.read_b();

            let mut d = self.read_b();

            let mut e = self.read_b();

            let v = ((b & 31) << 24) | (c << 16) | (d << 8) | e;
            if (b & 0x20 == 0) {
                v
            } else {
                -v
            }
        }
    }

    pub fn read_uindex(&self) -> u32 {
        let i = self.read_index();

        if (i < 0) {
            self.err = "Negative index";
            //Todo: debug break!
            return 0;
        }

        i
    }

    pub fn get_type(&self) -> ValueType<T> {
        let mut i = self.read_index();
       
        let code = self.code;

        if (i < 0 || i >= code.types.len()) {
            self.err ="Invalid type index";
            //Todo: debug break!
            i = 0;
        }

        code.types.get(i).unwrap()
    }

    pub fn read_string(&self) -> &str {
        let i = self.read_index();
        
        if (i < 0 || i >= self.code.strings.len()) {
            self.err = "Invalid string index";
            //Todo: debug break!
            return "";
        }

        self.code.strings.get(i).unwrap()
    }

    pub fn get_ustring(&self, index: i32) -> &string::Str {
        let mut ustrings = self.code.ustrings;
        let _str: &string::Str = ustrings.get(index).unwrap();
        if _str.len() <= 0 {
            let other = self.code.strings.get(index).unwrap();
            let mut sstr = string::Str::new(self.code.alloc, other);
            self.code.ustrings[index] = sstr;
        }

        _str
    }

    pub fn read_ustring(&self) -> &string::Str {
        let mut i = self.read_index();
       
        if (i < 0 || i >= self.code.strings.len()) {
            // Todo: store error in debug
            // Err("Invalid string index");
            i = 0;
        }

        self.get_ustring(i)
    }

    pub fn read_type(&self, t:&mut ValueType<T>){
        t.kind = TypeKind::try_from(self.read_b());
        match! t.kind {
            TypeKind::HFUN | TypeKind::HMETHOD  => {
                let nargs:i32 = self.read_b();
                let mut fun = FuncType{ 
                    args: Vector::new(self.code.alloc), 
                    ret: {}, 
                    parent: None, 
                    closure_type: None 
                };
                for i in 0..nargs {
                    fun.args[i] = &mut self.get_type();
                }
                fun.ret = &mut self.get_type();
                t.u.func = self.code.alloc.allocate(fun, comet::gc_base::AllocationSpace::New);
            },
            TypeKind::HOBJ | TypeKind::HSTRUCT => {
                let name = self.read_ustring();
                let super_index = self.read_index();
                let global_index = self.read_uindex();
                let nfields = self.read_uindex();
                let nproto = self.read_uindex();
                let nbindings = self.read_uindex();
                let mut obj = ObjType{
                    name,
                    super_type: Some(self.code.types[super_index]),
                    global_value: unsafe { GLOBAL_TABLE[global_index] },
                    fields: Vector::new(self.code.alloc),
                    proto: Vector::new(self.code.alloc),
                    bindings: Vector::new(self.code.alloc),
                    rt: None,
                    module: None,
                };
                for i in 0..nfields{
                    let mut f:ObjField<T> = t.u.obj.fields[i];
                    f.name = self.read_ustring();
                    f.hashed_name = -1; // Todo: compute hash
                    f.valtype = self.get_type();
                }
                for i in 0..nproto{
                    let mut f:ObjProto<T> = t.u.obj.proto[i];
                    f.name = self.read_ustring();
                    f.hashed_name = -1; // Todo: compute hash
                    f.findex = self.read_uindex();
                    f.pindex = self.read_index();
                }
                for i in 0..nbindings {
                    t.u.obj.bindings[i<<1] = self.read_uindex();
                    t.u.obj.bindings[i<<1|1] = self.read_uindex();
                }
                t.u.obj = self.code.alloc.allocate(obj, comet::gc_base::AllocationSpace::New);
            },
            TypeKind::HREF => {
                t.u.type_params = self.get_type();
            },
            TypeKind::HVIRTUAL => {
                let nfields = self.read_uindex();
                let mut virt = VirtualType{
                    fields: Vector::new(self.code.alloc),
                    data_size: None,
                    indexes: None,
                    lookup: None,
                };
                for i in 0..nfields{
                    let mut f:ObjField<T> = t.u.virtual.fields[i];
                    f.name = self.read_ustring();
                    f.hashed_name = -1; // Todo: compute hash
                    f.valtype = self.get_type();
                }
                t.u.virtual = self.code.alloc.allocate(virt, comet::gc_base::AllocationSpace::New);
            },
            TypeKind::HABSTRACT => {
                t.u.abs_name = self.read_ustring();
            },
            TypeKind::HENUM => {
                let name = self.read_ustring();
                let global_index = self.read_uindex();
                let nconstructs = self.read_uindex();
                let mut _enum = EnumType{
                    name,
                    global_value: unsafe { GLOBAL_TABLE[global_index] },
                    constructs: Vector::new(self.code.alloc)
                };
                
                for i in 0..nconstructs {
                    let name = self.read_ustring();
                    let nparams = self.read_uindex();
                    let mut c:EnumConstruct<T> = EnumConstruct{
                        name,
                        params: Vector::new(self.code.alloc),
                        offsets: Vector::new(self.code.alloc),
                        size: None,
                        hasptr: None
                    };
                    for j in 0..nparams {
                        c.params[j] = self.get_type();
                    }

                    _enum.constructs[i] = c;
                }

                t.u.enum_type = self.code.alloc.allocate(_enum, comet::gc_base::AllocationSpace::New);
            },
            TypeKind::HNULL => {
                t.u.type_params = self.get_type();
            },
            default =>{
                if t.kind >= TypeKind::HLAST {
                    self.err = "Invalid type";
                    // Todo: debug break;
                }
            }
        }
    }

    pub fn read_opcode(&self, f: &mut Function<T>, o: &mut OpCode){
        o.op = Op::try_from(self.read_b()).unwrap();

        if(o.op >= Op::OLast){
            self.err = "Invalid opcode";
            // Todo: debug break;
            return;
        }

        let nargs:i8 = OP_NARGS[o.op as i8];
        match! nargs {
            0 => {},
            1 => {
                o.p1 = self.read_index();
            },
            2 => {
                o.p1 = self.read_index();
		        o.p2 = self.read_index();
            },
            3 => {
                o.p1 = self.read_index();
		        o.p2 = self.read_index();
                o.p3 = self.read_index();
            },
            4 => {
                o.p1 = self.read_index();
		        o.p2 = self.read_index();
                o.p3 = self.read_index();
                o.extra = Vector::new(self.code.alloc);
                o.extra[i] = self.read_index();
            }
            -1 => {
                match! o.op {
                    Op::OCallN | 
                    Op::OCallClosure |
                    Op::OCallMethod |
                    Op::OCallThis |
                    Op::OMakeEnum => {
                        o.p1 = self.read_index();
                        o.p2 = self.read_index();
                        o.p3 = self.read();
                        extra = Vector::new(self.code.alloc);
                        
                        for i in 0..o.p3 {
                            extra[i] = self.read_index();
                        }
                        o.extra = extra;
                    },
                    Op::OSwitch => {
                        o.p1 = self.read_uindex();
                        o.p2 = self.read_uindex();
                        extra = Vector::new(self.code.alloc);
                        
                        for i in 0..o.p2 {
                            extra[i] = self.read_uindex();
                        }
                        o.extra = extra;
                        o.p3 = self.read_uindex();
                    }
                    default =>{
                        self.err = "Don't know how to process opcode";
                        // Todo: debug break;
                    }
                }
            },
            default => {
                let size:i8 = OP_NARGS[o.op as i8] - 3;
                o.p1 = self.read_index();
		        o.p2 = self.read_index();
                o.p3 = self.read_index();
                extra = Vector::new(self.code.alloc);

                for i in 0..size {
                    extra[i] = self.read_index();
                }
                o.extra = extra;
            }
        }
    }

    pub fn read_function(&self) -> &mut Function<T> {
        let mut f:Function<T> = Function{
            Type: &mut self.get_type(),
            findex: self.read_uindex(),
            regs: Vector::new(self.code.falloc),
            debug: Box::new_zeroed(),
            obj: None,
            field: None,
            ops: Vector::new(self.code.falloc)
        };
        let nregs = self.read_uindex();
        let nops = self.read_uindex();

        for i in 0..nregs {
            f.regs[i] = self.get_type();
        }

        if self.err.len() > 0 {
            return &mut f;
        }

        for i in 0..nops {
            self.read_opcode(&mut f, f.ops[i]);
        }

        &mut f
    }

    pub fn op_name(&self, op:Op) -> &'static str {
        if op < 0 || op >= Op::OLast  {
            return "UnknownOp";
        }
        
        let ret:&'static str = op.into();

        ret
    }

    pub fn read_strings(&self, nstrings:i32)-> Vector<&'static Str> {
        let mut size = self.read_i32();
        let mut c = self.code;

        let s = self.read_bytes(size);
        let mut strings:Vector<&Str> = Vector::new(c.alloc);
        let mut count = 0;
        for i in 0..nstrings {
            let sz = self.read_uindex();
            strings[i] = String::from_utf8(s[count..sz]).unwrap().as_str();
            count += sz;
        }

        strings
    }


    pub fn check_error(&self) -> bool {
        if self.err.len() > 0 {
            // todo: free allocated blocks
        }
    }

    pub fn exit(&self, msg: &'static str) {}

    pub fn read_debug_infos(&self){}
}

pub fn code_read(path: &'static str, error_msg:&'static mut str) -> Result<&mut Code> {
    let max_version = 5;
    let mut reader = CodeReader::new(path);
    if reader.read_b() != 'H' || reader.read_b() != 'L' || reader.read_b() != 'B' {
        reader.exit("Invalid HL bytecode header");
        return;
    }

   

    reader.code = &mut Code::default();

    let mut c = reader.code;

    c.version = reader.read_b();

    if c.version <= 1 || c.version > max_version  {
        print!("Found version {} while HL {}.{} supports up to {}\n", c.version, HL_VERSION>>16, (HL_VERSION>>8)&0xFF, max_version);
        reader.exit("Unsupported bytecode version");
        return Err("Unsupported bytecode version");
    }

    let mut r = reader;

    let flags = r.read_uindex();
    let nints = r.read_uindex();
    let nfloats = r.read_uindex();
    let nstrings = r.read_uindex();

    let mut nbytes = 0;

    if c.version >= 5 {
        nbytes = r.read_uindex();
    }
    let ntypes  = r.read_uindex();
    let nglobals = r.read_uindex();
    let nnatives = r.read_uindex();
    let nfunctions = r.read_uindex();
    let nconstants = if c.version >= 4 {r.read_uindex()} else {0};

    c.entrypoint = r.read_uindex();

    c.hasdebug = flags & 1;

    if r.check_error() {
        return Err(r.err);
    }

    for i in 0..nints {
        c.ints[i] = r.read_i32();
    }
    if r.check_error() {
        return Err(r.err);
    }
    for i in 0..nfloats {
        c.floats[i] = r.read_double();
    }
    if r.check_error() {
        return Err(r.err);
    }

    c.strings = &mut r.read_strings(nstrings);
    if r.check_error() {
        return Err(r.err);
    }

    if c.version >= 5 {
        let size = r.read_i32();
        c.bytes = &mut r.read_bytes(size);
        if r.check_error() {
            return Err(r.err);
        }
        for i in 0..nbytes {
            c.bytes_pos[i] = r.read_uindex();
        }
        if r.check_error() {
            return Err(r.err);
        }
    }

    if c.hasdebug {
        let ndebugfiles = r.read_uindex();
        c.debug_files = &mut r.read_strings(ndebugfiles);
        if r.check_error() {
            return Err(r.err);
        }
    }

    for i in 0..ntypes {
        let mut t = ValueType{ kind: {}, u: {} };
        r.read_type(&mut t);
        c.types[i] = t;
    }
    if r.check_error() {
        return Err(r.err);
    }

    for i in 0..nnatives {
        let mut n = Native{ lib: {}, name: {}, t: {}, findex: {} };
        n.lib = r.read_string();
        n.name = r.read_string();
        n.t = r.get_type();
        n.findex = r.read_uindex();
        c.natives[i] = n;
    }
    if r.check_error() {
        return Err(r.err);
    }

    for i in 0..nfunctions {
        c.functions[i] = r.read_function();
        if r.check_error() {
            return Err(r.err);
        }
        if c.hasdebug {
            let mut f:Function = c.functions[i];
            f.debug = r.read_debug_infos();
            if c.version >= 3 {
                // skip assigns (no need here)
                let nassigns = r.read_uindex();
                for j in nassigns {
                    r.read_uindex();
                    r.read_index();
                }
            }
        }
    }
    if r.check_error() {
        return Err(r.err);
    }

    for i in nconstants {
        let mut k = Constant{ global: {}, nfields: {}, fields: {} };
        k.global = r.read_uindex();
        k.nfields = r.read_uindex();
        k.fields = Vector::new(r.code.alloc);
        for j in 0..k.nfields {
            k.fields[j] = r.read_uindex();
        }
        if r.check_error() {
            return Err(r.err);
        }
    }

    Ok(c)
}


static CRC32_TABLE:[u32] = [
    0x00000000, 0x04c11db7, 0x09823b6e, 0x0d4326d9,
    0x130476dc, 0x17c56b6b, 0x1a864db2, 0x1e475005,
    0x2608edb8, 0x22c9f00f, 0x2f8ad6d6, 0x2b4bcb61,
    0x350c9b64, 0x31cd86d3, 0x3c8ea00a, 0x384fbdbd,
    0x4c11db70, 0x48d0c6c7, 0x4593e01e, 0x4152fda9,
    0x5f15adac, 0x5bd4b01b, 0x569796c2, 0x52568b75,
    0x6a1936c8, 0x6ed82b7f, 0x639b0da6, 0x675a1011,
    0x791d4014, 0x7ddc5da3, 0x709f7b7a, 0x745e66cd,
    0x9823b6e0, 0x9ce2ab57, 0x91a18d8e, 0x95609039,
    0x8b27c03c, 0x8fe6dd8b, 0x82a5fb52, 0x8664e6e5,
    0xbe2b5b58, 0xbaea46ef, 0xb7a96036, 0xb3687d81,
    0xad2f2d84, 0xa9ee3033, 0xa4ad16ea, 0xa06c0b5d,
    0xd4326d90, 0xd0f37027, 0xddb056fe, 0xd9714b49,
    0xc7361b4c, 0xc3f706fb, 0xceb42022, 0xca753d95,
    0xf23a8028, 0xf6fb9d9f, 0xfbb8bb46, 0xff79a6f1,
    0xe13ef6f4, 0xe5ffeb43, 0xe8bccd9a, 0xec7dd02d,
    0x34867077, 0x30476dc0, 0x3d044b19, 0x39c556ae,
    0x278206ab, 0x23431b1c, 0x2e003dc5, 0x2ac12072,
    0x128e9dcf, 0x164f8078, 0x1b0ca6a1, 0x1fcdbb16,
    0x018aeb13, 0x054bf6a4, 0x0808d07d, 0x0cc9cdca,
    0x7897ab07, 0x7c56b6b0, 0x71159069, 0x75d48dde,
    0x6b93dddb, 0x6f52c06c, 0x6211e6b5, 0x66d0fb02,
    0x5e9f46bf, 0x5a5e5b08, 0x571d7dd1, 0x53dc6066,
    0x4d9b3063, 0x495a2dd4, 0x44190b0d, 0x40d816ba,
    0xaca5c697, 0xa864db20, 0xa527fdf9, 0xa1e6e04e,
    0xbfa1b04b, 0xbb60adfc, 0xb6238b25, 0xb2e29692,
    0x8aad2b2f, 0x8e6c3698, 0x832f1041, 0x87ee0df6,
    0x99a95df3, 0x9d684044, 0x902b669d, 0x94ea7b2a,
    0xe0b41de7, 0xe4750050, 0xe9362689, 0xedf73b3e,
    0xf3b06b3b, 0xf771768c, 0xfa325055, 0xfef34de2,
    0xc6bcf05f, 0xc27dede8, 0xcf3ecb31, 0xcbffd686,
    0xd5b88683, 0xd1799b34, 0xdc3abded, 0xd8fba05a,
    0x690ce0ee, 0x6dcdfd59, 0x608edb80, 0x644fc637,
    0x7a089632, 0x7ec98b85, 0x738aad5c, 0x774bb0eb,
    0x4f040d56, 0x4bc510e1, 0x46863638, 0x42472b8f,
    0x5c007b8a, 0x58c1663d, 0x558240e4, 0x51435d53,
    0x251d3b9e, 0x21dc2629, 0x2c9f00f0, 0x285e1d47,
    0x36194d42, 0x32d850f5, 0x3f9b762c, 0x3b5a6b9b,
    0x0315d626, 0x07d4cb91, 0x0a97ed48, 0x0e56f0ff,
    0x1011a0fa, 0x14d0bd4d, 0x19939b94, 0x1d528623,
    0xf12f560e, 0xf5ee4bb9, 0xf8ad6d60, 0xfc6c70d7,
    0xe22b20d2, 0xe6ea3d65, 0xeba91bbc, 0xef68060b,
    0xd727bbb6, 0xd3e6a601, 0xdea580d8, 0xda649d6f,
    0xc423cd6a, 0xc0e2d0dd, 0xcda1f604, 0xc960ebb3,
    0xbd3e8d7e, 0xb9ff90c9, 0xb4bcb610, 0xb07daba7,
    0xae3afba2, 0xaafbe615, 0xa7b8c0cc, 0xa379dd7b,
    0x9b3660c6, 0x9ff77d71, 0x92b45ba8, 0x9675461f,
    0x8832161a, 0x8cf30bad, 0x81b02d74, 0x857130c3,
    0x5d8a9099, 0x594b8d2e, 0x5408abf7, 0x50c9b640,
    0x4e8ee645, 0x4a4ffbf2, 0x470cdd2b, 0x43cdc09c,
    0x7b827d21, 0x7f436096, 0x7200464f, 0x76c15bf8,
    0x68860bfd, 0x6c47164a, 0x61043093, 0x65c52d24,
    0x119b4be9, 0x155a565e, 0x18197087, 0x1cd86d30,
    0x029f3d35, 0x065e2082, 0x0b1d065b, 0x0fdc1bec,
    0x3793a651, 0x3352bbe6, 0x3e119d3f, 0x3ad08088,
    0x2497d08d, 0x2056cd3a, 0x2d15ebe3, 0x29d4f654,
    0xc5a92679, 0xc1683bce, 0xcc2b1d17, 0xc8ea00a0,
    0xd6ad50a5, 0xd26c4d12, 0xdf2f6bcb, 0xdbee767c,
    0xe3a1cbc1, 0xe760d676, 0xea23f0af, 0xeee2ed18,
    0xf0a5bd1d, 0xf464a0aa, 0xf9278673, 0xfde69bc4,
    0x89b8fd09, 0x8d79e0be, 0x803ac667, 0x84fbdbd0,
    0x9abc8bd5, 0x9e7d9662, 0x933eb0bb, 0x97ffad0c,
    0xafb010b1, 0xab710d06, 0xa6322bdf, 0xa2f33668,
    0xbcb4666d, 0xb8757bda, 0xb5365d03, 0xb1f740b4
];