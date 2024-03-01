use crate::{error::GeneratorError, utils::table_col_width};
use std::fmt::Write;

pub mod datasheet;

pub fn md_table(out: &mut dyn Write, rows: &Vec<Vec<String>>) -> Result<(), GeneratorError> {
    if rows.len() < 2 {
        return Err(GeneratorError::Error(format!(
            "Cannot generate markdown table from {} rows, need at least 2.",
            rows.len()
        )));
    }

    let col_widths = table_col_width(rows);

    md_table_row(out, &rows[0], &col_widths)?;
    md_sep_row(out, &col_widths)?;
    for row_idx in 1..rows.len() {
        md_table_row(out, &rows[row_idx], &col_widths)?;
    }

    Ok(())
}

fn md_table_row(out: &mut dyn Write, row: &Vec<String>, widths: &Vec<usize>) -> Result<(), GeneratorError> {
    write!(out, "| ")?;
    for (col_idx, width) in widths.iter().enumerate() {
        let col = row.get(col_idx).map_or("".to_string(), |col| col.to_owned());
        if col_idx != 0 {
            write!(out, " | ")?;
        }
        write!(out, "{col: <width$}", width = width)?;
    }
    writeln!(out, " |")?;

    Ok(())
}

fn md_sep_row(out: &mut dyn Write, widths: &Vec<usize>) -> Result<(), GeneratorError> {
    write!(out, "| ")?;
    for (col_idx, width) in widths.iter().enumerate() {
        if col_idx != 0 {
            write!(out, " | ")?;
        }
        for _ in 0..*width {
            write!(out, "-")?;
        }
    }
    writeln!(out, " |")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_md_table() {
        let mut is = String::new();

        md_table(
            &mut is,
            &vec![
                vec!["1".into(), "22222".into(), "3".into()],
                vec!["A".into(), "B".into(), "CCCC".into()],
                vec![":)".into(), "!".into()],
            ],
        )
        .unwrap();
        let should = "| 1  | 22222 | 3    |\n\
                      | -- | ----- | ---- |\n\
                      | A  | B     | CCCC |\n\
                      | :) | !     |      |\n";
        assert_eq!(is, should);
    }
}
