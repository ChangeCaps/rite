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
    notes:  Vec<Note>,
}

#[derive(Debug)]
pub struct Label {
    message: String,
    span:    Span,
}

#[derive(Debug)]
pub struct Note {
    message: String,
}

impl Diagnostic {
    #[track_caller]
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            #[cfg(debug_assertions)]
            location: Location::caller(),

            severity,
            message: message.into(),

            labels: Vec::new(),
            notes: Vec::new(),
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

    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(Note {
            message: message.into(),
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

    pub fn notes(&self) -> &[Note] {
        &self.notes
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

impl Note {
    pub fn message(&self) -> &str {
        &self.message
    }
}
