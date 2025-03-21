use crate::error::unexpected_token_message;
use crate::error::Error;
use crate::error::ErrorKind;
use crate::error::SingleError;
use crate::SourceFile;
use crate::Span;

use std::io;
use std::io::Write;
use std::rc::Rc;

use ariadne::Color;
use ariadne::Label;
use ariadne::ReportKind;
use ariadne::Source;

impl ariadne::Span for Span {
    type SourceId = String;

    fn source(&self) -> &Self::SourceId {
        self.source.id()
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl From<&ErrorKind> for ReportKind<'static> {
    fn from(value: &ErrorKind) -> Self {
        match value {
            ErrorKind::Silent
            | ErrorKind::Custom { .. }
            | ErrorKind::UnknownCharacter(_)
            | ErrorKind::UnterminatedGroup { .. }
            | ErrorKind::UnterminatedChar(_)
            | ErrorKind::LongChar(_)
            | ErrorKind::UnterminatedString(_)
            | ErrorKind::UnexpectedToken { .. }
            | ErrorKind::EndOfFile(_) => ReportKind::Error,
        }
    }
}

/// A reported error, ready to be written to stderr.
///
/// This type exposes a very similar API to [`ariadne::Report`].
///
/// A `Vec<Report>` can be created from an [`Error`], but in most cases, the
/// [`Error::eprint`] method will suffice.
pub struct Report {
    report: ariadne::Report<'static, Span>,
    source: Rc<SourceFile>,
}

impl Report {
    /// Writes this diagnostic to an implementor of [`Write`].
    ///
    /// For more details, see [`ariadne::Report::write`].
    pub fn write<W: Write>(&self, w: W) -> io::Result<()> {
        self.report.write(
            (
                self.source.id().to_owned(),
                Source::from(&self.source.contents),
            ),
            w,
        )
    }

    /// Writes this diagnostic to an implementor of [`Write`].
    ///
    /// For more details, see [`ariadne::Report::write_for_stdout`].
    pub fn write_for_stdout<W: Write>(&self, w: W) -> io::Result<()> {
        self.report.write_for_stdout(
            (
                self.source.id().to_owned(),
                Source::from(&self.source.contents),
            ),
            w,
        )
    }

    /// Prints this diagnostic to stderr.
    ///
    /// For more details, see [`ariadne::Report::eprint`].
    pub fn eprint(&self) -> io::Result<()> {
        self.report.eprint((
            self.source.id().to_owned(),
            Source::from(&self.source.contents),
        ))
    }

    /// Prints this diagnostic to stdout. In most cases, [`Report::eprint`] is
    /// preferable.
    ///
    /// For more details, see [`ariadne::Report::print`].
    pub fn print(&self) -> io::Result<()> {
        self.report.print((
            self.source.id().to_owned(),
            Source::from(&self.source.contents),
        ))
    }
}

impl From<&SingleError> for Report {
    fn from(value: &SingleError) -> Self {
        let mut builder =
            ariadne::Report::build((&value.kind).into(), value.source.id(), value.kind.start())
                .with_code(value.kind.code());
        match &value.kind {
            ErrorKind::Silent => unreachable!(),
            ErrorKind::Custom { message, span, .. } => {
                builder.set_message(message);
                builder.add_label(Label::new(span.clone()).with_color(Color::Red));
            }
            ErrorKind::UnknownCharacter(span) => {
                builder.set_message("Unrecognised character");
                builder.add_label(Label::new(span.clone()).with_color(Color::Red));
            }
            ErrorKind::UnterminatedGroup { start, span } => {
                builder.set_message(format!("Unmatched '{start}'"));
                builder.add_label(Label::new(span.clone()).with_color(Color::Red));
            }
            ErrorKind::UnterminatedChar(span) => {
                builder.set_message("Expect \"'\" after character literal");
                builder.add_label(Label::new(span.clone()).with_color(Color::Red));
            }
            ErrorKind::LongChar(span) => {
                builder.set_message("Character literals must be exactly one character long");
                builder.add_label(Label::new(span.clone()).with_color(Color::Red));
            }
            ErrorKind::UnterminatedString(span) => {
                builder.set_message("Expect '\"' at end of string literal");
                builder.add_label(Label::new(span.clone()).with_color(Color::Red));
            }
            ErrorKind::UnexpectedToken { expected, span } => {
                builder.set_message("Unexpected token");
                builder.add_label(
                    Label::new(span.clone())
                        .with_color(Color::Red)
                        .with_message(unexpected_token_message(expected)),
                );
            }
            ErrorKind::EndOfFile(_) => builder.set_message("Unexpected end of file while parsing"),
        }
        Report {
            report: builder.finish(),
            source: Rc::clone(&value.source),
        }
    }
}

impl From<&Error> for Vec<Report> {
    fn from(value: &Error) -> Self {
        let mut reports = Vec::with_capacity(value.errors.len());
        for error in &value.errors {
            if !matches!(&error.kind, ErrorKind::Silent) {
                reports.push(Report::from(error));
            }
        }
        reports
    }
}

impl Error {
    /// Prints this error to stderr.
    pub fn eprint(&self) -> io::Result<()> {
        let reports: Vec<Report> = self.into();
        for report in reports {
            report.eprint()?;
        }
        Ok(())
    }
}
