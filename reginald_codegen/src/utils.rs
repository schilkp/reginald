use std::fmt::Write;
use std::ops::RangeInclusive;
use std::path::Path;
use std::usize;

use crate::error::GeneratorError;
use crate::regmap::TypeValue;

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

pub fn numbers_as_ranges(mut i: Vec<TypeValue>) -> Vec<RangeInclusive<TypeValue>> {
    if i.is_empty() {
        return vec![];
    }

    if i.len() == 1 {
        let val = i[0];
        return vec![val..=val];
    }

    let mut ranges = vec![];

    let mut current_range: Option<(TypeValue, TypeValue)> = None;
    i.sort();

    for val in i {
        current_range = match current_range {
            Some((start, end)) if { end == val } => Some((start, end)),
            Some((start, end)) if { end + 1 == val } => Some((start, val)),
            Some((start, end)) => {
                ranges.push(start..=end);
                Some((val, val))
            }
            None => Some((val, val)),
        }
    }

    if let Some((start, end)) = current_range {
        ranges.push(start..=end);
    }

    ranges
}

pub fn filename(s: &Path) -> Result<String, GeneratorError> {
    s.file_name()
        .ok_or(GeneratorError::Error("".into()))
        .map(|x| x.to_string_lossy().to_string())
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

    #[test]
    fn test_numbers_as_ranges() {
        assert_eq!(numbers_as_ranges(vec![]), vec![]);
        assert_eq!(numbers_as_ranges(vec![1]), vec![1..=1]);
        assert_eq!(numbers_as_ranges(vec![2]), vec![2..=2]);
        assert_eq!(numbers_as_ranges(vec![1, 2]), vec![1..=2]);
        assert_eq!(numbers_as_ranges(vec![2, 1]), vec![1..=2]);
        assert_eq!(numbers_as_ranges(vec![2, 1, 0, 0, 4, 6, 5]), vec![0..=2, 4..=6]);
    }
}
