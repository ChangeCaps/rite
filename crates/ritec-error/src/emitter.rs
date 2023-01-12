use crate::Diagnostic;

pub trait Emitter {
    fn emit(&mut self, diagnostic: Diagnostic);
}

impl Emitter for Vec<Diagnostic> {
    fn emit(&mut self, diagnostic: Diagnostic) {
        self.push(diagnostic);
    }
}

impl Emitter for () {
    fn emit(&mut self, _: Diagnostic) {}
}
