use std::{ops::Range, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term,
    term::termcolor::{ColorChoice, StandardStream},
};

pub fn emit_ron_error(
    path: &Path,
    content: &str,
    e: &ron::error::SpannedError,
) -> anyhow::Result<()> {
    let mut files = SimpleFiles::new();

    let file_id = files.add(path.display().to_string(), &content);
    let diagnostic = Diagnostic::error()
        .with_message("Failed to parse RON file")
        .with_labels(vec![
            Label::primary(
                file_id,
                line_col_range_to_byte_range(
                    content,
                    e.span.start.line,
                    e.span.start.col,
                    e.span.end.line,
                    e.span.end.col,
                )
                .unwrap(),
            )
            .with_message(&e.code),
        ]);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = term::Config::default();

    term::emit_to_write_style(&mut writer.lock(), &config, &files, &diagnostic)?;
    Ok(())
}

/// Converts a line-column range to a byte index range in the source string.
/// Lines and columns are 1-indexed.
/// Returns None if any position is out of bounds.
pub fn line_col_range_to_byte_range(
    source: &str,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
) -> Option<Range<usize>> {
    let start = position_to_byte_index(source, start_line, start_col)?;
    let end = position_to_byte_index(source, end_line, end_col)?;
    Some(start..end)
}

/// Converts a line-column position to a byte index in the source string.
/// Lines and columns are 1-indexed.
/// Returns None if the position is out of bounds.
fn position_to_byte_index(source: &str, line: usize, col: usize) -> Option<usize> {
    let mut current_line = 1;
    let mut current_col = 1;
    for (byte_index, ch) in source.char_indices() {
        if current_line == line && current_col == col {
            return Some(byte_index);
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }
    None
}
