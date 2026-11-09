#![allow(clippy::module_inception)]

pub mod ast;
pub mod diagnostic;
pub mod ir;
pub mod lower;
pub mod parse;
pub mod rename;
pub mod runtime;
pub mod typecheck;
pub mod var;
