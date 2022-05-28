use crate::{
    code::Code,
    native::Native,
    op::{Op, OP_NARGS},
    types::{HLFunction, TypeKind, ValueType, ValueTypeU},
};
use crc32fast::Hasher as Crc32Hasher;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::hash::Hasher;

#[derive(Clone, Debug)]
pub struct CodeHash {
    pub code: Code,
    pub type_hashes: Vec<u32>,
    pub globals_signs: Vec<i32>,
    pub functions_signs: Vec<u32>,
    pub functions_hashes: Vec<u32>,
    pub functions_indexes: Vec<i32>,
}

impl CodeHash {
    pub fn alloc(mut code: &Code) -> Self {
        let c = &mut code;
        let fi = c.nfunctions + c.nnatives;
        let mut code_hash = CodeHash {
            code: c.clone(),
            type_hashes: vec![0; c.ntypes],
            globals_signs: vec![0; c.nglobals],
            functions_signs: Vec::new(),
            functions_hashes: Vec::new(),
            functions_indexes: vec![0; fi],
        };

        for i in 0..c.nfunctions {
            let functions = c.functions.clone();
            let f = functions[i].clone();
            code_hash.functions_indexes[f.findex] = i as i32; 
        }

        for i in 0..c.nnatives {
            let natives = c.natives.clone();
            let n = natives[i].clone();
            code_hash.functions_indexes[n.findex] = (i + c.nfunctions) as i32;
        }

        for i in 0..c.ntypes {
            let types = c.types.clone();
            code_hash.type_hashes[i] = types[i].clone().hash(false);
        }

        let mut type_hashes = vec![0; c.ntypes]; // use a second buffer for order-indepedent
        for i in 0..c.ntypes {
            type_hashes.insert(i, code_hash.type_hashes[i] ^ code_hash.hash_type_rec(i));
        }
        code_hash.type_hashes.clear();
        code_hash.type_hashes.clone_from(&type_hashes);

        for i in 0..c.nglobals {
            let types = c.globals.clone();
            code_hash.globals_signs[i] = (i | 0x80000000) as i32;
            if let TypeKind::HABSTRACT = types[i].kind {
                code_hash.globals_signs[i] = code_hash.code_hash_type(types[i].clone()) as i32;
                // some global abstracts allocated by compiler
            }
        }

        for i in 0..c.ntypes {
            let types = c.types.clone();
            let t = types[i].clone();
            match t.kind {
                TypeKind::HOBJ | TypeKind::HSTRUCT => {
                    if let ValueTypeU::ObjType {
                        name: _,
                        super_type: _,
                        fields: _,
                        nfields: _,
                        nproto: _,
                        nbindings: _,
                        proto: _,
                        bindings: _,
                        ref global_value,
                        rt: _,
                    } = t.union
                    {
                        code_hash.globals_signs[((*global_value)[0] - 1) as usize] =
                            code_hash.code_hash_type(t.clone()) as i32;
                    }
                }
                TypeKind::HENUM => {
                    if let ValueTypeU::EnumType {
                        name: _,
                        nconstructs: _,
                        constructs: _,
                        ref global_value,
                    } = t.union
                    {
                        code_hash.globals_signs[((*global_value)[0] - 1) as usize] =
                            code_hash.code_hash_type(t.clone()) as i32;
                    }
                }
                _ => {}
            }
        }
        for i in 0..c.nconstants {
            let conn = c.constants[i].clone();
            let tt = &c.globals[conn.global as usize];
            let mut hasher = Crc32Hasher::new();
            for k in 0..conn.nfields {
                let index = conn.fields[k];
                if let ValueTypeU::ObjType {
                    name: _,
                    super_type: _,
                    fields,
                    nfields: _,
                    nproto: _,
                    nbindings: _,
                    proto: _,
                    bindings: _,
                    global_value: _,
                    rt: _,
                } = &tt.union
                {
                    match fields[k].t.kind {
                        TypeKind::HI32 => {
                            hasher.write_i32(c.ints[index as usize]);
                        }
                        TypeKind::HBYTES => {
                            hasher.write(c.strings[index as usize].as_bytes());
                        }
                        _ => {}
                    }
                }
            }
            code_hash.globals_signs[conn.global as usize] = hasher.finalize() as i32;
        }

        // look into boot code to identify globals that are constant enum constructors
        // this is a bit hackish but we need them for remap and there's no metatada
        let f = &c.functions[code_hash.functions_indexes[c.entrypoint as usize] as usize];
        for i in 4..f.nops {
            let op = &f.ops[i];
            if let Op::OSetGlobal = op.op {
                let t = &c.globals[i];
                if t.kind == TypeKind::HENUM
                    && f.ops[i - 2].op == Op::OGetArray
                    && f.ops[i - 3].op == Op::OInt
                {
                    code_hash.globals_signs[op.p1.unwrap() as usize] =
                        c.ints[(f.ops[i - 3].p2.unwrap() as usize)] as i32;
                }
            }
        }

        for i in 0..c.nglobals {
            code_hash.globals_signs[i] ^= code_hash.code_hash_type(c.globals[i].clone()) as i32;
        }

        code_hash
    }
    pub fn hash_type_rec(&mut self, pos: usize) -> u32 {
        let mut hasher = Crc32Hasher::new();
        let t = self.code.types.get(pos).unwrap();
        match t.kind {
            TypeKind::HFUN | TypeKind::HMETHOD => {
                if let ValueTypeU::FuncType { args, nargs, ret } = &t.union {
                    for i in 0..*nargs {
                        let a = args.get(i).unwrap().clone();
                        let i = self.code.types.iter().position(|v| *v == a).unwrap();
                        let h = self.type_hashes[i];
                        hasher.write_u32(h);
                    }

                    let r = ret.as_ref().clone();
                    let i = self.code.types.iter().position(|v| *v == r).unwrap();
                    hasher.write_u32(self.type_hashes[i]);
                }
            }
            TypeKind::HOBJ | TypeKind::HSTRUCT => {
                if let ValueTypeU::ObjType {
                    name: _,
                    super_type: _,
                    fields,
                    nfields,
                    nproto: _,
                    nbindings: _,
                    proto: _,
                    bindings: _,
                    global_value: _,
                    rt: _,
                } = &t.union
                {
                    for i in 0..*nfields {
                        let f = fields.get(i).unwrap().clone().t;
                        let i = self.code.types.iter().position(|v| *v == f).unwrap();
                        hasher.write_u32(self.type_hashes[i]);
                    }
                }
            }
            TypeKind::HREF | TypeKind::HNULL => {
                let p = t.tparam.as_ref().unwrap();
                let i = self
                    .code
                    .types
                    .iter()
                    .position(|v| v == p.as_ref())
                    .unwrap();
                hasher.write_u32(self.type_hashes[i]);
            }
            TypeKind::HENUM => {
                if let ValueTypeU::EnumType {
                    name: _,
                    nconstructs,
                    constructs,
                    global_value: _,
                } = &t.union
                {
                    for i in 0..*nconstructs {
                        let con = constructs.get(i).unwrap().clone();

                        for k in 0..con.nparams {
                            let p = con.params.get(k).unwrap();
                            let i = self.code.types.iter().position(|v| v == p).unwrap();
                            hasher.write_u32(self.type_hashes[i]);
                        }
                    }
                }
            }
            _ => {}
        }

        hasher.finalize()
    }

    pub fn hash_native(&self, n: Native) -> u32 {
        let mut hasher = Crc32Hasher::new();
        hasher.write(n.lib.as_bytes());
        hasher.write(n.name.as_bytes());
        let i = self.code.types.iter().position(|v| *v == n.t).unwrap();
        hasher.write_u32(self.type_hashes[i]);
        hasher.finalize()
    }

    pub fn hash_fun_sign(&self, f: HLFunction) -> u32 {
        let mut hasher = Crc32Hasher::new();
        let i = self.code.types.iter().position(|v| *v == f.t).unwrap();
        hasher.write_u32(self.type_hashes[i]);
        let field = f.field.as_ref();

        if let Some(ValueTypeU::ObjType {
            name,
            super_type: _,
            fields: _,
            nfields: _,
            nproto: _,
            nbindings: _,
            proto: _,
            bindings: _,
            global_value: _,
            rt: _,
        }) = f.obj
        {
            hasher.write(name.as_bytes());
            hasher.write(field.unwrap().name.as_bytes());
        }

        if let Some(rf) = &field.unwrap().rf {
            if let Some(ValueTypeU::ObjType {
                name,
                super_type: _,
                fields: _,
                nfields: _,
                nproto: _,
                nbindings: _,
                proto: _,
                bindings: _,
                global_value: _,
                rt: _,
            }) = &rf.obj
            {
                hasher.write(name.as_bytes());
            }
            hasher.write(rf.field.as_ref().unwrap().name.as_bytes());
            hasher.write_u32(f.rf);
        }
        hasher.finalize()
    }

    fn hfun(&self, hasher: &mut Crc32Hasher, idx: usize) {
        hasher.write_u32(self.functions_hashes[self.functions_indexes[idx] as usize]);
    }

    pub fn hash_fun(&self, f: HLFunction) -> u32 {
        let mut hasher = Crc32Hasher::new();
        let c = &self.code;
        for i in 0..f.nregs {
            let fi = self
                .code
                .types
                .iter()
                .position(|v| *v == f.regs[i])
                .unwrap();
            hasher.write_u32(self.type_hashes[fi]);
        }
        for k in 0..f.nops {
            let o = &f.ops[k];
            hasher.write_u8(o.op as u8);

            match o.op {
                Op::OInt => {
                    hasher.write_i32(o.p1.unwrap());
                    let p2: usize = o.p2.unwrap().try_into().unwrap();
                    hasher.write_i32(*(*c).ints.get(p2).unwrap());
                }
                Op::OFloat => {
                    hasher.write_i32(o.p1.unwrap());
                    let p2: usize = o.p2.unwrap().try_into().unwrap();
                    hasher.write_i32((*(*c).floats.get(p2 << 1).unwrap()) as i32);
                    hasher.write_i32((*(*c).floats.get(p2 << 1 | 1).unwrap()) as i32);
                }
                Op::OString => {
                    hasher.write_i32(o.p1.unwrap());
                    let p2: usize = o.p2.unwrap().try_into().unwrap();
                    hasher.write((*(*c).strings.get(p2).unwrap()).as_bytes());
                }
                Op::OType => {
                    hasher.write_i32(o.p1.unwrap());
                    let p2: usize = o.p2.unwrap().try_into().unwrap();
                    let ti = self
                        .code
                        .types
                        .iter()
                        .position(|v| *v == c.types[p2])
                        .unwrap();
                    hasher.write_u32(self.type_hashes[ti]);
                }
                Op::OCall0 => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                }
                Op::OCall1 => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                    hasher.write_i32(o.p3.unwrap());
                }
                Op::OCall2 => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                    hasher.write_i32(o.p3.unwrap());
                    hasher.write_isize(o.extra[0]);
                }
                Op::OCall3 => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                    hasher.write_i32(o.p3.unwrap());
                    hasher.write_isize(o.extra[0]);
                    hasher.write_isize(o.extra[1]);
                }
                Op::OCall4 => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                    hasher.write_i32(o.p3.unwrap());
                    hasher.write_isize(o.extra[0]);
                    hasher.write_isize(o.extra[1]);
                    hasher.write_isize(o.extra[2]);
                }
                Op::OCallN => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                    hasher.write_i32(o.p3.unwrap());
                    for i in 0..o.p3.unwrap() {
                        hasher.write_isize(o.extra[i as usize]);
                    }
                }
                Op::OStaticClosure => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                }
                Op::OInstanceClosure => {
                    hasher.write_i32(o.p1.unwrap());
                    self.hfun(&mut hasher, o.p2.unwrap().try_into().unwrap());
                    hasher.write_i32(o.p3.unwrap());
                }
                Op::ODynGet => {
                    hasher.write_i32(o.p1.unwrap());
                    hasher.write_i32(o.p2.unwrap());
                    hasher.write(c.strings[o.p3.unwrap() as usize].as_bytes());
                }
                Op::ODynSet => {
                    hasher.write_i32(o.p1.unwrap());
                    hasher.write(c.strings[o.p2.unwrap() as usize].as_bytes());
                    hasher.write_i32(o.p3.unwrap());
                }
                _ => match OP_NARGS[(o.op as u8) as usize] {
                    0 => {}
                    1 => {
                        hasher.write_i32(o.p1.unwrap());
                    }
                    2 => {
                        hasher.write_i32(o.p1.unwrap());
                        hasher.write_i32(o.p2.unwrap());
                    }
                    3 => {
                        hasher.write_i32(o.p1.unwrap());
                        hasher.write_i32(o.p2.unwrap());
                        hasher.write_i32(o.p3.unwrap());
                    }
                    4 => {
                        hasher.write_i32(o.p1.unwrap());
                        hasher.write_i32(o.p2.unwrap());
                        hasher.write_i32(o.p3.unwrap());
                        hasher.write_isize(o.extra[0]);
                    }
                    -1 => match o.op {
                        Op::OCallN
                        | Op::OCallClosure
                        | Op::OCallMethod
                        | Op::OCallThis
                        | Op::OMakeEnum => {
                            hasher.write_i32(o.p1.unwrap());
                            hasher.write_i32(o.p2.unwrap());
                            hasher.write_i32(o.p3.unwrap());
                            for i in 0..o.p3.unwrap() {
                                hasher.write_isize(o.extra[i as usize]);
                            }
                        }
                        Op::OSwitch => {
                            hasher.write_i32(o.p1.unwrap());
                            hasher.write_i32(o.p2.unwrap());
                            for i in 0..o.p2.unwrap() {
                                hasher.write_isize(o.extra[i as usize]);
                            }
                            hasher.write_i32(o.p3.unwrap());
                        }
                        _ => {
                            let s: &'static str = o.op.into();
                            println!("Don't know how to process opcode {}", s);
                        }
                    },
                    _ => {
                        let size = OP_NARGS[(o.op as u8) as usize] - 3;
                        hasher.write_i32(o.p1.unwrap());
                        hasher.write_i32(o.p2.unwrap());
                        hasher.write_i32(o.p3.unwrap());
                        for i in 0..size {
                            hasher.write_isize(o.extra[i as usize]);
                        }
                    }
                },
            }
        }
        hasher.finalize()
    }
    pub fn code_hash_type(&self, t: ValueType) -> u32 {
        let mut hasher = Crc32Hasher::new();
        let ti = self.code.types.iter().position(|v| *v == t).unwrap();
        hasher.write_u32(self.type_hashes[ti]);
        hasher.finalize()
    }

    pub fn finalize(&mut self) {
        let c = &self.code;
        self.functions_signs = vec![0; c.nfunctions + c.nnatives];
        for i in 0..c.nfunctions {
            let f = &c.functions[i];
            self.functions_signs[i] = self.hash_fun_sign(f.clone());
        }
        for i in 0..c.nnatives {
            let f = &c.natives[i];
            self.functions_signs[i] = self.hash_native(f.clone());
        }

        self.functions_hashes = vec![0; c.nfunctions];
        for i in 0..c.nfunctions {
            let f = &c.functions[i];
            self.functions_hashes[i] = self.hash_fun(f.clone());
        }
    }
}
