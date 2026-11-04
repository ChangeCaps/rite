use crate::{
    ast::{ErrorExpr, Expr, ItemExpr, Parse},
    parse::{Parser, Token},
};

pub fn expr(parser: &mut Parser<'_>) -> Expr<Parse> {
    match parser.peek() {
        Token::Ident(name) => Expr::Item(item_expr(parser, name)),

        _ => {
            let span = parser.expected("expression");
            Expr::Error(ErrorExpr { ty: (), span })
        }
    }
}

pub fn item_expr(parser: &mut Parser<'_>, name: String) -> ItemExpr<Parse> {
    let span = parser.consume();

    ItemExpr { name, ty: (), span }
}
