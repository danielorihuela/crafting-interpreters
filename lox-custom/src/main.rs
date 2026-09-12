use std::{fs, io::Write};

use crate::vm::VM;

mod collections;
mod compiler;
mod memory;
mod scanner;
mod types;
mod vm;

pub const DEBUG_STRESS_GC: bool = false;
pub const DEBUG_LOG_GC: bool = false;

fn main() {
    let mut vm = VM::new();

    let mut exit_code = None;
    let args = std::env::args();
    match args.len() {
        1 => repl(&mut vm),
        2 => exit_code = Some(run_file(&mut vm, args)),
        _ => {
            println!("Usage: clox [path]");
            std::process::exit(64);
        }
    }

    vm.free();

    if let Some(code) = exit_code {
        std::process::exit(code);
    }
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

        vm.interpret(line);
    }
}

fn run_file(vm: &mut VM, mut args: std::env::Args) -> i32 {
    let Ok(source) = fs::read_to_string(args.next_back().expect("Length already checked")) else {
        println!("Could not read the file");
        return 74;
    };

    let result = vm.interpret(&source);

    result.to_exit_code()
}
