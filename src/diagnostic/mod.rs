mod reporter;
mod source;
mod span;
mod writer;

use std::panic::Location;

pub use reporter::*;
pub use source::*;
pub use span::*;
pub use writer::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
pub struct Diagnostic {
    #[cfg(debug_assertions)]
    location: &'static Location<'static>,

    severity: Severity,
    message:  String,

    labels: Vec<Label>,
}

#[derive(Debug)]
pub struct Label {
    message: String,
    span:    Span,
}

impl Diagnostic {
    #[track_caller]
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            location: Location::caller(),

            severity,
            message: message.into(),

            labels: Vec::new(),
        }
    }

    #[track_caller]
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    #[track_caller]
    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            message: message.into(),
            span,
        });

        self
    }

    pub fn location(&self) -> Option<&'static Location<'static>> {
        #[cfg(debug_assertions)]
        return Some(self.location);

        #[cfg(not(debug_assertions))]
        return None;
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }
}

impl Label {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
