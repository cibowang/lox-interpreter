use miette::{Error, LabeledSpan, Result};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'de> {
    origin: &'de str,
    kind: TokenKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
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
    String,
    Ident,
    Number(f64),
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
        let i = self.origin;
        match self.kind {
            TokenKind::LeftParen => write!(f, "LEFT_PAREN {i} null"),
            TokenKind::RightParen => write!(f, "RIGHT_PAREN {i} null"),
            TokenKind::LeftBrace => write!(f, "LEFT_BRACE {i} null"),
            TokenKind::RightBrace => write!(f, "RIGHT_BRACE {i} null"),
            TokenKind::Semicolon => write!(f, "SEMICOLON {i} null"),
            TokenKind::Comma => write!(f, "COMMA {i} null"),
            TokenKind::Plus => write!(f, "PLUS {i} null"),
            TokenKind::Minus => write!(f, "MINUS {i} null"),
            TokenKind::Star => write!(f, "STAR {i} null"),
            TokenKind::Bang => write!(f, "BANG {i} null"),
            TokenKind::Equal => write!(f, "EQUAL {i} null"),
            TokenKind::EqualEqual => write!(f, "EQUAL_EQUAL {i} null"),
            TokenKind::LessEqual => write!(f, "LESS_EQUAL {i} null"),
            TokenKind::GreaterEqual => write!(f, "GREATER_EQUAL {i} null"),
            TokenKind::BangEqual => write!(f, "BANG_EQUAL {i} null"),
            TokenKind::Less => write!(f, "LESS null {i} null"),
            TokenKind::Greater => write!(f, "GREATER {i} null"),
            TokenKind::Slash => write!(f, "SLASH {i} null"),
            TokenKind::Dot => write!(f, "DOT {i} null"),
            TokenKind::String => write!(f, "STRING {i} {}", Token::unescape(i)),
            TokenKind::Ident => write!(f, "IDENTIFIER {i} null"),
            TokenKind::Number(n) => write!(f, "NUMBER {i} {n}"),
            TokenKind::And => write!(f, "AND {i} null"),
            TokenKind::Class => write!(f, "CLASS {i} null"),
            TokenKind::Else => write!(f, "ELSE {i} null"),
            TokenKind::False => write!(f, "FALSE {i} null"),
            TokenKind::For => write!(f, "FOR {i} null"),
            TokenKind::Fun => write!(f, "FUNCTION {i} null"),
            TokenKind::If => write!(f, "IF {i} null"),
            TokenKind::Nil => write!(f, "NIL {i} null"),
            TokenKind::Or => write!(f, "OR {i} null"),
            TokenKind::Return => write!(f, "RETURN {i} null"),
            TokenKind::Super => write!(f, "SUPER {i} null"),
            TokenKind::This => write!(f, "THIS {i} null"),
            TokenKind::True => write!(f, "TRUE {i} null"),
            TokenKind::Var => write!(f, "VAR {i} null"),
            TokenKind::While => write!(f, "WHILE {i} null"),
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
        // literal should be derived from &str not chars (chars to derive c-related);
        loop {
            // NOTE: this must be in the loop for indices to match-up c_onwards
            let mut chars = self.remainder.chars();
            let c = chars.next()?;
            let literal = &self.remainder[..c.len_utf8()];
            let c_onwards = self.remainder;
            self.remainder = chars.as_str();
            self.byte += c.len_utf8();

            enum Started {
                // rm match "
                //Less,
                //Greater,
                //Bang,
                //Equal,
                Ident,
                Number,
                String,
                IfEqualElse(TokenKind, TokenKind),
            }

            let helper = move |kind: TokenKind| {
                Some(Ok(Token {
                    origin: literal,
                    kind,
                }))
            };

            // only 1 char to scan at one time
            let started = match c {
                '(' => return helper(TokenKind::LeftParen),
                ')' => return helper(TokenKind::RightParen),
                '{' => return helper(TokenKind::LeftBrace),
                '}' => return helper(TokenKind::RightBrace),
                ';' => return helper(TokenKind::Semicolon),
                ',' => return helper(TokenKind::Comma),
                '+' => return helper(TokenKind::Plus),
                '-' => return helper(TokenKind::Minus),
                '*' => return helper(TokenKind::Star),
                '/' => return helper(TokenKind::Slash),
                '.' => return helper(TokenKind::Dot),
                //'==' => Some(Ok(Token::EqualEqual)),
                //'<=' => Some(Ok(Token::LessEqual)),
                //'>=' => Some(Ok(Token::GreaterEqual)),
                //'!=' => Some(Ok(Token::BangEqual)),
                '<' => Started::IfEqualElse(TokenKind::LessEqual, TokenKind::Less),
                '>' => Started::IfEqualElse(TokenKind::GreaterEqual, TokenKind::Greater),
                '!' => Started::IfEqualElse(TokenKind::BangEqual, TokenKind::Bang),
                '=' => Started::IfEqualElse(TokenKind::EqualEqual, TokenKind::Equal),
                '"' => Started::String,
                '0'..='9' => Started::Number,
                'a'..='z' | 'A'..='Z' | '_' => Started::Ident,
                c if c.is_whitespace() => continue,
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

            break match started {
                Started::IfEqualElse(yes, no) => {
                    //if self.remainder.starts_with('<') {
                    //    self.remainder = &self.remainder[1..];
                    //    self.byte += 1;
                    //    return Some(Ok(Token::LessEqual));
                    //} else {
                    //    return Some(Ok(Token::Less));
                    //}
                    self.remainder = self.remainder.trim_start();
                    let trimmed = c_onwards.len() - self.remainder.len() - 1;
                    self.byte += trimmed;
                    if self.remainder.trim_start().starts_with('=') {
                        let span = &c_onwards[..c.len_utf8() + trimmed + 1];
                        self.remainder = &self.remainder.trim_start()[1..];
                        self.byte += 1;
                        Some(Ok(Token {
                            origin: span,
                            kind: yes,
                        }))
                    } else {
                        Some(Ok(Token {
                            origin: literal,
                            kind: no,
                        }))
                    }
                }
                Started::Ident => todo!(),
                Started::Number => todo!(),
                Started::String => todo!(),
            };
        }
    }
}
