use std::sync::Mutex;

use crate::diagnostic::Diagnostic;

pub trait Reporter {
    fn report(&self, diagnostic: Diagnostic);
}

impl Reporter for Mutex<Vec<Diagnostic>> {
    fn report(&self, diagnostic: Diagnostic) {
        if let Ok(mut diagnostics) = self.lock() {
            diagnostics.push(diagnostic);
        }
    }
}
