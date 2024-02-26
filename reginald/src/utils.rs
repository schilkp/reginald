use std::fmt::Write;
use std::usize;

use regex::Regex;

use crate::regmap::TypeBitwidth;

pub fn c_sanitize(s: &str) -> String {
    let re = Regex::new(r"[^_a-zA-Z0-9]").unwrap();
    re.replace_all(s, "_").into()
}

pub fn c_fitting_unsigned_type(width: TypeBitwidth) -> String {
    match width {
        1..=8 => "uint8_t".to_string(),
        9..=16 => "uint16_t".to_string(),
        17..=32 => "uint32_t".to_string(),
        33..=64 => "uint64_t".to_string(),
        _ => panic!(),
    }
}

pub fn str_pad_to_length(s: &str, pad_char: char, len: usize) -> String {
    let mut s = s.to_string();
    while s.len() < len {
        s.push(pad_char);
    }
    s
}

pub fn str_pad_to_table(rows: &Vec<Vec<String>>, prefix: &str, seperator: &str) -> String {
    if rows.is_empty() {
        return String::new();
    }

    // Number of cols:
    let max_cols = rows.iter().map(|row| row.len()).max().unwrap();

    // Determine maximum width of all columns:
    let mut col_widths: Vec<usize> = vec![];
    for col_idx in 0..max_cols {
        let mut col_width: usize = 0;
        for row in rows {
            if let Some(content) = row.get(col_idx) {
                col_width = usize::max(content.len(), col_width);
            }
        }
        col_widths.push(col_width);
    }

    // Output each row:
    let mut result = String::new();
    for row in rows {
        let mut line = String::new();
        for (col_idx, col) in row.iter().enumerate() {
            if col_idx == 0 {
                line.push_str(prefix);
            } else {
                line.push_str(seperator);
            }
            write!(&mut line, "{col: <width$}", width = col_widths[col_idx]).unwrap();
        }
        result.push_str(line.trim_end());
        result.push('\n')
    }
    result
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_str_pad_to_table() {
        let is = str_pad_to_table(
            &vec![
                vec!["1".into(), "22222".into(), "3".into()],
                vec!["A".into(), "B".into(), "CCCC".into()],
                vec![":)".into(), "!".into()],
            ],
            "$ ",
            " | ",
        );
        let should = "$ 1  | 22222 | 3\n$ A  | B     | CCCC\n$ :) | !\n";
        assert_eq!(is, should);
    }
}
