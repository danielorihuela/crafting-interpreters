use std::{
    ffi::CString,
    fs::{self},
    io::Write,
};

use crate::{opcode::OpCode, vm::VM};

mod chunk;
mod collections;
mod compiler;
mod opcode;
mod scanner;
mod token;
mod value;
mod vm;

pub type AsciiChar = u8;

fn main() {
    let vm = &mut VM::init();

    let args = std::env::args();
    match args.len() {
        1 => repl(vm),
        2 => run_file(args, vm),
        _ => {
            println!("Usage: clox [path]");
            std::process::exit(64);
        }
    }

    vm.free();
}

fn repl(vm: &mut VM) {
    let line = &mut String::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        line.clear();
        if let Err(e) = std::io::stdin().read_line(line) {
            println!("Could not read the input: {e}");
        }

        let source = CString::new(line.as_str()).expect("Input doesn't contain null bytes");
        vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);
    }
}

fn run_file(mut args: std::env::Args, vm: &mut VM) {
    let Ok(source) = fs::read_to_string(args.next_back().expect("Length already checked")) else {
        println!("Could not read the file");
        return;
    };

    let source = CString::new(source.as_str()).expect("Input doesn't contain null bytes");
    let result = vm.interpret(source.as_bytes_with_nul().as_ptr() as *const AsciiChar);

    std::process::exit(result.to_exit_code());
}
