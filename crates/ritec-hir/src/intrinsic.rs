use ritec_core::{Generic, Ident, Span};

use crate::{Body, Function, FunctionArgument, Generics};

pub fn build_intrinsic_bitcast() -> Function {
    let ident = Ident::from("bitcast");
    let t = Generic::new("T");
    let u = Generic::new("U");

    let generics = Generics::new(vec![t.clone(), u.clone()], Span::DUMMY);

    let mut body = Body::new();

    let value = body.local("value", t);

    let arguments = vec![FunctionArgument {
        ident: Ident::new("value", Span::DUMMY),
        local: value,
        span: Span::DUMMY,
    }];

    let local = body.local_expr(value);
    let bitcast = body.bitcast_expr(local, u.clone());
    let ret = body.return_expr(Some(bitcast));
    body.expr_stmt(ret);

    Function {
        ident,
        generics,
        arguments,
        return_type: u.into(),
        body,
        span: Span::DUMMY,
    }
}
