//! The dump interface (examples/dump.rs): it reproduces G1, G4 and G5 from their lines,
//! and every key it prints carries exactly the matching field of the solve.

use std::collections::BTreeMap;

use oracle::{dump, Economy, Eq1a, Params, Regime};

use crate::g5_general_instances::durable;
use crate::goldens::*;
use crate::support::*;

/// The SSRN Appendix B instance (SSRN p.30) as a dump input line.
const APPENDIX_B_LINE: &str = "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 \
                               g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1";

/// G4 case B, (ρ, δ, J_b) = (0.05, 0.1, 1): u = 0.15, with interest.
const G4_B_LINE: &str = "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 \
                         g1=0.8 k=1 chi_max=1 rho=0.05 delta=0.1 build_lag=1";

/// G4 case D, (0.05, 0.1, 3): u = 0.165375, with a build lag.
const G4_D_LINE: &str = "workers=4 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 \
                         g1=0.8 k=1 chi_max=1 rho=0.05 delta=0.1 build_lag=3";

/// G8's N = 7.35: funded = true but lemma_b1 = false, so the two flags differ.
const G8_N7_35_LINE: &str = "workers=7.35 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 \
                             g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1";

/// G8's N = 8: funded = false and lemma_b1 = false.
const G8_N8_LINE: &str = "workers=8 land=10 space=1 a=0.3 lam=0.05 b=0.4 eta=1 g0=0.2 \
                          g1=0.8 k=1 chi_max=1 rho=0 delta=1 build_lag=1";

/// G5's durable instance: fourteen distinct values, none of them 1.
const GENERAL_LINE: &str = "workers=5.2 land=12.5 space=0.7 a=0.22 lam=0.08 b=0.55 eta=2.3 \
                            g0=0.15 g1=0.9 k=2.5 chi_max=1.6 rho=0.04 delta=0.35 build_lag=2";

/// The key=value pairs of a line. A key printed twice fails.
fn fields(line: &str) -> BTreeMap<&str, &str> {
    let mut map = BTreeMap::new();
    for t in line.split_whitespace() {
        let (key, value) = t
            .split_once('=')
            .unwrap_or_else(|| panic!("bad token {t} in {line}"));
        assert!(map.insert(key, value).is_none(), "{key} twice in {line}");
    }
    map
}

/// What the dump should print for one key.
enum Want {
    Float(f64),
    Flag(bool),
    Option(Option<f64>),
    Count(u32),
}

/// Every key an interior line carries, with the [`Eq1a`] field it must print. Written
/// out here from the field docs, independently of src/dump.rs.
fn expected(eq: &Eq1a) -> Vec<(&'static str, Want)> {
    use Want::*;
    let cs = eq.cost_system;
    let r = eq.residuals;
    vec![
        ("x_star", Float(eq.x_star)),
        ("one_minus_x_star", Float(eq.one_minus_x_star)),
        ("gamma_star", Float(eq.gamma_star)),
        ("j_star", Float(eq.j_star)),
        ("u", Float(eq.u)),
        ("v", Float(eq.v)),
        ("p_m", Float(eq.p_m)),
        ("v_m", Float(eq.v_m)),
        ("p", Float(eq.p)),
        ("p_s", Float(eq.p_s)),
        ("y", Float(eq.y)),
        ("k", Float(eq.k)),
        ("final_hours", Float(eq.final_hours)),
        ("machine_hours", Float(eq.machine_hours)),
        ("n_a", Float(eq.n_a)),
        ("participation", Float(eq.participation)),
        ("income", Float(eq.income)),
        ("interest", Float(eq.interest)),
        ("labor_share", Float(eq.labor_share)),
        ("capital_share", Float(eq.capital_share)),
        ("real_wage", Float(eq.real_wage)),
        ("support_cost", Float(eq.support_cost)),
        ("worker_baskets", Float(eq.worker_baskets)),
        ("provider_baskets", Float(eq.provider_baskets)),
        ("funded", Flag(eq.funded)),
        ("lemma_b1", Flag(eq.lemma_b1)),
        ("phi_w", Option(eq.phi_w)),
        ("phi_r", Option(eq.phi_r)),
        ("lambda_tilde_good", Option(cs.map(|c| c.lambda_tilde[0]))),
        (
            "lambda_tilde_machine",
            Option(cs.map(|c| c.lambda_tilde[1])),
        ),
        ("lambda_tilde_space", Option(cs.map(|c| c.lambda_tilde[2]))),
        ("b_tilde_good", Option(cs.map(|c| c.b_tilde[0]))),
        ("b_tilde_machine", Option(cs.map(|c| c.b_tilde[1]))),
        ("b_tilde_space", Option(cs.map(|c| c.b_tilde[2]))),
        ("l_s", Option(cs.map(|c| c.l_s))),
        ("b_s", Option(cs.map(|c| c.b_s))),
        ("res_labor", Float(r.labor)),
        ("res_income", Float(r.income)),
        ("res_land", Float(r.land)),
        ("res_services", Float(r.services)),
        ("res_user_cost", Float(r.user_cost)),
        ("bisection_steps", Count(eq.bisection_steps)),
    ]
}

/// Solves `line` through the library and through the dump. The dump must print exactly
/// the keys of [`expected`] and `regime`, each float with the solve's bits. Returns the
/// dump's line.
fn matches_the_solve(line: &str) -> String {
    matches_the_solve_of(line, dump::parse(line).expect("a valid line"))
}

/// As [`matches_the_solve`], with the library's economy built from `params` rather than
/// from the dump's own parse of the line.
fn matches_the_solve_of(line: &str, params: Params) -> String {
    let economy = Economy::new(params).expect("valid");
    let eq = match economy.solve() {
        Ok(Regime::Interior(eq)) => eq,
        other => panic!("{line} is not interior: {other:?}"),
    };
    let out = dump::line(line);
    let f = fields(&out);
    assert_eq!(f.get("regime"), Some(&"Interior"), "{out}");
    let want = expected(&eq);
    let mut keys: Vec<&str> = want.iter().map(|(k, _)| *k).collect();
    keys.push("regime");
    keys.sort_unstable();
    assert_eq!(f.keys().copied().collect::<Vec<_>>(), keys, "{out}");
    let bits = |key: &str, text: &str| -> u64 {
        text.parse::<f64>()
            .unwrap_or_else(|_| panic!("{key}={text} is not a float"))
            .to_bits()
    };
    for (key, want) in want {
        let text = f[key];
        match want {
            Want::Float(v) | Want::Option(Some(v)) => {
                assert_eq!(bits(key, text), v.to_bits(), "{key}={text}, want {v:?}")
            }
            Want::Option(None) => assert_eq!(text, "none", "{key}"),
            Want::Flag(b) => assert_eq!(text, b.to_string(), "{key}"),
            Want::Count(n) => assert_eq!(text, n.to_string(), "{key}"),
        }
    }
    out
}

#[test]
fn every_key_matches_the_solve_at_u_one() {
    matches_the_solve(APPENDIX_B_LINE);
}

#[test]
fn funded_and_lemma_b1_print_under_their_own_keys() {
    // At N = 7.35 the flags differ (G8), so printing one under the other's key shows.
    let out = matches_the_solve(G8_N7_35_LINE);
    let f = fields(&out);
    assert_eq!((f["funded"], f["lemma_b1"]), ("true", "false"), "{out}");
    let out = matches_the_solve(G8_N8_LINE);
    let f = fields(&out);
    assert_eq!((f["funded"], f["lemma_b1"]), ("false", "false"), "{out}");
    let out = matches_the_solve(APPENDIX_B_LINE);
    let f = fields(&out);
    assert_eq!((f["funded"], f["lemma_b1"]), ("true", "true"), "{out}");
}

#[test]
fn every_key_matches_a_solve_built_without_the_parser() {
    // The reference economy comes from a Params literal, not from dump::parse, and every
    // one of its fourteen values is distinct, so a key parsed into the wrong field fails.
    assert_eq!(dump::parse(GENERAL_LINE).expect("a valid line"), durable());
    let out = matches_the_solve_of(GENERAL_LINE, durable());
    // And the printed values are G5's durable goldens.
    let f = fields(&out);
    let num = |key: &str| -> f64 { f[key].parse().expect("a float") };
    for (key, want) in [
        ("u", G5_DURABLE_U),
        ("x_star", G5_DURABLE_X_STAR),
        ("one_minus_x_star", G5_DURABLE_ONE_MINUS_X_STAR),
        ("gamma_star", G5_DURABLE_GAMMA_STAR),
        ("v", G5_DURABLE_V),
        ("y", G5_DURABLE_Y),
        ("n_a", G5_DURABLE_N_A),
        ("income", G5_DURABLE_INCOME),
        ("interest", G5_DURABLE_INTEREST),
        ("labor_share", G5_DURABLE_LABOR_SHARE),
        ("real_wage", G5_DURABLE_REAL_WAGE),
    ] {
        close(key, num(key), want);
    }
}

#[test]
fn every_key_matches_the_solve_with_interest() {
    // u ≠ 1: interest and the capital share are nonzero and distinct, and the flow-only
    // outputs print as none.
    for line in [G4_B_LINE, G4_D_LINE] {
        let out = matches_the_solve(line);
        let f = fields(&out);
        let num = |key: &str| -> f64 { f[key].parse().expect("a float") };
        assert!(num("interest") > 0.0 && num("capital_share") > 0.0, "{out}");
        assert_ne!(num("interest"), num("capital_share"), "{out}");
        assert_eq!(f["phi_w"], "none", "{out}");
    }
}

#[test]
fn reproduces_g4_b() {
    let out = dump::line(G4_B_LINE);
    let f = fields(&out);
    assert_eq!(f["regime"], "Interior", "{out}");
    let num = |key: &str| -> f64 { f[key].parse().expect("a float") };
    for (key, want) in [
        ("u", G4_B_U),
        ("x_star", G4_B_X_STAR),
        ("v", G4_B_V),
        ("y", G4_B_Y),
        ("n_a", G4_B_N_A),
        ("p_m", G4_B_P_M),
        ("v_m", G4_B_V_M),
        ("k", G4_B_K),
        ("income", G4_B_INCOME),
        ("capital_share", G4_B_CAPITAL_SHARE),
    ] {
        close(key, num(key), want);
    }
    // interest = capital share · income, from two goldens.
    close(
        "interest",
        num("interest"),
        G4_B_CAPITAL_SHARE * G4_B_INCOME,
    );
}

#[test]
fn reproduces_g1() {
    let out = dump::line(APPENDIX_B_LINE);
    let f = fields(&out);
    assert_eq!(f["regime"], "Interior", "{out}");
    let num = |key: &str| -> f64 {
        f.get(key)
            .unwrap_or_else(|| panic!("{key} missing"))
            .parse()
            .expect("a float")
    };
    for (key, want) in [
        ("x_star", G1_X_STAR),
        ("gamma_star", G1_GAMMA_STAR),
        ("j_star", G1_J_STAR),
        ("v", G1_V),
        ("p_m", G1_P_M),
        ("v_m", G1_P_M),
        ("p", G1_P),
        ("p_s", G1_P_S),
        ("y", G1_Y),
        ("k", G1_K),
        ("final_hours", G1_FINAL_HOURS),
        ("machine_hours", G1_MACHINE_HOURS),
        ("n_a", G1_N_A),
        ("participation", G1_PARTICIPATION),
        ("income", G1_INCOME),
        ("labor_share", G1_LABOR_SHARE),
        ("real_wage", G1_REAL_WAGE),
        ("support_cost", G1_SUPPORT_COST),
        ("worker_baskets", G1_WORKER_BASKETS),
        ("provider_baskets", G1_PROVIDER_BASKETS),
        ("phi_w", G1_PHI_W),
        ("phi_r", G1_PHI_R),
        ("lambda_tilde_good", G1_LAMBDA_TILDE_GOOD),
        ("lambda_tilde_machine", G1_LAMBDA_TILDE_MACHINE),
        ("b_tilde_good", G1_B_TILDE_GOOD),
        ("b_tilde_machine", G1_B_TILDE_MACHINE),
        ("l_s", G1_L_S),
        ("b_s", G1_B_S),
    ] {
        close(key, num(key), want);
    }
    for (key, want) in [
        ("x_star", PUB_G1_X_STAR),
        ("v", PUB_G1_V),
        ("y", PUB_G1_Y),
        ("n_a", PUB_G1_N_A),
        ("final_hours", PUB_G1_FINAL_HOURS),
        ("machine_hours", PUB_G1_MACHINE_HOURS),
        ("support_cost", PUB_G1_SUPPORT_COST),
    ] {
        near(key, num(key), want, PUBLISHED);
    }
    assert_eq!(num("u"), 1.0);
    assert_eq!(num("interest"), 0.0);
    assert_eq!(f["funded"], G1_FUNDED.to_string());
    assert_eq!(f["lemma_b1"], G1_LEMMA_B1.to_string());
    for key in ["res_income", "res_land", "res_services", "res_user_cost"] {
        assert!(num(key) <= FULL, "{key}");
    }
}

#[test]
fn every_line_gives_one_line() {
    for input in [APPENDIX_B_LINE, "", "workers=4", "nonsense", "a=b=c"] {
        let out = dump::line(input);
        assert_eq!(out.lines().count(), 1, "{input:?} gave {out:?}");
        assert!(
            out.starts_with("regime=") || out.starts_with("error="),
            "{out}"
        );
    }
}
