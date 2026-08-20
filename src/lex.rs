use miette::{Diagnostic, Error, LabeledSpan, SourceSpan};
use std::{borrow::Cow, fmt::Display};
use thiserror::Error;

// for f64, no Eq will be allowed
#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Comma,
    Colon,
    Semicolon,
    Dot,
    Bang,
    Slash,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BangEqual,
    EqualEqual,
    Star,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Plus,
    Minus,
    Ident,
    Number(f64),
    String,
    Comment,
    BlockComment,
    Var,
    Callee,
    While,
    For,
    Field,
    If,
    Else,
    Nil,
    Print,
    Return,
    Block,
    Class,
    Fn,
    True,
    False,
    And,
    This,
    Or,
    Super,
}

#[derive(Debug, Clone, PartialEq)]
// used to derive the literal from original
// Tokenization (raw str => literal ++ closing delimiter)
pub struct Token<'de> {
    pub origin: &'de str,
    pub kind: TokenKind,
    // used to derive literal in labelspan
    pub offset: usize,
}

// Special char e.g. BACKSLASH, QUOTES, NEWLINE, CR, TAB for token only string literal is
// relevant)
// lox has NO escape, so string CANNOT have quote, so ONLY match on the inner
impl<'de> Token<'de> {
    //pub fn escape(raw: &'de str) -> Cow<'de, &str> {
    //    todo!()
    //}
    pub fn unescape(raw: &'de str) -> Cow<'de, &'de str> {
        Cow::Owned(raw.trim_matches('"'))
    }
}

// Print out token as native representation
impl<'de> Display for Token<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let i = self.origin;
        match self.kind {
            TokenKind::Comma => write!(f, "COMMA {i} null"),
            TokenKind::Colon => write!(f, "COLON {i} null"),
            TokenKind::Semicolon => write!(f, "SEMICOLON {i} null"),
            TokenKind::Dot => write!(f, "DOT {i} null"),
            TokenKind::Bang => write!(f, "BANG {i} null"),
            TokenKind::Slash => write!(f, "SLASH {i} null"),
            TokenKind::Equal => write!(f, "EQUAL {i} null"),
            TokenKind::Greater => write!(f, "GREATER {i} null"),
            TokenKind::GreaterEqual => write!(f, "GREATER_EQUAL {i} null"),
            TokenKind::Less => write!(f, "LESS {i} null"),
            TokenKind::LessEqual => write!(f, "LESS_EQUAL {i} null"),
            TokenKind::Star => write!(f, "STAR {i} null"),
            TokenKind::LeftParen => write!(f, "LEFT_PAREN {i} null"),
            TokenKind::RightParen => write!(f, "RIGHT_PAREN {i} null"),
            TokenKind::LeftBrace => write!(f, "LEFT_BRACE {i} null"),
            TokenKind::RightBrace => write!(f, "RIGHT_BRACE {i} null"),
            TokenKind::Plus => write!(f, "PLUS {i} null"),
            TokenKind::Minus => write!(f, "MINUS {i} null"),
            TokenKind::Ident => write!(f, "IDENT {i} null"),
            TokenKind::Number(n) => write!(f, "NUM {i} {n} null"),
            TokenKind::String => write!(f, "STRING {} null", Token::unescape(i)),
            TokenKind::Comment => write!(f, "COMMENT {i} null"),
            TokenKind::BlockComment => write!(f, "BLOCKCOMMENT {i} null"),
            TokenKind::Print => write!(f, "PRINT {i} null"),
            TokenKind::Class => write!(f, "CLASS {i} null"),
            TokenKind::Fn => write!(f, "FN {i} null"),
            _ => Ok(()),
        }
    }
}

// the scanner
pub struct Lexer<'de> {
    // use to derive err msg
    entire: &'de str,
    // src (one past c)
    // inluding leading '\0'
    // always need to be keep track of
    remainder: &'de str,
    // keep track of the lexed token in literal
    cursor: usize,
    // keep track of the peeked token in original (None)
    peeked: Option<Result<Token<'de>, miette::Error>>,
}

impl<'de> Lexer<'de> {
    pub fn new(raw: &'de str) -> Self {
        Lexer {
            entire: raw,
            remainder: raw,
            cursor: 0,
            peeked: None,
        }
    }
    // return type align with parser (NO optional)
    pub fn expect(
        &mut self,
        expected: TokenKind,
        unexpected: &'de str,
    ) -> Result<Token<'de>, miette::Error> {
        self.expect_where(|next| next.kind == expected, unexpected)
    }
    // lookahead token might be expected, unexpected, or simply propagated err
    // chk: match!(...)
    pub fn expect_where(
        &mut self,
        mut chk: impl FnMut(&Token<'de>) -> bool,
        unexpected: &'de str,
    ) -> Result<Token<'de>, miette::Error> {
        match self.next() {
            Some(Ok(token)) if chk(&token) => Ok(token),
            Some(Ok(token)) => Err(miette::miette! {
                labels = vec![
                    // derive source after token
                    LabeledSpan::at(token.offset..token.offset + token.origin.len(), "this ident literal")
                ],
                help = format!("expected {token:?}"),
                "{unexpected}"
            }.with_source_code(self.entire.to_string())),
            Some(Err(e)) => Err(e),
            None => Err(Eof.into()),
        }
    }
    // for parsering to get the inner &T for lookahead char
    // peek at ZERO token
    // parser can advance the cursor (ptional)
    pub fn peek(&mut self) -> Option<&Result<Token<'de>, miette::Error>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }
        self.peeked = self.next();
        self.peeked.as_ref()
    }
}

// Map the valid lookahead char to token
// c as the (next) valid token
// the terminator is ONLY a valid cursor pos in sliced literal (but exlusive in literal as self)
impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        // need loop to keep track of c_onwards (literal scanner)
        loop {
            let mut chars = self.remainder.chars();
            let c = chars.next()?;
            // Used to derive literal (exclusive of c)
            let c_str = &self.remainder[..c.len_utf8()];
            // lexing from c up to remainder
            // Used to keep track of the sliding window of literal
            let c_onwards = &self.remainder[c.len_utf8()..];
            // used to derive lexing at token offset
            let c_at = self.cursor;

            let fitting = move |kind: TokenKind| {
                Some(Ok(Token {
                    origin: c_str,
                    kind,
                    offset: c_at,
                }))
            };

            enum Leading {
                String,
                Ident,
                Num,
                Slash,
                // if has '=' as leading or not
                // yes: assignment stmt (leading with '=')
                // no: logical expression (various)
                IfEqualElse(TokenKind, TokenKind),
            }

            // How to identify token
            let token = match c {
                ',' => return fitting(TokenKind::Comma),
                ':' => return fitting(TokenKind::Colon),
                ';' => return fitting(TokenKind::Semicolon),
                '.' => return fitting(TokenKind::Dot),
                '*' => return fitting(TokenKind::Star),
                '(' => return fitting(TokenKind::LeftParen),
                ')' => return fitting(TokenKind::RightParen),
                '{' => return fitting(TokenKind::LeftBrace),
                '}' => return fitting(TokenKind::RightBrace),
                '+' => return fitting(TokenKind::Plus),
                '-' => return fitting(TokenKind::Minus),
                '>' => Leading::IfEqualElse(TokenKind::GreaterEqual, TokenKind::Greater),
                '<' => Leading::IfEqualElse(TokenKind::LessEqual, TokenKind::Less),
                '=' => Leading::IfEqualElse(TokenKind::EqualEqual, TokenKind::Equal),
                '!' => Leading::IfEqualElse(TokenKind::BangEqual, TokenKind::Bang),
                'A'..='Z' | 'a'..='z' | '_' => Leading::Ident,
                '0'..='9' => Leading::Num,
                '"' => Leading::String,
                '/' => Leading::Slash,
                // for '\0' scan on (since Eof already handled)
                c if c.is_whitespace() => continue,
                // handling single token err in c_str
                _ => {
                    return Some(Err(SingleTokenError {
                        src: self.entire.to_string(),
                        token: c,
                        err_span: SourceSpan::from(self.cursor - c.len_utf8()..self.cursor),
                    }
                    .into()))
                }
            };

            // How to generate valid span literal (wo escape)
            // in most case ONLY need to consider null terminator
            // for string literal, need to consider opening quote
            match token {
                // heuristics: if can find open '/', there must be close one
                // verify if start with '/'
                // get the terminator (scanner need to know)
                // update cursor & remainder
                Leading::Slash => {
                    if self.remainder.starts_with('/') {
                        let terminator = self.remainder.find('\n').unwrap_or(self.remainder.len());
                        self.cursor += terminator;
                        self.remainder = &self.remainder[terminator..];
                        // if valid token, then scan on
                        continue;
                    } else {
                        return Some(Ok(Token {
                            origin: c_str,
                            kind: TokenKind::Slash,
                            offset: c_at,
                        }));
                    }
                }
                // heuristics: if can find end quote, there must be open quote (wo escape)
                // get the end quote (scanner need to know)
                // get literal
                // get the terminator (scanner need to know)
                Leading::String => {
                    if let Some(end_quote) = self.remainder.find('"') {
                        let terminator = self.remainder.find('\0').unwrap_or(self.remainder.len());
                        // count the end_quote + terminator in literal
                        let string_literal = &c_onwards[..end_quote + 2];
                        self.cursor += terminator;
                        // ONLY count the closing quote in remainder
                        self.remainder = &self.remainder[end_quote + 1..];
                        return Some(Ok(Token {
                            origin: string_literal,
                            kind: TokenKind::String,
                            offset: c_at,
                        }));
                    } else {
                        let err = StringTerminationError {
                            src: self.entire.to_string(),
                            err_span: SourceSpan::from(
                                self.cursor - c.len_utf8()..self.entire.len(),
                            ),
                        };
                        // consume all remainder
                        self.cursor += self.remainder.len();
                        self.remainder = &self.remainder[self.remainder.len()..];
                        return Some(Err(err.into()));
                    }
                }
                // get the 1st non_ident token in remainder
                // get ident literal (no terminator)
                // update valid extra bytes to peek(to keep track of where Ident may terminate)
                // update remainder & cursor (from extra bytes onwards)
                // match token kind on ident_literal to derive corresponding keyword kind
                // return token
                Leading::Ident => {
                    let first_non_ident = c_onwards
                        .find(|c| !matches!(c,'0'..='9' | 'A'..='Z' | 'a'..='z'))
                        .unwrap_or(c_onwards.len());
                    // exlude first_non_ident
                    let ident_literal = &c_onwards[..first_non_ident];
                    let valid_extra_bytes = ident_literal.len() - c.len_utf8();
                    self.remainder = &self.remainder[valid_extra_bytes..];
                    self.cursor += valid_extra_bytes;
                    let kind = match ident_literal {
                        "and" => TokenKind::And,
                        "or" => TokenKind::Or,
                        "var" => TokenKind::Var,
                        "class" => TokenKind::Class,
                        "return" => TokenKind::Return,
                        "print" => TokenKind::Print,
                        "super" => TokenKind::Super,
                        "this" => TokenKind::This,
                        "fn" => TokenKind::Fn,
                        "false" => TokenKind::False,
                        "true" => TokenKind::True,
                        "else" => TokenKind::Else,
                        "if" => TokenKind::If,
                        "for" => TokenKind::For,
                        "while" => TokenKind::While,
                        "nil" => TokenKind::Nil,
                        // if other case, should be ident
                        _ => TokenKind::Ident,
                    };

                    return Some(Ok(Token {
                        origin: ident_literal,
                        kind,
                        offset: c_at,
                    }));
                }
                // get the 1st_non_digit token in remainder
                // get & update num literal(int/float)
                // update remainder
                // update remainder & cursor (from extra bytes onwards)
                // since no else clause, need to use match to proprigate err in parsing
                // return token
                Leading::Num => {
                    let first_non_digit = c_onwards
                        .find(|c| !matches!(c, '0'..='9' | '.'))
                        .unwrap_or(c_onwards.len());
                    let mut num_literal = &c_onwards[..first_non_digit];
                    // handle float
                    let mut section = num_literal.splitn(3, '.');
                    match (section.next(), section.next(), section.next()) {
                        // 9. => will just capture 9 (truncate dot)
                        (Some(first), Some(""), None) => {
                            num_literal = &num_literal[..first.len()];
                        }
                        // 1.22.
                        // including dot pos
                        (Some(first), Some(second), Some(_)) => {
                            num_literal = &num_literal[..first.len() + 1 + second.len()];
                        }
                        // leave literal as-is
                        _ => {}
                    }
                    let valid_extra_bytes = num_literal.len() - c.len_utf8();
                    self.remainder = &self.remainder[valid_extra_bytes..];
                    self.cursor += valid_extra_bytes;

                    let n = match num_literal.parse() {
                        Ok(n) => n,
                        Err(e) => {
                            return Some(Err(miette::miette! {
                            labels = vec![
                                // derive literal up to lexer cursor
                                LabeledSpan::at(self.cursor - num_literal.len()..self.cursor, "this literal")
                            ],
                            "{e}"
                            }.with_source_code(self.entire.to_string())));
                        }
                    };

                    return Some(Ok(Token {
                        origin: num_literal,
                        kind: TokenKind::Number(n),
                        offset: c_at,
                    }));
                }
                // trim src
                // but need to keep track of trimmed pos (counted as byte peeked)
                // if start with '=', then should be assignment stmt, return full span text
                // update remainder, account for the leading '='
                // otherwise return literal
                Leading::IfEqualElse(yes, no) => {
                    self.remainder = self.remainder.trim_start();
                    let trimmed_pos = c_onwards.len() - self.remainder.len() - 1;
                    self.cursor += trimmed_pos;
                    // lead by '=' as assignment stmt
                    if self.remainder.trim_start().starts_with('=') {
                        // count '=' (since this is a token_pos)
                        let span = &c_onwards[..c.len_utf8() + trimmed_pos + 1];
                        // count '='
                        self.remainder = &self.remainder.trim_start()[1..];
                        return Some(Ok(Token {
                            origin: span,
                            kind: yes,
                            offset: c_at,
                        }));
                        // logical expression so parse on
                    } else {
                        return Some(Ok(Token {
                            origin: c_str,
                            kind: no,
                            offset: c_at,
                        }));
                    }
                }
            };
        }
    }
}

#[derive(Debug, Diagnostic, Error)]
#[error("unexpected end of file")]
pub struct Eof;

#[derive(Debug, Diagnostic, Error)]
#[error("unexpected token {token}")]
pub struct SingleTokenError {
    #[source_code]
    src: String,
    pub token: char,
    #[label = "this input char"]
    err_span: SourceSpan,
}

impl SingleTokenError {
    pub fn line(&self) -> usize {
        let until_recognized = &self.src[..=self.err_span.offset()];
        until_recognized.lines().count()
    }
}

#[derive(Debug, Diagnostic, Error)]
#[error("unterminated string")]
pub struct StringTerminationError {
    #[source_code]
    src: String,
    #[label = "this string literal"]
    err_span: SourceSpan,
}

//  used in main()
#[allow(dead_code, unused)]
impl StringTerminationError {
    pub fn line(&self) -> usize {
        let until_recognized = &self.src[..=self.err_span.offset()];
        until_recognized.lines().count()
    }
}
