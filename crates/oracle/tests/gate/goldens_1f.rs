//! The golden numbers of unit 1f, one constant each.
//!
//! Every constant equals its line in `goldens/goldens_1f.txt` rounded to 20 significant
//! digits (half up), and `f11_goldens_file::constants_match_goldens_1f_txt` enforces that.
//! `goldens_1f.txt` is written by `goldens/generate_1f.py`, which solves each instance with
//! mpmath at 70 digits from the equations of docs/unit-1f.md §4. Each constant's doc comment
//! is the generator's note; laborformal references are at 31b3482.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE_1F` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE_1F: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // H0: unit 1e's instances in household form equal generate_1e.py's solve (docs/unit-1f.md section 9)

    // TX: three-taxes' worked instance inside the closure (check_three_taxes.py T1 :27-61, T6 :63-85, T5 :105-134)
    /// goldens/generate_1f.py: x* = 1/2 exactly: labour demand T L_s/B_s = N at x = 1/2.
    TX_X_STAR: f64 = 0.5;
    /// goldens/generate_1f.py: gamma(x*) = 3.
    TX_GAMMA_STAR: f64 = 3.0;
    /// goldens/generate_1f.py: p_m = 1.
    TX_P_M: f64 = 1.0;
    /// goldens/generate_1f.py: v = gamma* p_m = 3.
    TX_V: f64 = 3.0;
    /// goldens/generate_1f.py: the good's price 11/4.
    TX_P_GOOD: f64 = 2.75;
    /// goldens/generate_1f.py: P = 15/4.
    TX_P: f64 = 3.75;
    /// goldens/generate_1f.py: L_s = 3/4.
    TX_L_S: f64 = 0.75;
    /// goldens/generate_1f.py: B_s = 3/2.
    TX_B_S: f64 = 1.5;
    /// goldens/generate_1f.py: Y = 16/3.
    TX_Y: f64 = 5.3333333333333333333;
    /// goldens/generate_1f.py: N_a = N = 4, supply saturated.
    TX_N_A: f64 = 4.0;
    /// goldens/generate_1f.py: the wage bill 12.
    TX_W: f64 = 12.0;
    /// goldens/generate_1f.py: rent 8.
    TX_R: f64 = 8.0;
    /// goldens/generate_1f.py: I = 20.
    TX_I: f64 = 20.0;
    /// goldens/generate_1f.py: lambda-tilde_m = lambda/(1 - a) = 0.2.
    TX_LAMBDA_M: f64 = 0.2;
    /// goldens/generate_1f.py: b-tilde_m = b/(1 - a) = 0.4.
    TX_B_M: f64 = 0.4;
    /// goldens/generate_1f.py: phi_w = lambda gamma*/(1 - a) = 0.6, the machine service and the basket.
    TX_PHI_W: f64 = 0.6;
    /// goldens/generate_1f.py: phi_r = 0.4.
    TX_PHI_R: f64 = 0.4;
    /// goldens/generate_1f.py: kappa = 8/15.
    TX_KAPPA: f64 = 0.53333333333333333333;
    /// goldens/generate_1f.py: the net real wage v/P = 0.8.
    TX_OMEGA_NET: f64 = 0.8;
    /// goldens/generate_1f.py: the consumption tax's revenue t_c Y P = 5.
    TX1_G_C: f64 = 5.0;
    /// goldens/generate_1f.py: d = G_c/N = 5/4, returned uniformly.
    TX1_D: f64 = 1.25;
    /// goldens/generate_1f.py: per unit of machine service: t v lambda-tilde = 0.15.
    TX1_LEG_WAGE_MACHINE: f64 = 0.15;
    /// goldens/generate_1f.py: per unit of machine service: t r b-tilde = 0.10.
    TX1_LEG_RENT_MACHINE: f64 = 0.1;
    /// goldens/generate_1f.py: per composite: t v L_s = 0.5625.
    TX1_LEG_WAGE_BASKET: f64 = 0.5625;
    /// goldens/generate_1f.py: per composite: t r B_s = 0.375.
    TX1_LEG_RENT_BASKET: f64 = 0.375;
    /// goldens/generate_1f.py: in aggregate, t_c W = 3.
    TX1_WAGE_LEG: f64 = 3.0;
    /// goldens/generate_1f.py: t_c R = 2.
    TX1_RENT_LEG: f64 = 2.0;
    /// goldens/generate_1f.py: d = tau_R R/N = 2: (1 - 1) 8 + 4 2 = 8 (Prop 6 (iii)).
    TX2_D: f64 = 2.0;
    /// goldens/generate_1f.py: phi^q_r = r B^q_s/P = 0.4.
    TX2_RENT_SHARE: f64 = 0.4;
    /// goldens/generate_1f.py: R_0 = phi^q_r (C - G_R) = 4.8.
    TX2_R0: f64 = 4.8;
    /// goldens/generate_1f.py: 1/(1 - phi^q_r tau_R) = 5/3.
    TX2_MULTIPLIER: f64 = 1.6666666666666666667;
    /// goldens/generate_1f.py: x* = 1/2 at T 12.
    TX3_X_STAR: f64 = 0.5;
    /// goldens/generate_1f.py: p_m = b/(1 - a) = 0.4.
    TX3_P_M: f64 = 0.4;
    /// goldens/generate_1f.py: v = 3 p_m = 1.2.
    TX3_V: f64 = 1.2;
    /// goldens/generate_1f.py: P = 2.1.
    TX3_P: f64 = 2.1;
    /// goldens/generate_1f.py: Y = 8.
    TX3_Y: f64 = 8.0;
    /// goldens/generate_1f.py: N_a = 4.
    TX3_N_A: f64 = 4.0;
    /// goldens/generate_1f.py: a 30% consumption tax and a 30% rent tax each take 0.12 per unit.
    TX3_TAX_PER_UNIT: f64 = 0.12;
    /// goldens/generate_1f.py: tau_R = N P/R = 15/8, outside the rent: kappa = 8/15 < 1.
    TXD_RENT_TAX: f64 = 1.875;

    // G: Appendix B's economy with a government (docs/unit-1f.md section 3.3)
    /// goldens/generate_1f.py: GA, a replacing transfer of one composite: tau_R = N P/R = 1/kappa (eq 16).
    GA_RENT_TAX: f64 = 0.54463033271192370986;
    /// goldens/generate_1f.py: G1's coverage kappa = T/(N P).
    G1_KAPPA: f64 = 1.8361077963113360457;
    /// goldens/generate_1f.py: x*.
    GB_X_STAR: f64 = 0.93746083398747312221;
    /// goldens/generate_1f.py: v, the pool's wage.
    GB_V: f64 = 0.58235487285097562042;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GB_P: f64 = 1.3668561558660557364;
    /// goldens/generate_1f.py: N_a, hours worked.
    GB_N_A: f64 = 0.77247596469536914465;
    /// goldens/generate_1f.py: x*.
    GC_X_STAR: f64 = 0.93360426237043449653;
    /// goldens/generate_1f.py: v, the pool's wage.
    GC_V: f64 = 0.58032633255676976674;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GC_P: f64 = 1.3666475439978276467;
    /// goldens/generate_1f.py: N_a, hours worked.
    GC_N_A: f64 = 0.80124305493560295052;
    /// goldens/generate_1f.py: x*.
    GP_X_STAR: f64 = 0.87627170039731020321;
    /// goldens/generate_1f.py: v, the pool's wage.
    GP_V: f64 = 0.55028234488275430929;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GP_P: f64 = 1.3627008536384357176;
    /// goldens/generate_1f.py: N_a, hours worked.
    GP_N_A: f64 = 1.2400309957754075556;
    /// goldens/generate_1f.py: x*.
    GT_X_STAR: f64 = 0.89015871146432740031;
    /// goldens/generate_1f.py: v, the pool's wage.
    GT_V: f64 = 0.55754023103552006644;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GT_P: f64 = 1.3638017912497742457;
    /// goldens/generate_1f.py: N_a, hours worked.
    GT_N_A: f64 = 1.1318355224715726678;
    /// goldens/generate_1f.py: x*.
    GW_X_STAR: f64 = 0.8037264408356073074;
    /// goldens/generate_1f.py: v, the pool's wage.
    GW_V: f64 = 0.51256666115635029646;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GW_P: f64 = 1.355454792774879952;
    /// goldens/generate_1f.py: N_a, hours worked.
    GW_N_A: f64 = 1.8250157761403085431;
    /// goldens/generate_1f.py: x*.
    GE_X_STAR: f64 = 0.94717837462152922992;
    /// goldens/generate_1f.py: v, the pool's wage.
    GE_V: f64 = 0.58747051381971391848;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GE_P: f64 = 1.3673498760635844705;
    /// goldens/generate_1f.py: N_a, hours worked.
    GE_N_A: f64 = 0.70040916318589496291;
    /// goldens/generate_1f.py: x*.
    GU_X_STAR: f64 = 0.88543913357893768838;
    /// goldens/generate_1f.py: v, the pool's wage.
    GU_V: f64 = 0.55507221628355679682;
    /// goldens/generate_1f.py: P, the composite's producer price.
    GU_P: f64 = 1.3634379975171398545;
    /// goldens/generate_1f.py: N_a, hours worked.
    GU_N_A: f64 = 1.1684693399719529172;
    /// goldens/generate_1f.py: the levy's rate, (N d)/R.
    GB_RENT_TAX: f64 = 0.54674246234642229456;
    /// goldens/generate_1f.py: kappa at GB.
    GB_KAPPA: f64 = 1.82901469863591559;
    /// goldens/generate_1f.py: d = tau_R R/N = 5/4.
    GC_D: f64 = 1.25;
    /// goldens/generate_1f.py: the net wage in producer composites, kappa_w v/P, as at tau_w = t/(1 + t).
    GT_OMEGA_NET: f64 = 0.32705059319483415896;

    // INC: payroll incidence at G1, workers bear eps_D/(eps_D + eps_S) (SSRN D.2 p.32; main.tex:829-833)
    /// goldens/generate_1f.py: -d ln omega_net/d tau_w at tau_w = 0 = eps_D/(eps_D + eps_S).
    INC_SHARE: f64 = 0.88585841800425697123;
    /// goldens/generate_1f.py: eps_D = -d ln n_D/d ln omega along the line at x*.
    INC_EPS_D: f64 = 6.5922191918464243292;
    /// goldens/generate_1f.py: eps_S = d ln S/d ln omega_net at x*.
    INC_EPS_S: f64 = 0.84939795358634117975;

    // W: the wall, where labour demand does not move with the wage (docs/unit-1f.md section 4.10 (e))
    /// goldens/generate_1f.py: the wall's gross wage at tau_w 0.
    W1_V_0: f64 = 12.347776347728814794;
    /// goldens/generate_1f.py: the wall's gross wage at tau_w 0.05.
    W1_V_005: f64 = 17.304632676489533436;
    /// goldens/generate_1f.py: the wall's gross wage at tau_w 0.1.
    W1_V_01: f64 = 28.910302576745416974;
    /// goldens/generate_1f.py: the net real wage, the same at every tau_w on the wall.
    W1_OMEGA_NET: f64 = 1.6050368175205035052;
    /// goldens/generate_1f.py: (1 - 0.2) 35/18 = 14/9 on idle land.
    WI_OMEGA_NET: f64 = 1.5555555555555555556;
    /// goldens/generate_1f.py: the market's land in use.
    WI_T_M: f64 = 9.7997051141928276615;
    /// goldens/generate_1f.py: N_a.
    WI_N_A: f64 = 3.7530785543717212321;
    /// goldens/generate_1f.py: Y.
    WI_Y: f64 = 7.2976527446116801735;
    /// goldens/generate_1f.py: SurplusLabour: f_0 = n_D(0) - S(0) = 10 - 20.
    SL_F_START: f64 = -10.0;

    // AP: SSRN's automation path with a payroll tax, the corollary and eq 17 (SSRN p.16, p.31)
    /// goldens/generate_1f.py: x* at eta 2^-0.
    AP0_X_STAR: f64 = 0.83299773278259040079;
    /// goldens/generate_1f.py: v/r at eta 2^-0.
    AP0_V: f64 = 1.0474272758757659433;
    /// goldens/generate_1f.py: the labour share W/I at eta 2^-0.
    AP0_LABOR_SHARE: f64 = 0.094595044181221527777;
    /// goldens/generate_1f.py: kappa = r T/(N P) at eta 2^-0.
    AP0_KAPPA: f64 = 1.3519547213846445738;
    /// goldens/generate_1f.py: T/(N B_s) at eta 2^-0.
    AP0_T_NB: f64 = 1.4932044635894895278;
    /// goldens/generate_1f.py: payroll revenue per person over P, G_w/(N P) at eta 2^-0.
    AP0_GW_NP: f64 = 0.070624871102422476998;
    /// goldens/generate_1f.py: the corollary's bound tau_w (v/r)/B_s at eta 2^-0.
    AP0_BOUND: f64 = 0.31280461672461467008;
    /// goldens/generate_1f.py: C/R, the consumption base over the rent base at eta 2^-0.
    AP0_C_R: f64 = 1.1044781603781669819;
    /// goldens/generate_1f.py: x* at eta 2^-4.
    AP4_X_STAR: f64 = 0.98606048927049178458;
    /// goldens/generate_1f.py: v/r at eta 2^-4.
    AP4_V: f64 = 0.070930731759660420878;
    /// goldens/generate_1f.py: the labour share W/I at eta 2^-4.
    AP4_LABOR_SHARE: f64 = 0.00093846788733786065451;
    /// goldens/generate_1f.py: kappa = r T/(N P) at eta 2^-4.
    AP4_KAPPA: f64 = 2.3728891707796336079;
    /// goldens/generate_1f.py: T/(N B_s) at eta 2^-4.
    AP4_T_NB: f64 = 2.3751181428853650595;
    /// goldens/generate_1f.py: payroll revenue per person over P, G_w/(N P) at eta 2^-4.
    AP4_GW_NP: f64 = 0.0011144860528657258004;
    /// goldens/generate_1f.py: the corollary's bound tau_w (v/r)/B_s at eta 2^-4.
    AP4_BOUND: f64 = 0.033693773578100928186;
    /// goldens/generate_1f.py: C/R, the consumption base over the rent base at eta 2^-4.
    AP4_C_R: f64 = 1.0009393494366191166;
    /// goldens/generate_1f.py: x* at eta 2^-8.
    AP8_X_STAR: f64 = 0.99910853518017626618;
    /// goldens/generate_1f.py: v/r at eta 2^-8.
    AP8_V: f64 = 0.0044622958374557505941;
    /// goldens/generate_1f.py: the labour share W/I at eta 2^-8.
    AP8_LABOR_SHARE: f64 = 3.9647050760798191961e-6;
    /// goldens/generate_1f.py: kappa = r T/(N P) at eta 2^-8.
    AP8_KAPPA: f64 = 2.491657399310504859;
    /// goldens/generate_1f.py: T/(N B_s) at eta 2^-8.
    AP8_T_NB: f64 = 2.4916672780364099919;
    /// goldens/generate_1f.py: payroll revenue per person over P, G_w/(N P) at eta 2^-8.
    AP8_GW_NP: f64 = 4.9393629525664704434e-6;
    /// goldens/generate_1f.py: the corollary's bound tau_w (v/r)/B_s at eta 2^-8.
    AP8_BOUND: f64 = 0.0022237113046213145368;
    /// goldens/generate_1f.py: C/R, the consumption base over the rent base at eta 2^-8.
    AP8_C_R: f64 = 1.0000039647207950285;
    /// goldens/generate_1f.py: x* at eta 2^-12.
    AP12_X_STAR: f64 = 0.99994420187746205746;
    /// goldens/generate_1f.py: v/r at eta 2^-12.
    AP12_V: f64 = 0.00027901007280656557721;
    /// goldens/generate_1f.py: the labour share W/I at eta 2^-12.
    AP12_LABOR_SHARE: f64 = 1.5564981051039897804e-8;
    /// goldens/generate_1f.py: kappa = r T/(N P) at eta 2^-12.
    AP12_KAPPA: f64 = 2.4994769509734139025;
    /// goldens/generate_1f.py: T/(N B_s) at eta 2^-12.
    AP12_T_NB: f64 = 2.4994769898777258874;
    /// goldens/generate_1f.py: payroll revenue per person over P, G_w/(N P) at eta 2^-12.
    AP12_GW_NP: f64 = 1.9452155992478522944e-8;
    /// goldens/generate_1f.py: the corollary's bound tau_w (v/r)/B_s at eta 2^-12.
    AP12_BOUND: f64 = 0.00013947585138482393442;
    /// goldens/generate_1f.py: C/R, the consumption base over the rent base at eta 2^-12.
    AP12_C_R: f64 = 1.0000000155649812933;
    /// goldens/generate_1f.py: x* at eta 2^-16.
    AP16_X_STAR: f64 = 0.99999651229807292214;
    /// goldens/generate_1f.py: v/r at eta 2^-16.
    AP16_V: f64 = 0.000017438585661081132482;
    /// goldens/generate_1f.py: the labour share W/I at eta 2^-16.
    AP16_LABOR_SHARE: f64 = 6.0819793355895615236e-11;
    /// goldens/generate_1f.py: kappa = r T/(N P) at eta 2^-16.
    AP16_KAPPA: f64 = 2.4999673030225087319;
    /// goldens/generate_1f.py: T/(N B_s) at eta 2^-16.
    AP16_T_NB: f64 = 2.4999673031745562267;
    /// goldens/generate_1f.py: payroll revenue per person over P, G_w/(N P) at eta 2^-16.
    AP16_GW_NP: f64 = 7.6023747387786077003e-11;
    /// goldens/generate_1f.py: the corollary's bound tau_w (v/r)/B_s at eta 2^-16.
    AP16_BOUND: f64 = 8.7191787932622969095e-6;
    /// goldens/generate_1f.py: C/R, the consumption base over the rent base at eta 2^-16.
    AP16_C_R: f64 = 1.0000000000608197934;
    /// goldens/generate_1f.py: x* at eta 2^-20.
    AP20_X_STAR: f64 = 0.9999997820173822609;
    /// goldens/generate_1f.py: v/r at eta 2^-20.
    AP20_V: f64 = 1.0899133856731863081e-6;
    /// goldens/generate_1f.py: the labour share W/I at eta 2^-20.
    AP20_LABOR_SHARE: f64 = 2.3758197871006702888e-13;
    /// goldens/generate_1f.py: kappa = r T/(N P) at eta 2^-20.
    AP20_KAPPA: f64 = 2.4999979564138496286;
    /// goldens/generate_1f.py: T/(N B_s) at eta 2^-20.
    AP20_T_NB: f64 = 2.4999979564144435831;
    /// goldens/generate_1f.py: payroll revenue per person over P, G_w/(N P) at eta 2^-20.
    AP20_GW_NP: f64 = 2.9697723062803370768e-13;
    /// goldens/generate_1f.py: the corollary's bound tau_w (v/r)/B_s at eta 2^-20.
    AP20_BOUND: f64 = 5.4495624737034261262e-7;
    /// goldens/generate_1f.py: C/R, the consumption base over the rent base at eta 2^-20.
    AP20_C_R: f64 = 1.000000000000237582;

    // K: coverage with parcels, RentRate d-hat 1 in Replace mode (SSRN eq 16; docs/unit-1f.md section 3.3)
    /// goldens/generate_1f.py: kappa = r T/(N P).
    KR4_KAPPA: f64 = 0.92864068860820635539;
    /// goldens/generate_1f.py: the levy's rate at d-hat 1: 1/kappa with T_m = T, N P/(r T_m) with rented plots.
    KR4_RENT_TAX: f64 = 1.0768427576641541589;
    /// goldens/generate_1f.py: kappa = r T/(N P).
    KR5_KAPPA: f64 = 1.0055247647582589377;
    /// goldens/generate_1f.py: the levy's rate at d-hat 1: 1/kappa with T_m = T, N P/(r T_m) with rented plots.
    KR5_RENT_TAX: f64 = 0.99450559056137502286;
    /// goldens/generate_1f.py: kappa = r T/(N P).
    KR1_KAPPA: f64 = 1.0333754799030553077;
    /// goldens/generate_1f.py: the levy's rate at d-hat 1: 1/kappa with T_m = T, N P/(r T_m) with rented plots.
    KR1_RENT_TAX: f64 = 1.47315087881406432;
    /// goldens/generate_1f.py: x*.
    KT_X_STAR: f64 = 0.91429046477069489349;
    /// goldens/generate_1f.py: v, the pool's wage.
    KT_V: f64 = 0.57018176639823643615;
    /// goldens/generate_1f.py: P, the composite's producer price.
    KT_P: f64 = 1.3654947150845808806;
    /// goldens/generate_1f.py: N_a, hours worked.
    KT_N_A: f64 = 0.91754722960930240794;
    /// goldens/generate_1f.py: enclosed land in rented plots.
    KT_T_P: f64 = 0.30824527703906975921;

    // E: a walled type under a government (docs/unit-1f.md section 4.5)
    /// goldens/generate_1f.py: the pool's wage, on the wall.
    ER_V: f64 = 2.0187867868541043205;
    /// goldens/generate_1f.py: P.
    ER_P: f64 = 2.9365136181386231297;
    /// goldens/generate_1f.py: N_a.
    ER_N_A: f64 = 2.78125;
    /// goldens/generate_1f.py: the trained's wage c_T P at its wall.
    ER_TRAINED_WAGE: f64 = 5.6700659367586602131;
    /// goldens/generate_1f.py: c_T = ((a_T + mu_e) zeta_T - (mu_w - mu_e)) (1 + t_c)/(1 - tau_w).
    ER_TRAINED_RATE: f64 = 1.9308835830813419669;

    // C: the CES household on G1's economy, alpha 0.3 (SSRN eq 26; docs/unit-1f.md section 3.3)
    /// goldens/generate_1f.py: x*.
    C1_X_STAR: f64 = 0.89176202197842091755;
    /// goldens/generate_1f.py: v, the pool's wage.
    C1_V: f64 = 0.55837897627480756256;
    /// goldens/generate_1f.py: P, the composite's producer price.
    C1_P: f64 = 0.80009907662364586351;
    /// goldens/generate_1f.py: N_a, hours worked.
    C1_N_A: f64 = 2.1175388302542343684;
    /// goldens/generate_1f.py: space's share = eq 26's alpha(q).
    C1_SPACE_SHARE: f64 = 0.52042802976305350621;
    /// goldens/generate_1f.py: q = r/p_g.
    C1_Q: f64 = 2.7478344256621054222;
    /// goldens/generate_1f.py: x*.
    C2_X_STAR: f64 = 0.90066033465721472141;
    /// goldens/generate_1f.py: v, the pool's wage.
    C2_V: f64 = 0.5630369706472617337;
    /// goldens/generate_1f.py: P, the composite's producer price.
    C2_P: f64 = 0.49345674059338766913;
    /// goldens/generate_1f.py: N_a, hours worked.
    C2_N_A: f64 = 3.0451027517319873678;
    /// goldens/generate_1f.py: space's share = eq 26's alpha(q).
    C2_SPACE_SHARE: f64 = 0.3;
    /// goldens/generate_1f.py: q = r/p_g.
    C2_Q: f64 = 2.7429354773817037774;
    /// goldens/generate_1f.py: x*.
    C3_X_STAR: f64 = 0.90960722009951245118;
    /// goldens/generate_1f.py: v, the pool's wage.
    C3_V: f64 = 0.56772550609720382536;
    /// goldens/generate_1f.py: P, the composite's producer price.
    C3_P: f64 = 0.23495322058734605667;
    /// goldens/generate_1f.py: N_a, hours worked.
    C3_N_A: f64 = 4.0;
    /// goldens/generate_1f.py: space's share = eq 26's alpha(q).
    C3_SPACE_SHARE: f64 = 0.062859066150003403983;
    /// goldens/generate_1f.py: q = r/p_g.
    C3_Q: f64 = 2.7383150461502721144;

    // CP: C1-C3's baskets on AP's path, space's share toward eq 26's limits (SSRN App. C p.31)
    /// goldens/generate_1f.py: space's share at sigma 0.5, eta 2^-0.
    CP05_0_SHARE: f64 = 0.41725290983466708842;
    /// goldens/generate_1f.py: q = r/p_g at sigma 0.5, eta 2^-0.
    CP05_0_Q: f64 = 1.1962316976021548035;
    /// goldens/generate_1f.py: space's share at sigma 0.5, eta 2^-8.
    CP05_8_SHARE: f64 = 0.91878969786344465734;
    /// goldens/generate_1f.py: q = r/p_g at sigma 0.5, eta 2^-8.
    CP05_8_Q: f64 = 298.66667471266946468;
    /// goldens/generate_1f.py: space's share at sigma 0.5, eta 2^-16.
    CP05_16_SHARE: f64 = 0.99450607818891702033;
    /// goldens/generate_1f.py: q = r/p_g at sigma 0.5, eta 2^-16.
    CP05_16_Q: f64 = 76458.666666666813051;
    /// goldens/generate_1f.py: space's share at sigma 1, eta 2^-0.
    CP1_0_SHARE: f64 = 0.3;
    /// goldens/generate_1f.py: q = r/p_g at sigma 1, eta 2^-0.
    CP1_0_Q: f64 = 1.1971441554974369223;
    /// goldens/generate_1f.py: space's share at sigma 1, eta 2^-8.
    CP1_8_SHARE: f64 = 0.3;
    /// goldens/generate_1f.py: q = r/p_g at sigma 1, eta 2^-8.
    CP1_8_Q: f64 = 298.66668366291228841;
    /// goldens/generate_1f.py: space's share at sigma 1, eta 2^-16.
    CP1_16_SHARE: f64 = 0.3;
    /// goldens/generate_1f.py: q = r/p_g at sigma 1, eta 2^-16.
    CP1_16_Q: f64 = 76458.66666666950928;
    /// goldens/generate_1f.py: space's share at sigma 2, eta 2^-0.
    CP2_0_SHARE: f64 = 0.1330155577229585115;
    /// goldens/generate_1f.py: q = r/p_g at sigma 2, eta 2^-0.
    CP2_0_Q: f64 = 1.1971685353520635771;
    /// goldens/generate_1f.py: space's share at sigma 2, eta 2^-8.
    CP2_8_SHARE: f64 = 0.00061459980206261191883;
    /// goldens/generate_1f.py: q = r/p_g at sigma 2, eta 2^-8.
    CP2_8_Q: f64 = 298.66684482128279355;
    /// goldens/generate_1f.py: space's share at sigma 2, eta 2^-16.
    CP2_16_SHARE: f64 = 2.4022525655170948765e-6;
    /// goldens/generate_1f.py: q = r/p_g at sigma 2, eta 2^-16.
    CP2_16_Q: f64 = 76458.66666736420493;

    // CW: W3 under CES, a free category at the wall's end: no idle stretch (docs/unit-1f.md section 2.12)
    /// goldens/generate_1f.py: v, on the wall with land scarce.
    CW05_V: f64 = 29.267492270362484658;
    /// goldens/generate_1f.py: P.
    CW05_P: f64 = 10.598475271797954213;
    /// goldens/generate_1f.py: N_a.
    CW05_N_A: f64 = 1.7664171568510081241;
    /// goldens/generate_1f.py: f_inf = -S_inf, the path's end.
    CW05_F_END: f64 = -2.1040692273899681792;
    /// goldens/generate_1f.py: v, on the wall with land scarce.
    CW2_V: f64 = 5.7935510403550846102;
    /// goldens/generate_1f.py: P.
    CW2_P: f64 = 1.4165173771906560161;
    /// goldens/generate_1f.py: N_a.
    CW2_N_A: f64 = 2.1697028434673954233;
    /// goldens/generate_1f.py: f_inf = -S_inf, the path's end.
    CW2_F_END: f64 = -4.0;

    // CI: W3 with space given one human-required hour, C1's basket: the idle stretch (docs/unit-1f.md section 2.12)
    /// goldens/generate_1f.py: the market's land in use.
    CI_T_M: f64 = 0.79084422487257830458;
    /// goldens/generate_1f.py: Y.
    CI_Y: f64 = 1.0065351353131344715;
    /// goldens/generate_1f.py: N_a.
    CI_N_A: f64 = 0.95773784128167540424;
    /// goldens/generate_1f.py: P in pool wages.
    CI_P: f64 = 0.95151953238445256894;
    /// goldens/generate_1f.py: T_idle.
    CI_IDLE: f64 = 9.2091557751274216954;
    /// goldens/generate_1f.py: space's share at the idle prices.
    CI_SPACE_SHARE: f64 = 0.47722557505166113457;

    // CA: C3's basket at the all-human corner, the good free as v -> 0 (docs/unit-1f.md section 5.3 step 2)
    /// goldens/generate_1f.py: v, at the all-human corner.
    CA_V: f64 = 0.63900965042269379903;
    /// goldens/generate_1f.py: P.
    CA_P: f64 = 0.39261840952342306052;
    /// goldens/generate_1f.py: N_a.
    CA_N_A: f64 = 40.0;
    /// goldens/generate_1f.py: space's share = eq 26's alpha(q).
    CA_SPACE_SHARE: f64 = 0.10504059707820474271;

    // CF: C1's basket on W3 with few workers, the wall far out (docs/unit-1f.md section 5.5)
    /// goldens/generate_1f.py: v, on the wall with land scarce, N 0.01.
    CF2_V: f64 = 4337700.4654140980326;
    /// goldens/generate_1f.py: P.
    CF2_P: f64 = 1128980.9807283347639;
    /// goldens/generate_1f.py: N_a.
    CF2_N_A: f64 = 0.005257854369391122216;
    /// goldens/generate_1f.py: Y.
    CF2_Y: f64 = 0.020210258396441351896;
    /// goldens/generate_1f.py: v, on the wall with land scarce, N 0.001.
    CF3_V: f64 = 433699594.49873698627;
    /// goldens/generate_1f.py: P.
    CF3_P: f64 = 112790744.58595681438;
    /// goldens/generate_1f.py: N_a.
    CF3_N_A: f64 = 0.00052599411517003631223;
    /// goldens/generate_1f.py: Y.
    CF3_Y: f64 = 0.0020226254848784003373;

    // CS: W1 under a steep CES (sigma 20) with eq 26's weights, space's weight 0.3^20 (docs/unit-1f.md section 5.5)
    /// goldens/generate_1f.py: x*.
    CS_X_STAR: f64 = 0.99349127735404220362;
    /// goldens/generate_1f.py: v, the pool's wage.
    CS_V: f64 = 3.8586215400145048667;
    /// goldens/generate_1f.py: P, the composite's producer price.
    CS_P: f64 = 0.0018238205950235484236;
    /// goldens/generate_1f.py: N_a, hours worked.
    CS_N_A: f64 = 4.0;
    /// goldens/generate_1f.py: Y.
    CS_Y: f64 = 13945.717155217023861;
    /// goldens/generate_1f.py: space's share.
    CS_SPACE_SHARE: f64 = 0.28964691334049453218;
    /// goldens/generate_1f.py: c_good, units of the good per composite.
    CS_GOOD_CONTENT: f64 = 0.00055669580489493360075;
    /// goldens/generate_1f.py: c_space, units of space per composite.
    CS_SPACE_CONTENT: f64 = 0.00052826400583539490349;

    // AJ: check_pinning's A-joint household, Ces { sigma: 1 } over (good, space) (check_pinning.py:225-267)
    /// goldens/generate_1f.py: AJ1 (gamma = 1 + 3.9x): x*.
    AJ1_X_STAR: f64 = 0.89631061966178256886;
    /// goldens/generate_1f.py: AJ1 (gamma = 1 + 3.9x): v.
    AJ1_V: f64 = 17.825984034366138326;
    /// goldens/generate_1f.py: AJ1 (gamma = 1 + 3.9x): P = p^0.7 r^0.3.
    AJ1_P: f64 = 5.5653499074688290542;
    /// goldens/generate_1f.py: AJ1 (gamma = 1 + 3.9x): w/p.
    AJ1_W_P: f64 = 1.5348440806825329101;
    /// goldens/generate_1f.py: AJ1 (gamma = 1 + 3.9x): r/p.
    AJ1_R_P: f64 = 0.08610150652685184878;
    /// goldens/generate_1f.py: AJ1 (gamma = 1 + 3.9x): goods, the good's final output.
    AJ1_GOODS: f64 = 1.6771014021657359785;
    /// goldens/generate_1f.py: AJW (lambda 0.08), the wall in closed form: goods 25/12 = N/lambda-tilde_g.
    AJW_GOODS: f64 = 2.0833333333333333333;
    /// goldens/generate_1f.py: AJW (lambda 0.08), the wall in closed form: r/p = 5/42.
    AJW_R_P: f64 = 0.11904761904761904762;
    /// goldens/generate_1f.py: AJW (lambda 0.08), the wall in closed form: p = 8.4.
    AJW_P_GOOD: f64 = 8.4;
    /// goldens/generate_1f.py: AJW (lambda 0.08), the wall in closed form: v = 15.
    AJW_V: f64 = 15.0;
    /// goldens/generate_1f.py: AJW (lambda 0.08), the wall in closed form: w/p = 25/14.
    AJW_W_P: f64 = 1.7857142857142857143;
    /// goldens/generate_1f.py: AJW (lambda 0.08), the wall in closed form: housing 7.5 = T - 1.2 goods.
    AJW_HOUSING: f64 = 7.5;

    // X: the wrong units of docs/unit-1f.md section 3.3
    /// goldens/generate_1f.py: x* with C1's composite frozen at one good and (3/7)^0.5 space.
    X1_X_STAR: f64 = 0.87389221689154025529;
    /// goldens/generate_1f.py: space's share with the composite frozen.
    X1_SPACE_SHARE: f64 = 0.64361147559656076219;
    /// goldens/generate_1f.py: x* with C3's composite frozen at one good and (3/7)^2 space.
    X3_X_STAR: f64 = 0.90236871855688530804;
    /// goldens/generate_1f.py: space's share with the composite frozen.
    X3_SPACE_SHARE: f64 = 0.33494635440890407453;
}
