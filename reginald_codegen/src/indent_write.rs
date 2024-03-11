use std::fmt::Write;

pub struct IndentWrite<'a> {
    pub w: &'a mut dyn Write,
    indent: String,
    current_indent: usize,
    newline_buffered: bool,
}

impl Write for IndentWrite<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            self.process_char(&c)?
        }
        Ok(())
    }
}

impl IndentWrite<'_> {
    pub fn new<'a>(w: &'a mut dyn Write, indent: &str) -> IndentWrite<'a> {
        IndentWrite {
            w,
            indent: indent.to_string(),
            current_indent: 0,
            newline_buffered: false,
        }
    }

    pub fn push_indent(&mut self) {
        self.current_indent += 1;
    }

    pub fn pop_indent(&mut self) {
        if self.current_indent == 0 {
            panic!("Cannot reduce indent below 0");
        }
        self.current_indent -= 1;
    }

    fn process_char(&mut self, c: &char) -> std::fmt::Result {
        match (self.newline_buffered, c) {
            (false, '\n') => {
                // Buffer newline;
                self.newline_buffered = true;
                Ok(())
            }
            (false, c) => {
                // Write-through character:
                self.w.write_char(*c)
            }
            (true, '\n') => {
                // Empty newline. Write-through empty newline without
                // indent and keep new newline buffered.
                self.w.write_char('\n')
            }
            (true, c) => {
                // First character of new line. Emit newline, indent and character.
                self.w.write_char('\n')?;
                self.newline_buffered = false;
                for _ in 0..self.current_indent {
                    self.w.write_str(&self.indent)?
                }
                self.w.write_char(*c)
            }
        }
    }

    pub fn flush(&mut self) -> std::fmt::Result {
        if self.newline_buffered {
            self.w.write_char('\n')?;
        }
        self.newline_buffered = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write, iter::zip};

    use super::IndentWrite;
    #[test]
    fn test_indent_write() {
        let mut string = String::new();
        let mut w = IndentWrite::new(&mut string, "_.-");

        writeln!(&mut w, "| no indent").unwrap();
        w.push_indent();
        writeln!(&mut w, "| 1").unwrap();
        writeln!(&mut w, "| 1").unwrap();
        w.push_indent();
        writeln!(&mut w, "| 2").unwrap();
        writeln!(&mut w).unwrap();
        w.pop_indent();
        w.pop_indent();
        writeln!(&mut w, "| no indent").unwrap();

        w.flush().unwrap();

        println!("is: ");
        println!("{}", string);

        let should = "| no indent\n\
                      _.-| 1\n\
                      _.-| 1\n\
                      _.-_.-| 2\n\
                      \n\
                      | no indent\n";

        for (idx, (is, should)) in zip(string.lines(), should.lines()).enumerate() {
            println!("{idx}");
            assert_eq!(is, should);
        }

        assert_eq!(string, should);
    }
}
