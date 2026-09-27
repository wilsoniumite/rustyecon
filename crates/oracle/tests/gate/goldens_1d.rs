//! The golden numbers of unit 1d, one constant each.
//!
//! Every constant equals its line in `goldens/goldens_1d.txt` rounded to 20 significant
//! digits (half up), and `d9_goldens_file::constants_match_goldens_1d_txt` enforces that.
//! `goldens_1d.txt` is written by `goldens/generate_1d.py`, which solves each instance with
//! mpmath at 70 digits from the equations of docs/unit-1d.md §4. Each constant's doc comment
//! is the generator's note; laborformal references are at 31b3482.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE_1D` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE_1D: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // D0: unit 1c's boundary rows in one-type form, solved (docs/unit-1d.md section 3.3)
    /// goldens/generate_1d.py: M4 at eta 0.25 (1c's BoundaryNoMargin): the wall's wage.
    D0_M4_ETA025_V: f64 = 0.064963057288499518554;
    /// goldens/generate_1d.py: its f on the line at 1, 1c's f_at_1.
    D0_M4_ETA025_F_LINE_1: f64 = 0.39646115974723321568;
    /// goldens/generate_1d.py: M4 at N 200, chi_max 0.01 (1c's NoInteriorAtZero): the all-human wage.
    D0_M4_N200_V: f64 = 0.00066816690250631266227;
    /// goldens/generate_1d.py: its f on the line at 1e-12, 1c's f_at_0.
    D0_M4_N200_F_LINE_LO: f64 = -190.75346260388830846;
    /// goldens/generate_1d.py: M5b (H4): f at the end of the wall, labour-short: two sign changes, not 1c's three.
    D0_M5B_F_END: f64 = 6.2264150943396226415;

    // W: the corners in Appendix B's closure, 1a's G8 rows in one-type form (docs/unit-1d.md section 3.3)
    /// goldens/generate_1d.py: the wall's wage, zeta B_s/(1 - zeta L_s).
    W1_V: f64 = 12.347776347728814794;
    /// goldens/generate_1d.py: g = v/pi, above gamma(1) = 1.
    W1_G: f64 = 1.1069040032228707651;
    /// goldens/generate_1d.py: p_m = pi = (lambda v + b)/(1 - a) at the wall's own wage.
    W1_P_M: f64 = 11.155236869481841252;
    /// goldens/generate_1d.py: gamma(1) pi.
    W1_REPLACEMENT_TOP: f64 = 11.155236869481841252;
    /// goldens/generate_1d.py: P_s.
    W1_P_S: f64 = 7.6931421216891047511;
    /// goldens/generate_1d.py: Y = 350/47.
    W1_Y: f64 = 7.4468085106382978723;
    /// goldens/generate_1d.py: N_a = 180/47.
    W1_N_A: f64 = 3.8297872340425531915;
    /// goldens/generate_1d.py: I.
    W1_INCOME: f64 = 57.289356225344397083;
    /// goldens/generate_1d.py: v N_a/I.
    W1_LABOR_SHARE: f64 = 0.82544750615340180269;
    /// goldens/generate_1d.py: v/P_s = zeta = expm1(45/47).
    W1_REAL_WAGE: f64 = 1.6050368175205035052;
    /// goldens/generate_1d.py: provider baskets: not funded.
    W1_PROVIDER_BASKETS: f64 = -2.7001410032700134243;
    /// goldens/generate_1d.py: f on the line at 1, 1a's G8 golden.
    W1_F_LINE_1: f64 = 0.71896895969051973502;
    /// goldens/generate_1d.py: f at the end of the wall = -8/47.
    W1_F_END: f64 = -0.17021276595744680851;
    /// goldens/generate_1d.py: LaborShort: f_infinity = n_D(1) - N = 13/188.
    W2_EXCESS: f64 = 0.069148936170212765957;
    /// goldens/generate_1d.py: f on the line at 1, 1a's G8 golden.
    W2_F_LINE_1: f64 = 0.22635492751282969682;
    /// goldens/generate_1d.py: LaborShort at the real-wage ceiling: f_infinity.
    W3_EXCESS: f64 = 2.3898936931679436689;
    /// goldens/generate_1d.py: omega_infinity = 1/L_s = 35/18.
    W3_OMEGA_END: f64 = 1.9444444444444444444;
    /// goldens/generate_1d.py: the all-human wage omega/(1 - omega), omega = expm1(1/40).
    W4_V: f64 = 0.025972620543831183633;
    /// goldens/generate_1d.py: gamma(0) pi.
    W4_REPLACEMENT_BOTTOM: f64 = 0.11465675172205473119;
    /// goldens/generate_1d.py: pi, the delivered machine-task price at the wage.
    W4_PI: f64 = 0.57328375861027365597;
    /// goldens/generate_1d.py: P_s.
    W4_P_S: f64 = 1.0259726205438311836;
    /// goldens/generate_1d.py: Y = T/h.
    W4_Y: f64 = 10.0;
    /// goldens/generate_1d.py: N_a.
    W4_N_A: f64 = 10.0;
    /// goldens/generate_1d.py: I.
    W4_INCOME: f64 = 10.259726205438311836;
    /// goldens/generate_1d.py: f on the line at x = 0.
    W4_F_LINE_0: f64 = -10.0;
    /// goldens/generate_1d.py: f on the line at 1e-12, 1a's G8 golden.
    W4_F_LINE_LO: f64 = -10.000000000011;
    /// goldens/generate_1d.py: x*, a root below 1e-12 (T and chi_max the doubles).
    W5_X_STAR: f64 = 4.5458586390082282758e-13;
    /// goldens/generate_1d.py: v.
    W5_V: f64 = 0.1159420289857211242;
    /// goldens/generate_1d.py: P_s.
    W5_P_S: f64 = 1.1159420289857211242;
    /// goldens/generate_1d.py: Y.
    W5_Y: f64 = 10.000000000004480918;
    /// goldens/generate_1d.py: N_a.
    W5_N_A: f64 = 10.0;
    /// goldens/generate_1d.py: f on the line at x = 0.
    W5_F_LINE_0: f64 = 5.0004445029117050581e-12;
    /// goldens/generate_1d.py: f on the line at 1e-12.
    W5_F_LINE_LO: f64 = -5.999555497094538288e-12;
    /// goldens/generate_1d.py: x* with the decimal inputs, for reference: 8.9e-5 away.
    W5_X_STAR_DECIMAL: f64 = 4.5454545454521331974e-13;
    /// goldens/generate_1d.py: x*, a root below 1e-12 at a = 1 - 2^-53, lambda 0.
    W6_X_STAR: f64 = 3.6175751314293239386e-15;
    /// goldens/generate_1d.py: v.
    W6_V: f64 = 720575940379289.78695;
    /// goldens/generate_1d.py: Y.
    W6_Y: f64 = 2.7725887222397884922;
    /// goldens/generate_1d.py: N_a.
    W6_N_A: f64 = 2.7725887222397784621;
    /// goldens/generate_1d.py: f on the line at x = 0.
    W6_F_LINE_0: f64 = 7.2274112777602215379;
    /// goldens/generate_1d.py: f on the line at 1e-12.
    W6_F_LINE_LO: f64 = -2.7587301670408429713;

    // B1: the human-required economy, N 8, eta 1: a contestable margin with the tail
    /// goldens/generate_1d.py: x*.
    B1_X_STAR: f64 = 0.97556565124183680037;
    /// goldens/generate_1d.py: 1 - x*.
    B1_ONE_MINUS_X_STAR: f64 = 0.024434348758163199633;
    /// goldens/generate_1d.py: v.
    B1_V: f64 = 0.60244952296500789847;
    /// goldens/generate_1d.py: p_m.
    B1_P_M: f64 = 0.61446068021178627846;
    /// goldens/generate_1d.py: P_s.
    B1_P_S: f64 = 1.7955392957591152601;
    /// goldens/generate_1d.py: Y.
    B1_Y: f64 = 6.3459650067529959603;
    /// goldens/generate_1d.py: N_a.
    B1_N_A: f64 = 2.3145997896628033234;
    /// goldens/generate_1d.py: Y (H_y + L^H_y), with the tail.
    B1_FINAL_HOURS: f64 = 1.8578454155069278184;
    /// goldens/generate_1d.py: Y L^H_y = Y/4.
    B1_REQUIRED_HOURS: f64 = 1.5864912516882489901;
    /// goldens/generate_1d.py: I.
    B1_INCOME: f64 = 11.394429539137263482;
    /// goldens/generate_1d.py: p of services.
    B1_P_SERVICES: f64 = 0.42700963003462195409;
    /// goldens/generate_1d.py: p of goods.
    B1_P_GOODS: f64 = 0.36852966572449330596;
    /// goldens/generate_1d.py: f on the line at 1.
    B1_F_LINE_1: f64 = -0.32132496440441550864;

    // BP: check_pinning D1's automation path, N 8, every point at the wall (docs/unit-1d.md section 4.8)
    /// goldens/generate_1d.py: the wall's wage at eta 0.3.
    BP_ETA0_3_V: f64 = 0.43450367471671539576;
    /// goldens/generate_1d.py: p_m = (lambda v + b)/(1 - a) at eta 0.3.
    BP_ETA0_3_P_M: f64 = 0.6024645481940510997;
    /// goldens/generate_1d.py: p_services/v -> 1/4 at eta 0.3.
    BP_ETA0_3_P_SERVICES_OVER_V: f64 = 0.43718533061710840711;
    /// goldens/generate_1d.py: labour's share of services' price -> 1 at eta 0.3.
    BP_ETA0_3_PHI_W_SERVICES: f64 = 0.59389654446172425849;
    /// goldens/generate_1d.py: p_goods -> 0 at eta 0.3.
    BP_ETA0_3_P_GOODS: f64 = 0.10844361867492919795;
    /// goldens/generate_1d.py: p_services/p_goods, diverging at eta 0.3.
    BP_ETA0_3_RELATIVE_PRICE: f64 = 1.7516810579218691866;
    /// goldens/generate_1d.py: ces_share(0.3, 0.5, p_services/p_goods) -> 1 at eta 0.3.
    BP_ETA0_3_CES_SHARE: f64 = 0.46422101645782538085;
    /// goldens/generate_1d.py: v/(gamma(1) pi), growing as 1/eta: the machine no longer prices the hour at eta 0.3.
    BP_ETA0_3_V_OVER_REPLACEMENT: f64 = 2.4040345390124860479;
    /// goldens/generate_1d.py: the wall's wage at eta 0.1.
    BP_ETA0_1_V: f64 = 0.41390571390730221459;
    /// goldens/generate_1d.py: p_m = (lambda v + b)/(1 - a) at eta 0.1.
    BP_ETA0_1_P_M: f64 = 0.60099326527909301533;
    /// goldens/generate_1d.py: p_services/v -> 1/4 at eta 0.1.
    BP_ETA0_1_P_SERVICES_OVER_V: f64 = 0.31534023578040307229;
    /// goldens/generate_1d.py: labour's share of services' price -> 1 at eta 0.1.
    BP_ETA0_1_PHI_W_SERVICES: f64 = 0.8029875575111173409;
    /// goldens/generate_1d.py: p_goods -> 0 at eta 0.1.
    BP_ETA0_1_P_GOODS: f64 = 0.03605959591674558092;
    /// goldens/generate_1d.py: p_services/p_goods, diverging at eta 0.1.
    BP_ETA0_1_RELATIVE_PRICE: f64 = 3.6195947873551328935;
    /// goldens/generate_1d.py: ces_share(0.3, 0.5, p_services/p_goods) -> 1 at eta 0.1.
    BP_ETA0_1_CES_SHARE: f64 = 0.5546636639207168106;
    /// goldens/generate_1d.py: v/(gamma(1) pi), growing as 1/eta: the machine no longer prices the hour at eta 0.1.
    BP_ETA0_1_V_OVER_REPLACEMENT: f64 = 6.8870274896523189443;
    /// goldens/generate_1d.py: the wall's wage at eta 0.01.
    BP_ETA0_01_V: f64 = 0.40487078336474958949;
    /// goldens/generate_1d.py: p_m = (lambda v + b)/(1 - a) at eta 0.01.
    BP_ETA0_01_P_M: f64 = 0.60034791309748211353;
    /// goldens/generate_1d.py: p_services/v -> 1/4 at eta 0.01.
    BP_ETA0_01_P_SERVICES_OVER_V: f64 = 0.25667266130316155473;
    /// goldens/generate_1d.py: labour's share of services' price -> 1 at eta 0.01.
    BP_ETA0_01_PHI_W_SERVICES: f64 = 0.97525551533425132932;
    /// goldens/generate_1d.py: p_goods -> 0 at eta 0.01.
    BP_ETA0_01_P_GOODS: f64 = 0.0036020874785848926812;
    /// goldens/generate_1d.py: p_services/p_goods, diverging at eta 0.01.
    BP_ETA0_01_RELATIVE_PRICE: f64 = 28.849732847396458839;
    /// goldens/generate_1d.py: ces_share(0.3, 0.5, p_services/p_goods) -> 1 at eta 0.01.
    BP_ETA0_01_CES_SHARE: f64 = 0.77857845517218100888;
    /// goldens/generate_1d.py: v/(gamma(1) pi), growing as 1/eta: the machine no longer prices the hour at eta 0.01.
    BP_ETA0_01_V_OVER_REPLACEMENT: f64 = 67.439358833751501213;
    /// goldens/generate_1d.py: the wall's wage at eta 0.001.
    BP_ETA0_001_V: f64 = 0.40397656969686354193;
    /// goldens/generate_1d.py: p_m = (lambda v + b)/(1 - a) at eta 0.001.
    BP_ETA0_001_P_M: f64 = 0.60028404069263311014;
    /// goldens/generate_1d.py: p_services/v -> 1/4 at eta 0.001.
    BP_ETA0_001_P_SERVICES_OVER_V: f64 = 0.25066867199380989783;
    /// goldens/generate_1d.py: labour's share of services' price -> 1 at eta 0.001.
    BP_ETA0_001_PHI_W_SERVICES: f64 = 0.99746067535442658199;
    /// goldens/generate_1d.py: p_goods -> 0 at eta 0.001.
    BP_ETA0_001_P_GOODS: f64 = 0.00036017042441557986608;
    /// goldens/generate_1d.py: p_services/p_goods, diverging at eta 0.001.
    BP_ETA0_001_RELATIVE_PRICE: f64 = 281.15653973210352567;
    /// goldens/generate_1d.py: ces_share(0.3, 0.5, p_services/p_goods) -> 1 at eta 0.001.
    BP_ETA0_001_CES_SHARE: f64 = 0.91650699681441674216;
    /// goldens/generate_1d.py: v/(gamma(1) pi), growing as 1/eta: the machine no longer prices the hour at eta 0.001.
    BP_ETA0_001_V_OVER_REPLACEMENT: f64 = 672.9756953570484616;
    /// goldens/generate_1d.py: the wall's wage at eta 1e-6.
    BP_ETA1EM6_V: f64 = 0.40387742178231295406;
    /// goldens/generate_1d.py: p_m = (lambda v + b)/(1 - a) at eta 1e-6.
    BP_ETA1EM6_P_M: f64 = 0.60027695869873663958;
    /// goldens/generate_1d.py: p_services/v -> 1/4 at eta 1e-6.
    BP_ETA1EM6_P_SERVICES_OVER_V: f64 = 0.25000066882825542059;
    /// goldens/generate_1d.py: labour's share of services' price -> 1 at eta 1e-6.
    BP_ETA1EM6_PHI_W_SERVICES: f64 = 0.99999745326522020177;
    /// goldens/generate_1d.py: p_goods -> 0 at eta 1e-6.
    BP_ETA1EM6_P_GOODS: f64 = 3.6016617521924198375e-7;
    /// goldens/generate_1d.py: p_services/p_goods, diverging at eta 1e-6.
    BP_ETA1EM6_RELATIVE_PRICE: f64 = 280341.77698321327824;
    /// goldens/generate_1d.py: ces_share(0.3, 0.5, p_services/p_goods) -> 1 at eta 1e-6.
    BP_ETA1EM6_CES_SHARE: f64 = 0.99712330810402835357;
    /// goldens/generate_1d.py: v/(gamma(1) pi), growing as 1/eta: the machine no longer prices the hour at eta 1e-6.
    BP_ETA1EM6_V_OVER_REPLACEMENT: f64 = 672818.46475971186778;

    // BLIM: the limit of BP as eta -> 0
    /// goldens/generate_1d.py: v_infinity = zeta/(1 - zeta/4), zeta = expm1(5/16).
    BLIM_V_INF: f64 = 0.40387732254620515981;

    // B2: N 4, the tail decides the regime
    /// goldens/generate_1d.py: x* without the tail.
    B2_NO_TAIL_X_STAR: f64 = 0.9321231698467138939;
    /// goldens/generate_1d.py: v.
    B2_NO_TAIL_V: f64 = 0.57954753900939173627;
    /// goldens/generate_1d.py: P_s.
    B2_NO_TAIL_P_S: f64 = 1.6414896533693263696;
    /// goldens/generate_1d.py: Y.
    B2_NO_TAIL_Y: f64 = 6.5190489932451689352;
    /// goldens/generate_1d.py: N_a.
    B2_NO_TAIL_N_A: f64 = 1.2094805430763968357;
    /// goldens/generate_1d.py: provider baskets.
    B2_NO_TAIL_PROVIDER_BASKETS: f64 = 2.0920274334193769204;
    /// goldens/generate_1d.py: the wall's wage with the tail.
    B2_TAIL_V: f64 = 1.3486543727559672398;
    /// goldens/generate_1d.py: g = v/pi.
    B2_TAIL_G: f64 = 2.0196661964108324913;
    /// goldens/generate_1d.py: P_s.
    B2_TAIL_P_S: f64 = 2.0383126711456893529;
    /// goldens/generate_1d.py: Y = 6.25.
    B2_TAIL_Y: f64 = 6.25;
    /// goldens/generate_1d.py: N_a = 65/32.
    B2_TAIL_N_A: f64 = 2.03125;
    /// goldens/generate_1d.py: provider baskets.
    B2_TAIL_PROVIDER_BASKETS: f64 = 0.90601866021822189607;
    /// goldens/generate_1d.py: f on the line at 1.
    B2_TAIL_F_LINE_1: f64 = 0.85496251779779224568;

    // E: entrant and trained, on B's economy (docs/unit-1d.md section 3.3)
    /// goldens/generate_1d.py: x*.
    E1_X_STAR: f64 = 0.93526838038380481533;
    /// goldens/generate_1d.py: v, the pool's wage.
    E1_V: f64 = 0.58120153393571462686;
    /// goldens/generate_1d.py: P_s.
    E1_P_S: f64 = 1.8917089345791874112;
    /// goldens/generate_1d.py: Y.
    E1_Y: f64 = 6.5064160111838815002;
    /// goldens/generate_1d.py: N_a, hours of both types.
    E1_N_A: f64 = 3.3621110178126007371;
    /// goldens/generate_1d.py: the pool's efficiency hours.
    E1_N_POOL: f64 = 2.8003509824241698881;
    /// goldens/generate_1d.py: I.
    E1_INCOME: f64 = 12.308245300445626796;
    /// goldens/generate_1d.py: the entrant's hours.
    E1_ENTRANT_HOURS: f64 = 2.1433213245632650952;
    /// goldens/generate_1d.py: the trained's wage.
    E1_TRAINED_WAGE: f64 = 0.87180230090357194029;
    /// goldens/generate_1d.py: the trained's premium v_T/(1.5 v).
    E1_TRAINED_PREMIUM: f64 = 1.0;
    /// goldens/generate_1d.py: the trained's hours.
    E1_TRAINED_HOURS: f64 = 1.2187896932493356419;
    /// goldens/generate_1d.py: the trained's reserved hours D_T.
    E1_TRAINED_RESERVED_HOURS: f64 = 0.78076992134206578002;
    /// goldens/generate_1d.py: x*.
    E2_X_STAR: f64 = 0.99416996327993848043;
    /// goldens/generate_1d.py: v, the pool's wage.
    E2_V: f64 = 0.6122947695538158456;
    /// goldens/generate_1d.py: P_s.
    E2_P_S: f64 = 2.0418839426825230478;
    /// goldens/generate_1d.py: Y.
    E2_Y: f64 = 6.2728033691443532844;
    /// goldens/generate_1d.py: N_a, hours of both types.
    E2_N_A: f64 = 2.8508355049050816456;
    /// goldens/generate_1d.py: the pool's efficiency hours.
    E2_N_POOL: f64 = 2.0980991006077592515;
    /// goldens/generate_1d.py: I.
    E2_INCOME: f64 = 12.808336475060686125;
    /// goldens/generate_1d.py: the entrant's hours.
    E2_ENTRANT_HOURS: f64 = 2.0980991006077592515;
    /// goldens/generate_1d.py: the trained's wage.
    E2_TRAINED_WAGE: f64 = 2.0241898240265697611;
    /// goldens/generate_1d.py: the trained's premium v_T/(1.5 v).
    E2_TRAINED_PREMIUM: f64 = 2.2039382823206903753;
    /// goldens/generate_1d.py: the trained's hours.
    E2_TRAINED_HOURS: f64 = 0.75273640429732239413;
    /// goldens/generate_1d.py: the trained's reserved hours D_T.
    E2_TRAINED_RESERVED_HOURS: f64 = 0.75273640429732239413;
    /// goldens/generate_1d.py: v, the pool's wage.
    E3_V: f64 = 0.47204913358823543131;
    /// goldens/generate_1d.py: P_s.
    E3_P_S: f64 = 1.4105971789611369616;
    /// goldens/generate_1d.py: Y.
    E3_Y: f64 = 8.474576271186440678;
    /// goldens/generate_1d.py: N_a, hours of both types.
    E3_N_A: f64 = 3.3262711864406779661;
    /// goldens/generate_1d.py: the pool's efficiency hours.
    E3_N_POOL: f64 = 2.3093220338983050847;
    /// goldens/generate_1d.py: I.
    E3_INCOME: f64 = 11.95421338102658442;
    /// goldens/generate_1d.py: the entrant's hours.
    E3_ENTRANT_HOURS: f64 = 2.3093220338983050847;
    /// goldens/generate_1d.py: the trained's wage.
    E3_TRAINED_WAGE: f64 = 0.84969825048619005462;
    /// goldens/generate_1d.py: the trained's premium v_T/(1.5 v).
    E3_TRAINED_PREMIUM: f64 = 1.2000138545286472029;
    /// goldens/generate_1d.py: the trained's hours.
    E3_TRAINED_HOURS: f64 = 1.0169491525423728814;
    /// goldens/generate_1d.py: the trained's reserved hours D_T.
    E3_TRAINED_RESERVED_HOURS: f64 = 1.0169491525423728814;
    /// goldens/generate_1d.py: v, the pool's wage.
    E4_V: f64 = 0.40186205291748006285;
    /// goldens/generate_1d.py: P_s.
    E4_P_S: f64 = 1.3618425789451597284;
    /// goldens/generate_1d.py: Y.
    E4_Y: f64 = 8.474576271186440678;
    /// goldens/generate_1d.py: N_a, hours of both types.
    E4_N_A: f64 = 3.2460381913181589815;
    /// goldens/generate_1d.py: the pool's efficiency hours.
    E4_N_POOL: f64 = 2.3093220338983050847;
    /// goldens/generate_1d.py: I.
    E4_INCOME: f64 = 11.541038804619997699;
    /// goldens/generate_1d.py: the entrant's hours.
    E4_ENTRANT_HOURS: f64 = 2.068623048530748131;
    /// goldens/generate_1d.py: the trained's wage.
    E4_TRAINED_WAGE: f64 = 0.60279307937622009428;
    /// goldens/generate_1d.py: the trained's premium v_T/(1.5 v).
    E4_TRAINED_PREMIUM: f64 = 1.0;
    /// goldens/generate_1d.py: the trained's hours.
    E4_TRAINED_HOURS: f64 = 1.1774151427874108505;
    /// goldens/generate_1d.py: the trained's reserved hours D_T.
    E4_TRAINED_RESERVED_HOURS: f64 = 1.0169491525423728814;
    /// goldens/generate_1d.py: v, the pool's wage.
    E5_V: f64 = 0.029413782758211337152;
    /// goldens/generate_1d.py: P_s.
    E5_P_S: f64 = 1.1619056970251169848;
    /// goldens/generate_1d.py: Y.
    E5_Y: f64 = 10.0;
    /// goldens/generate_1d.py: N_a, hours of both types.
    E5_N_A: f64 = 21.2;
    /// goldens/generate_1d.py: the pool's efficiency hours.
    E5_N_POOL: f64 = 20.0;
    /// goldens/generate_1d.py: I.
    E5_INCOME: f64 = 11.619056970251169848;
    /// goldens/generate_1d.py: the entrant's hours.
    E5_ENTRANT_HOURS: f64 = 20.0;
    /// goldens/generate_1d.py: the trained's wage.
    E5_TRAINED_WAGE: f64 = 0.85898442923911925401;
    /// goldens/generate_1d.py: the trained's premium v_T/(1.5 v).
    E5_TRAINED_PREMIUM: f64 = 19.468977889270175376;
    /// goldens/generate_1d.py: the trained's hours.
    E5_TRAINED_HOURS: f64 = 1.2;
    /// goldens/generate_1d.py: the trained's reserved hours D_T.
    E5_TRAINED_RESERVED_HOURS: f64 = 1.2;
    /// goldens/generate_1d.py: gamma(0) pi.
    E5_REPLACEMENT_BOTTOM: f64 = 0.11470591118226016196;
    /// goldens/generate_1d.py: the trained's reserved demand at x = 1, above N_T 0.5.
    E6_TRAINED_DEMAND_AT_1: f64 = 0.75;

    // E7 and E8: three types and the walk
    /// goldens/generate_1d.py: x*.
    E7_X_STAR: f64 = 0.92712986242639353397;
    /// goldens/generate_1d.py: v.
    E7_V: f64 = 0.5769229810714520861;
    /// goldens/generate_1d.py: P_s.
    E7_P_S: f64 = 1.9276746171072055141;
    /// goldens/generate_1d.py: the trained's wage, pooled.
    E7_TRAINED_WAGE: f64 = 0.86538447160717812915;
    /// goldens/generate_1d.py: the master's wage, at its wall.
    E7_MASTER_WAGE: f64 = 1.1182679714505371933;
    /// goldens/generate_1d.py: the master's premium.
    E7_MASTER_PREMIUM: f64 = 1.0768508180856191135;
    /// goldens/generate_1d.py: the master's hours.
    E7_MASTER_HOURS: f64 = 0.3269568283179602348;
    /// goldens/generate_1d.py: the P_s above which the trained is walled.
    E7_TRAINED_THRESHOLD: f64 = 3.7854989805027272803;
    /// goldens/generate_1d.py: the P_s above which the master is walled, below the trained's.
    E7_MASTER_THRESHOLD: f64 = 1.7901036844956349594;
    /// goldens/generate_1d.py: x*.
    E8_X_STAR: f64 = 0.98812228934750603464;
    /// goldens/generate_1d.py: v.
    E8_V: f64 = 0.60909191982741945457;
    /// goldens/generate_1d.py: P_s, both walled.
    E8_P_S: f64 = 1.9568213969246348103;
    /// goldens/generate_1d.py: P after walling the master alone, above the trained's threshold.
    E8_P_S_MASTER_WALLED: f64 = 1.956623245446503853;
    /// goldens/generate_1d.py: the trained's threshold.
    E8_TRAINED_THRESHOLD: f64 = 1.9527320879312454996;
    /// goldens/generate_1d.py: the trained's wage.
    E8_TRAINED_WAGE: f64 = 0.91555117221039191283;
    /// goldens/generate_1d.py: the trained's premium.
    E8_TRAINED_PREMIUM: f64 = 1.0020941474863157432;
    /// goldens/generate_1d.py: the master's wage.
    E8_MASTER_WAGE: f64 = 1.3474426859707907976;
    /// goldens/generate_1d.py: the master's premium.
    E8_MASTER_PREMIUM: f64 = 1.2290087023250540863;

    // E9: E4 with chi_max 3 for both types: the end of the wall with supply unsaturated (docs/unit-1d.md section 12 item 18)
    /// goldens/generate_1d.py: the wall's wage.
    E9_V: f64 = 4.3862693691168219969;
    /// goldens/generate_1d.py: P_s.
    E9_P_S: f64 = 3.1845200222277115569;
    /// goldens/generate_1d.py: f at the end of the wall, both types unsaturated there.
    E9_F_END: f64 = -1.2628620845098934739;
    /// goldens/generate_1d.py: omega_infinity, from the walk with every type's efficiency.
    E9_OMEGA_END: f64 = 2.2099447513812154696;
    /// goldens/generate_1d.py: the trained's wage.
    E9_TRAINED_WAGE: f64 = 6.7438468261948130232;

    // F: 1c's M4 with L^H_care 0.2, the entrant and the trained (docs/unit-1d.md section 3.3)
    /// goldens/generate_1d.py: x*, with the engine.
    F1_X_STAR: f64 = 0.98889368696731094579;
    /// goldens/generate_1d.py: v.
    F1_V: f64 = 0.15114482308841387646;
    /// goldens/generate_1d.py: P_s.
    F1_P_S: f64 = 1.6036808524929569565;
    /// goldens/generate_1d.py: Y.
    F1_Y: f64 = 6.4914476902294503478;
    /// goldens/generate_1d.py: N_a.
    F1_N_A: f64 = 1.0873141491115720841;
    /// goldens/generate_1d.py: the pool's hours.
    F1_N_POOL: f64 = 0.36027200780587364517;
    /// goldens/generate_1d.py: I.
    F1_INCOME: f64 = 10.410210365780601306;
    /// goldens/generate_1d.py: interest.
    F1_INTEREST: f64 = 0.056415486787913071726;
    /// goldens/generate_1d.py: the trained's wage, at its wall.
    F1_TRAINED_WAGE: f64 = 0.4117252812492722115;
    /// goldens/generate_1d.py: the trained's premium.
    F1_TRAINED_PREMIUM: f64 = 1.9457463418022066925;
    /// goldens/generate_1d.py: the trained's hours.
    F1_TRAINED_HOURS: f64 = 0.72704214130569843896;
    /// goldens/generate_1d.py: p of manufactures.
    F1_MANUFACTURES_P: f64 = 0.043919939930458856896;
    /// goldens/generate_1d.py: reserved cost of manufactures through the chain.
    F1_MANUFACTURES_RESERVED_COST: f64 = 0.0;
    /// goldens/generate_1d.py: p of food.
    F1_FOOD_P: f64 = 0.68879942076695752269;
    /// goldens/generate_1d.py: reserved cost of food through the chain.
    F1_FOOD_RESERVED_COST: f64 = 0.020586264062463610575;
    /// goldens/generate_1d.py: p of care.
    F1_CARE_P: f64 = 0.42581136186527803663;
    /// goldens/generate_1d.py: reserved cost of care through the chain.
    F1_CARE_RESERVED_COST: f64 = 0.12763483718727438557;
    /// goldens/generate_1d.py: p of shelter.
    F1_SHELTER_P: f64 = 1.0206789717172577118;
    /// goldens/generate_1d.py: reserved cost of shelter through the chain.
    F1_SHELTER_RESERVED_COST: f64 = 0.0;
    /// goldens/generate_1d.py: the wall's wage, with the engine.
    F2_V: f64 = 0.1165053534428324599;
    /// goldens/generate_1d.py: g = v/pi.
    F2_G: f64 = 0.76791810604669908158;
    /// goldens/generate_1d.py: P_s.
    F2_P_S: f64 = 1.5215351426172399225;
    /// goldens/generate_1d.py: Y.
    F2_Y: f64 = 6.8110704414527195923;
    /// goldens/generate_1d.py: N_a.
    F2_N_A: f64 = 1.0579635716708509343;
    /// goldens/generate_1d.py: gamma_s of the switch loom -> engine on the wall.
    F2_WALL_SWITCH_GAMMA: f64 = 0.43150661211919930493;
    /// goldens/generate_1d.py: v_s, the loom's closure wage at gamma_s.
    F2_WALL_SWITCH_V: f64 = 0.064963057288499518554;
    /// goldens/generate_1d.py: the trained's premium.
    F2_TRAINED_PREMIUM: f64 = 2.5252998838906632167;
    /// goldens/generate_1d.py: x* = the switch loom -> engine on the line.
    F3_X_STAR: f64 = 0.82876653029799826234;
    /// goldens/generate_1d.py: v.
    F3_V: f64 = 0.064963057288499518554;
    /// goldens/generate_1d.py: sigma, the engine's share, by bisection.
    F3_SHARE: f64 = 0.8933594540375401821;
    /// goldens/generate_1d.py: 1c's closed form at the same switch, for reference.
    F3_SHARE_CLOSED_FORM: f64 = 0.8937435873915790651;
    /// goldens/generate_1d.py: P_s.
    F3_P_S: f64 = 1.6060227418448017258;
    /// goldens/generate_1d.py: Y.
    F3_Y: f64 = 6.715477852810912082;
    /// goldens/generate_1d.py: N_a.
    F3_N_A: f64 = 1.3865811223593493908;
    /// goldens/generate_1d.py: the trained's premium.
    F3_TRAINED_PREMIUM: f64 = 10.457936453153349916;

    // X: switches at the wall, 1c's M5 at rho 0.15, chi_max 3 (docs/unit-1d.md section 3.3)
    /// goldens/generate_1d.py: gamma_s of the switch flow -> durable, above gamma(1).
    X_WALL_SWITCH_GAMMA: f64 = 5.7939090413420924643;
    /// goldens/generate_1d.py: v_s, the flow type's closure wage at gamma_s.
    X_WALL_SWITCH_V: f64 = 6.8875222841006163301;
    /// goldens/generate_1d.py: sigma toward the durable type.
    X1_SHARE: f64 = 0.47890358118953065926;
    /// goldens/generate_1d.py: P_s.
    X1_P_S: f64 = 1.7132513370460369798;
    /// goldens/generate_1d.py: Y.
    X1_Y: f64 = 8.5692697142403971175;
    /// goldens/generate_1d.py: N_a.
    X1_N_A: f64 = 0.26890987054373998522;
    /// goldens/generate_1d.py: interest.
    X1_INTEREST: f64 = 2.8291900696458505285;
    /// goldens/generate_1d.py: v, the flow type.
    X2_V: f64 = 4.7431304705568019961;
    /// goldens/generate_1d.py: P_s.
    X2_P_S: f64 = 1.5845878282334081198;
    /// goldens/generate_1d.py: Y = 100/13.
    X2_Y: f64 = 7.6923076923076923077;
    /// goldens/generate_1d.py: N_a = 6/13.
    X2_N_A: f64 = 0.46153846153846153846;
    /// goldens/generate_1d.py: v, the durable type.
    X3_V: f64 = 16.892543391434617742;
    /// goldens/generate_1d.py: P_s.
    X3_P_S: f64 = 1.7850520472154803024;
    /// goldens/generate_1d.py: Y.
    X3_Y: f64 = 9.7828213656818626492;
    /// goldens/generate_1d.py: N_a.
    X3_N_A: f64 = 0.0023478771277636470358;
    /// goldens/generate_1d.py: interest.
    X3_INTEREST: f64 = 7.4231836900952454918;

    // J: the edge of a reserved shortage (docs/unit-1d.md section 12 item 16)
    /// goldens/generate_1d.py: x*, where the trained's reserved demand D_T = 0.12 Y reaches N_T 1.
    J1_X_STAR: f64 = 0.5;
    /// goldens/generate_1d.py: v.
    J1_V: f64 = 0.35820895522388059701;
    /// goldens/generate_1d.py: P_s.
    J1_P_S: f64 = 29.723913718591018298;
    /// goldens/generate_1d.py: Y.
    J1_Y: f64 = 8.3333333333333333333;
    /// goldens/generate_1d.py: N_a.
    J1_N_A: f64 = 10.583333333333333333;
    /// goldens/generate_1d.py: the pool's efficiency hours.
    J1_N_POOL: f64 = 9.5833333333333333333;
    /// goldens/generate_1d.py: the entrant's hours.
    J1_ENTRANT_HOURS: f64 = 9.5833333333333333333;
    /// goldens/generate_1d.py: the trained's wage, kappa nu P_s.
    J1_TRAINED_WAGE: f64 = 234.26644516736296343;
    /// goldens/generate_1d.py: kappa, above expm1(0.8): set by the pool's clearing.
    J1_TRAINED_CLEARING: f64 = 6.5678443117457383283;
    /// goldens/generate_1d.py: the trained's premium.
    J1_TRAINED_PREMIUM: f64 = 435.99588406148107083;
    /// goldens/generate_1d.py: N_T, the double midway between D_T(0) and D_T(1e-12).
    J1B_N_T: f64 = 1.1999999999998798295;
    /// goldens/generate_1d.py: x*, the edge below 1e-12.
    J1B_X_STAR: f64 = 5.0071058410549431977e-13;
    /// goldens/generate_1d.py: v.
    J1B_V: f64 = 0.1159420289857428254;
    /// goldens/generate_1d.py: P_s.
    J1B_P_S: f64 = 4.5799516883169634236;
    /// goldens/generate_1d.py: the trained's wage.
    J1B_TRAINED_WAGE: f64 = 27.900563586212314773;
    /// goldens/generate_1d.py: kappa.
    J1B_TRAINED_CLEARING: f64 = 5.0765753085314729389;
    /// goldens/generate_1d.py: N_T, the double midway between D_T under the loom and the engine.
    J2_N_T: f64 = 0.75421467109117334982;
    /// goldens/generate_1d.py: sigma, where the mix's D_T reaches N_T.
    J2_SHARE: f64 = 0.49824924569934011727;
    /// goldens/generate_1d.py: v.
    J2_V: f64 = 0.064963057288499518554;
    /// goldens/generate_1d.py: P_s.
    J2_P_S: f64 = 1.9517792445568691801;
    /// goldens/generate_1d.py: Y.
    J2_Y: f64 = 6.7340595633140477662;
    /// goldens/generate_1d.py: N_a.
    J2_N_A: f64 = 1.7364776126193999188;
    /// goldens/generate_1d.py: the trained's wage.
    J2_TRAINED_WAGE: f64 = 4.0382429662537090722;
    /// goldens/generate_1d.py: kappa.
    J2_TRAINED_CLEARING: f64 = 1.7241716660646857619;
}
