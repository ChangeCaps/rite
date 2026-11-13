use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{diagnostic::Span, parse::Token};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    PipeGt,
    LtPipe,
    BackGt,
    LtBack,
    GtGtEqual,
    GtGt,
    LtLt,
    EqualEqual,
    BangEqual,
    Comma,
    Colon,
    PipePipe,
    AmpAmp,
    Plus,
    Minus,
    Star,
    Slash,
}

impl Operator {
    pub const ALL: &[Self] = &[
        Operator::PipeGt,
        Operator::LtPipe,
        Operator::BackGt,
        Operator::LtBack,
        Operator::GtGtEqual,
        Operator::GtGt,
        Operator::LtLt,
        Operator::EqualEqual,
        Operator::BangEqual,
        Operator::Comma,
        Operator::Colon,
        Operator::PipePipe,
        Operator::AmpAmp,
        Operator::Plus,
        Operator::Minus,
        Operator::Star,
        Operator::Slash,
    ];

    pub fn as_token(self) -> &'static Token {
        match self {
            Operator::PipeGt => &Token::PipeGt,
            Operator::LtPipe => &Token::LtPipe,
            Operator::BackGt => &Token::BackGt,
            Operator::LtBack => &Token::LtBack,
            Operator::GtGtEqual => &Token::GtGtEqual,
            Operator::GtGt => &Token::GtGt,
            Operator::LtLt => &Token::LtLt,
            Operator::EqualEqual => &Token::EqualEqual,
            Operator::BangEqual => &Token::BangEqual,
            Operator::Comma => &Token::Comma,
            Operator::Colon => &Token::Colon,
            Operator::PipePipe => &Token::PipePipe,
            Operator::AmpAmp => &Token::AmpAmp,
            Operator::Plus => &Token::Plus,
            Operator::Minus => &Token::Minus,
            Operator::Star => &Token::Star,
            Operator::Slash => &Token::Slash,
        }
    }

    pub fn precedence(self) -> u8 {
        match self {
            Self::PipeGt | Self::BackGt | Self::GtGtEqual | Self::GtGt => 0,
            Self::LtPipe | Self::LtBack | Self::LtLt => 1,
            Self::EqualEqual | Self::BangEqual => 2,
            Self::Comma => 3,
            Self::Colon => 4,
            Self::PipePipe => 5,
            Self::AmpAmp => 6,
            Self::Plus | Self::Minus => 7,
            Self::Star | Self::Slash => 8,
        }
    }

    pub fn assoc(self) -> Assoc {
        match self {
            Operator::PipeGt
            | Operator::LtPipe
            | Operator::BackGt
            | Operator::LtBack
            | Operator::GtGtEqual
            | Operator::GtGt
            | Operator::LtLt
            | Operator::EqualEqual
            | Operator::BangEqual
            | Operator::Comma
            | Operator::PipePipe
            | Operator::AmpAmp
            | Operator::Plus
            | Operator::Minus
            | Operator::Star
            | Operator::Slash => Assoc::Left,

            Operator::Colon => Assoc::Right,
        }
    }

    pub fn as_str(&self) -> &'static str {
        self.as_token().as_str()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Assoc {
    Left,
    Right,
}

#[derive(Clone)]
pub struct Ident {
    inner: IdentInner,
    span:  Span,
}

#[derive(Clone, PartialEq)]
enum IdentInner {
    Name(Arc<str>),
    Operator(Operator),
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            inner: IdentInner::Name(name.into().into()),
            span,
        }
    }

    pub fn operator(operator: Operator, span: Span) -> Self {
        Self {
            inner: IdentInner::Operator(operator),
            span,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn as_str(&self) -> &str {
        match self.inner {
            IdentInner::Name(ref name) => name.as_ref(),
            IdentInner::Operator(operator) => operator.as_str(),
        }
    }

    pub fn as_operator(&self) -> Option<Operator> {
        match self.inner {
            IdentInner::Name(_) => None,
            IdentInner::Operator(operator) => Some(operator),
        }
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Clone)]
pub struct Var {
    ident:  Option<Ident>,
    unique: Unique,
}

impl From<Option<Ident>> for Var {
    fn from(ident: Option<Ident>) -> Self {
        Self {
            ident,
            unique: Unique::new(),
        }
    }
}

impl Var {
    pub fn new(ident: Ident) -> Self {
        Self {
            ident:  Some(ident),
            unique: Unique::new(),
        }
    }

    pub fn unnamed() -> Self {
        Self {
            ident:  None,
            unique: Unique::new(),
        }
    }

    pub fn ident(&self) -> Option<&Ident> {
        self.ident.as_ref()
    }

    pub fn unique(&self) -> Unique {
        self.unique
    }

    pub fn span(&self) -> Option<Span> {
        self.ident().map(Ident::span)
    }

    pub fn as_str(&self) -> Option<&str> {
        self.ident().map(Ident::as_str)
    }
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        self.unique == other.unique
    }
}

impl Eq for Var {}

impl Hash for Var {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unique.hash(state);
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(name) => write!(f, "{}:{}", name, self.unique.data),
            None => write!(f, "?:{}", self.unique.data),
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(name) => f.write_str(name),
            None => f.write_str("?"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unique {
    data: u64,
}

impl Default for Unique {
    fn default() -> Self {
        Self::new()
    }
}

impl Unique {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let data = NEXT.fetch_add(1, Ordering::SeqCst);

        Self { data }
    }
}

impl fmt::Debug for Unique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "u{}", self.data)
    }
}

impl fmt::Display for Unique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
