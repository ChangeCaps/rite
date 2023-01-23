use ritec_core::{Generic, Ident, Span};

use crate::{Body, Function, FunctionArgument, Generics, IntType, PointerType, Type};

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

pub fn build_intrinsic_sizeof() -> Function {
    let ident = Ident::from("sizeof");
    let t = Generic::new("T");

    let generics = Generics::new(vec![t.clone()], Span::DUMMY);

    let mut body = Body::new();

    let arguments = vec![];

    let size = body.sizeof_expr(t.clone());
    let ret = body.return_expr(Some(size));
    body.expr_stmt(ret);

    Function {
        ident,
        generics,
        arguments,
        return_type: Type::Int(IntType {
            signed: false,
            size: None,
            span: Span::DUMMY,
        }),
        body,
        span: Span::DUMMY,
    }
}

pub fn build_intrinsic_alignof() -> Function {
    let ident = Ident::from("alignof");
    let t = Generic::new("T");

    let generics = Generics::new(vec![t.clone()], Span::DUMMY);

    let mut body = Body::new();

    let arguments = vec![];

    let align = body.alignof_expr(t.clone());
    let ret = body.return_expr(Some(align));
    body.expr_stmt(ret);

    Function {
        ident,
        generics,
        arguments,
        return_type: Type::Int(IntType {
            signed: false,
            size: None,
            span: Span::DUMMY,
        }),
        body,
        span: Span::DUMMY,
    }
}

pub fn build_intrinsic_malloc() -> Function {
    let ident = Ident::from("malloc");
    let t = Generic::new("T");

    let generics = Generics::new(vec![t.clone()], Span::DUMMY);

    let mut body = Body::new();

    let size = body.local(
        "size",
        Type::Int(IntType {
            signed: false,
            size: None,
            span: Span::DUMMY,
        }),
    );

    let arguments = vec![FunctionArgument {
        ident: Ident::new("size", Span::DUMMY),
        local: size,
        span: Span::DUMMY,
    }];

    let local = body.local_expr(size);
    let malloc = body.malloc_expr(t.clone(), local);
    let ret = body.return_expr(Some(malloc));
    body.expr_stmt(ret);

    Function {
        ident,
        generics,
        arguments,
        return_type: Type::Pointer(PointerType {
            pointee: Box::new(t.into()),
            span: Span::DUMMY,
        }),
        body,
        span: Span::DUMMY,
    }
}

pub fn build_intrinsic_free() -> Function {
    let ident = Ident::from("free");
    let t = Generic::new("T");

    let generics = Generics::new(vec![t.clone()], Span::DUMMY);

    let mut body = Body::new();

    let ptr = body.local(
        "ptr",
        Type::Pointer(PointerType {
            pointee: Box::new(t.clone().into()),
            span: Span::DUMMY,
        }),
    );

    let arguments = vec![FunctionArgument {
        ident: Ident::new("ptr", Span::DUMMY),
        local: ptr,
        span: Span::DUMMY,
    }];

    let local = body.local_expr(ptr);
    let free = body.free_expr(local);
    let ret = body.return_expr(Some(free));
    body.expr_stmt(ret);

    Function {
        ident,
        generics,
        arguments,
        return_type: Type::void(Span::DUMMY),
        body,
        span: Span::DUMMY,
    }
}

pub fn build_intrinsic_memcpy() -> Function {
    let ident = Ident::from("memcpy");
    let t = Generic::new("T");

    let generics = Generics::new(vec![t.clone()], Span::DUMMY);

    let mut body = Body::new();

    let dst = body.local(
        "dst",
        Type::Pointer(PointerType {
            pointee: Box::new(t.clone().into()),
            span: Span::DUMMY,
        }),
    );

    let src = body.local(
        "src",
        Type::Pointer(PointerType {
            pointee: Box::new(t.clone().into()),
            span: Span::DUMMY,
        }),
    );

    let size = body.local(
        "size",
        Type::Int(IntType {
            signed: false,
            size: None,
            span: Span::DUMMY,
        }),
    );

    let arguments = vec![
        FunctionArgument {
            ident: Ident::new("dst", Span::DUMMY),
            local: dst,
            span: Span::DUMMY,
        },
        FunctionArgument {
            ident: Ident::new("src", Span::DUMMY),
            local: src,
            span: Span::DUMMY,
        },
        FunctionArgument {
            ident: Ident::new("size", Span::DUMMY),
            local: size,
            span: Span::DUMMY,
        },
    ];

    let dst_local = body.local_expr(dst);
    let src_local = body.local_expr(src);
    let size_local = body.local_expr(size);
    let memcpy = body.memcpy_expr(dst_local, src_local, size_local);
    let ret = body.return_expr(Some(memcpy));
    body.expr_stmt(ret);

    Function {
        ident,
        generics,
        arguments,
        return_type: Type::void(Span::DUMMY),
        body,
        span: Span::DUMMY,
    }
}
