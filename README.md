# Rite
Rite is a programming

# Compiler
The compiler is separated into a few different steps.

## Parsing
The first step of any compilation is creating an [`AST (abstract syntax tree)`](crates/ritec-ast)
this is done by first converting the input into a [`TokenStream`](crates/ritec-parser), this is done by the [`Lexer`](crates/ritec-parser).
Thereafter the [`TokenStream`](crates/ritec-parser) is parsed into an [`AST`](crates/ritec-ast).

## Lowering to IR
After parsing, the [`AST`](crates/ritec-ast) is lowered into [`IR (intermediary representation)`](crates/ritec-ir).

### Type inference

### Validation

## Codegen
