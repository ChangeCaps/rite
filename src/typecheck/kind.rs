use crate::{
    diagnostic::{Diagnostic, Span},
    rename,
    typecheck::{State, Ty},
    var::Var,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Kind {
    pub inferrable: bool,
    pub subst:      Option<Ty>,
    pub arity:      Option<usize>,
    pub traits:     Vec<Var>,
}

impl Default for Kind {
    fn default() -> Self {
        Self {
            inferrable: true,
            subst:      None,
            arity:      None,
            traits:     Vec::new(),
        }
    }
}

impl Kind {
    pub fn intersect(&mut self, other: &Self, span: Span) -> Result<(), Diagnostic> {
        if let Some(other_arity) = other.arity {
            if let Some(self_arity) = self.arity
                && other_arity != self_arity
            {
                let diagnostic = Diagnostic::error("kind mismatch").label(
                    span,
                    format!(
                        "expected `{}` but found `{}`",
                        Self::format_arity(self_arity),
                        Self::format_arity(other_arity),
                    ),
                );

                return Err(diagnostic);
            }

            self.arity = Some(other_arity);
        }

        for trait_ in &other.traits {
            if self.traits.contains(trait_) {
                continue;
            };

            if !self.inferrable {
                let diagnostic = Diagnostic::error("kind mismatch").label(
                    span,
                    format!("expected to implement `{trait_}`"),
                );

                return Err(diagnostic);
            }

            self.traits.push(trait_.clone());
        }

        Ok(())
    }

    pub fn format_arity(arity: usize) -> String {
        (0..=arity).map(|_| "*").collect::<Vec<_>>().join(" -> ")
    }
}

impl State<'_> {
    pub fn kind(&mut self, kind: rename::Kind) -> Kind {
        match kind {
            rename::Kind::Var(kind) => self.var_kind(kind),
            rename::Kind::Arity(kind) => self.arity_kind(kind),
            rename::Kind::Error(kind) => self.error_kind(kind),
        }
    }

    fn var_kind(&mut self, kind: rename::VarKind) -> Kind {
        Kind {
            traits: vec![kind.var],
            ..Default::default()
        }
    }

    fn arity_kind(&mut self, kind: rename::ArityKind) -> Kind {
        Kind {
            arity: Some(kind.arity),
            ..Default::default()
        }
    }

    fn error_kind(&mut self, _kind: rename::ErrorKind) -> Kind {
        Kind::default()
    }
}
