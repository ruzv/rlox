mod ast;
mod scan;

use anyhow::Result;
use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CLIError {
    #[error("Too many arguments")]
    TooManyArguments,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");

        Err(CLIError::TooManyArguments)?;
    }

    if args.len() == 2 {
        run_file(&args[1])?;

        return Ok(());
    }

    println!("Starting REPL");

    run_prompt()?;

    return Ok(());
}

fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        run(&line?)?;
    }

    Ok(())
}

fn run_file(path: &str) -> Result<()> {
    run(&fs::read_to_string(path)?)?;

    Ok(())
}

fn run(source: &str) -> Result<()> {
    println!("running {}", source);

    let mut scanner = scan::Scanner::new(source.to_string());

    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{}", token);
    }

    Ok(())
}
