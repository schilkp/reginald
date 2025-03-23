use std::{collections::VecDeque, fmt::Write};

struct Section {
    has_been_emitted: bool,
    header_lines: Vec<String>,
}

/// A wrapper around a `write`, that indents all lines to a set value.
pub struct HeaderWriter<'a> {
    w: &'a mut dyn Write,
    sections: VecDeque<Section>,
}

impl Write for HeaderWriter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if !s.is_empty() { self.process_str(s) } else { Ok(()) }
    }
}

impl HeaderWriter<'_> {
    /// Wrap a given `write` into an `HeaderWrite` that buffers
    /// section headers and only emits them if a section contains any content.
    pub fn new(w: &mut dyn Write) -> HeaderWriter<'_> {
        HeaderWriter {
            w,
            sections: VecDeque::new(),
        }
    }

    fn process_str(&mut self, s: &str) -> std::fmt::Result {
        for section in self.sections.iter_mut() {
            if !section.has_been_emitted {
                for line in &section.header_lines {
                    self.w.write_str(line)?
                }
                section.has_been_emitted = true;
            }
        }

        self.w.write_str(s)
    }

    pub fn push_section(&mut self) {
        self.sections.push_back(Section {
            has_been_emitted: false,
            header_lines: vec![],
        });
    }

    /// Start a new section that, if non-empty, should have the given header.
    pub fn push_section_with_header(&mut self, header: &[&str]) {
        let header_lines: Vec<String> = header.iter().map(|x| x.to_string()).collect();
        self.sections.push_back(Section {
            has_been_emitted: false,
            header_lines,
        });
    }

    pub fn pop_section(&mut self) {
        assert!(self.sections.pop_back().is_some(), "Attempted to pop more sections than have been pushed!");
    }

    pub fn pop_section_with_footer(&mut self, footer: &[&str]) -> std::fmt::Result {
        let section = self
            .sections
            .pop_back()
            .expect("Attempted to pop more sections than have been pushed!");

        if section.has_been_emitted {
            for line in footer {
                self.w.write_str(line)?
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use std::{fmt::Write, iter::zip};

    use std::fmt::Write;

    use super::HeaderWriter;
    #[test]
    fn test_header_write() {
        let mut string = String::new();
        let mut w = HeaderWriter::new(&mut string);

        w.push_section_with_header(&["--> Section1.A <--\n"]);
        w.push_section_with_header(&["--> Section1.B <--\n"]);
        writeln!(&mut w, "S1 content!").unwrap();
        w.push_section_with_header(&["--> Section2 <--\n"]);
        writeln!(&mut w, "S1+2 content!").unwrap();
        w.push_section_with_header(&["--> Section3 <--\n"]);
        w.pop_section_with_footer(&["<-- Section3 --->\n"]).unwrap();
        w.pop_section_with_footer(&["<-- Section2 --->\n"]).unwrap();
        w.pop_section();
        w.pop_section();

        let expected =
            "--> Section1.A <--\n--> Section1.B <--\nS1 content!\n--> Section2 <--\nS1+2 content!\n<-- Section2 --->\n";
        assert_eq!(string, expected);
    }
}
