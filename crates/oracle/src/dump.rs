//! A one-line text interface to the solver, for differential testing.
//!
//! `examples/dump.rs` hands stdin and stdout to [`run`], which passes each input line to
//! [`line()`] and writes the result. An input line is whitespace-separated `key=value`
//! pairs, one for each of [`KEYS`]:
//!
//! ```text
//! workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1
//! ```
//!
//! The output line starts with `regime=<name>`. An interior equilibrium then lists every
//! output of [`Eq1a::outputs`](crate::Eq1a::outputs) as `key=value`, in that order; the
//! other regimes list their diagnostic. Floats are printed with Rust's `{:?}` formatting:
//! the shortest digits that parse back to the same double, always with a decimal point or
//! an exponent (`1.0`, `0.863150418162437`, `-6.000533403494046e-12`, `1e30`), so a tiny
//! or huge value stays short. Any `float()` or `str::parse::<f64>` reads them. A flag
//! prints as `true` or `false`, a flow-only output that is absent as `none`, and
//! `bisection_steps` as an integer. A line that cannot be read or solved gives
//! `error=<message>`, where the message runs to the end of the line. Every input line
//! gives exactly one output line.

use std::io::{self, BufRead, Write};
use std::num::IntErrorKind;

use crate::params::{Economy, Params, UniformWorkCost};
use crate::schedule::PowerSchedule;
use crate::solve::{Output, Regime};

/// The keys an input line must carry, each exactly once. `eta`, `g0`, `g1` and `k` are
/// the [`PowerSchedule`]'s, and `chi_max` is the [`UniformWorkCost`]'s.
pub const KEYS: [&str; 14] = [
    "workers",
    "land",
    "space",
    "a",
    "lam",
    "b",
    "eta",
    "g0",
    "g1",
    "k",
    "chi_max",
    "rho",
    "delta",
    "build_lag",
];

/// Solves the instance on one input line and returns one output line, without a newline.
pub fn line(input: &str) -> String {
    let params = match parse(input) {
        Ok(p) => p,
        Err(message) => return format!("error={message}"),
    };
    let economy = match Economy::new(params) {
        Ok(e) => e,
        Err(e) => return format!("error=invalid parameter: {e}"),
    };
    match economy.solve() {
        Ok(regime) => format_regime(&regime),
        Err(e) => format!("error=solve failed: {e}"),
    }
}

/// Parses one input line into parameters, checking keys but not ranges.
pub fn parse(input: &str) -> Result<Params, String> {
    let mut values: [Option<&str>; KEYS.len()] = [None; KEYS.len()];
    let mut tokens = 0;
    for token in input.split_whitespace() {
        tokens += 1;
        let (key, value) = token
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {token:?}"))?;
        let slot = KEYS
            .iter()
            .position(|k| *k == key)
            .ok_or_else(|| format!("unknown key {key:?}"))?;
        if values[slot].replace(value).is_some() {
            return Err(format!("duplicate key {key:?}"));
        }
    }
    if tokens == 0 {
        return Err("empty line".to_string());
    }
    let missing: Vec<&str> = KEYS
        .iter()
        .zip(&values)
        .filter(|(_, v)| v.is_none())
        .map(|(k, _)| *k)
        .collect();
    if !missing.is_empty() {
        return Err(format!("missing keys {}", missing.join(" ")));
    }
    let text = |key: &str| -> Result<&str, String> {
        KEYS.iter()
            .position(|k| *k == key)
            .and_then(|slot| values[slot])
            .ok_or_else(|| format!("missing key {key:?}"))
    };
    let num = |key: &str| -> Result<f64, String> {
        let raw = text(key)?;
        raw.parse::<f64>()
            .map_err(|_| format!("{key}={raw} is not a number"))
    };
    let raw_lag = text("build_lag")?;
    let build_lag = raw_lag.parse::<u32>().map_err(|e| match e.kind() {
        IntErrorKind::PosOverflow => format!(
            "build_lag={raw_lag} is too large: at most u32::MAX = {}",
            u32::MAX
        ),
        _ => format!("build_lag={raw_lag} is not a non-negative integer"),
    })?;
    Ok(Params {
        workers: num("workers")?,
        land: num("land")?,
        space: num("space")?,
        a: num("a")?,
        lam: num("lam")?,
        b: num("b")?,
        schedule: PowerSchedule {
            eta: num("eta")?,
            g0: num("g0")?,
            g1: num("g1")?,
            k: num("k")?,
        },
        work_cost: UniformWorkCost {
            chi_max: num("chi_max")?,
        },
        rho: num("rho")?,
        delta: num("delta")?,
        build_lag,
    })
}

/// Reads economies from `input`, one per line, and writes one result line per economy
/// to `output`, then flushes it.
///
/// Lines end at `\n`; a `\r` before it is whitespace to [`line()`], and a last line
/// without a newline is still read. A line that is not UTF-8 gives
/// `error=line is not UTF-8`. Every input line, blank ones included, gives exactly one
/// output line.
///
/// A failure to read `input` or to write or flush `output` ends the run with that error,
/// after the lines already written. `examples/dump.rs` then reports it on stderr and exits
/// with status 1, so a truncated run is never mistaken for a complete one.
pub fn run(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    for chunk in input.split(b'\n') {
        let bytes = chunk?;
        let result = match std::str::from_utf8(&bytes) {
            Ok(text) => line(text),
            Err(_) => "error=line is not UTF-8".to_string(),
        };
        writeln!(output, "{result}")?;
    }
    output.flush()
}

fn format_regime(regime: &Regime) -> String {
    let mut out = format!("regime={}", regime.name());
    let diagnostic = match regime {
        Regime::Interior(eq) => {
            for (key, value) in eq.outputs() {
                let text = match value {
                    Output::Float(v) | Output::FlowOnly(Some(v)) => format!("{v:?}"),
                    Output::FlowOnly(None) => "none".to_string(),
                    Output::Flag(b) => b.to_string(),
                    Output::Count(n) => n.to_string(),
                };
                out.push_str(&format!(" {key}={text}"));
            }
            return out;
        }
        Regime::BoundaryNoMargin { f_at_1 } => ("f_at_1", *f_at_1),
        Regime::NotViable { d_at_1 } => ("d_at_1", *d_at_1),
        Regime::NoInteriorAtZero { f_at_0 } => ("f_at_0", *f_at_0),
    };
    out.push_str(&format!(" {}={:?}", diagnostic.0, diagnostic.1));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const APPENDIX_B: &str = "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 \
                              g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1";

    fn with(key: &str, value: &str) -> String {
        APPENDIX_B
            .split_whitespace()
            .map(|t| {
                if t.split('=').next() == Some(key) {
                    format!("{key}={value}")
                } else {
                    t.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn parses_every_key_into_its_own_field() {
        // Fourteen distinct values, none of them 1, so a key read into the wrong field
        // cannot go unnoticed (the review's parser mutants swapped eta, k and space).
        let input = "workers=5.2 land=12.5 space=0.7 a=0.22 lam=0.08 b=0.55 eta=2.3 g0=0.15 \
                     g1=0.9 k=2.5 chi_max=1.6 rho=0.04 delta=0.35 build_lag=2";
        let want = Params {
            workers: 5.2,
            land: 12.5,
            space: 0.7,
            a: 0.22,
            lam: 0.08,
            b: 0.55,
            schedule: PowerSchedule {
                eta: 2.3,
                g0: 0.15,
                g1: 0.9,
                k: 2.5,
            },
            work_cost: UniformWorkCost { chi_max: 1.6 },
            rho: 0.04,
            delta: 0.35,
            build_lag: 2,
        };
        assert_eq!(parse(input).unwrap(), want);
        // Each key moved to the front of the line still lands in its field.
        let tokens: Vec<&str> = input.split_whitespace().collect();
        for i in 0..tokens.len() {
            let mut moved = tokens.clone();
            let token = moved.remove(i);
            moved.insert(0, token);
            assert_eq!(parse(&moved.join(" ")).unwrap(), want, "{token} first");
        }
    }

    #[test]
    fn parses_the_appendix_b_line() {
        let p = parse(APPENDIX_B).unwrap();
        assert_eq!((p.workers, p.land, p.a, p.build_lag), (4.0, 10.0, 0.3, 1));
        assert_eq!(
            p.schedule,
            PowerSchedule {
                eta: 1.0,
                g0: 0.2,
                g1: 0.8,
                k: 1.0
            }
        );
        // Order and spacing do not matter; CRLF line ends are whitespace.
        let shuffled: Vec<&str> = APPENDIX_B.split_whitespace().rev().collect();
        assert_eq!(parse(&format!("  {}\r", shuffled.join("\t"))).unwrap(), p);
    }

    #[test]
    fn bad_lines_are_error_lines() {
        for (input, needle) in [
            (String::new(), "empty line"),
            ("   \t ".to_string(), "empty line"),
            (format!("{APPENDIX_B} colour=blue"), "unknown key"),
            (format!("{APPENDIX_B} a=0.3"), "duplicate key"),
            (format!("{APPENDIX_B} stray"), "expected key=value"),
            (APPENDIX_B.replace(" rho=0", ""), "missing keys rho"),
            (with("a", "zero"), "a=zero is not a number"),
            (
                with("build_lag", "1.5"),
                "build_lag=1.5 is not a non-negative integer",
            ),
            (
                with("build_lag", "-1"),
                "build_lag=-1 is not a non-negative integer",
            ),
            // One above u32::MAX is a non-negative integer, only too large.
            (
                with("build_lag", "4294967296"),
                "build_lag=4294967296 is too large: at most u32::MAX = 4294967295",
            ),
            (with("a", "1.5"), "invalid parameter: a = 1.5"),
            (
                with("delta", "NaN"),
                "invalid parameter: delta = NaN is not finite",
            ),
            (with("build_lag", "0"), "invalid parameter: build_lag"),
        ] {
            let out = line(&input);
            assert!(out.starts_with("error="), "{input:?} gave {out}");
            assert!(
                out.contains(needle),
                "{input:?} gave {out}, expected {needle:?}"
            );
            assert!(!out.contains('\n'));
        }
    }

    #[test]
    fn non_interior_lines_carry_their_diagnostic() {
        let out = line(&with("lam", "0.8"));
        assert!(out.starts_with("regime=NotViable d_at_1=-0.1"), "{out}");
        let out = line(&with("workers", "0.25"));
        assert!(
            out.starts_with("regime=BoundaryNoMargin f_at_1=0.22635"),
            "{out}"
        );
    }

    #[test]
    fn interior_line_spot_checks() {
        // The gate's dump_line tests compare every key with the solve; this checks the
        // line's shape.
        let out = line(APPENDIX_B);
        let economy = Economy::new(parse(APPENDIX_B).unwrap()).unwrap();
        let eq = economy.solve().unwrap();
        let eq = eq.interior().unwrap();
        let field = |key: &str| -> &str {
            out.split_whitespace()
                .find_map(|t| t.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
                .unwrap_or_else(|| panic!("{key} missing from {out}"))
        };
        assert_eq!(field("regime"), "Interior");
        assert_eq!(
            field("x_star").parse::<f64>().unwrap().to_bits(),
            eq.x_star.to_bits()
        );
        assert_eq!(field("u"), "1.0");
        assert_eq!(field("interest"), "0.0");
        assert_eq!(field("funded"), "true");
        assert_eq!(field("lambda_tilde_space"), "0.0");
        assert_eq!(field("b_tilde_space"), "1.0");
        assert_eq!(field("bisection_steps"), eq.bisection_steps.to_string());
        // At u ≠ 1 the flow-only outputs print as none.
        let durable = line(&with("rho", "0.05"));
        assert!(
            durable.contains(" phi_w=none phi_r=none lambda_tilde_good=none"),
            "{durable}"
        );
    }

    /// A reader whose every read fails.
    struct Broken;

    impl io::Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("disk on fire"))
        }
    }

    impl BufRead for Broken {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(io::Error::other("disk on fire"))
        }
        fn consume(&mut self, _: usize) {}
    }

    /// A writer that accepts `room` bytes and then fails, like a full disk.
    struct Full {
        written: Vec<u8>,
        room: usize,
    }

    impl Write for Full {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.written.len() + buf.len() > self.room {
                return Err(io::Error::new(io::ErrorKind::StorageFull, "no space"));
            }
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn run_writes_one_line_per_input_line() {
        // A valid line, a blank line, bytes that are not UTF-8, a CRLF line, and a last
        // line with no newline: five lines in, five out.
        let mut input = Vec::new();
        input.extend_from_slice(APPENDIX_B.as_bytes());
        input.extend_from_slice(b"\n\n\xff\xfe\n");
        input.extend_from_slice(APPENDIX_B.as_bytes());
        input.extend_from_slice(b"\r\n");
        input.extend_from_slice(with("lam", "0.8").as_bytes());
        let mut output = Vec::new();
        run(&input[..], &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 5, "{text}");
        assert!(text.ends_with('\n'));
        assert_eq!(lines[0], line(APPENDIX_B));
        assert_eq!(lines[1], "error=empty line");
        assert_eq!(lines[2], "error=line is not UTF-8");
        assert_eq!(lines[3], line(APPENDIX_B));
        assert!(lines[4].starts_with("regime=NotViable"), "{}", lines[4]);
        // No input, no output.
        let mut output = Vec::new();
        run(&b""[..], &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn run_reports_read_and_write_failures() {
        let err = run(Broken, Vec::new()).unwrap_err();
        assert_eq!(err.to_string(), "disk on fire");
        // Room for the first result line but not the second.
        let first = line(APPENDIX_B);
        let input = format!("{APPENDIX_B}\n{APPENDIX_B}\n");
        let mut full = Full {
            written: Vec::new(),
            room: first.len() + 1,
        };
        let err = run(input.as_bytes(), &mut full).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::StorageFull);
        assert_eq!(full.written, format!("{first}\n").into_bytes());
    }

    /// A writer that takes every byte but cannot flush them, like a closed pipe behind a
    /// buffer.
    struct Unflushable(Vec<u8>);

    impl Write for Unflushable {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
    }

    #[test]
    fn run_reports_a_failed_flush() {
        let mut out = Unflushable(Vec::new());
        let err = run(APPENDIX_B.as_bytes(), &mut out).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(out.0, format!("{}\n", line(APPENDIX_B)).into_bytes());
    }

    #[test]
    fn negative_zero_prints_as_zero() {
        // rho = -0.0 and lambda = -0.0 are the economies at 0 and print identically, with
        // no -0.0 anywhere (Economy::new stores +0.0).
        for key in ["rho", "lam", "a"] {
            let minus = line(&with(key, "-0.0"));
            assert_eq!(minus, line(&with(key, "0")), "{key}");
            assert!(!minus.contains("-0.0"), "{minus}");
        }
    }

    #[test]
    fn extreme_floats_print_with_an_exponent() {
        // g1 at SCALE_CEIL = 1e30 makes D(1) about -5e28: printed as an exponent, not 29
        // digits.
        let out = line(&with("g1", "1e30"));
        assert!(out.starts_with("regime=NotViable d_at_1=-"), "{out}");
        assert!(out.len() < 50, "{out}");
        assert!(out.contains("e28"), "{out}");
        let d: f64 = out.rsplit('=').next().unwrap().parse().unwrap();
        assert!(d < -1e28, "{out}");
        // A diagnostic of order 1e-12 is short too.
        let out = line(
            &with("workers", "10")
                .replace("chi_max=1", "chi_max=1e-3")
                .replace("land=10", "land=10.000000000005"),
        );
        assert!(out.starts_with("regime=NoInteriorAtZero f_at_0=-"), "{out}");
        assert!(out.contains("e-12"), "{out}");
    }
}
