mod expr;
mod kind;
mod lex;
mod operator;
mod parser;
mod pat;
mod stmt;
mod token;
mod ty;

pub use expr::*;
pub use kind::*;
pub use lex::*;
pub use operator::*;
pub use parser::*;
pub use pat::*;
pub use stmt::*;
pub use token::*;
pub use ty::*;

use crate::{
    ast::Module,
    diagnostic::{Reporter, SourceId},
};

pub fn module(reporter: &dyn Reporter, source: SourceId, input: &str, path: String) -> Module {
    let tokens = lex(reporter, source, input);
    let mut parser = Parser::new(reporter, &tokens);
    let stmts = stmts(&mut parser);

    Module { path, stmts }
}
