use miette::{Diagnostic, Error, LabeledSpan, SourceSpan};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Diagnostic, Debug, Error)]
#[error("Unexpected token '{token}' in input")]
pub struct SingleTokenError {
    #[source_code]
    src: String,
    pub token: char,

    #[label = "This input char"]
    err_span: SourceSpan,
}

impl SingleTokenError {
    pub fn line(&self) -> usize {
        let until_unrecognized = &self.src[..=self.err_span.offset()];
        until_unrecognized.lines().count()
    }
}

#[derive(Diagnostic, Debug, Error)]
#[error("Unterminated string in input")]
pub struct StringTerminationError {
    #[source_code]
    src: String,

    #[label = "This input string"]
    err_span: SourceSpan,
}

impl StringTerminationError {
    pub fn line(&self) -> usize {
        let until_unrecognized = &self.src[..=self.err_span.offset()];
        until_unrecognized.lines().count()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'de> {
    pub origin: &'de str,
    pub offset: usize,
    pub kind: TokenKind,
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
    Print,
    //(){};,+-*!===<=>=!=<>/.
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let origin = self.origin;
        match self.kind {
            TokenKind::LeftParen => write!(f, "LEFT_PAREN {origin} null"),
            TokenKind::RightParen => write!(f, "RIGHT_PAREN {origin} null"),
            TokenKind::LeftBrace => write!(f, "LEFT_BRACE {origin} null"),
            TokenKind::RightBrace => write!(f, "RIGHT_BRACE {origin} null"),
            TokenKind::Semicolon => write!(f, "SEMICOLON {origin} null"),
            TokenKind::Comma => write!(f, "COMMA {origin} null"),
            TokenKind::Plus => write!(f, "PLUS {origin} null"),
            TokenKind::Minus => write!(f, "MINUS {origin} null"),
            TokenKind::Star => write!(f, "STAR {origin} null"),
            TokenKind::Bang => write!(f, "BANG {origin} null"),
            TokenKind::Equal => write!(f, "EQUAL {origin} null"),
            TokenKind::EqualEqual => write!(f, "EQUAL_EQUAL {origin} null"),
            TokenKind::LessEqual => write!(f, "LESS_EQUAL {origin} null"),
            TokenKind::GreaterEqual => write!(f, "GREATER_EQUAL {origin} null"),
            TokenKind::BangEqual => write!(f, "BANG_EQUAL {origin} null"),
            TokenKind::Less => write!(f, "LESS null {origin} null"),
            TokenKind::Greater => write!(f, "GREATER {origin} null"),
            TokenKind::Slash => write!(f, "SLASH {origin} null"),
            TokenKind::Dot => write!(f, "DOT {origin} null"),
            TokenKind::String => write!(f, "STRING {origin} {}", Token::unescape(origin)),
            TokenKind::Ident => write!(f, "IDENTIFIER {origin} null"),
            TokenKind::Number(n) => {
                if n == n.trunc() {
                    write!(f, "NUMBER {origin} {n}.0")
                } else {
                    write!(f, "NUMBER {origin} {n}")
                }
            }
            TokenKind::And => write!(f, "AND {origin} null"),
            TokenKind::Class => write!(f, "CLASS {origin} null"),
            TokenKind::Else => write!(f, "ELSE {origin} null"),
            TokenKind::False => write!(f, "FALSE {origin} null"),
            TokenKind::For => write!(f, "FOR {origin} null"),
            TokenKind::Fun => write!(f, "FUNCTION {origin} null"),
            TokenKind::If => write!(f, "IF {origin} null"),
            TokenKind::Nil => write!(f, "NIL {origin} null"),
            TokenKind::Or => write!(f, "OR {origin} null"),
            TokenKind::Return => write!(f, "RETURN {origin} null"),
            TokenKind::Super => write!(f, "SUPER {origin} null"),
            TokenKind::This => write!(f, "THIS {origin} null"),
            TokenKind::True => write!(f, "TRUE {origin} null"),
            TokenKind::Var => write!(f, "VAR {origin} null"),
            TokenKind::While => write!(f, "WHILE {origin} null"),
            TokenKind::Print => write!(f, "PRINT {origin} null"),
            //need to escape & unescape double quotes
        }
    }
}

impl Token<'_> {
    pub fn unescape<'de>(s: &'de str) -> Cow<'de, str> {
        // Lox no support escaping
        Cow::Borrowed(s.trim_matches('"'))
    }
}

pub struct Lexer<'de> {
    whole: &'de str,
    remainder: &'de str,
    byte: usize,
    peeked: Option<Result<Token<'de>, miette::Error>>,
}

impl<'de> Lexer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            remainder: input,
            byte: 0,
            peeked: None,
        }
    }
    pub fn peek(&mut self) -> Option<&Result<Token<'de>, miette::Error>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }
        self.peeked = self.next();
        self.peeked.as_ref()
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
            let c_at = self.byte;
            let mut chars = self.remainder.chars();
            let c = chars.next()?;
            let literal = &self.remainder[..c.len_utf8()]; // literal is up until c
            let c_onwards = self.remainder;
            self.remainder = chars.as_str();
            self.byte += c.len_utf8();

            enum Started {
                // rm match "
                //Less,
                //Greater,
                //Bang,
                //Equal,
                Slash,
                Ident,
                Number,
                String,
                IfEqualElse(TokenKind, TokenKind),
            }

            let helper = move |kind: TokenKind| {
                Some(Ok(Token {
                    origin: literal,
                    offset: c_at,
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
                '.' => return helper(TokenKind::Dot),
                //'==' => Some(Ok(Token::EqualEqual)),
                //'<=' => Some(Ok(Token::LessEqual)),
                //'>=' => Some(Ok(Token::GreaterEqual)),
                //'!=' => Some(Ok(Token::BangEqual)),
                '<' => Started::IfEqualElse(TokenKind::LessEqual, TokenKind::Less),
                '>' => Started::IfEqualElse(TokenKind::GreaterEqual, TokenKind::Greater),
                '!' => Started::IfEqualElse(TokenKind::BangEqual, TokenKind::Bang),
                '=' => Started::IfEqualElse(TokenKind::EqualEqual, TokenKind::Equal),
                '"' => Started::String,                        // special
                '/' => Started::Slash,                         // special
                '0'..='9' => Started::Number,                  // special
                'a'..='z' | 'A'..='Z' | '_' => Started::Ident, // special
                c if c.is_whitespace() => continue,            // temination (\n, '')
                c => {
                    return Some(Err(SingleTokenError {
                        src: self.whole.to_string(),
                        token: c,
                        err_span: SourceSpan::from(self.byte - c.len_utf8()..self.byte),
                    }
                    .into()))
                }
            };

            break match started {
                Started::Slash => {
                    if self.remainder.starts_with('/') {
                        // terminator is char
                        let terminator = self.remainder.find('\n').unwrap_or(self.remainder.len());
                        self.byte += terminator;
                        self.remainder = &self.remainder[terminator..];
                        continue;
                    } else {
                        Some(Ok(Token {
                            origin: literal,
                            offset: c_at,
                            kind: TokenKind::Slash,
                        }))
                    }
                }
                Started::IfEqualElse(yes, no) => {
                    //if self.remainder.starts_with('<') {
                    //    self.remainder = &self.remainder[1..];
                    //    self.byte += 1;
                    //    return Some(Ok(Token::LessEqual));
                    //} else {
                    //    return Some(Ok(Token::Less));
                    //}
                    self.remainder = self.remainder.trim_start(); // need to consider trim
                    let trimmed = c_onwards.len() - self.remainder.len() - 1; // get trimmed pos
                    self.byte += trimmed; // count trim in all bytes
                    if self.remainder.trim_start().starts_with('=') {
                        let span = &c_onwards[..c.len_utf8() + trimmed + 1]; // span mv beyond
                                                                             // cur_c, it's
                                                                             // everything text
                        self.remainder = &self.remainder.trim_start()[1..];
                        self.byte += 1;
                        Some(Ok(Token {
                            origin: span,
                            offset: c_at,
                            kind: yes,
                        }))
                    } else {
                        Some(Ok(Token {
                            origin: literal,
                            offset: c_at,
                            kind: no,
                        }))
                    }
                }
                Started::Number => {
                    let first_non_digit = c_onwards // NOTE: Already started parsing so first_non_digit should include c
                        .find(|c| !matches!(c, '.' | '0'..='9')) // NOTE: '_'cannot
                        // support
                        .unwrap_or(c_onwards.len());
                    let mut num_literal = &c_onwards[..first_non_digit];
                    let mut dotted = literal.splitn(3, '.');
                    match (dotted.next(), dotted.next(), dotted.next()) {
                        (Some(one), Some(two), Some(_)) => {
                            num_literal = &num_literal[..one.len() + 1 + two.len()];
                        }
                        (Some(one), Some(two), None) if two.is_empty() => {
                            num_literal = &num_literal[..one.len()];
                        }
                        _ => {}
                    }
                    let extra_bytes = num_literal.len() - c.len_utf8();
                    eprintln!("num literal: '{num_literal}'");
                    self.remainder = &self.remainder[extra_bytes..];
                    self.byte += extra_bytes;

                    let n = match num_literal.parse() {
                        Ok(n) => n,
                        Err(e) => {
                            return Some(Err(miette::miette! {
                                labels = vec![
                                    LabeledSpan::at(self.byte - num_literal.len()..self.byte, "this identifier literal"),
                                ],
                                "{e}",
                            }.with_source_code(self.whole.to_string())));
                        }
                    };
                    return Some(Ok(Token {
                        origin: num_literal,
                        offset: c_at,
                        kind: TokenKind::Number(n),
                    }));
                }
                Started::Ident => {
                    let first_non_identifier = c_onwards
                        .find(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
                        .unwrap_or(c_onwards.len());
                    let identifier_literal = &c_onwards[..first_non_identifier];
                    let extra_bytes = identifier_literal.len() - c.len_utf8();
                    eprintln!("identifier literal: '{identifier_literal}'");
                    self.remainder = &self.remainder[extra_bytes..];
                    self.byte += extra_bytes;

                    let kind = match identifier_literal {
                        "and" => TokenKind::And,
                        "class" => TokenKind::Class,
                        "else" => TokenKind::Else,
                        "false" => TokenKind::False,
                        "for" => TokenKind::For,
                        "fun" => TokenKind::Fun,
                        "if" => TokenKind::If,
                        "nil" => TokenKind::Nil,
                        "or" => TokenKind::Or,
                        "return" => TokenKind::Return,
                        "super" => TokenKind::Super,
                        "this" => TokenKind::This,
                        "true" => TokenKind::True,
                        "var" => TokenKind::Var,
                        "while" => TokenKind::While,
                        _ => TokenKind::Ident,
                    };

                    return Some(Ok(Token {
                        origin: identifier_literal,
                        offset: c_at,
                        kind,
                    }));
                }
                Started::String => {
                    // assign close double quote to end
                    if let Some(end) = self.remainder.find('"') {
                        // need to include literal(with open double quote) + end + terminator
                        let string_literal = &c_onwards[..end + 1 + 1];
                        // need to include end + terminator in byte offset
                        self.byte += end + 1;
                        // need to include end + terminator in remainder
                        self.remainder = &self.remainder[end + 1..];
                        Some(Ok(Token {
                            origin: string_literal,
                            offset: c_at,
                            kind: TokenKind::String,
                        }))
                    } else {
                        let err = StringTerminationError {
                            src: self.whole.to_string(),
                            err_span: SourceSpan::from(self.byte - c.len_utf8()..self.whole.len()),
                        };

                        // Consume all remainder of input as string
                        self.byte += self.remainder.len();
                        self.remainder = &self.remainder[self.remainder.len()..];

                        return Some(Err(err.into()));
                    }
                }
            };
        }
    }
}
