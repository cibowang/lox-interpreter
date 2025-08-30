use miette::{Error, LabeledSpan, Result};
use std::borrow::Cow;

pub enum Token<'de> {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    Plus,
    Minus,
    Star,
    Bang,
    EqualEqual,
    LessEqual,
    GreaterEqual,
    BangEqual,
    Less,
    Greater,
    Slash,
    Dot,
    String(&'de str),
    //(){};,+-*!===<=>=!=<>/.
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::LeftParen => "LEFT_PAREN ( null",
                Token::RightParen => "RIGHT_PAREN ) null",
                Token::LeftBrace => "LEFT_BRACE { null",
                Token::RightBrace => "RIGHT_BRACE } null",
                Token::Semicolon => "SEMICOLON ; null",
                Token::Comma => "COMMA , null",
                Token::Plus => "PLUS + null",
                Token::Minus => "MINUS - null",
                Token::Star => "STAR * null",
                Token::Bang => "BANG ! null",
                Token::EqualEqual => "EQUAL_EQUAL == null",
                Token::LessEqual => "LESS_EQUAL =< null",
                Token::GreaterEqual => "GREATER_EQUAL null",
                Token::BangEqual => "BANG_EQUAL null",
                Token::Less => "LESS < null",
                Token::Greater => "GREATER > null",
                Token::Slash => "SLASH / null",
                Token::Dot => "DOT . null",
                Token::String(s) => return write!(f, "STRING \"{s}\" {}", Token::unescape(s)),
            }
        )
    }
}

impl Token<'_> {
    pub fn unescape<'de>(_s: &'de str) -> Cow<'de, str> {
        todo!()
    }
}

pub struct Lexer<'de> {
    whole: &'de str,
    remainder: &'de str,
    byte: usize,
}

impl<'de> Lexer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            remainder: input,
            byte: 0,
        }
    }
}

impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        // let c = self.remainder.chars().next()?;
        // self.remainder = self.remainder[c.len_utf8()..];
        let mut chars = self.remainder.chars();
        let c = chars.next()?;
        self.remainder = chars.as_str();
        self.byte += c.len_utf8();

        // only 1 char to scan at one time
        match c {
            '(' => Some(Ok(Token::LeftParen)),
            ')' => Some(Ok(Token::RightParen)),
            '{' => Some(Ok(Token::LeftBrace)),
            '}' => Some(Ok(Token::RightBrace)),
            ';' => Some(Ok(Token::Semicolon)),
            ',' => Some(Ok(Token::Comma)),
            '+' => Some(Ok(Token::Plus)),
            '-' => Some(Ok(Token::Minus)),
            '*' => Some(Ok(Token::Star)),
            '!' => Some(Ok(Token::Bang)),
            //'==' => Some(Ok(Token::EqualEqual)),
            //'<=' => Some(Ok(Token::LessEqual)),
            //'>=' => Some(Ok(Token::GreaterEqual)),
            //'!=' => Some(Ok(Token::BangEqual)),
            '<' => Some(Ok(Token::Less)),
            '>' => Some(Ok(Token::Greater)),
            '/' => Some(Ok(Token::Slash)),
            '.' => Some(Ok(Token::Dot)),
            _ => Some(Err(miette::miette! {
                labels = vec![
                    LabeledSpan::at(self.byte - c.len_utf8()..self.byte, "this char"),
                ],
                "Uexpected token '{c}' input",
            }
            .with_source_code(self.whole.to_string()))),
        }
    }
}
