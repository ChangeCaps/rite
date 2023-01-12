# Rite
Rite is a programming

# Compiler
The compiler is separated into a few different steps.

## Parsing
The first step of any compilation is creating an [`AST (abstract syntax tree)`](crates/ritec-ast)
this is done by first converting the input into a [`TokenStream`], this is done by the [`Lexer`].
Thereafter the [`TokenStream`] is parsed into an [`AST`].

## Lowering to IR
After parsing, the [`AST`] is lowered into [`IR (intermediary representation)`](crates/ritec-ir).

### Type inference

### Validation

## Codegen
