use crate::{
    ast::{AppTy, ErrorTy, LamTy, Parse, TyTy, TypeLit, VarTy},
    parse::{Parser, Token},
};

pub fn is_ty(token: Token) -> bool {
    matches!(token, Token::Ident(..) | Token::Star)
}

pub fn ty(parser: &mut Parser<'_>) -> TypeLit<Parse> {
    lambda_ty(parser)
}

pub fn lambda_ty(parser: &mut Parser<'_>) -> TypeLit<Parse> {
    let mut ty = apply_ty(parser);

    while parser.peek() == Token::Arrow {
        parser.expect(Token::Arrow);

        let output = apply_ty(parser);
        let span = ty.span().join(output.span());

        ty = TypeLit::Lam(LamTy {
            input: Box::new(ty),
            output: Box::new(output),
            span,
        });
    }

    ty
}

pub fn apply_ty(parser: &mut Parser<'_>) -> TypeLit<Parse> {
    let mut ty = term_ty(parser);

    while is_ty(parser.peek()) {
        let input = term_ty(parser);
        let span = ty.span().join(input.span());

        ty = TypeLit::App(AppTy {
            lambda: Box::new(ty),
            input: Box::new(input),
            span,
        });
    }

    ty
}

pub fn term_ty(parser: &mut Parser<'_>) -> TypeLit<Parse> {
    match parser.peek() {
        Token::Ident(name) => TypeLit::Var(var_ty(parser, name)),

        Token::Star => TypeLit::Ty(ty_ty(parser)),

        _ => {
            let span = parser.expected("type");
            TypeLit::Error(ErrorTy { span })
        }
    }
}

pub fn var_ty(parser: &mut Parser<'_>, name: String) -> VarTy<Parse> {
    let span = parser.consume();

    VarTy { var: name, span }
}

pub fn ty_ty(parser: &mut Parser<'_>) -> TyTy {
    let span = parser.expect(Token::Star);

    TyTy { span }
}
