use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::diagnostic::{Diagnostic, Label, Sources};

use super::Severity;

pub struct DiagnosticWriter<'a, W> {
    sources: &'a Sources,
    writer:  W,
}

impl<'a, W> DiagnosticWriter<'a, W>
where
    W: Write,
{
    pub fn new(sources: &'a Sources, writer: W) -> Self {
        Self { sources, writer }
    }

    pub fn write(&mut self, diagnostics: &[Diagnostic]) -> io::Result<bool> {
        let mut errors = 0;

        for diagnostic in diagnostics {
            match diagnostic.severity() {
                Severity::Error => errors += 1,
            }

            self.diagnostic(diagnostic)?;
        }

        if errors > 0 {
            let plural = if errors > 1 { "s" } else { "" };

            writeln!(
                self.writer,
                "{RED}{BOLD}error{RESET}: failed to compile due to {errors} error{plural}",
            )?;
        }

        Ok(errors == 0)
    }

    fn diagnostic(&mut self, diagnostic: &Diagnostic) -> io::Result<()> {
        self.header(diagnostic)?;

        let labels = diagnostic
            .labels
            .iter()
            .map(|label| self.resolve_label(label))
            .collect::<Vec<_>>();

        let label_indent = labels.iter().map(ResolvedLabel::indent).max().unwrap_or(1);

        for label in labels {
            self.label(diagnostic, label_indent, &label)?;
        }

        Ok(())
    }

    fn header(&mut self, diagnostic: &Diagnostic) -> io::Result<()> {
        let color = Self::severity_color(diagnostic.severity());

        writeln!(
            self.writer,
            "{color}{BOLD}error{WHITE}: {}{RESET}",
            diagnostic.message()
        )?;

        Ok(())
    }

    fn label(
        &mut self,
        _diagnostic: &Diagnostic,
        indent: usize,
        label: &ResolvedLabel,
    ) -> io::Result<()> {
        writeln!(
            self.writer,
            "{}{BLUE}{BOLD}--> {RESET}{WHITE}{}:{}:{}",
            " ".repeat(indent),
            label.path.display(),
            label.line,
            label.column,
        )?;

        writeln!(
            self.writer,
            "{}{BLUE}{BOLD}|",
            " ".repeat(indent + 1)
        )?;

        let number = label.line.to_string();

        let line = label.lines.first().unwrap();

        writeln!(
            self.writer,
            "{BLUE}{BOLD}{number}{}|{RESET}{WHITE} {line}",
            " ".repeat(indent - number.len() + 1),
        )?;

        let highlight = line
            .get(label.line_start..label.line_end)
            .into_iter()
            .flat_map(str::chars)
            .count()
            .max(1);

        writeln!(
            self.writer,
            "{}{BLUE}{BOLD}|{}{RED}{} {WHITE}{}{RESET}",
            " ".repeat(indent + 1),
            " ".repeat(label.column),
            "^".repeat(highlight),
            label.message,
        )?;

        writeln!(self.writer)?;

        Ok(())
    }

    fn severity_color(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => RED,
        }
    }

    fn resolve_label(&self, label: &Label) -> ResolvedLabel {
        let source = &self.sources[label.span.source];

        let mut offset = 0;

        let mut line = 1;
        let mut column = 1;

        let mut line_start = 0;
        let mut line_end = 0;

        let mut lines = Vec::new();

        let start = label.span.start as usize;
        let end = label.span.end as usize;

        for s in source.contents.split('\n') {
            offset += s.len() + 1;

            if offset < start {
                line += 1;
            }

            if offset > start {
                if lines.is_empty() {
                    line_start = s
                        .len()
                        .saturating_sub((offset - start).saturating_sub(1))
                        .min(s.len());

                    column = s[..line_start].chars().count() + 1;
                }

                lines.push(s.to_string());
            }

            if offset > end {
                line_end = s.len().saturating_sub(offset - end) + 1;
                break;
            }
        }

        ResolvedLabel {
            path: source.path.clone(),
            line,
            column,
            line_start,
            line_end,
            lines,
            message: label.message().to_string(),
        }
    }
}

#[derive(Debug)]
struct ResolvedLabel {
    path:       PathBuf,
    line:       usize,
    column:     usize,
    line_start: usize,
    line_end:   usize,
    lines:      Vec<String>,
    message:    String,
}

impl ResolvedLabel {
    fn indent(&self) -> usize {
        self.line.to_string().len()
    }
}

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const WHITE: &str = "\x1b[37m";

const BOLD: &str = "\x1b[1m";

const RESET: &str = "\x1b[0m";
