//! A small CSV reader for the world tables: comma-separated, a header row, fields quoted with
//! `"` when they hold a comma or a quote (a quote inside is doubled), no line break inside a
//! field. Blank lines are skipped. It is strict: every row has the header's width, every column
//! the table expects is there, and no other.

use crate::CompileError;

/// A parsed table: its file name, its header, and each row with its line number.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Table {
    pub file: String,
    pub header: Vec<String>,
    pub rows: Vec<Row>,
}

/// One row: its 1-based line number in the file and its fields, in header order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Row {
    pub line: usize,
    pub fields: Vec<String>,
}

impl Table {
    /// Parse `text` as the table `file`, whose header must hold exactly the columns `expect`,
    /// in that order.
    pub fn parse(file: &str, text: &str, expect: &[&str]) -> Result<Table, CompileError> {
        if text.contains('\r') {
            return Err(CompileError::new(
                file,
                "line endings are LF only (.gitattributes)",
            ));
        }
        let mut lines = text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty());
        let Some((_, head)) = lines.next() else {
            return Err(CompileError::new(file, "the table is empty"));
        };
        let header = split(file, 1, head)?;
        if header != expect {
            return Err(CompileError::new(
                format!("{file}:1"),
                format!(
                    "the header is `{}`, not the expected `{}`",
                    header.join(","),
                    expect.join(",")
                ),
            ));
        }
        let mut rows = Vec::new();
        for (i, l) in lines {
            let fields = split(file, i + 1, l)?;
            if fields.len() != header.len() {
                return Err(CompileError::new(
                    format!("{file}:{}", i + 1),
                    format!(
                        "{} fields where the header has {}",
                        fields.len(),
                        header.len()
                    ),
                ));
            }
            rows.push(Row {
                line: i + 1,
                fields,
            });
        }
        Ok(Table {
            file: file.to_string(),
            header,
            rows,
        })
    }

    /// The index of a column the header was checked to hold.
    pub fn col(&self, name: &str) -> usize {
        self.header
            .iter()
            .position(|h| h == name)
            .unwrap_or(usize::MAX)
    }
}

impl Row {
    /// The field of column `i`.
    pub fn get(&self, i: usize) -> &str {
        self.fields.get(i).map_or("", String::as_str)
    }
}

/// Split one line into fields.
fn split(file: &str, line: usize, l: &str) -> Result<Vec<String>, CompileError> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut at_start = true;
    let mut chars = l.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            ('"', true) => {
                quoted = false;
                if !matches!(chars.peek(), None | Some(',')) {
                    return Err(CompileError::new(
                        format!("{file}:{line}"),
                        "a closing quote is followed by something other than a comma",
                    ));
                }
            }
            ('"', false) if at_start => quoted = true,
            ('"', false) => {
                return Err(CompileError::new(
                    format!("{file}:{line}"),
                    "a quote inside an unquoted field",
                ))
            }
            (',', false) => {
                out.push(std::mem::take(&mut cur));
                at_start = true;
                continue;
            }
            _ => cur.push(c),
        }
        at_start = false;
    }
    if quoted {
        return Err(CompileError::new(
            format!("{file}:{line}"),
            "a quoted field is not closed",
        ));
    }
    out.push(cur);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_fields_and_doubled_quotes() {
        let t = Table::parse(
            "t.csv",
            "a,b,c\n1,\"x, y\",\"say \"\"hi\"\"\"\n\n2,,z\n",
            &["a", "b", "c"],
        )
        .unwrap();
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0].fields, ["1", "x, y", "say \"hi\""]);
        assert_eq!(t.rows[1].line, 4);
        assert_eq!(t.rows[1].fields, ["2", "", "z"]);
    }

    #[test]
    fn strict_about_shape() {
        assert!(Table::parse("t.csv", "a,b\n1\n", &["a", "b"]).is_err());
        assert!(Table::parse("t.csv", "a,c\n1,2\n", &["a", "b"]).is_err());
        assert!(Table::parse("t.csv", "a,b\n1,\"2\n", &["a", "b"]).is_err());
        assert!(Table::parse("t.csv", "a,b\n1,x\"y\n", &["a", "b"]).is_err());
        assert!(Table::parse("t.csv", "a,b\r\n1,2\r\n", &["a", "b"]).is_err());
        assert!(Table::parse("t.csv", "", &["a"]).is_err());
    }
}
