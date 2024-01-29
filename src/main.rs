use anyhow::Result;
use std::env;
use std::fmt;
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

struct Token {
    r#type: TokenType,
    lexeme: String,
    literal: String,
    line: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {:?} {} {}",
            self.line, self.r#type, self.lexeme, self.literal
        )
    }
}

#[derive(Debug)]
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
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

    Ok(())
}
