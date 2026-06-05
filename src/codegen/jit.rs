// Submodule: jit
//
// Linux:   JIT via inkwell's MCJIT engine.  dlsym(RTLD_DEFAULT) resolves
//          C runtime symbols automatically.
// Windows: The MCJIT engine cannot resolve CRT symbols reliably on Windows,
//          so we emit bitcode, compile it with clang, and run the result.
//          We also ship a tiny dirent wrapper (opendir/readdir/closedir) that
//          adapts the Windows FindFirstFile/FindNextFile API to the POSIX
//          signatures the runtime IR expects.

use std::io::Write;
use std::process::Command;

use super::CodeGen;

#[cfg(target_os = "windows")]
const DIRENT_COMPAT_C: &str = r#"
/* Windows dirent compatibility — provides opendir / readdir / closedir
   on top of FindFirstFileA / FindNextFileA / FindClose.

   The struct layout must match the hardcoded offset (19) that the LLVM
   IR in runtime.rs uses for d_name.

   Also defines stdin / stdout / stderr as real global variables because
   modern UCRT only provides them as macros (&__iob_func()[N]).  The LLVM
   IR references them as symbols, so they must exist at link time. */

#pragma comment(lib, "legacy_stdio_definitions.lib")

#define _CRT_SECURE_NO_WARNINGS
#include <windows.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* stdio.h #defines stdin / stdout / stderr as macros on UCRT.
   We need them as real global symbols, so undefine the macros first. */
#undef stdin
#undef stdout
#undef stderr

/* ── dirent types ──────────────────────────────────────────── */

struct dirent {
    uint64_t d_ino;
    uint64_t d_off;
    unsigned short d_reclen;
    unsigned char d_type;
    char d_name[256];
};

typedef struct {
    HANDLE handle;
    WIN32_FIND_DATAA data;
    struct dirent entry;
    int first;
    int done;
} DIR;

/* ── dirent implementation ─────────────────────────────────── */

void* opendir(const char* path) {
    char search_path[MAX_PATH];
    int len = (int)strlen(path);
    if (len + 3 > MAX_PATH) return NULL;
    memcpy(search_path, path, len);
    search_path[len] = '\\';
    search_path[len+1] = '*';
    search_path[len+2] = '\0';

    DIR* dir = (DIR*)malloc(sizeof(DIR));
    if (!dir) return NULL;
    memset(dir, 0, sizeof(DIR));

    dir->handle = FindFirstFileA(search_path, &dir->data);
    if (dir->handle == INVALID_HANDLE_VALUE) {
        free(dir);
        return NULL;
    }
    dir->first = 1;
    return dir;
}

void* readdir(void* dirp) {
    DIR* dir = (DIR*)dirp;
    if (!dir || dir->done) return NULL;

    if (!dir->first) {
        if (!FindNextFileA(dir->handle, &dir->data)) {
            dir->done = 1;
            return NULL;
        }
    }
    dir->first = 0;

    dir->entry.d_ino = 0;
    dir->entry.d_off = 0;
    dir->entry.d_reclen = 0;
    dir->entry.d_type = 0;
    strncpy(dir->entry.d_name, dir->data.cFileName, 255);
    dir->entry.d_name[255] = '\0';

    return &dir->entry;
}

int closedir(void* dirp) {
    DIR* dir = (DIR*)dirp;
    if (!dir) return -1;
    FindClose(dir->handle);
    free(dir);
    return 0;
}

/* ── stdin / stdout / stderr globals for LLVM IR ─────────────
   The runtime IR declares @stdin = external global ptr (and
   likewise for stdout / stderr).  On UCRT these are macros, not
   linker symbols, so we provide real definitions initialised via
   __iob_func() before main.  The .CRT$XCU callback runs during
   CRT init; dllexport keeps the linker from discarding the globals
   even when LTCG would otherwise prove them unreferenced from
   this TU. */

__declspec(dllexport) FILE *stdin = NULL;
__declspec(dllexport) FILE *stdout = NULL;
__declspec(dllexport) FILE *stderr = NULL;

/* Forward declaration — provided by legacy_stdio_definitions.lib. */
FILE **__cdecl __iob_func(void);

static void init_std_streams(void) {
    FILE **iob = __iob_func();
    stdin  = iob[0];
    stdout = iob[1];
    stderr = iob[2];
}

__declspec(allocate(".CRT$XCU"))
static void (*init_std_streams_ptr)(void) = init_std_streams;
"#;

impl<'ctx> CodeGen<'ctx> {
    pub fn run_jit(&self) -> Result<(), String> {
        // Module verification can trigger analysis passes that may call into
        // unresolved symbols on Windows.  Skip it there — the IR we emit will
        // be verified again by clang anyway.
        #[cfg(not(target_os = "windows"))]
        if let Err(e) = self.module.verify() {
            return Err(format!("LLVM module verification failed: {}", e));
        }

        #[cfg(target_os = "linux")]
        {
            run_via_jit(self)
        }
        #[cfg(target_os = "windows")]
        {
            run_via_clang(self)
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            run_via_jit(self)
        }
    }
}

#[cfg(any(
    target_os = "linux",
    not(any(target_os = "linux", target_os = "windows"))
))]
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

#[cfg(target_os = "windows")]
fn run_via_clang(cg: &CodeGen) -> Result<(), String> {
    eprintln!("[atomic] run_via_clang: starting");

    let tmp_dir = std::env::temp_dir();
    let pid = std::process::id();
    let bc_path = tmp_dir.join(format!("atomic_tmp_{}.bc", pid));
    let c_path = tmp_dir.join(format!("atomic_tmp_{}.c", pid));
    let exe_path = tmp_dir.join(format!("atomic_tmp_{}.exe", pid));

    // 1. Emit LLVM bitcode — uses a simpler LLVM code path than print_ir().
    eprintln!(
        "[atomic] run_via_clang: emitting bitcode to {}",
        bc_path.display()
    );
    cg.emit_bitcode(&bc_path)?;
    eprintln!("[atomic] run_via_clang: bitcode emitted");

    // 2. Write the Windows dirent compatibility shim alongside the bitcode
    //    so that opendir / readdir / closedir (POSIX, absent from msvcrt)
    //    resolve to FindFirstFileA / FindNextFileA / FindClose.
    eprintln!(
        "[atomic] run_via_clang: writing dirent compat to {}",
        c_path.display()
    );
    std::fs::write(&c_path, DIRENT_COMPAT_C)
        .map_err(|e| format!("Failed to write dirent compat: {}", e))?;

    // 3. Compile bitcode + dirent compat to executable with clang.
    //    On modern Windows UCRT, stdin and sprintf are not direct exports
    //    — they live in legacy_stdio_definitions.lib.  We let clang choose
    //    the default linker so that most CRT symbols resolve automatically.
    eprintln!("[atomic] run_via_clang: running clang...");
    let output = Command::new("clang")
        .arg("-target")
        .arg("x86_64-pc-windows-msvc")
        .arg(&c_path)
        .arg(&bc_path)
        .arg("-llegacy_stdio_definitions")
        .arg("-o")
        .arg(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    let _ = std::fs::remove_file(&bc_path);
    let _ = std::fs::remove_file(&c_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "clang compilation failed:\nstdout:{}\nstderr:{}",
            stdout, stderr
        ));
    }
    eprintln!("[atomic] run_via_clang: clang succeeded");

    // 4. Run the executable.
    eprintln!("[atomic] run_via_clang: running {}", exe_path.display());
    let run_output = Command::new(&exe_path).output().map_err(|e| {
        let _ = std::fs::remove_file(&exe_path);
        format!("Failed to run executable: {}", e)
    })?;

    std::io::stdout()
        .write_all(&run_output.stdout)
        .map_err(|e| e.to_string())?;
    std::io::stderr()
        .write_all(&run_output.stderr)
        .map_err(|e| e.to_string())?;

    let _ = std::fs::remove_file(&exe_path);

    if !run_output.status.success() {
        return Err(format!(
            "Process exited with code: {}",
            run_output.status.code().unwrap_or(-1)
        ));
    }

    eprintln!("[atomic] run_via_clang: done");
    Ok(())
}
