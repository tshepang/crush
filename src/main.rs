#[macro_use]
extern crate lalrpop_util;

mod lang;
mod lib;
mod util;

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use lib::declare;
use crate::lang::errors::{CrushResult, to_crush_error};
use crate::lang::printer;
use crate::lang::stream::empty_channel;
use crate::lang::pretty_printer::create_pretty_printer;
use crate::util::file::home;
use std::path::Path;
use std::fs;
use crate::lang::parser::parse;
use crate::lang::scope::Scope;
use crate::lang::execution_context::JobContext;
use crate::lang::printer::Printer;

fn crush_history_file() -> Box<str> {
    Box::from(
        home()
            .unwrap_or_else(|_| Box::from(Path::new(".")))
            .join(Path::new(".crush_history"))
            .to_str()
            .unwrap_or(".crush_history"))
}

fn run_interactive(global_env: Scope, printer: Printer) -> CrushResult<()> {
    printer.line("Welcome to Crush");
    printer.line(r#"Type "help" for... help."#);

    let pretty_printer = create_pretty_printer(printer.clone());

    let mut rl = Editor::<()>::new();
    let _ = rl.load_history(crush_history_file().as_ref());
    loop {
        let readline = rl.readline("crush> ");

        match readline {
            Ok(cmd) => {
                if !cmd.is_empty() {
                    rl.add_history_entry(cmd.as_str());
                    match parse(&cmd.as_str(), &global_env) {//&mut Lexer::new(&cmd)) {
                        Ok(jobs) => {
                            for job_definition in jobs {
                                match job_definition.invoke(JobContext::new(
                                    empty_channel(), pretty_printer.clone(), global_env.clone(), printer.clone())) {
                                    Ok(handle) => {
                                        handle.join(&printer);
                                    }
                                    Err(e) => printer.crush_error(e),
                                }
                            }
                        }
                        Err(error) => {
                            printer.crush_error(error);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                printer.line("^C");
            }
            Err(ReadlineError::Eof) => {
                printer.line("exit");
                break;
            }
            Err(err) => {
                printer.handle_error::<()>(to_crush_error(Err(err)));
                break;
            }
        }
        match rl.save_history(crush_history_file().as_ref()) {
            Ok(_) => {}
            Err(_) => {
                printer.line("Error: Failed to save history.");
            }
        }
    }
    Ok(())
}


fn run_script(global_env: Scope, filename: &str, printer: Printer) -> CrushResult<()> {
    let cmd = to_crush_error(fs::read_to_string(filename))?;
    match parse(&cmd.as_str(), &global_env) {//&mut Lexer::new(&cmd)) {
        Ok(jobs) => {
            for job_definition in jobs {
                let last_output = create_pretty_printer(printer.clone());
                match job_definition.invoke(JobContext::new(
                    empty_channel(), last_output, global_env.clone(), printer.clone())) {
                    Ok(handle) => {
                        handle.join(&printer);
                    }
                    Err(e) => printer.crush_error(e),
                }
            }
        }
        Err(error) => {
            printer.crush_error(error);
        }
    }
    Ok(())
}

fn run() -> CrushResult<()> {
    let global_env = lang::scope::Scope::create_root();
    let (printer, print_handle) = printer::init();
    declare(&global_env)?;
    let my_scope = global_env.create_child(&global_env, false);

    let args = std::env::args().collect::<Vec<String>>();
    match args.len() {
        1 => run_interactive(my_scope, printer)?,
        2 => run_script(my_scope, args[1].as_str(), printer)?,
        _ => {}
    }
    let _ = print_handle.join();
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => println!("Error during initialization: {}", e.message),
    }
}
