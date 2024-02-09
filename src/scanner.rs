use std::ops::Range;


use rug::{ops::CompleteRound, Float};

use crate::{ast::Literal, PREC_BITS};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    LParen,
    RParen,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
    Exp,
    Indentifier,
    Equal,
    Number,
    Eof,
    Unkown,
}


#[derive(Debug)]
pub enum ScanError {
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: Option<Literal>,
    pub start: usize, 
    pub end: usize,

}
impl Token {
    pub fn span(&self) -> Range<usize> {
        self.start..self.end
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
}

impl <'a> Scanner <'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
        }
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn add_token(&mut self, kind: TokenKind, literal: Option<Literal>) {
        self.tokens.push(Token {
            kind,
            literal,
           start: self.start,
           end: self.current
        })
    }
    pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            literal: None,
            start: self.current,
            end: self.current,
        });
        Ok(self.tokens)
    }
    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::LParen, None),
            ')' => self.add_token(TokenKind::RParen, None),
            ',' => self.add_token(TokenKind::Comma, None),
            '.' => self.add_token(TokenKind::Dot, None),
            '-' => self.add_token(TokenKind::Minus, None),
            '+' => self.add_token(TokenKind::Plus, None),
            '/' => self.add_token(TokenKind::Slash, None),
            '*' => self.add_token(TokenKind::Star, None),
            '=' => self.add_token(TokenKind::Equal, None),
            '^' => self.add_token(TokenKind::Exp, None),
            c if c.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphabetic() => self.literal(),
            ' ' => {},
         _ => self.add_token(TokenKind::Unkown, None),
        };
        Ok(())
    }
    fn advance_while<P>(&mut self, mut predicate: P)
    where
        P: FnMut(char) -> bool,
    {
        while self.peek().is_some_and(|c| predicate(c)) {
            self.advance();
        }
    }

    fn number(&mut self) -> Result<(), String> {
        self.advance_while(|c| c.is_ascii_digit());
        if self.peek() == Some('.') && self.peek_offset(1).is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
            self.advance_while(|c| c.is_ascii_digit());
        }

        self.add_token(
            TokenKind::Number,
            Some(Literal::Number(
                Float::parse(&self.source[self.start..self.current]).unwrap().complete(PREC_BITS)

            )),
        );
        Ok(())
    }
    fn literal(&mut self) {
        self.advance_while(|c| c.is_ascii_alphanumeric());
        self.add_token(TokenKind::Indentifier, None)
    }
    fn advance(&mut self) -> char {
        let c = self.source[self.current..].chars().next().unwrap();
        self.current += 1;
        c
    }
    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }
    fn peek_offset(&self, offset: usize) -> Option<char> {
        self.source[self.current + offset..].chars().next()
    }
    
}
