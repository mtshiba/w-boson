//! # w-boson
//!
//! Windows port of [backtrace-on-stack-overflow](https://crates.io/crates/backtrace-on-stack-overflow)
//!
//! ## Usage
//!
//! ```rust
//! use w_boson::enable;
//! // or use w_boson::enable_backtrace_on_stack_overflow;
//!
//! fn recursive(n: usize) {
//!     print!("{n}");
//!     recursive(n+1);
//! }
//!
//! fn main() {
//!     unsafe { enable(); }
//!     recursive(0);
//! }
//! ```
//!
//! ## Notes
//!
//! To get the correct function names even in the release build, add the following settings to `Cargo.toml`.
//!
//! ```toml
//! # Cargo.toml
//! [profile.release]
//! debug = true
//! ```
use std::ptr;
use std::sync::Once;
use windows::Win32::System::Diagnostics::Debug::{RtlCaptureStackBackTrace, SymFromAddr, SymGetLineFromAddr64, SymInitialize, EXCEPTION_POINTERS, IMAGEHLP_LINE64, SYMBOL_INFO};
use windows::Win32::Foundation::EXCEPTION_STACK_OVERFLOW;
use windows::Win32::System::Diagnostics::Debug::AddVectoredExceptionHandler;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::core::PCSTR;

// `backtrace::Backtrace::new()` causes an access violation, so we need to print the backtrace manually.
unsafe fn print_backtrace(buf_size: usize) {
    let mut addresses = vec![ptr::null_mut(); buf_size];
    let n_frames = RtlCaptureStackBackTrace(0, &mut addresses, None);
    let process = GetCurrentProcess();

    eprintln!("Stack Overflow:");
    for i in 0..n_frames {
        let addr = addresses[i as usize];
        let mut symbol: [u8; 512] = [0; 512];
        let symbol_info = symbol.as_mut_ptr() as *mut SYMBOL_INFO;
        (*symbol_info).SizeOfStruct = std::mem::size_of::<SYMBOL_INFO>() as u32;
        (*symbol_info).MaxNameLen = (symbol.len() - std::mem::size_of::<SYMBOL_INFO>()) as u32;

        if SymFromAddr(process, addr as u64, None, symbol_info).is_ok() {
            let name = std::ffi::CStr::from_ptr((*symbol_info).Name.as_ptr());
            eprintln!("{i}: {}", name.to_string_lossy());
        } else {
            eprintln!("{i}: <unknown>");
        }
        let mut line: IMAGEHLP_LINE64 = std::mem::zeroed();
        line.SizeOfStruct = std::mem::size_of::<IMAGEHLP_LINE64>() as u32;
        if SymGetLineFromAddr64(process, addr as u64, &mut 0, &mut line).is_ok() {
            let fine_name = String::from_utf8_lossy(line.FileName.as_bytes());
            eprintln!("        at {}:{}", fine_name, line.LineNumber);
        }
    }
}

// SEH Exception handler
unsafe extern "system" fn seh_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    let ex_record = *exception_info.as_ref().unwrap().ExceptionRecord;
    if ex_record.ExceptionCode == EXCEPTION_STACK_OVERFLOW {
        print_backtrace(10000);
        std::process::abort();
    }
    0
}

pub unsafe fn enable() {
    unsafe {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            let process = GetCurrentProcess();
            SymInitialize(process, PCSTR("./target/debug;./target/release".as_ptr()), true).unwrap();

            AddVectoredExceptionHandler(1, Some(seh_handler));
        });
    }
}

/// alias of `enable`
pub unsafe fn enable_backtrace_on_stack_overflow() {
    enable();
}
