use std::{
    ffi::CString,
    fs::{self},
    io::Write,
};

use crate::{types::AsciiChar, vm::VM};

mod collections;
mod compiler;
mod memory;
mod scanner;
mod types;
mod vm;

pub const DEBUG_STRESS_GC: bool = false;
pub const DEBUG_LOG_GC: bool = true;

pub use crate::types::value::obj::garbage_collect;

pub static mut VM_INSTANCE: *mut VM = std::ptr::null_mut();
pub static mut COMPILER_INSTANCE: *mut compiler::Compiler = std::ptr::null_mut();

fn initialize_vm() {
    unsafe {
        VM_INSTANCE = Box::into_raw(Box::new(VM::new()));
    }
}

pub fn vm_is_ready() -> bool {
    unsafe { !VM_INSTANCE.is_null() }
}

fn main() {
    initialize_vm();

    let mut exit_code = None;
    let args = std::env::args();
    match args.len() {
        1 => repl(),
        2 => exit_code = Some(run_file(args)),
        _ => {
            println!("Usage: clox [path]");
            std::process::exit(64);
        }
    }

    unsafe {
        if !VM_INSTANCE.is_null() {
            (*VM_INSTANCE).free();
        }
    }

    if let Some(code) = exit_code {
        std::process::exit(code);
    }
}

fn repl() {
    let line = &mut String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        line.clear();
        if let Err(e) = std::io::stdin().read_line(line) {
            println!("Could not read the input: {e}");
        }

        let source = CString::new(line.as_str()).expect("Input doesn't contain null bytes");
        unsafe {
            (*VM_INSTANCE).interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
        }
    }
}

fn run_file(mut args: std::env::Args) -> i32 {
    let Ok(source) = fs::read_to_string(args.next_back().expect("Length already checked")) else {
        println!("Could not read the file");
        return 74;
    };

    let source = CString::new(source.as_str()).expect("Input doesn't contain null bytes");
    let result = unsafe {
        (*VM_INSTANCE).interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar)
    };

    result.to_exit_code()
}
