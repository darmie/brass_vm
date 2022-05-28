use std::any::Any;

use cranelift::{
    codegen::{ir::{immediates::Offset32, GlobalValue}, Context},
    frontend::FunctionBuilder,
    prelude::{types, EntityRef, GlobalValueData, Type},
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use crate::{code::Code, code_hash::CodeHash, types::TypeKind};

const HOT_RELOAD_EXTRA_GLOBALS: i32 = 4096;

pub struct HLModule<'a> {
    pub module: JITModule,
    pub module_ctx: Context,
    pub code: &'a Code,
    pub codesize: usize,
    // pub globals_size: usize,
    // pub globals_indexes: Vec<i32>,
    // pub globals_data: Vec<u32>,
    pub functions_ptrs: Vec<FuncId>,
    pub functions_indexes: Vec<i32>,
    pub code_hash: Option<CodeHash>,
}

impl<'a> HLModule<'a> {
    pub fn new(code: &'a Code) -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        // The Module holds information about all functions and data objects defined in the current JIT
        let module = JITModule::new(builder);
        // This is the main Context object for compiling functions.
        let module_ctx = module.make_context();

        // if hot reloading
        let code_hash = None;

        // let gsize = 0;

        // let globals_data = vec![0; code.nglobals];
        // let globals_indexes = vec![0; code.nglobals];

        let functions_indexes = vec![0; code.nfunctions + code.nnatives];
        let functions_ptrs = vec![FuncId::new(0); code.nfunctions + code.nnatives];

        HLModule {
            module,
            module_ctx,
            code,
            codesize: 0,
            // globals_size: code.nglobals,
            // globals_indexes,
            // globals_data,
            functions_ptrs,
            functions_indexes,
            code_hash,
        }
    }
    pub fn init(&mut self, hot_reload: bool) {
        if hot_reload {
            self.code_hash = Some(CodeHash::alloc(self.code));
        }

        self.init_globals();
    }

    pub fn init_globals(&mut self) {
        for i in 0..self.code.nglobals {
            let t = &self.code.globals[i];

            match t.kind {
                TypeKind::HUI8 => {
                    self.translate_global(types::I8, format!("$global{}", i).to_string(), true, Offset32::from(i as i32));
                }
                TypeKind::HUI16 => todo!(),
                TypeKind::HI32 => todo!(),
                TypeKind::HI64 => todo!(),
                TypeKind::HF32 => todo!(),
                TypeKind::HF64 => todo!(),
                TypeKind::HBOOL => todo!(),
                TypeKind::HBYTES => todo!(),
                TypeKind::HDYN => todo!(),
                TypeKind::HFUN => todo!(),
                TypeKind::HOBJ => todo!(),
                TypeKind::HARRAY => todo!(),
                TypeKind::HTYPE => todo!(),
                TypeKind::HREF => todo!(),
                TypeKind::HVIRTUAL => todo!(),
                TypeKind::HDYNOBJ => todo!(),
                TypeKind::HABSTRACT => todo!(),
                TypeKind::HENUM => todo!(),
                TypeKind::HNULL => todo!(),
                TypeKind::HMETHOD => todo!(),
                TypeKind::HSTRUCT => todo!(),
                TypeKind::HPACKED => todo!(),
                TypeKind::HLAST => todo!(),
                _ => {}
            }
        }
    }

    fn translate_global(&mut self, t: Type, name: String, writable: bool, offset: Offset32)-> GlobalValue {
        let sym = self
            .module
            .declare_data(name.as_str(), Linkage::Export, writable, false)
            .expect("problem declaring data object");
        let local_id = self
            .module
            .declare_data_in_func(sym, &mut self.module_ctx.func);
        self.module_ctx
            .func
            .create_global_value(GlobalValueData::Load {
                base: local_id,
                offset,
                global_type: t,
                readonly: false,
            })
    }
}

pub struct FunctionCompiler<'a> {
    pub module: HLModule<'a>,
    pub builder: FunctionBuilder<'a>,
}
