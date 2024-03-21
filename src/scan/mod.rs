pub mod token;

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
enum ScannerError {
    #[error("Unexpected character '{0}'")]
    UnexpectedCharacter(char),

    #[error("Unexpected character")]
    UnterminatedString,
}

pub struct Scanner {
    source: String,
    tokens: Vec<token::Token>,
    start: usize,   // points to the first charector of a lexeme
    current: usize, // points to to the current charecter being considered as part of the lexeme
    line: u32,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<token::Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(token::Token {
            r#type: token::TokenType::Eof,
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
            '(' => self.add_token(token::TokenType::LeftParen),
            ')' => self.add_token(token::TokenType::RightParen),
            '{' => self.add_token(token::TokenType::LeftBrace),
            '}' => self.add_token(token::TokenType::RightBrace),
            ',' => self.add_token(token::TokenType::Comma),
            '.' => self.add_token(token::TokenType::Dot),
            '-' => self.add_token(token::TokenType::Minus),
            '+' => self.add_token(token::TokenType::Plus),
            ';' => self.add_token(token::TokenType::Semicolon),
            '*' => self.add_token(token::TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.add_token(token::TokenType::BangEqual)
                } else {
                    self.add_token(token::TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(token::TokenType::EqualEqual)
                } else {
                    self.add_token(token::TokenType::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(token::TokenType::LessEqual)
                } else {
                    self.add_token(token::TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(token::TokenType::GreaterEqual)
                } else {
                    self.add_token(token::TokenType::Greater)
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(token::TokenType::Slash)
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
            "and" => self.add_token(token::TokenType::And),
            "class" => self.add_token(token::TokenType::Class),
            "else" => self.add_token(token::TokenType::Else),
            "false" => self.add_token(token::TokenType::False),
            "for" => self.add_token(token::TokenType::For),
            "fun" => self.add_token(token::TokenType::Fun),
            "if" => self.add_token(token::TokenType::If),
            "nil" => self.add_token(token::TokenType::Nil),
            "or" => self.add_token(token::TokenType::Or),
            "print" => self.add_token(token::TokenType::Print),
            "return" => self.add_token(token::TokenType::Return),
            "super" => self.add_token(token::TokenType::Super),
            "this" => self.add_token(token::TokenType::This),
            "true" => self.add_token(token::TokenType::True),
            "var" => self.add_token(token::TokenType::Var),
            "while" => self.add_token(token::TokenType::While),
            _ => self.add_token_literal(token::TokenType::Identifier, text.to_string()),
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
            token::TokenType::Number,
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
            token::TokenType::String,
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

    fn add_token(&mut self, r#type: token::TokenType) {
        self.add_token_literal(r#type, "".to_string())
    }

    fn add_token_literal(&mut self, r#type: token::TokenType, literal: String) {
        self.tokens.push(token::Token {
            r#type,
            lexeme: self.source[self.start..self.current].to_string(),
            literal,
            line: self.line,
        });
    }
}
