// Taken from similar's 'terminal-inline' example:
// https://github.com/mitsuhiko/similar/blob/main/examples/terminal-inline.rs

use console::{style, Style};
use similar::{ChangeTag, TextDiff};
use std::fmt::{self, Write};

struct Line(Option<usize>);
impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn diff_report(file: &String, generated: &String) -> String {
    let diff = TextDiff::from_lines(file, generated);

    let mut out = String::new();

    writeln!(&mut out, "{}", Style::new().red().bold().apply_to("- File Content")).unwrap();
    writeln!(&mut out, "{}", Style::new().green().bold().apply_to("+ Generator Output")).unwrap();
    writeln!(&mut out).unwrap();

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            write!(&mut out, "{:-^1$}", "-", 80).unwrap();
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    &mut out,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )
                .unwrap();
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(&mut out, "{}", s.apply_to(value).underlined().on_black()).unwrap();
                    } else {
                        write!(&mut out, "{}", s.apply_to(value)).unwrap();
                    }
                }
                if change.missing_newline() {
                    writeln!(&mut out).unwrap();
                }
            }
        }
    }

    out
}
