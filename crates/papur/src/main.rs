//! papur — command-line interface for the papur markup compiler.
//!
//! Spec 001 wires the front door: parse arguments, enforce the `.papur`
//! extension, segment the source into blocks, and render any diagnostics with
//! `miette`. Target emitters (HTML/PDF/email/plain) arrive in later specs; for
//! now a successful parse prints a segmentation summary.

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use miette::{Diagnostic, IntoDiagnostic, NamedSource, Result, SourceSpan};
use papur_core::{segment, Block, ParseMode};
use thiserror::Error;

/// Compile a `.papur` source file.
#[derive(Parser, Debug)]
#[command(name = "papur", version, about = "The papur markup compiler")]
struct Cli {
    /// Path to a `.papur` source file.
    file: PathBuf,
    /// Parse in lenient mode: malformed typed content degrades to prose
    /// instead of erroring. Strict mode is the default.
    #[arg(long)]
    lenient: bool,
}

/// Whether `path` ends in the exact `.papur` extension. Middle segments
/// (`styles.css.papur`) are human signage and never change behavior (AC2).
fn has_papur_extension(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("papur")
}

#[derive(Debug, Error, Diagnostic)]
#[error("not a papur source file: {path}")]
#[diagnostic(
    code(papur::bad_extension),
    help("papur only parses files with the exact `.papur` extension")
)]
struct BadExtension {
    path: String,
}

/// A single parse diagnostic, dressed for `miette`'s source-highlighted output.
#[derive(Debug, Error, Diagnostic)]
#[error("{code}: {message}")]
struct ParseDiagnostic {
    code: &'static str,
    message: String,
    #[source_code]
    src: NamedSource<String>,
    #[label("here")]
    at: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("could not parse {path}")]
struct ParseFailed {
    path: String,
    #[related]
    diagnostics: Vec<ParseDiagnostic>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // AC1: only the exact `.papur` extension is accepted, before any parsing.
    if !has_papur_extension(&cli.file) {
        return Err(BadExtension {
            path: cli.file.display().to_string(),
        }
        .into());
    }

    let source = fs::read_to_string(&cli.file).into_diagnostic()?;
    let name = cli.file.display().to_string();
    let mode = if cli.lenient {
        ParseMode::Lenient
    } else {
        ParseMode::Strict
    };

    match segment(&source, mode) {
        Ok(stream) => {
            print_summary(&name, &stream.blocks);
            Ok(())
        }
        Err(diags) => {
            let diagnostics = diags
                .into_iter()
                .map(|d| ParseDiagnostic {
                    code: d.code.code(),
                    message: d.message,
                    src: NamedSource::new(name.as_str(), source.clone()),
                    at: (d.span.start_byte, d.span.len()).into(),
                })
                .collect();
            Err(ParseFailed {
                path: name,
                diagnostics,
            }
            .into())
        }
    }
}

/// Print a one-screen segmentation summary of a successful parse.
fn print_summary(name: &str, blocks: &[Block]) {
    println!("{name}: {} block(s)", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        let (kind, lines) = match block {
            Block::Content { text, .. } => ("content", text.lines().count()),
            Block::Layer { kind, body, .. } => (kind.keyword(), body.lines().count()),
        };
        println!("  {:>2}. {kind} ({lines} line(s))", i + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::has_papur_extension;
    use std::path::Path;

    #[test]
    fn accepts_only_papur_extension() {
        assert!(has_papur_extension(Path::new("page.papur")));
        assert!(!has_papur_extension(Path::new("page.md")));
        assert!(!has_papur_extension(Path::new("page")));
    }

    #[test]
    fn middle_segments_do_not_matter() {
        // AC2: middle segments are signage; the extension is still `.papur`.
        assert!(has_papur_extension(Path::new("styles.css.papur")));
        assert!(has_papur_extension(Path::new("mytheme.theme.papur")));
        assert!(has_papur_extension(Path::new("actions.js.papur")));
    }
}
