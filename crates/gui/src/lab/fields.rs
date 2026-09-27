//! A point's numbers, read from the oracle's own `Debug` (docs/GUI.md §7.3's "the lab keeps no
//! copy of the fields"): every point type of units 1a–1f derives `Debug`, which prints each
//! field by name and each float by the shortest digits that parse back to the same double, so
//! reading the text back gives the oracle's doubles bit for bit, and a field the oracle adds is
//! listed without a change here.
//!
//! The derived form is `Name { field: value, … }`, with `[a, b]` for a vector, `Some(v)` and
//! `None`, `Variant`, `Variant(v, …)` and `Variant { … }` for an enum, and `true` or `false`.
//! Every number becomes one leaf, named by its path: `gamma`, `prices[2]`, `short.worker`. A
//! count reads as its value. Flags, names and text are not numbers and are left out; so is an
//! absent `None`.

/// The name f(x) = n_D − n_S goes by among a point's fields.
pub const F: &str = "f";

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(f64),
    Text,
    Punct(char),
}

fn tokens(s: &str) -> Vec<Tok> {
    let c: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let n = c.len();
    while i < n {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch == '"' {
            i += 1;
            while i < n && c[i] != '"' {
                if c[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            out.push(Tok::Text);
        } else if ch.is_ascii_digit()
            || (ch == '-' && i + 1 < n && (c[i + 1].is_ascii_digit() || c[i + 1] == 'i'))
        {
            let start = i;
            i += 1;
            while i < n
                && (c[i].is_ascii_alphanumeric()
                    || c[i] == '.'
                    || ((c[i] == '-' || c[i] == '+') && matches!(c[i - 1], 'e' | 'E')))
            {
                i += 1;
            }
            let text: String = c[start..i].iter().collect();
            match text.parse::<f64>() {
                Ok(v) => out.push(Tok::Num(v)),
                Err(_) => out.push(Tok::Ident(text)),
            }
        } else if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < n && (c[i].is_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Ident(c[start..i].iter().collect()));
        } else {
            out.push(Tok::Punct(ch));
            i += 1;
        }
    }
    out
}

struct Reader {
    toks: Vec<Tok>,
    at: usize,
    out: Vec<(String, f64)>,
}

impl Reader {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn punct(&self, ch: char) -> bool {
        self.peek() == Some(&Tok::Punct(ch))
    }

    fn eat(&mut self, ch: char) -> bool {
        let yes = self.punct(ch);
        if yes {
            self.at += 1;
        }
        yes
    }

    /// One value at `path`; false when the text is not a derived `Debug` here.
    fn value(&mut self, path: &str) -> bool {
        match self.peek().cloned() {
            Some(Tok::Num(v)) => {
                self.at += 1;
                self.out.push((path.to_string(), v));
                true
            }
            Some(Tok::Text) => {
                self.at += 1;
                true
            }
            Some(Tok::Punct('[')) => {
                self.at += 1;
                let mut k = 0;
                while !self.eat(']') {
                    if !self.value(&format!("{path}[{k}]")) {
                        return false;
                    }
                    k += 1;
                    if !self.eat(',') && !self.punct(']') {
                        return false;
                    }
                }
                true
            }
            Some(Tok::Ident(name)) => {
                self.at += 1;
                // `NaN` and `inf` are how a float prints; as a field's name they come before
                // a colon, which this position never sees.
                let bare = !self.punct('{') && !self.punct('(');
                if bare && (name == "NaN" || name == "inf") {
                    let v = if name == "inf" {
                        f64::INFINITY
                    } else {
                        f64::NAN
                    };
                    self.out.push((path.to_string(), v));
                    return true;
                }
                if self.eat('{') {
                    while !self.eat('}') {
                        let Some(Tok::Ident(field)) = self.peek().cloned() else {
                            return false;
                        };
                        self.at += 1;
                        if !self.eat(':') {
                            return false;
                        }
                        let p = if path.is_empty() {
                            field
                        } else {
                            format!("{path}.{field}")
                        };
                        if !self.value(&p) {
                            return false;
                        }
                        if !self.eat(',') && !self.punct('}') {
                            return false;
                        }
                    }
                    true
                } else if self.eat('(') {
                    // `Some(v)` is its value; a tuple variant's fields are numbered.
                    let mut k = 0;
                    let mut items = Vec::new();
                    while !self.eat(')') {
                        let mark = self.out.len();
                        if !self.value(&format!("{path}.{k}")) {
                            return false;
                        }
                        items.push(mark);
                        k += 1;
                        if !self.eat(',') && !self.punct(')') {
                            return false;
                        }
                    }
                    if name == "Some" && k == 1 {
                        let prefix = format!("{path}.0");
                        for (p, _) in &mut self.out[items[0]..] {
                            if let Some(rest) = p.strip_prefix(&prefix) {
                                *p = format!("{path}{rest}");
                            }
                        }
                    }
                    true
                } else {
                    // A unit variant, `None`, `true` or `false`: no number.
                    true
                }
            }
            _ => false,
        }
    }
}

/// Every number in a derived `Debug` text, by its path, in the order printed. A text that does
/// not read as one gives the numbers read before the point it stopped at.
pub fn read(debug: &str) -> Vec<(String, f64)> {
    let mut r = Reader {
        toks: tokens(debug),
        at: 0,
        out: Vec::new(),
    };
    r.value("");
    r.out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    #[allow(dead_code)]
    enum Short {
        Pool,
        Reserved(usize),
        Walk { worker: usize, by: f64 },
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Probe {
        x: f64,
        tiny: f64,
        huge: f64,
        neg: f64,
        nan: f64,
        inf: f64,
        ninf: f64,
        count: usize,
        prices: Vec<f64>,
        walled: Vec<bool>,
        matrix: Vec<Vec<f64>>,
        short: Option<Short>,
        other: Option<Short>,
        none: Option<f64>,
        some: Option<f64>,
        name: &'static str,
        pool: Short,
    }

    #[test]
    fn every_number_of_a_derived_debug_is_read_bit_for_bit() {
        let p = Probe {
            x: 0.863150418162437,
            tiny: 4.545_858_639_008_228e-13,
            huge: -3.2e300,
            neg: -0.1,
            nan: f64::NAN,
            inf: f64::INFINITY,
            ninf: f64::NEG_INFINITY,
            count: 3,
            prices: vec![0.1 + 0.2, 1.0 / 3.0],
            walled: vec![true, false],
            matrix: vec![vec![1.0, 2.0], vec![]],
            short: Some(Short::Walk {
                worker: 2,
                by: 1e-300,
            }),
            other: Some(Short::Reserved(7)),
            none: None,
            some: Some(5e-324),
            name: "a \"quoted\", name",
            pool: Short::Pool,
        };
        let got = read(&format!("{p:?}"));
        let want: Vec<(&str, f64)> = vec![
            ("x", p.x),
            ("tiny", p.tiny),
            ("huge", p.huge),
            ("neg", p.neg),
            ("nan", f64::NAN),
            ("inf", f64::INFINITY),
            ("ninf", f64::NEG_INFINITY),
            ("count", 3.0),
            ("prices[0]", p.prices[0]),
            ("prices[1]", p.prices[1]),
            ("matrix[0][0]", 1.0),
            ("matrix[0][1]", 2.0),
            ("short.worker", 2.0),
            ("short.by", 1e-300),
            ("other.0", 7.0),
            ("some", 5e-324),
        ];
        assert_eq!(got.len(), want.len(), "{got:?}");
        for ((k, v), (wk, wv)) in got.iter().zip(&want) {
            assert_eq!(k, wk);
            assert_eq!(v.to_bits(), wv.to_bits(), "{k}");
        }
    }
}
