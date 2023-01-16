use std::panic::Location;

use ritec_core::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Debug)]
pub struct DiagnosticHint {
    pub message: Option<String>,
    pub span: Option<Span>,
}

impl DiagnosticHint {
    pub fn new() -> Self {
        Self {
            message: None,
            span: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
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

impl From<String> for DiagnosticHint {
    fn from(message: String) -> Self {
        Self::new().with_message(message)
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub title: String,
    pub level: DiagnosticLevel,
    pub messages: Vec<DiagnosticHint>,
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

    pub fn add_msg(&mut self, message: impl Into<DiagnosticHint>) -> &mut Self {
        self.messages.push(message.into());
        self
    }

    pub fn with_msg(mut self, message: impl Into<DiagnosticHint>) -> Self {
        self.add_msg(message);
        self
    }

    pub fn add_span(&mut self, span: Span) -> &mut Self {
        self.messages.push(DiagnosticHint::new().with_span(span));
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.add_span(span);
        self
    }

    pub fn add_msg_span(&mut self, message: impl Into<String>, span: Span) -> &mut Self {
        self.add_msg(DiagnosticHint::new().with_message(message).with_span(span));
        self
    }

    pub fn with_msg_span(mut self, message: impl Into<String>, span: Span) -> Self {
        self.add_msg_span(message, span);
        self
    }
}
