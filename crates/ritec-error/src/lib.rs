mod emitter;

use std::panic::Location;

pub use emitter::*;
use ritec_core::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Debug)]
pub struct DiagnosticMessage {
    pub message: String,
    pub span: Option<Span>,
}

impl DiagnosticMessage {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn set_span(&mut self, span: Span) -> &mut Self {
        self.span = Some(span);
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

impl From<String> for DiagnosticMessage {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub title: String,
    pub level: DiagnosticLevel,
    pub messages: Vec<DiagnosticMessage>,
    pub location: &'static Location<'static>,
}

impl Diagnostic {
    #[track_caller]
    pub fn new(title: impl Into<String>, level: DiagnosticLevel) -> Self {
        Self {
            title: title.into(),
            level,
            messages: Vec::new(),
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn error(title: impl Into<String>) -> Self {
        Self::new(title, DiagnosticLevel::Error)
    }

    #[track_caller]
    pub fn warning(title: impl Into<String>) -> Self {
        Self::new(title, DiagnosticLevel::Warning)
    }

    pub fn add_message(&mut self, message: impl Into<DiagnosticMessage>) -> &mut Self {
        self.messages.push(message.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<DiagnosticMessage>) -> Self {
        self.add_message(message);
        self
    }

    pub fn add_message_span(&mut self, message: impl Into<String>, span: Span) -> &mut Self {
        self.add_message(DiagnosticMessage::new(message).with_span(span));
        self
    }

    pub fn with_message_span(mut self, message: impl Into<String>, span: Span) -> Self {
        self.add_message_span(message, span);
        self
    }
}
