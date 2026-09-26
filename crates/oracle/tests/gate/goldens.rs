//! The golden numbers of unit 1a, one constant each.
//!
//! Every constant equals its line in `goldens/goldens.txt` rounded to 20 significant
//! digits, and `goldens_file::constants_match_goldens_txt` enforces that. `goldens.txt`
//! is written by `goldens/generate.py`, which solves each instance with mpmath at 70
//! digits from the equations (docs/unit-1a.md §3). `PUB_` constants are the paper's
//! published figures, transcribed. Each constant's doc comment names its source;
//! laborformal references are at 31b3482.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // G1: the SSRN Appendix B instance (SSRN p.30): N 4, T 10, h 1, a 0.3, b 0.4, lambda 0.05, gamma = 0.2 + 0.8x, chi ~ U[0, 1], rho 0, delta 1, J_b 1
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_X_STAR: f64 = 0.86315;
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_V: f64 = 0.54344;
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_Y: f64 = 7.88061;
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_N_A: f64 = 1.34338;
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_FINAL_HOURS: f64 = 1.07846;
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_MACHINE_HOURS: f64 = 0.26492;
    /// Published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482.
    PUB_G1_SUPPORT_COST: f64 = 5.44630;
    /// goldens/generate.py: x*, the root of n_D = n_S (SSRN eq 24, Lemma B.1, p.29).
    G1_X_STAR: f64 = 0.86315041816243703192;
    /// goldens/generate.py: gamma(x*).
    G1_GAMMA_STAR: f64 = 0.89052033452994962553;
    /// goldens/generate.py: J(x*), SSRN eq 21.
    G1_J_STAR: f64 = 0.47064154138208336959;
    /// goldens/generate.py: v = w/r, SSRN eq 21.
    G1_V: f64 = 0.54343596069677832842;
    /// goldens/generate.py: p_m = V_m at u = 1, SSRN eq 21.
    G1_P_M: f64 = 0.61024542576405559489;
    /// goldens/generate.py: p, SSRN eq 22.
    G1_P: f64 = 0.36157583177980927464;
    /// goldens/generate.py: P_s, SSRN eq 22.
    G1_P_S: f64 = 1.3615758317798092746;
    /// goldens/generate.py: Y, SSRN eq 23.
    G1_Y: f64 = 7.8806055249729076768;
    /// goldens/generate.py: K (the SSRN's X), SSRN eq 23.
    G1_K: f64 = 5.2984861875677308081;
    /// goldens/generate.py: Y (1 - x*).
    G1_FINAL_HOURS: f64 = 1.0784575707193308057;
    /// goldens/generate.py: lambda delta K.
    G1_MACHINE_HOURS: f64 = 0.2649243093783865404;
    /// goldens/generate.py: N_a = n_D(x*), SSRN eq 24.
    G1_N_A: f64 = 1.3433818800977173461;
    /// goldens/generate.py: N_a / N.
    G1_PARTICIPATION: f64 = 0.33584547002442933653;
    /// goldens/generate.py: N P_s.
    G1_SUPPORT_COST: f64 = 5.4463033271192370986;
    /// goldens/generate.py: I = v N_a + T, SSRN eq 15 and p.30.
    G1_INCOME: f64 = 10.730042022593547301;
    /// goldens/generate.py: v N_a / I.
    G1_LABOR_SHARE: f64 = 0.068037200698407852314;
    /// goldens/generate.py: w / P_s.
    G1_REAL_WAGE: f64 = 0.3991228016925181936;
    /// goldens/generate.py: N + v N_a / P_s, SSRN p.30.
    G1_WORKER_BASKETS: f64 = 4.5361743397275634938;
    /// goldens/generate.py: T / P_s - N, SSRN p.30.
    G1_PROVIDER_BASKETS: f64 = 3.344431185245344183;
    /// goldens/generate.py: T / (N P_s).
    G1_COVERAGE: f64 = 1.8361077963113360457;
    /// goldens/generate.py: T + interest > N P_s(x*), macro.py:113 at 31b3482.
    G1_FUNDED: bool = true;
    /// goldens/generate.py: n_S(1) > n_D(1) and T > N P_s(1), SSRN eq 25.
    G1_LEMMA_B1: bool = true;
    /// goldens/generate.py: lambda gamma(x*) / (1 - a), check_three_taxes.py:48 at 31b3482.
    G1_PHI_W: f64 = 0.063608595323567830395;
    /// goldens/generate.py: 1 - phi_w.
    G1_PHI_R: f64 = 0.9363914046764321696;
    /// goldens/generate.py: first row of (I - A)^-1 lambda, SSRN eq 4, p.29.
    G1_LAMBDA_TILDE_GOOD: f64 = 0.17046683479342606591;
    /// goldens/generate.py: lambda / (1 - a) = 1/14.
    G1_LAMBDA_TILDE_MACHINE: f64 = 0.071428571428571428571;
    /// goldens/generate.py: first row of (I - A)^-1 b, SSRN eq 4, p.29.
    G1_B_TILDE_GOOD: f64 = 0.26893802364690478262;
    /// goldens/generate.py: b / (1 - a) = 4/7.
    G1_B_TILDE_MACHINE: f64 = 0.57142857142857142857;
    /// goldens/generate.py: z' lambda-tilde, SSRN p.29.
    G1_L_S: f64 = 0.17046683479342606591;
    /// goldens/generate.py: z' b-tilde, SSRN p.29.
    G1_B_S: f64 = 1.2689380236469047826;
    /// goldens/generate.py: n_D - n_S at x = 1e-12, the bracket's left end.
    G1_F_AT_LO: f64 = 9.6046166614411005933;
    /// goldens/generate.py: n_D(1).
    G1_N_D_AT_1: f64 = 0.31914893617021276596;
    /// goldens/generate.py: n_S(1).
    G1_N_S_AT_1: f64 = 1.4847041385181291062;
    /// goldens/generate.py: n_D(1) - n_S(1).
    G1_F_AT_1: f64 = -1.1655552023479163402;
    /// goldens/generate.py: P_s(1) = 89/65.
    G1_P_S_AT_1: f64 = 1.3692307692307692308;
    /// goldens/generate.py: N P_s(1).
    G1_SUPPORT_COST_AT_1: f64 = 5.4769230769230769231;
    /// goldens/generate.py: v(1) = 8/13.
    G1_V_AT_1: f64 = 0.61538461538461538462;
    /// goldens/generate.py: D(1) = 1 - a - lambda gamma(1) = 0.65.
    G1_D_AT_1: f64 = 0.65;

    // G2: SSRN Figure 3 (SSRN p.12). Captions are published to 2 decimals
    /// Published caption, SSRN p.12 (Figure 3).
    PUB_G2_BASE_REAL_WAGE: f64 = 0.40;
    /// Published caption, SSRN p.12 (Figure 3).
    PUB_G2_BASE_PARTICIPATION: f64 = 0.34;
    /// Published caption, SSRN p.12 (Figure 3).
    PUB_G2_LAM0_REAL_WAGE: f64 = 0.37;
    /// Published caption, SSRN p.12 (Figure 3).
    PUB_G2_LAM0_PARTICIPATION: f64 = 0.32;
    /// Published caption, SSRN p.12 (Figure 3).
    PUB_G2_HALF_REAL_WAGE: f64 = 0.24;
    /// Published caption, SSRN p.12 (Figure 3).
    PUB_G2_HALF_PARTICIPATION: f64 = 0.21;
    /// goldens/generate.py: x*, lambda = 0 (recursive automation).
    G2_LAM0_X_STAR: f64 = 0.84058612875612342345;
    /// goldens/generate.py: v, lambda = 0 (recursive automation).
    G2_LAM0_V: f64 = 0.49855365885994213644;
    /// goldens/generate.py: Y, lambda = 0 (recursive automation).
    G2_LAM0_Y: f64 = 7.9518301153058752463;
    /// goldens/generate.py: N_a, lambda = 0 (recursive automation).
    G2_LAM0_N_A: f64 = 1.2676320221545510279;
    /// goldens/generate.py: w / P_s, lambda = 0 (recursive automation).
    G2_LAM0_REAL_WAGE: f64 = 0.37287626910307467929;
    /// goldens/generate.py: N_a / N, lambda = 0 (recursive automation).
    G2_LAM0_PARTICIPATION: f64 = 0.31690800553863775697;
    /// goldens/generate.py: x*, eta = 0.5 (task automation halves gamma).
    G2_HALF_X_STAR: f64 = 0.92147755772173964136;
    /// goldens/generate.py: v, eta = 0.5 (task automation halves gamma).
    G2_HALF_V: f64 = 0.27703901279652668537;
    /// goldens/generate.py: Y, eta = 0.5 (task automation halves gamma).
    G2_HALF_Y: f64 = 8.6979344529225284357;
    /// goldens/generate.py: N_a, eta = 0.5 (task automation halves gamma).
    G2_HALF_N_A: f64 = 0.84574124940438527541;
    /// goldens/generate.py: w / P_s, eta = 0.5 (task automation halves gamma).
    G2_HALF_REAL_WAGE: f64 = 0.23545004468505030702;
    /// goldens/generate.py: N_a / N, eta = 0.5 (task automation halves gamma).
    G2_HALF_PARTICIPATION: f64 = 0.21143531235109631885;

    // G3: the automation path (SSRN p.30; check_macro.py:35-43 at 31b3482): lambda 0, gamma = eta (1 + x)
    /// goldens/generate.py: N_a / N at eta = 1.
    G3_ETA_1_PARTICIPATION: f64 = 0.43065595858144168109;
    /// goldens/generate.py: v at eta = 1.
    G3_ETA_1_V: f64 = 0.98840122029213958952;
    /// goldens/generate.py: N_a / N at eta = 0.3.
    G3_ETA_0_3_PARTICIPATION: f64 = 0.22927792970224215316;
    /// goldens/generate.py: v at eta = 0.3.
    G3_ETA_0_3_V: f64 = 0.32367862903339933844;
    /// goldens/generate.py: N_a / N at eta = 0.1.
    G3_ETA_0_1_PARTICIPATION: f64 = 0.098066745653698975909;
    /// goldens/generate.py: v at eta = 0.1.
    G3_ETA_0_1_V: f64 = 0.11186280479118877161;
    /// goldens/generate.py: N_a / N at eta = 0.03.
    G3_ETA_0_03_PARTICIPATION: f64 = 0.032663064734812997025;
    /// goldens/generate.py: v at eta = 0.03.
    G3_ETA_0_03_V: f64 = 0.034056082139179303478;
    /// goldens/generate.py: N_a / N at eta = 0.01.
    G3_ETA_0_01_PARTICIPATION: f64 = 0.011242317671650493691;
    /// goldens/generate.py: v at eta = 0.01.
    G3_ETA_0_01_V: f64 = 0.011402655775197234064;
    /// goldens/generate.py: n_S(0.25) at eta = 1e-6, where z = v/P_s is about 7e-7. A
    /// point evaluation with no root, where f64 ln(1 + z) is off by 1.2e-10 and ln_1p(z)
    /// is not.
    G3_ETA_1E_6_N_S_AT_QUARTER: f64 = 0.0000028571398469420684487;
    /// goldens/generate.py: 1 - x* at eta = 1e-6.
    G3_ETA_1E_6_ONE_MINUS_X_STAR: f64 = 0.00000045714249142895853789;
    /// goldens/generate.py: N_a = Y (1 - x*) at eta = 1e-6 (lambda = 0).
    G3_ETA_1E_6_N_A: f64 = 0.0000045714209959311200557;
    /// goldens/generate.py: N_a / N at eta = 1e-6.
    G3_ETA_1E_6_PARTICIPATION: f64 = 0.0000011428552489827800139;
    /// goldens/generate.py: v N_a / I at eta = 1e-6.
    G3_ETA_1E_6_LABOR_SHARE: f64 = 5.2244799440381028585e-13;
    /// goldens/generate.py: 1 - x* at eta = 1e-20.
    G3_ETA_1E_20_ONE_MINUS_X_STAR: f64 = 4.5714285714285714285e-21;
    /// goldens/generate.py: N_a = Y (1 - x*) at eta = 1e-20 (lambda = 0).
    G3_ETA_1E_20_N_A: f64 = 4.5714285714285714285e-20;
    /// goldens/generate.py: N_a / N at eta = 1e-20.
    G3_ETA_1E_20_PARTICIPATION: f64 = 1.1428571428571428571e-20;
    /// goldens/generate.py: v N_a / I at eta = 1e-20.
    G3_ETA_1E_20_LABOR_SHARE: f64 = 5.2244897959183673468e-41;

    // G4: durability and interest through u = (rho + delta)(1 + rho)^(J_b - 1) (check_dynamics.py:45 at 31b3482; SSRN A.4, pp.27-28)
    /// goldens/generate.py: u at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_U: f64 = 1.05;
    /// goldens/generate.py: x at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_X_STAR: f64 = 0.85606144502130589985;
    /// goldens/generate.py: v at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_V: f64 = 0.58200502938266013801;
    /// goldens/generate.py: Y at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_Y: f64 = 7.9030007613895119881;
    /// goldens/generate.py: n at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_N_A: f64 = 1.3996714144162366065;
    /// goldens/generate.py: pm at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_P_M: f64 = 0.6577449110110797916;
    /// goldens/generate.py: Vm at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_V_M: f64 = 0.62642372477245694438;
    /// goldens/generate.py: income at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_INCOME: f64 = 10.978817061910314775;
    /// goldens/generate.py: capital_share at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_CAPITAL_SHARE: f64 = 0.014956188659577909534;
    /// goldens/generate.py: labor_share at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_LABOR_SHARE: f64 = 0.074198868428148129303;
    /// goldens/generate.py: real_wage at (rho, delta, J_b) = (0.05, 1, 1).
    G4_A_REAL_WAGE: f64 = 0.41895098209636805043;
    /// goldens/generate.py: u at (0.05, 0.1, 1).
    G4_B_U: f64 = 0.15;
    /// goldens/generate.py: x at (0.05, 0.1, 1).
    G4_B_X_STAR: f64 = 0.97912908840923014792;
    /// goldens/generate.py: v at (0.05, 0.1, 1).
    G4_B_V: f64 = 0.062258997183627810226;
    /// goldens/generate.py: Y at (0.05, 0.1, 1).
    G4_B_Y: f64 = 9.7666856299694170224;
    /// goldens/generate.py: n at (0.05, 0.1, 1).
    G4_B_N_A: f64 = 0.23300392857175693251;
    /// goldens/generate.py: pm at (0.05, 0.1, 1).
    G4_B_P_M: f64 = 0.063316170134949956625;
    /// goldens/generate.py: Vm at (0.05, 0.1, 1).
    G4_B_V_M: f64 = 0.4221078008996663775;
    /// goldens/generate.py: K at (0.05, 0.1, 1).
    G4_B_K: f64 = 5.8328592507645744403;
    /// goldens/generate.py: income at (0.05, 0.1, 1).
    G4_B_INCOME: f64 = 10.13761136049759874;
    /// goldens/generate.py: capital_share at (0.05, 0.1, 1).
    G4_B_CAPITAL_SHARE: f64 = 0.012143370384523498204;
    /// goldens/generate.py: labor_share at (0.05, 0.1, 1).
    G4_B_LABOR_SHARE: f64 = 0.001430967356792732955;
    /// goldens/generate.py: real_wage at (0.05, 0.1, 1).
    G4_B_REAL_WAGE: f64 = 0.059980998630410869347;
    /// goldens/generate.py: u at (0, 0.1, 1).
    G4_C_U: f64 = 0.1;
    /// goldens/generate.py: x at (0, 0.1, 1).
    G4_C_X_STAR: f64 = 0.98694968535939866455;
    /// goldens/generate.py: v at (0, 0.1, 1).
    G4_C_V: f64 = 0.041015801922299965682;
    /// goldens/generate.py: Y at (0, 0.1, 1).
    G4_C_Y: f64 = 9.7636520559177819562;
    /// goldens/generate.py: n at (0, 0.1, 1).
    G4_C_N_A: f64 = 0.15696222438135841406;
    /// goldens/generate.py: u at (0.05, 0.1, 3).
    G4_D_U: f64 = 0.165375;
    /// goldens/generate.py: x at (0.05, 0.1, 3).
    G4_D_X_STAR: f64 = 0.97675691133921725842;
    /// goldens/generate.py: v at (0.05, 0.1, 3).
    G4_D_V: f64 = 0.06889724102020382201;
    /// goldens/generate.py: Y at (0.05, 0.1, 3).
    G4_D_Y: f64 = 9.7676023542515746937;
    /// goldens/generate.py: n at (0.05, 0.1, 3).
    G4_D_N_A: f64 = 0.25607895324169275037;
    /// goldens/generate.py: income at (0.05, 0.1, 3).
    G4_D_INCOME: f64 = 10.178880949831178601;
    /// goldens/generate.py: x* at (N, rho, delta, J_b) = (7.3, 0.05, 1, 1).
    G4_E_X_STAR: f64 = 0.74326159093471752974;
    /// goldens/generate.py: N P_s at (7.3, 0.05, 1, 1), above T = 10.
    G4_E_SUPPORT_COST: f64 = 10.034041293511349215;
    /// goldens/generate.py: interest at (7.3, 0.05, 1, 1).
    G4_E_INTEREST: f64 = 0.13554159365380831009;
    /// goldens/generate.py: (T + interest)/P_s - N at (7.3, 0.05, 1, 1).
    G4_E_PROVIDER_BASKETS: f64 = 0.073843845103477730290;
    /// goldens/generate.py: funded at (7.3, 0.05, 1, 1): only through interest.
    G4_E_FUNDED: bool = true;
    /// goldens/generate.py: v N_a / I at (N, rho, delta, J_b) = (7.3, 0.05, 1, 1).
    G4_E_LABOR_SHARE: f64 = 0.10686847331204750894;
    /// goldens/generate.py: w / P_s at (N, rho, delta, J_b) = (7.3, 0.05, 1, 1).
    G4_E_REAL_WAGE: f64 = 0.37743990302044527907;
    /// goldens/generate.py: x* at (rho, delta, J_b) = (0.5, 0.5, 1), where u = 1.
    G4_F_X_STAR: f64 = 0.86447496527289638279;
    /// goldens/generate.py: Y at (rho, delta, J_b) = (0.5, 0.5, 1), where u = 1.
    G4_F_Y: f64 = 9.0007632618460365696;
    /// goldens/generate.py: phi_w = lambda gamma(x*) / (1 - a) at (rho, delta, J_b) = (0.5, 0.5, 1), where u = 1.
    G4_F_PHI_W: f64 = 0.063684283729879793302;
    /// goldens/generate.py: phi_r = 1 - phi_w at (rho, delta, J_b) = (0.5, 0.5, 1), where u = 1.
    G4_F_PHI_R: f64 = 0.9363157162701202067;
    /// goldens/generate.py: D(x*) near the viability edge, a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5.
    G4_G_D_STAR: f64 = 0.00000083644619286869630732;
    /// goldens/generate.py: x* at a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5.
    G4_G_X_STAR: f64 = 0.62292259686651390085;
    /// goldens/generate.py: p_m = u b / D at a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5.
    G4_G_P_M: f64 = 448325.31153485298836;
    /// goldens/generate.py: v at a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5.
    G4_G_V: f64 = 503434.62306970597671;
    /// goldens/generate.py: P_s at a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5.
    G4_G_P_S: f64 = 416453.21351772792983;
    /// goldens/generate.py: I at a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5.
    G4_G_INCOME: f64 = 3500919.7380127001637;

    // G5: two general instances, every parameter off the paper's values (constructed on 2026-09-25). Flow: N 5.2, T 12.5, h 0.7, a 0.22, lambda 0.08, b 0.55, gamma = 2.3 (0.15 + 0.9 x^4.5), chi_max 1.6, rho 0, delta 1, J_b 1. Durable: the same with k 2.5, rho 0.04, delta 0.35, J_b 2, so u = 0.39 * 1.04 = 0.4056
    /// goldens/generate.py: x*, flow instance.
    G5_FLOW_X_STAR: f64 = 0.86829772362082105489;
    /// goldens/generate.py: 1 - x*, flow instance.
    G5_FLOW_ONE_MINUS_X_STAR: f64 = 0.13170227637917894511;
    /// goldens/generate.py: gamma(x*), flow instance.
    G5_FLOW_GAMMA_STAR: f64 = 1.4414260022762480672;
    /// goldens/generate.py: J(x*), flow instance.
    G5_FLOW_J_STAR: f64 = 0.47265802408466387503;
    /// goldens/generate.py: v, flow instance.
    G5_FLOW_V: f64 = 1.1927201669461128271;
    /// goldens/generate.py: p_m, flow instance.
    G5_FLOW_P_M: f64 = 0.82745847866113977714;
    /// goldens/generate.py: V_m, flow instance.
    G5_FLOW_V_M: f64 = 0.82745847866113977714;
    /// goldens/generate.py: p, flow instance.
    G5_FLOW_P: f64 = 0.54818885060623373729;
    /// goldens/generate.py: P_s, flow instance.
    G5_FLOW_P_S: f64 = 1.2481888506062337373;
    /// goldens/generate.py: Y, flow instance.
    G5_FLOW_Y: f64 = 12.097345841970594874;
    /// goldens/generate.py: K, flow instance.
    G5_FLOW_K: f64 = 7.3306507465828792512;
    /// goldens/generate.py: Y (1 - x*), flow instance.
    G5_FLOW_FINAL_HOURS: f64 = 1.593247985533722505;
    /// goldens/generate.py: lambda delta K, flow instance.
    G5_FLOW_MACHINE_HOURS: f64 = 0.58645205972663034009;
    /// goldens/generate.py: N_a, flow instance.
    G5_FLOW_N_A: f64 = 2.1797000452603528451;
    /// goldens/generate.py: N_a / N, flow instance.
    G5_FLOW_PARTICIPATION: f64 = 0.41917308562699093175;
    /// goldens/generate.py: I, flow instance.
    G5_FLOW_INCOME: f64 = 15.099772201875377731;
    /// goldens/generate.py: interest, flow instance.
    G5_FLOW_INTEREST: f64 = 0.0;
    /// goldens/generate.py: v N_a / I, flow instance.
    G5_FLOW_LABOR_SHARE: f64 = 0.17217294189063914684;
    /// goldens/generate.py: interest / I, flow instance.
    G5_FLOW_CAPITAL_SHARE: f64 = 0.0;
    /// goldens/generate.py: w / P_s, flow instance.
    G5_FLOW_REAL_WAGE: f64 = 0.95556066405081227456;
    /// goldens/generate.py: N P_s, flow instance.
    G5_FLOW_SUPPORT_COST: f64 = 6.4905820231524154339;
    /// goldens/generate.py: N + v N_a / P_s, flow instance.
    G5_FLOW_WORKER_BASKETS: f64 = 7.2828356226805683347;
    /// goldens/generate.py: (T + interest) / P_s - N, flow instance.
    G5_FLOW_PROVIDER_BASKETS: f64 = 4.8145102192900265394;
    /// goldens/generate.py: funded, flow instance.
    G5_FLOW_FUNDED: bool = true;
    /// goldens/generate.py: lemma_b1, flow instance.
    G5_FLOW_LEMMA_B1: bool = true;
    /// goldens/generate.py: phi_w = lambda gamma(x*) / (1 - a), flow instance.
    G5_FLOW_PHI_W: f64 = 0.14783856433602544279;
    /// goldens/generate.py: phi_r = 1 - phi_w, flow instance.
    G5_FLOW_PHI_R: f64 = 0.85216143566397455721;
    /// goldens/generate.py: lambda-tilde good, flow instance.
    G5_FLOW_LAMBDA_TILDE_GOOD: f64 = 0.18018002243914447076;
    /// goldens/generate.py: lambda-tilde machine = lambda / (1 - a), flow instance.
    G5_FLOW_LAMBDA_TILDE_MACHINE: f64 = 0.1025641025641025641;
    /// goldens/generate.py: b-tilde good, flow instance.
    G5_FLOW_B_TILDE_GOOD: f64 = 0.33328450416226298881;
    /// goldens/generate.py: b-tilde machine = b / (1 - a), flow instance.
    G5_FLOW_B_TILDE_MACHINE: f64 = 0.70512820512820512821;
    /// goldens/generate.py: L_s, flow instance.
    G5_FLOW_L_S: f64 = 0.18018002243914447076;
    /// goldens/generate.py: B_s = h + b J / (1 - a), flow instance.
    G5_FLOW_B_S: f64 = 1.0332845041622629888;
    /// goldens/generate.py: u = (rho + delta)(1 + rho), durable instance.
    G5_DURABLE_U: f64 = 0.4056;
    /// goldens/generate.py: x*, durable instance.
    G5_DURABLE_X_STAR: f64 = 0.92239054108259182071;
    /// goldens/generate.py: 1 - x*, durable instance.
    G5_DURABLE_ONE_MINUS_X_STAR: f64 = 0.077609458917408179293;
    /// goldens/generate.py: gamma(x*), durable instance.
    G5_DURABLE_GAMMA_STAR: f64 = 2.0364433122513665055;
    /// goldens/generate.py: J(x*), durable instance.
    G5_DURABLE_J_STAR: f64 = 0.76398796867294253865;
    /// goldens/generate.py: v, durable instance.
    G5_DURABLE_V: f64 = 0.53781866694372592563;
    /// goldens/generate.py: p_m, durable instance.
    G5_DURABLE_P_M: f64 = 0.26409704788155712414;
    /// goldens/generate.py: V_m, durable instance.
    G5_DURABLE_V_M: f64 = 0.65112684388944064136;
    /// goldens/generate.py: p, durable instance.
    G5_DURABLE_P: f64 = 0.24350678288073599942;
    /// goldens/generate.py: P_s, durable instance.
    G5_DURABLE_P_S: f64 = 0.94350678288073599942;
    /// goldens/generate.py: Y, durable instance.
    G5_DURABLE_Y: f64 = 14.546104478511574628;
    /// goldens/generate.py: K, durable instance.
    G5_DURABLE_K: f64 = 12.040139558659209144;
    /// goldens/generate.py: Y (1 - x*), durable instance.
    G5_DURABLE_FINAL_HOURS: f64 = 1.1289152979333711791;
    /// goldens/generate.py: lambda delta K, durable instance.
    G5_DURABLE_MACHINE_HOURS: f64 = 0.33712390764245785603;
    /// goldens/generate.py: N_a, durable instance.
    G5_DURABLE_N_A: f64 = 1.4660392055758290351;
    /// goldens/generate.py: N_a / N, durable instance.
    G5_DURABLE_PARTICIPATION: f64 = 0.28193061645689019906;
    /// goldens/generate.py: I, durable instance.
    G5_DURABLE_INCOME: f64 = 13.724348239967521793;
    /// goldens/generate.py: interest, durable instance.
    G5_DURABLE_INTEREST: f64 = 0.43588498873749045294;
    /// goldens/generate.py: v N_a / I, durable instance.
    G5_DURABLE_LABOR_SHARE: f64 = 0.057449959549547047305;
    /// goldens/generate.py: interest / I, durable instance.
    G5_DURABLE_CAPITAL_SHARE: f64 = 0.031759977313029908957;
    /// goldens/generate.py: w / P_s, durable instance.
    G5_DURABLE_REAL_WAGE: f64 = 0.57002098628442919087;
    /// goldens/generate.py: N P_s, durable instance.
    G5_DURABLE_SUPPORT_COST: f64 = 4.906235270979827197;
    /// goldens/generate.py: N + v N_a / P_s, durable instance.
    G5_DURABLE_WORKER_BASKETS: f64 = 6.0356731138939751094;
    /// goldens/generate.py: (T + interest) / P_s - N, durable instance.
    G5_DURABLE_PROVIDER_BASKETS: f64 = 8.5104313646175995189;
    /// goldens/generate.py: funded, durable instance.
    G5_DURABLE_FUNDED: bool = true;
    /// goldens/generate.py: lemma_b1, durable instance.
    G5_DURABLE_LEMMA_B1: bool = true;

    // G6: the replacement closure, price block only (main.tex:325-336, check_pinning.py:59-73 at 31b3482); exact rationals
    /// goldens/generate.py: p_m (main.tex c) at (a, lambda, gamma*, b, r, u) = (0.5, 0.1, 3, 0.2, 1, 1).
    G6_P_M: f64 = 1.0;
    /// goldens/generate.py: w = gamma* p_m.
    G6_W: f64 = 3.0;
    /// goldens/generate.py: D = 1 - u (a + lambda gamma*).
    G6_D: f64 = 0.2;
    /// goldens/generate.py: p_m at lambda = 0.
    G6_LAM0_P_M: f64 = 0.4;
    /// goldens/generate.py: w at lambda = 0.
    G6_LAM0_W: f64 = 1.2;
    /// goldens/generate.py: 1 - w(lambda = 0) / w, the 60% cut.
    G6_LAM0_WAGE_CUT: f64 = 0.6;
    /// goldens/generate.py: D at (a, lambda, gamma*) = (0.5, 0.2, 3): not viable.
    G6_NOT_VIABLE_D: f64 = -0.1;

    // G7: three-taxes resolution shares on the G6 instance (check_three_taxes.py:58-61 at 31b3482; SSRN p.9)
    /// goldens/generate.py: phi_w = lambda gamma* / (1 - a).
    G7_PHI_W: f64 = 0.6;
    /// goldens/generate.py: phi_r = 1 - phi_w.
    G7_PHI_R: f64 = 0.4;
    /// goldens/generate.py: phi_w at lambda = 0.
    G7_LAM0_PHI_W: f64 = 0.0;
    /// goldens/generate.py: phi_r at lambda = 0.
    G7_LAM0_PHI_R: f64 = 1.0;

    // G8: regime recognition (constructed on 2026-09-25; laborformal has no instance)
    /// goldens/generate.py: f(1) at N = 0.25: BoundaryNoMargin.
    G8_N025_F_AT_1: f64 = 0.22635492751282969682;
    /// goldens/generate.py: f(1) at lambda = 0.6: BoundaryNoMargin.
    G8_LAM06_F_AT_1: f64 = 0.71896895969051973502;
    /// goldens/generate.py: D(1) at lambda = 0.8: NotViable.
    G8_LAM08_D_AT_1: f64 = -0.1;
    /// goldens/generate.py: D(1) at rho = 0.1, a = 0.6, lambda = 0.35: NotViable only through u = 1.1.
    G8_RHO01_D_AT_1: f64 = -0.045;
    /// goldens/generate.py: f(1e-12) at N = 20, chi_max = 0.05: NoInteriorAtZero.
    G8_N20_F_AT_LO: f64 = -10.000000000011;
    /// goldens/generate.py: x* at N = 8: Interior although Lemma B.1 fails.
    G8_N8_X_STAR: f64 = 0.73337119933110234855;
    /// goldens/generate.py: N P_s(1) at N = 8, above T = 10.
    G8_N8_SUPPORT_COST_AT_1: f64 = 10.953846153846153846;
    /// goldens/generate.py: lemma_b1 at N = 8.
    G8_N8_LEMMA_B1: bool = false;
    /// goldens/generate.py: funded at x* when N = 8.
    G8_N8_FUNDED: bool = false;
    /// goldens/generate.py: x* at N = 7.35: funded although Lemma B.1 fails.
    G8_N7_35_X_STAR: f64 = 0.75208427516884260890;
    /// goldens/generate.py: N P_s(x*) at N = 7.35, below T = 10.
    G8_N7_35_SUPPORT_COST: f64 = 9.9135366206749867864;
    /// goldens/generate.py: N P_s(1) at N = 7.35, above T = 10.
    G8_N7_35_SUPPORT_COST_AT_1: f64 = 10.063846153846153846;
    /// goldens/generate.py: lemma_b1 at N = 7.35.
    G8_N7_35_LEMMA_B1: bool = false;
    /// goldens/generate.py: funded at x* when N = 7.35.
    G8_N7_35_FUNDED: bool = true;
    /// goldens/generate.py: D(1) at lambda = 0.7: exactly 0, NotViable since D(1) <= 0.
    G8_LAM07_D_AT_1: f64 = 0.0;
    /// goldens/generate.py: f(1e-12) at a = 1 - 2^-53, lambda = 0: NoInteriorAtZero although T/h > N.
    G8_A_NEAR_1_F_AT_LO: f64 = -2.7587301670408429713;
    /// goldens/generate.py: x* at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_X_STAR: f64 = 0.99798105131089916681;
    /// goldens/generate.py: 1 - x* at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_ONE_MINUS_X_STAR: f64 = 0.0020189486891008331935;
    /// goldens/generate.py: gamma(x*) at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_GAMMA_STAR: f64 = 0.30099979286523141438;
    /// goldens/generate.py: J(x*) at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_J_STAR: f64 = 0.19969454770556112770;
    /// goldens/generate.py: v at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_V: f64 = 0.17577913033884822469;
    /// goldens/generate.py: p_m at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_P_M: f64 = 0.58398422359563201605;
    /// goldens/generate.py: P_s at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_P_S: f64 = 1.1169733544428819161;
    /// goldens/generate.py: Y at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_Y: f64 = 8.9757649579491023495;
    /// goldens/generate.py: K at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_K: f64 = 2.5605876051272441264;
    /// goldens/generate.py: Y (1 - x*) at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_FINAL_HOURS: f64 = 0.018121608895528535362;
    /// goldens/generate.py: N_a at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_N_A: f64 = 0.14615098915189074168;
    /// goldens/generate.py: I at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_INCOME: f64 = 10.025690293771281796;
    /// goldens/generate.py: w / P_s at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_REAL_WAGE: f64 = 0.15737092531318476533;
    /// goldens/generate.py: T / P_s - N at N = 1, k = CURVATURE_CEIL = 1024.
    G8_K_CEIL_PROVIDER_BASKETS: f64 = 7.9527650415508320747;
}
