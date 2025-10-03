use crate::{
    lex::{Token, TokenKind},
    Lexer,
};
use miette::{Context, Diagnostic, Error, LabeledSpan};
use std::{borrow::Cow, fmt};
use thiserror::Error;

#[derive(Diagnostic, Debug, Error)]
#[error("unexpected EOF")]
pub struct Eof;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Op {
    // Op as Ast root displayed first
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
    And,
    Or,
    If,
    For,
    Print,
    Class,
    Fun,
    Var,
    While,
    Return,
    Group,
    Call,
    Field,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Plus => "+",
                Op::Minus => "-",
                Op::Star => "*",
                Op::Bang => "!",
                Op::Equal => "=",
                Op::EqualEqual => "==",
                Op::LessEqual => "<=",
                Op::GreaterEqual => ">=",
                Op::BangEqual => "!=",
                Op::Less => "-",
                Op::Greater => ">",
                Op::Slash => "/",
                Op::And => "and",
                Op::Or => "or",
                Op::If => "if",
                Op::For => "for",
                Op::Print => "print",
                Op::Class => "class",
                Op::Fun => "fun",
                Op::Call => "call",
                Op::Var => "var",
                Op::While => "while",
                Op::Return => "return",
                Op::Group => "group",
                Op::Field => "field",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom<'de> {
    // Single char & var
    String(Cow<'de, str>),
    Number(f64),
    Bool(bool),
    Ident(&'de str),
    Nil,
    This,
    Super,
}

// Format Atom into string
impl std::fmt::Display for Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::String(s) => write!(f, "\"{s}\""),
            Atom::Number(n) => {
                if *n == n.trunc() {
                    write!(f, "NUMBER {n}.0")
                } else {
                    write!(f, "NUMBER {n}")
                }
            }
            Atom::Nil => write!(f, "nil"),
            Atom::This => write!(f, "this"),
            Atom::Super => write!(f, "super"),
            Atom::Bool(b) => write!(f, "{b:?}"),
            Atom::Ident(i) => write!(f, "{i}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
// Raw text in expr(), w.o. operator
pub enum Ast<'de> {
    Atom(Atom<'de>),
    Cons(Op, Vec<Ast<'de>>),
    Fun {
        name: Atom<'de>,
        parameters: Vec<Token<'de>>, // Vec
        body: Box<Ast<'de>>,         // Whole fn
    },
    Call {
        arguments: Vec<Ast<'de>>, // Vec of Ast
        callee: Box<Ast<'de>>,    // Whole fn
    },
    If {
        cond: Box<Token<'de>>,     // Box as a intergral
        yes: Box<Ast<'de>>,        // Whole fn
        no: Option<Box<Ast<'de>>>, // Whole fn (optional)
    },
}

// Format Ast into string
impl std::fmt::Display for Ast<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ast::Atom(i) => write!(f, "{}", i),
            Ast::Cons(root, branch) => {
                write!(f, "({root}")?;
                for s in branch {
                    write!(f, "{s}")? // Parsing might fail
                }
                write!(f, ")") // No parsing
            }
            Ast::Call { arguments, callee } => {
                write!(f, "({callee}")?;
                for a in arguments {
                    write!(f, "{a}")?;
                }
                write!(f, ")")
            }
            Ast::If { cond, yes, no } => {
                write!(f, "(if {cond} {yes}")?;
                if let Some(no) = no {
                    write!(f, "{no}")?
                }
                write!(f, ")")
            }

            Ast::Fun {
                name,
                parameters,
                body,
            } => {
                write!(f, "def {name}")?;
                for p in parameters {
                    write!(f, "{p}")?;
                }
                write!(f, "{body}")
            }
        }
    }
}

pub struct Parser<'de> {
    // Parser to wrap lexer
    whole: &'de str,
    lexer: Lexer<'de>,
}

impl<'de> Parser<'de> {
    pub fn new(input: &'de str) -> Self {
        Parser {
            whole: input,
            lexer: Lexer::new(input),
        }
    }

    // Include all stmt elements for parsing {}
    pub fn parse_block(mut self) -> Result<Ast<'de>, Error> {
        self.lexer.expect(TokenKind::LeftBrace, "missing {")?;
        let block = self.parse_stmt_within(0)?;
        Ok(block)
    }

    pub fn parse_fun_call(&mut self) -> Result<Ast<'de>, Error> {
        todo!()
    }
    pub fn parse_expr(mut self) -> Result<Ast<'de>, Error> {
        self.parse_expr_within(0)
    }
    pub fn parse_expr_within(&mut self, min_bp: u8) -> Result<Ast<'de>, Error> {
        let lhs = match self.lexer.next() {
            Some(Ok(token)) => token,
            None => return Ok(Ast::Atom(Atom::Nil)),
            Some(Err(e)) => return Err(e).wrap_err("on lhs"),
        };
        let mut lhs = match lhs {
            Token {
                origin, // Origin is for literal repre
                kind: TokenKind::String,
                ..
            } => return Ok(Ast::Atom(Atom::String(Token::unescape(origin)))),
            Token {
                origin,
                kind: TokenKind::Ident,
                ..
            } => return Ok(Ast::Atom(Atom::Ident(origin))),
            Token {
                kind: TokenKind::True,
                ..
            } => return Ok(Ast::Atom(Atom::Bool(true))),
            Token {
                kind: TokenKind::False,
                ..
            } => return Ok(Ast::Atom(Atom::Bool(false))),
            Token {
                kind: TokenKind::Nil,
                ..
            } => return Ok(Ast::Atom(Atom::Nil)),
            Token {
                kind: TokenKind::Number(n),
                ..
            } => return Ok(Ast::Atom(Atom::Number(n))),
            Token {
                kind: TokenKind::Super,
                ..
            } => return Ok(Ast::Atom(Atom::Super)),
            Token {
                kind: TokenKind::This,
                ..
            } => return Ok(Ast::Atom(Atom::This)),
            Token {
                kind: TokenKind::LeftParen,
                ..
            } => {
                let lhs = self
                    .parse_expr_within(0)
                    .wrap_err("Within a bracket expr")?;
                self.lexer
                    .expect(TokenKind::RightParen, "Unexpected end")
                    .wrap_err("to the bracket expr")?;
                Ast::Cons(Op::Group, vec![lhs])
            }
        };
    }

    pub fn parse(mut self) -> Result<Ast<'de>, Error> {
        self.parse_stmt_within(0)
    }

    // Parser to consume token so need mut self
    pub fn parse_stmt_within(&mut self, min_bp: u8) -> Result<Ast<'de>, Error> {
        // match on lexer.next to lhs
        let lhs = match self.lexer.next() {
            Some(Ok(token)) => token,
            None => return Ok(Ast::Atom(Atom::Nil)),
            Some(Err(e)) => return Err(e).wrap_err("on lhs"),
        };
        // Match on valid token & return Ast-formatted ver
        let mut lhs = match lhs {
            // Atom
            Token {
                // identifier
                origin,
                kind: TokenKind::Ident,
                ..
            } => return Ok(Ast::Atom(Atom::Ident(origin))),
            Token {
                // Super
                kind: TokenKind::Super,
                ..
            } => return Ok(Ast::Atom(Atom::Super)),
            Token {
                // This
                kind: TokenKind::This,
                ..
            } => return Ok(Ast::Atom(Atom::This)),
            Token {
                // Paren
                kind: TokenKind::LeftParen,
                ..
            } => {
                todo!()
            }
            // TBD: Prefix
            Token {
                kind: TokenKind::Minus | TokenKind::Bang | TokenKind::Return | TokenKind::Print,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::Minus => Op::Minus,
                    TokenKind::Bang => Op::Bang,
                    TokenKind::Return => Op::Return,
                    TokenKind::Print => Op::Print,
                };
                let ((), r_bp) = prefix_binding_power(op);
                let rhs = self.parse_expr_within(r_bp).wrap_err("Cannot parse RHS")?;
                Ast::Cons(op, vec![rhs])
            }
            //TBD: Prefix (double)
            Token {
                kind: TokenKind::For | TokenKind::While,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::For => Op::For,
                    TokenKind::While => Op::While,
                    _ => unreachable!("OB"),
                };
                let init = self.parse_expr_within(0).wrap_err("in for loop")?;
                let cond = self.parse_expr_within(0).wrap_err("in for loop")?;
                let incr = self.parse_expr_within(0).wrap_err("in for loop")?;
                let block = self.parse_expr_within(0).wrap_err("in for loop")?;
                Ast::Cons(Op::For | Op::While, vec![cond, block])
            }

            Token {
                kind: TokenKind::Var | TokenKind::Class,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::Var => Op::Var,
                    TokenKind::Class => Op::Class,
                    _ => unreachable!("OB"),
                };
                let first_cons = self
                    .parse_expr_within(min_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;

                if lhs.kind == TokenKind::Var {
                    if !matches!(first_cons, Ast::Atom(Atom::Ident(_))) {
                        todo!()
                    }
                }

                match self.lexer.next() {
                    Some(Ok(token)) if token.kind == TokenKind::Equal => {}
                    Some(Ok(token)) => {
                        return Err(miette::miette! {
                            labels = vec![
                                LabeledSpan::at(token.offset..token.offset + token.origin.len(), "this identifier literal")
                            ],
                           help = "Expected =",
                           "Malform var assignment",
                        }.with_source_code(self.whole.to_string())).wrap_err("Target operator error")?;
                    }
                    Some(Err(e)) => {
                        return Err(e).wrap_err("Target operator error")?;
                    }
                    None => {
                        return Err(Eof).wrap_err("Target operator error")?;
                    }
                }

                let second_cons = self
                    .parse_expr_within(min_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                Ast::Cons(op, vec![first_cons, second_cons])
            }
            //TBD: Prefix (triple)
            Token {
                kind: TokenKind::Fun,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::Fun => Op::Fun,
                    _ => unreachable!("OB"),
                };
                let first_cons = self
                    .parse_expr_within(in_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                let second_cons = self
                    .parse_expr_within(min_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                let third_cons = self
                    .parse_expr_within(in_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                Ast::Cons(op, vec![first_cons, second_cons, third_cons])
            }
            //TBD: Prefix (quardruple)
            Token {
                kind: TokenKind::For | TokenKind::If,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::Var => Op::Var,
                    TokenKind::Fun => Op::Fun,
                    _ => unreachable!("OB"),
                };
                let first_cons = self
                    .parse_expr_within(min_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                let second_cons = self
                    .parse_expr_within(min_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                let third_cons = self
                    .parse_expr_within(min_bp)
                    .wrap_err_with(|| format!("In {op:?} expression"))?;
                Ast::Cons(op, vec![first_cons, second_cons, third_cons])
            }

            // Group
            Token {
                kind: TokenKind::LeftParen | TokenKind::LeftBrace,
                ..
            } => {
                let terminator = match lhs.kind {
                    TokenKind::LeftParen => TokenKind::RightParen,
                    TokenKind::LeftBrace => TokenKind::RightBrace,
                    _ => unreachable!("OB"),
                };
                let lhs = self
                    .parse_expr_within(min_bp)
                    .wrap_err("bracket expression")?;

                // find the terminator in expr
                match self.lexer.next() {
                    Some(Ok(token)) if token.kind == terminator => {}
                    Some(Ok(token)) => {
                        return Err(miette::miette! {
                            labels = vec![
                                LabeledSpan::at(token.offset..token.offset + token.origin.len(), "this identifier literal")
                            ],
                           help = "Expecting {terminator:?}",
                           "Unexpected terminator",
                        }.with_source_code(self.whole.to_string())).wrap_err("Target operator error")?;
                    }
                    Some(Err(e)) => {
                        return Err(e).wrap_err("Target operator error")?;
                    }
                    None => {
                        return Err(Eof).wrap_err("Target operator error")?;
                    }
                }
                lhs
            }
            Token {
                kind: TokenKind::LeftParen | TokenKind::LeftBrace,
                ..
            } => {
                let terminator = match lhs.kind {
                    TokenKind::LeftParen => TokenKind::RightParen,
                    TokenKind::LeftBrace => TokenKind::RightBrace,
                    _ => unreachable!("OB"),
                };
                let lhs = self
                    .parse_expr_within(min_bp)
                    .wrap_err("bracket expression")?;
                match self.lexer.next() {
                    Some(Ok(token)) if token.kind == terminator => {}
                    Some(Ok(token)) => {
                        return Err(miette::miette! {
                            labels = vec![
                                LabeledSpan::at(token.offset..token.offset + token.origin.len(), "this identifier literal")
                            ],
                           help = "Expected {terminator:?}",
                           "Unexpected terminator",
                        }.with_source_code(self.whole.to_string())).wrap_err("Target operator error")?;
                    }
                    Some(Err(e)) => {
                        return Err(e).wrap_err("Target operator error")?;
                    }
                    None => {
                        return Err(Eof).wrap_err("Target operator error")?;
                    }
                }
                lhs
            }
            Token {
                kind: TokenKind::LeftParen | TokenKind::LeftBrace,
                ..
            } => {
                let terminator = match lhs.kind {
                    TokenKind::LeftParen => TokenKind::RightParen,
                    TokenKind::LeftBrace => TokenKind::RightBrace,
                    _ => unreachable!("OB"),
                };
                let lhs = self
                    .parse_expr_within(min_bp)
                    .wrap_err("bracket expression")?;
                match self.lexer.next() {
                    Some(Ok(token)) if token.kind == terminator => {}
                    Some(Ok(token)) => {
                        return Err(miette::miette! {
                            labels = vec![
                                LabeledSpan::at(token.offset..token.offset + token.origin.len(), "this identifier literal")
                            ],
                           help = "Expected {terminator:?}",
                           "Unexpected terminator",
                        }.with_source_code(self.whole.to_string())).wrap_err("Target operator error")?;
                    }
                    Some(Err(e)) => {
                        return Err(e).wrap_err("Target operator error")?;
                    }
                    None => {
                        return Err(Eof).wrap_err("Target operator error")?;
                    }
                }
                lhs
            }
        };

        loop {
            let op = self.lexer.peek();
            if op.is_err() {
                return Err(self
                    .lexer
                    .next()
                    .expect("checked some")
                    .expect_err("check err"));
            }
            let op = match op.map(|res| res.expect("handle above err")) {
                None => break,
                Some(Token {
                    kind:
                        TokenKind::LeftParen
                        | TokenKind::Dot
                        | TokenKind::Minus
                        | TokenKind::Plus
                        | TokenKind::Star
                        | TokenKind::BangEqual
                        | TokenKind::EqualEqual
                        | TokenKind::LessEqual
                        | TokenKind::GreaterEqual
                        | TokenKind::Less
                        | TokenKind::Greater
                        | TokenKind::Slash
                        | TokenKind::And
                        | TokenKind::Or,
                    ..
                }) => op,
                t => panic!("bad token: {:?}", t),
            };

            if let Some((l_bp, ())) = postfix_binding_power(op) {
                if l_bp < min_bp {
                    break;
                }
                lexer.next();

                lhs = if op == '[' {
                    assert_eq!(lexer.next(), Token::Op(']'));
                    Ast::Cons(op, vec![lhs, rhs])
                } else {
                    Ast::Cons(op, vec![lhs])
                };
                continue;
            }

            if let Some((l_bp, r_bp)) = infix_binding_power(op) {
                if l_bp < min_bp {
                    break;
                }
                lexer.next();

                lhs = if op == '?' {
                    assert_eq!(lexer.next(), Token::Op(':'));
                    Ast::Cons(op, vec![lhs, mhs, rhs])
                } else {
                    Ast::Cons(op, vec![lhs, rhs])
                };
                continue;
            }
        }
    }
}

// Pos of operator in proportion to operands
fn prefix_binding_power(op: Op) -> ((), u8) {
    match op {
        Op::Return | Op::Print => ((), 1),
        Op::Bang | Op::Minus => ((), 11),
        _ => panic!("Bad op: {:?}", op),
    }
}

fn infix_binding_power(op: Op) -> Option<(u8, u8)> {
    let res = match op {
        Op::Equal => (2, 1),
        Op::EqualEqual | Op::GreaterEqual | Op::LessEqual | Op::Greater | Op::Less => (5, 6),
        Op::Plus | Op::Minus => (7, 8),
        Op::Star | Op::Slash => (9, 10),
        Op::Field => (16, 15),
        _ => return None,
    };
    Some(res)
}

fn postfix_binding_power(op: Op) -> Option<(u8, ())> {
    let res = match op {
        Op::Call => (13, ()),
        _ => return None,
    };
    Some(res)
}
