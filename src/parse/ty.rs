use crate::{
    ast::{AppTy, ErrorTy, LamTy, Parse, Ty, TyTy, VarTy},
    parse::{Parser, Token},
};

pub fn is_ty(token: Token) -> bool {
    matches!(token, Token::Ident(..) | Token::Star)
}

pub fn ty(parser: &mut Parser<'_>) -> Ty<Parse> {
    lambda_ty(parser)
}

pub fn lambda_ty(parser: &mut Parser<'_>) -> Ty<Parse> {
    let mut ty = apply_ty(parser);

    while parser.peek() == Token::Arrow {
        parser.expect(Token::Arrow);

        let output = apply_ty(parser);
        let span = ty.span().join(output.span());

        ty = Ty::Lam(LamTy {
            input: Box::new(ty),
            output: Box::new(output),
            span,
        });
    }

    ty
}

pub fn apply_ty(parser: &mut Parser<'_>) -> Ty<Parse> {
    let mut ty = term_ty(parser);

    while is_ty(parser.peek()) {
        let input = term_ty(parser);
        let span = ty.span().join(input.span());

        ty = Ty::App(AppTy {
            lambda: Box::new(ty),
            input: Box::new(input),
            span,
        });
    }

    ty
}

pub fn term_ty(parser: &mut Parser<'_>) -> Ty<Parse> {
    match parser.peek() {
        Token::Ident(name) => Ty::Var(var_ty(parser, name)),

        Token::Star => Ty::Ty(ty_ty(parser)),

        _ => {
            let span = parser.expected("type");
            Ty::Error(ErrorTy { span })
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
