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

#[derive(Debug, Clone)]
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
            "line: {}, type: {:?}, lexeme: {}, literal: {}",
            self.line, self.r#type, self.lexeme, self.literal
        )
    }
}

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Unexpected character '{0}'")]
    UnexpectedCharacter(char),

    #[error("Unexpected character")]
    UnterminatedString,
}

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,   // points to the first charector of a lexeme
    current: usize, // points to to the current charecter being considered as part of the lexeme
    line: u32,
}

impl Scanner {
    fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token {
            r#type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: "".to_string(),
            line: self.line,
        });

        Ok(self.tokens.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {} // ignore whitespace
            '\n' => self.line += 1,
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.identifier()?,
            _ => Err(ScannerError::UnexpectedCharacter(c))?,
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<()> {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = &self.source[self.start..self.current];

        match text {
            "and" => self.add_token(TokenType::And),
            "class" => self.add_token(TokenType::Class),
            "else" => self.add_token(TokenType::Else),
            "false" => self.add_token(TokenType::False),
            "for" => self.add_token(TokenType::For),
            "fun" => self.add_token(TokenType::Fun),
            "if" => self.add_token(TokenType::If),
            "nil" => self.add_token(TokenType::Nil),
            "or" => self.add_token(TokenType::Or),
            "print" => self.add_token(TokenType::Print),
            "return" => self.add_token(TokenType::Return),
            "super" => self.add_token(TokenType::Super),
            "this" => self.add_token(TokenType::This),
            "true" => self.add_token(TokenType::True),
            "var" => self.add_token(TokenType::Var),
            "while" => self.add_token(TokenType::While),
            _ => self.add_token_literal(TokenType::Identifier, text.to_string()),
        }

        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance(); //consume the '.'

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        self.add_token_literal(
            TokenType::Number,
            self.source[self.start..self.current].to_string(),
        );

        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            Err(ScannerError::UnterminatedString)?;
        }

        // The closing ".
        self.advance();

        self.add_token_literal(
            TokenType::String,
            self.source[self.start + 1..self.current - 1].to_string(),
        );

        Ok(())
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        return self.source.chars().nth(self.current + 1).unwrap();
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn add_token(&mut self, r#type: TokenType) {
        self.add_token_literal(r#type, "".to_string())
    }

    fn add_token_literal(&mut self, r#type: TokenType, literal: String) {
        self.tokens.push(Token {
            r#type,
            lexeme: self.source[self.start..self.current].to_string(),
            literal,
            line: self.line,
        });
    }
}

#[derive(Debug, Clone, Copy)]
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

    let mut scanner = Scanner::new(source.to_string());

    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{}", token);
    }

    Ok(())
}
