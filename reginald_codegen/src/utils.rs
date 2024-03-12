use std::fmt::Write;
use std::ops::{Add, RangeInclusive};
use std::path::Path;
use std::usize;

use crate::error::Error;

/// Pad string 's' to length 'len' with characters char.
///
/// Example:
/// ```rust
/// # use reginald_codegen::utils::str_pad_to_length;
/// let s = str_pad_to_length("Hi!", '_', 10);
/// assert_eq!(s, String::from("Hi!_______"));
/// ```
pub fn str_pad_to_length(s: &str, pad_char: char, len: usize) -> String {
    let mut s = s.to_string();
    while s.len() < len {
        s.push(pad_char);
    }
    s
}

/// Determine the width of each column from a list of rows.
///
/// Returns the vector `len`, where `len[i]` is the maximum length of
/// the set of strings in position i in every row.
///
/// Example:
/// ```rust
/// # use reginald_codegen::utils::table_col_width;
/// let t: Vec<Vec<String>> = vec![vec![":)".into(), "Loooooong".into(), "!".into()],
///                                vec![                                           ],
///                                vec!["?".into(),  "".into()                     ]];
/// assert_eq!(table_col_width(&t), vec![2, 9, 1]);
/// ```
pub fn table_col_width(rows: &Vec<Vec<String>>) -> Vec<usize> {
    if rows.is_empty() {
        return vec![];
    }

    // Number of cols:
    let max_cols = rows.iter().map(std::vec::Vec::len).max().unwrap();

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

    col_widths
}

/// Takes a list of strings and formats them into a left-aligned table. Columns
/// are seperated by 'seperator', and every line is prefixed by 'prefix'. All
/// trailing spaces are trimmed.
///
/// Example:
/// ```rust
/// # use reginald_codegen::utils::str_table;
/// let rows: Vec<Vec<String>> = vec![vec![":)".into(), "Loooooong".into(), "!".into()],
///                                   vec![                                           ],
///                                   vec!["?".into(),  "".into()                     ]];
///
/// let should = "-> :) | Loooooong | !\n\
///               ->\n\
///               -> ?  |\n".to_string();
///
/// assert_eq!(str_table(&rows, "-> ", " | "), should);
/// ```
pub fn str_table(rows: &Vec<Vec<String>>, prefix: &str, seperator: &str) -> String {
    if rows.is_empty() {
        return String::from('\n');
    }

    let col_widths = table_col_width(rows);

    // Output each row:
    let mut result = String::new();
    for row in rows {
        let mut line = String::new();
        line.push_str(prefix);
        for (col_idx, col) in row.iter().enumerate() {
            if col_idx != 0 {
                line.push_str(seperator);
            }
            write!(&mut line, "{col: <width$}", width = col_widths[col_idx]).unwrap();
        }
        result.push_str(line.trim_end());
        result.push('\n');
    }
    result
}

/// Takes a list of numbers, and greedily collects each consecutive
/// range into an inclusive range.
///
/// Example:
/// ```rust
/// # use reginald_codegen::utils::numbers_as_ranges;
/// let ranges = numbers_as_ranges(vec![1,3,2,5,101,100]);
/// assert_eq!(ranges, vec![1..=3, 5..=5, 100..=101]);
/// ```
pub fn numbers_as_ranges<T>(mut i: Vec<T>) -> Vec<RangeInclusive<T>>
where
    T: Ord + From<u8> + Add<T, Output = T> + Eq + Copy,
{
    if i.is_empty() {
        return vec![];
    }

    if i.len() == 1 {
        let val = i[0];
        return vec![val..=val];
    }

    let mut ranges = vec![];

    let mut current_range: Option<(T, T)> = None;
    i.sort();

    for val in i {
        current_range = match current_range {
            Some((start, end)) if { end == val } => Some((start, end)),
            Some((start, end)) if { end + T::from(1) == val } => Some((start, val)),
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

/// Attempt to extract the filename from a path.
pub fn filename(s: &Path) -> Result<String, Error> {
    s.file_name()
        .ok_or(Error::GeneratorError(String::from("Could not extract filename from path")))
        .map(|x| x.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_str_table() {
        let is = str_table(
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
        assert_eq!(numbers_as_ranges::<u8>(vec![]), vec![]);
        assert_eq!(numbers_as_ranges(vec![1]), vec![1..=1]);
        assert_eq!(numbers_as_ranges(vec![2]), vec![2..=2]);
        assert_eq!(numbers_as_ranges(vec![1, 2]), vec![1..=2]);
        assert_eq!(numbers_as_ranges(vec![2, 1]), vec![1..=2]);
        assert_eq!(numbers_as_ranges(vec![2, 1, 0, 0, 4, 6, 5]), vec![0..=2, 4..=6]);
    }
}
