use miette::{Error, LabeledSpan, Result};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
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
    Equal,
    EqualEqual,
    LessEqual,
    GreaterEqual,
    BangEqual,
    Less,
    Greater,
    Slash,
    Dot,
    String(&'de str),
    Ident(&'de str),
    Number(&'de str, f64),
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    //(){};,+-*!===<=>=!=<>/.
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LeftParen => write!(f, "LEFT_PAREN ( null"),
            Token::RightParen => write!(f, "RIGHT_PAREN ) null"),
            Token::LeftBrace => write!(f, "LEFT_BRACE {{ null"),
            Token::RightBrace => write!(f, "RIGHT_BRACE }} null"),
            Token::Semicolon => write!(f, "SEMICOLON ; null"),
            Token::Comma => write!(f, "COMMA , null"),
            Token::Plus => write!(f, "PLUS + null"),
            Token::Minus => write!(f, "MINUS - null"),
            Token::Star => write!(f, "STAR * null"),
            Token::Bang => write!(f, "BANG ! null"),
            Token::Equal => write!(f, "EQUAL = null"),
            Token::EqualEqual => write!(f, "EQUAL_EQUAL == null"),
            Token::LessEqual => write!(f, "LESS_EQUAL =< null"),
            Token::GreaterEqual => write!(f, "GREATER_EQUAL null"),
            Token::BangEqual => write!(f, "BANG_EQUAL null"),
            Token::Less => write!(f, "LESS < null"),
            Token::Greater => write!(f, "GREATER > null"),
            Token::Slash => write!(f, "SLASH / null"),
            Token::Dot => write!(f, "DOT . null"),
            Token::String(s) => write!(f, "STRING \"{s}\" {}", Token::unescape(s)),
            Token::Ident(i) => write!(f, "IDENTIFIER {i} null"),
            Token::Number(lit, n) => write!(f, "NUMBER {lit} {n}"),
            Token::And => write!(f, "AND and null"),
            Token::Class => write!(f, "CLASS class null"),
            Token::Else => write!(f, "ELSE else null"),
            Token::False => write!(f, "FALSE false null"),
            Token::For => write!(f, "FOR for null"),
            Token::Fun => write!(f, "FUNCTION fun null"),
            Token::If => write!(f, "IF if null"),
            Token::Nil => write!(f, "NIL nil null"),
            Token::Or => write!(f, "OR or null"),
            Token::Return => write!(f, "RETURN return null"),
            Token::Super => write!(f, "SUPER super null"),
            Token::This => write!(f, "THIS this null"),
            Token::True => write!(f, "TRUE true null"),
            Token::Var => write!(f, "VAR var null"),
            Token::While => write!(f, "WHILE while null"),
            //need to escape & unescape double quotes
        }
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

        enum Started<'de> {
            // rm match "
            //Less,
            //Greater,
            //Bang,
            //Equal,
            Ident,
            Number,
            String,
            IfEqualElse(Token<'de>, Token<'de>),
        }
        // only 1 char to scan at one time
        let started = match c {
            '(' => return Some(Ok(Token::LeftParen)),
            ')' => return Some(Ok(Token::RightParen)),
            '{' => return Some(Ok(Token::LeftBrace)),
            '}' => return Some(Ok(Token::RightBrace)),
            ';' => return Some(Ok(Token::Semicolon)),
            ',' => return Some(Ok(Token::Comma)),
            '+' => return Some(Ok(Token::Plus)),
            '-' => return Some(Ok(Token::Minus)),
            '*' => return Some(Ok(Token::Star)),
            //'==' => Some(Ok(Token::EqualEqual)),
            //'<=' => Some(Ok(Token::LessEqual)),
            //'>=' => Some(Ok(Token::GreaterEqual)),
            //'!=' => Some(Ok(Token::BangEqual)),
            '<' => Started::IfEqualElse(Token::LessEqual, Token::Less),
            '>' => Started::IfEqualElse(Token::GreaterEqual, Token::Greater),
            '!' => Started::IfEqualElse(Token::BangEqual, Token::Bang),
            '=' => Started::IfEqualElse(Token::EqualEqual, Token::Equal),
            '/' => return Some(Ok(Token::Slash)),
            '.' => return Some(Ok(Token::Dot)),
            '"' => Started::String,
            '0'..='9' => Started::Number,
            'a'..='z' | 'A'..='Z' | '_' => Started::Ident,
            c => {
                return Some(Err(miette::miette! {
                    labels = vec![
                        LabeledSpan::at(self.byte - c.len_utf8()..self.byte, "this char"),
                    ],
                    "Uexpected token '{c}' input",
                }
                .with_source_code(self.whole.to_string())))
            }
        };

        match started {
            Started::IfEqualElse(yes, no) => {
                //yif self.remainder.starts_with('<') {
                //    self.remainder = &self.remainder[1..];
                //    self.byte += 1;
                //    return Some(Ok(Token::LessEqual));
                //} else {
                //    return Some(Ok(Token::Less));
                //}
                if self.remainder.starts_with('=') {
                    self.remainder = &self.remainder[1..];
                    self.byte += 1;
                    Some(Ok(yes))
                } else {
                    Some(Ok(no))
                }
            }
            Started::Ident => todo!(),
            Started::Number => todo!(),
            Started::String => todo!(),
        }
    }
}
