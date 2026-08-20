use crate::{
    lex::{Token, TokenKind},
    Lexer,
};
use miette::{Error, LabeledSpan, WrapErr};
use std::{borrow::Cow, fmt::Display};

// arithmetic op: (+, -, *, /, %, **, ++, --)
// logical op: (&&, ||, NOT)
// relational op: (>, <, >=, <=, ==, !=)
// bitwise op: (&, |, ^, <<, >>, ~)
// others: (., callee)
#[derive(Debug, Clone)]
pub enum Op {
    Plus,
    Minus,
    Star,
    Slash,
    Modulo,
    Exponential,
    Increment,
    Decrement,
    And,
    Or,
    Not,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    EqualEqual,
    BangEqual,
    Bang,
    Dot,
    Return,
    Print,
    Call,
    LeftShift,
    RightShift,
    Unit,
    For,
    While,
    Class,
    Var,
    Fn,
    If,
    Field,
    Nil,
}

// every token is &str in Ast (non-cap)
// support binary op
impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Plus => "+",
                Op::Minus => "-",
                Op::Star => "*",
                Op::Slash => "/",
                Op::Modulo => "%",
                Op::Exponential => "**",
                Op::Increment => "++",
                Op::Decrement => "--",
                Op::And => "and",
                Op::Or => "or",
                Op::Not => "not",
                Op::Greater => ">",
                Op::Less => "<",
                Op::GreaterEqual => ">=",
                Op::LessEqual => "<=",
                Op::EqualEqual => "==",
                Op::BangEqual => "!=",
                Op::Bang => "!",
                Op::Dot => ".",
                Op::Return => "return",
                Op::Print => "print",
                Op::Call => "call",
                Op::LeftShift => "<<",
                Op::RightShift => ">>",
                Op::For => "for",
                Op::While => "while",
                Op::Class => "class",
                Op::Var => "var",
                Op::Fn => "fn",
                Op::If => "if",
                Op::Field => ".",
                Op::Nil => "",
                Op::Unit => "group",
            }
        )
    }
}

// typical word, collection of tokens (including keyword)
// num as f64
// callee is special
#[derive(Debug, Clone, PartialEq)]
pub enum Atom<'de> {
    Ident(&'de str),
    String(Cow<'de, &'de str>),
    True,
    False,
    Num(f64),
    Var,
    While,
    For,
    Field,
    If,
    Else,
    Nil,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Bang,
    Minus,
    Bool(bool),
}

// for num, need to consider x.0
// no escape to consider for string in Display
impl<'de> Display for Atom<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Ident(i) => write!(f, "{i}"),
            Atom::String(s) => write!(f, "\"{s}\""),
            Atom::True => write!(f, "true"),
            Atom::False => write!(f, "false"),
            Atom::Num(n) => {
                // int ONLY
                if *n == n.trunc() {
                    write!(f, "{n}.0")
                } else {
                    write!(f, "{n}")
                }
            }
            Atom::Var => write!(f, "var"),
            Atom::While => write!(f, "while"),
            Atom::For => write!(f, "for"),
            Atom::Field => write!(f, "field"),
            Atom::If => write!(f, "if"),
            Atom::Else => write!(f, "else"),
            Atom::Nil => write!(f, "nil"),
            Atom::LeftBrace => write!(f, "{{"),
            Atom::RightBrace => write!(f, "}}"),
            Atom::LeftParen => write!(f, "(("),
            Atom::RightParen => write!(f, "))"),
            Atom::Bang => write!(f, "!"),
            Atom::Minus => write!(f, "-"),
            Atom::Bool(b) => write!(f, "{b}"),
        }
    }
}

// "comma" isnon-exsistent in Ast
// "paren" will be eaten by parser & printed by Disaply
// cons is op + operand(lhs/rhs),  used for grouping expr in paran (special case)
// call is special Ast pattern: callee(paran_list)
// No Option used in Ast
#[derive(Debug, Clone)]
pub enum Ast<'de> {
    Atom(Atom<'de>),
    Cons(Op, Vec<Ast<'de>>),
    Fn {
        name: Atom<'de>,
        // growable as per callee
        // in Ast form
        // MUST align with return type of parse_fn_args()
        paran_list: Vec<Ast<'de>>,
        definition_block: Box<Ast<'de>>,
    },
    Call {
        // fixed (use literal)
        // align with fn (acceptable)
        callee: Box<Ast<'de>>,
        arguments: Vec<Ast<'de>>,
    },
    If {
        // fixed
        predicate: Box<Ast<'de>>,
        exiting_block: Box<Ast<'de>>,
        exit_block: Option<Box<Ast<'de>>>,
    },
}

// parsing of root/op might fail
impl<'de> Display for Ast<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Atom(i) => write!(f, "{i}"),
            // TBD: Group
            Ast::Cons(root, branch) => {
                write!(f, "{root}")?;
                for s in branch {
                    write!(f, "{:?}", s)?
                }
                write!(f, ")")
            }
            Ast::Fn {
                name,
                paran_list,
                definition_block,
            } => {
                write!(f, "fn {name}")?;
                for paran in paran_list {
                    write!(f, "{paran}")?
                }
                write!(f, "{definition_block}")
            }
            Ast::Call { arguments, callee } => {
                for arg in arguments {
                    write!(f, "{arg}")?
                }
                write!(f, "{callee}")?;
                write!(f, ")")
            }
            Ast::If {
                predicate,
                exiting_block,
                exit_block,
            } => {
                let _ = write!(f, "if {predicate} {exiting_block}");
                if let Some(exit_block) = exit_block {
                    write!(f, "{:?}", exit_block)?
                }
                write!(f, "if {predicate} {exiting_block} {:?}", exit_block)
            }
        }
    }
}

pub struct Parser<'de> {
    pub span: &'de str,
    pub lexer: Lexer<'de>,
}

// parse all return Result<Ast>
// NOT omit lifetime annotation for return type
// callee should NOT use mut self to give away ownership to the caller (while caller can as builder)
impl<'de> Parser<'de> {
    pub fn new(input: &'de str) -> Self {
        Parser {
            span: input,
            lexer: Lexer::new(input),
        }
    }
    // derive lhs/rhs/args_list, which starts after keyword
    // next() will eat the left_paren
    pub fn parse_expr(&mut self, _min_bp: u8) -> Result<Ast<'de>, Error> {
        let lhs = match self.lexer.next() {
            Some(Ok(token)) => token,
            None => return Ok(Ast::Atom(Atom::Nil)),
            Some(Err(e)) => return Err(e),
        };
        let lhs = match lhs {
            Token {
                origin,
                kind: TokenKind::Ident,
                ..
            } => Ast::Atom(Atom::Ident(origin)),
            Token {
                origin,
                kind: TokenKind::String,
                ..
            } => Ast::Atom(Atom::String(Token::unescape(origin))),
            Token {
                kind: TokenKind::True,
                ..
            } => Ast::Atom(Atom::Bool(true)),
            Token {
                kind: TokenKind::False,
                ..
            } => Ast::Atom(Atom::Bool(false)),
            Token {
                kind: TokenKind::Number(n),
                ..
            } => Ast::Atom(Atom::Num(n)),
            Token {
                kind: TokenKind::Var,
                ..
            } => Ast::Atom(Atom::Var),
            // No need to map callee
            //Token {
            //    origin,
            //    kind: TokenKind::Callee,
            //} => return Ok(Ast::Atom(Atom::Callee)),
            Token {
                kind: TokenKind::While,
                ..
            } => Ast::Atom(Atom::While),
            Token {
                kind: TokenKind::For,
                ..
            } => Ast::Atom(Atom::For),
            Token {
                kind: TokenKind::Field,
                ..
            } => Ast::Atom(Atom::Field),
            Token {
                kind: TokenKind::If,
                ..
            } => Ast::Atom(Atom::If),
            Token {
                kind: TokenKind::Else,
                ..
            } => Ast::Atom(Atom::Else),
            Token {
                kind: TokenKind::Nil,
                ..
            } => Ast::Atom(Atom::Nil),
            Token {
                kind: TokenKind::LeftParen,
                ..
            } => {
                let lhs = self.parse_expr(1)?;
                let _ = self
                    .lexer
                    .expect(TokenKind::RightParen, "in this expression");
                Ast::Cons(Op::Unit, vec![lhs])
            }
            _ => unreachable!("abc"),
        };
        Ok(lhs)
    }
    // everything start being the `lhs`
    // paren is associated with lhs
    // op need arbitration, to be derived from lhs
    // this is a callee (mut self as more useful than &)
    pub fn parse_stmt(&mut self, min_bp: u8) -> Result<Ast<'de>, Error> {
        let lhs = match self.lexer.next() {
            Some(Ok(token)) => token,
            None => return Ok(Ast::Atom(Atom::Nil)),
            Some(Err(e)) => return Err(e),
        };
        let mut lhs = match lhs {
            // special case 0: Unit
            Token {
                kind: TokenKind::LeftParen,
                ..
            } => {
                let lhs = self.parse_expr(1)?;
                let _ = self
                    .lexer
                    .expect(TokenKind::RightParen, "in this expression");
                Ast::Cons(Op::Unit, vec![lhs])
            }
            // special case I: unary prefix (lhs is always 0 for bp)
            // return rhs
            Token {
                kind: TokenKind::Print | TokenKind::Return,
                ..
            } => {
                let op = match lhs.kind {
                    TokenKind::Print => Op::Print,
                    TokenKind::Return => Op::Return,
                    _ => unreachable!(),
                };
                // fn prefix_bp(op: Op) -> Option<((), u8)>
                let ((), r_bp) = prefix_bp(op);
                let rhs = self.parse_expr(r_bp)?;
                Ast::Cons(Op::Print, vec![rhs])
            }
            // for (i=0; i < 5; i++) {}
            Token {
                kind: TokenKind::For,
                ..
            } => {
                let init = self.parse_expr(1)?;
                let _ = self.lexer.expect(TokenKind::Semicolon, "in this lhs");
                let cond = self.parse_expr(1)?;
                let _ = self.lexer.expect(TokenKind::Semicolon, "in this lhs");
                let incre = self.parse_expr(1)?;
                let _ = self.lexer.expect(TokenKind::RightParen, "in this lhs");
                let block = self.parse_block()?;
                Ast::Cons(Op::For, vec![init, cond, incre, block])
            }
            // while (i < 5) {}
            Token {
                kind: TokenKind::While,
                ..
            } => {
                let cond = self.parse_expr(1)?;
                let _ = self.lexer.expect(TokenKind::RightParen, "in this lhs");
                let block = self.parse_block()?;
                Ast::Cons(Op::While, vec![cond, block])
            }
            // special case II: class | var
            // need assert! (same with fn)
            // ident is token literal
            Token {
                kind: TokenKind::Class,
                ..
            } => {
                let token = self.lexer.expect(TokenKind::Ident, "in this lhs")?;
                assert!(token.kind == TokenKind::Ident);
                let ident = Ast::Atom(Atom::Ident(token.origin));
                if lhs.kind == TokenKind::Var {
                    let _ = self.lexer.expect(TokenKind::Equal, "in this lhs");
                }

                let block = self.parse_block()?;
                Ast::Cons(Op::Class, vec![ident, block])
            }
            Token {
                kind: TokenKind::Var,
                ..
            } => {
                let token = self.lexer.expect(TokenKind::Ident, "in this lhs")?;
                assert!(token.kind == TokenKind::Ident);
                let ident = Ast::Atom(Atom::Ident(token.origin));
                let _ = self.lexer.expect(TokenKind::Equal, "in this lhs");
                let block = self.parse_block()?;
                Ast::Cons(Op::Var, vec![ident, block])
            }
            // special case III: fn
            // need to derive args_list (peek + check end)
            // need to derive token to break
            Token {
                kind: TokenKind::Fn,
                ..
            } => {
                let token = self.lexer.expect(TokenKind::Ident, "in this lhs")?;
                assert!(token.kind == TokenKind::Ident);
                let ident = Atom::Ident(token.origin);
                let _paran_list = self.parse_fn_args();
                let _ = self.lexer.expect(TokenKind::LeftParen, "in this lhs");
                let block = self.parse_block()?;
                if lhs.kind == TokenKind::Var {
                    let _ = self.lexer.expect(TokenKind::Equal, "in this lhs");
                }
                let mut args_list = vec![];
                if matches!(
                    self.lexer.peek(),
                    Some(Ok(Token {
                        kind: TokenKind::RightParen,
                        ..
                    }))
                ) {
                } else {
                    loop {
                        let argument = self.parse_expr(1)?;
                        args_list.push(argument);
                        let token = self.lexer.expect_where(
                            |token| matches!(token.kind, TokenKind::Comma | TokenKind::RightParen),
                            "continue",
                        )?;
                        // until run into ')'
                        if token.kind == TokenKind::RightParen {
                            break;
                        }
                    }
                }
                Ast::Fn {
                    name: ident,
                    paran_list: args_list,
                    definition_block: Box::new(block),
                }
            }
            // special case IV: if
            // otherwiser init to None
            // if run into else, return nothing (if not parse on)
            Token {
                kind: TokenKind::If,
                ..
            } => {
                let _ = self.lexer.expect(TokenKind::LeftParen, "in this lhs");
                let cond = self.parse_expr(1)?;
                let _ = self.lexer.expect(TokenKind::RightParen, "in this lhs");
                let block = self.parse_block()?;
                let mut otherwise = None;
                if matches!(
                    self.lexer.peek(),
                    Some(Ok(Token {
                        kind: TokenKind::Else,
                        ..
                    }))
                ) {
                    // TODO
                    otherwise = Some(self.parse_block().wrap_err("in body of else")?);
                } else {
                    self.lexer.next();
                }
                Ast::If {
                    predicate: Box::new(cond),
                    exiting_block: Box::new(block),
                    exit_block: otherwise.map(Box::new),
                }
            }
            _ => unreachable!("abc"),
        };
        // handle infix & postfix
        // need loop to keep track of c_onwards (literal scanner)
        loop {
            let op = self.lexer.peek();
            // 2 special cases: call + access
            let op = match op.map(|res| res.as_ref().expect("handle err")) {
                None => break,
                Some(Token {
                    kind: TokenKind::LeftParen,
                    ..
                }) => Op::Call,
                Some(Token {
                    kind: TokenKind::Dot,
                    ..
                }) => Op::Field,
                Some(token) => return Err(miette::miette! {
                labels = vec![
                    // derive source after token
                    LabeledSpan::at(token.offset..token.offset + token.origin.len(), "this ident literal")
                ],
                help = format!("expected {token:?}"),
                "expect operator"
            }.with_source_code(self.span.to_string())),
            };
            // call
            if let Some((l_bp, ())) = postfix_bp(&op) {
                if l_bp < min_bp {
                    break;
                }
                self.lexer.next();
                lhs = match op {
                    Op::Call => Ast::Call {
                        arguments: self.parse_fn_args()?,
                        callee: Box::new(lhs),
                    },
                    _ => Ast::Cons(op.clone(), vec![lhs]),
                };
            }
            // access
            if let Some((l_bp, r_bp)) = infix_bp(&op) {
                if l_bp < min_bp {
                    break;
                }
                self.lexer.next();
                let rhs = self.parse_expr(r_bp)?;
                return Ok(Ast::Cons(op, vec![lhs, rhs]));
            }
        }
        Ok(lhs)
    }
    // block is a combination of stmts
    // block is identified by '{' in parse_stmt()
    pub fn parse_block(&mut self) -> Result<Ast<'de>, Error> {
        let _ = self.lexer.expect(TokenKind::LeftBrace, "missing {");
        let block = self.parse_stmt(1)?;
        let _ = self.lexer.expect(TokenKind::RightBrace, "missing {");
        Ok(block)
    }
    // Need loop
    // args_list is identified by ')'
    pub fn parse_fn_args(&mut self) -> Result<Vec<Ast<'de>>, Error> {
        let mut args_list = vec![];
        if matches!(
            self.lexer.peek(),
            Some(Ok(Token {
                kind: TokenKind::RightParen,
                ..
            }))
        ) {
        } else {
            loop {
                let argument = self.parse_expr(1)?;
                args_list.push(argument);
                let token = self.lexer.expect_where(
                    |token| matches!(token.kind, TokenKind::Comma | TokenKind::RightParen),
                    "continue",
                )?;
                if token.kind == TokenKind::RightParen {
                    break;
                }
            }
        }
        Ok(args_list)
    }
    // used in main
    pub fn parse(mut self) -> Result<Ast<'de>, Error> {
        self.parse_stmt(0)
    }
}

// take Op and return bp (NO Option)
// priority in expression should never be equal between (lhs, rhs)
// always have to lookahead over entire span to identify if a drop to decide on priority (but NOT
// drop below min_bp: 1)
// prefix api has NO Option<> since will always bind rhs
fn prefix_bp(op: Op) -> ((), u8) {
    let op = match op {
        Op::Return | Op::Print => ((), 1),
        Op::Bang => ((), 10),
        Op::Minus => ((), 9),
        _ => panic!("Oops"),
    };
    op
}
// NOT always bind both lhs/rhs
fn infix_bp(op: &Op) -> Option<(u8, u8)> {
    let op = match op {
        Op::Plus => (1, 2),
        Op::Minus => (3, 4),
        Op::Star => (5, 6),
        Op::Slash => (7, 8),
        Op::Dot => (11, 12),
        _ => panic!("Oops"),
    };
    Some(op)
}
// NOT always bind lhs
fn postfix_bp(op: &Op) -> Option<(u8, ())> {
    let op = match op {
        Op::Call => (13, ()),
        _ => panic!("Oops"),
    };
    Some(op)
}
