// Submodule: jit
//
// JIT execution via inkwell's MCJIT engine. All platforms use the same
// code path now that the binary is statically linked and all CRT symbols
// are resolvable at JIT time.

use std::io::Write;

use super::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    pub fn run_jit(&self) -> Result<(), String> {
        #[cfg(not(target_os = "windows"))]
        if let Err(e) = self.module.verify() {
            return Err(format!("LLVM module verification failed: {}", e));
        }

        run_via_jit(self)
    }
}

fn run_via_jit(cg: &CodeGen) -> Result<(), String> {
    let opt = match cg.opt_level {
        0 => inkwell::OptimizationLevel::None,
        1 => inkwell::OptimizationLevel::Less,
        2 => inkwell::OptimizationLevel::Default,
        _ => inkwell::OptimizationLevel::Aggressive,
    };
    let engine = cg
        .module
        .create_jit_execution_engine(opt)
        .map_err(|e| e.to_string())?;

    unsafe {
        let main: inkwell::execution_engine::JitFunction<unsafe extern "C" fn() -> u64> =
            engine.get_function("main").map_err(|e| e.to_string())?;
        let _exit_code = main.call();
        extern "C" {
            fn fflush(stream: *mut std::ffi::c_void) -> std::ffi::c_int;
        }
        fflush(std::ptr::null_mut());
    }
    std::io::stdout().flush().ok();
    Ok(())
}
