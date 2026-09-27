//! The golden numbers of unit 1e, one constant each.
//!
//! Every constant equals its line in `goldens/goldens_1e.txt` rounded to 20 significant
//! digits (half up), and `e10_goldens_file::constants_match_goldens_1e_txt` enforces that.
//! `goldens_1e.txt` is written by `goldens/generate_1e.py`, which solves each instance with
//! mpmath at 70 digits from the equations of docs/unit-1e.md §4. Each constant's doc comment
//! is the generator's note; laborformal references are at 31b3482.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE_1E` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE_1E: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // P: the exit value alone, check_pinning P3 and check_enclosure N-ii, N-iii, N-vi (docs/unit-1e.md section 4.2)
    /// goldens/generate_1e.py: check_pinning P3 (10, 4, 6): q_enc = (s0 - s_)/h = 1.
    P3_Q_ENC: f64 = 1.0;
    /// goldens/generate_1e.py: check_enclosure (1.5, 0, 1): q_enc = 1.5.
    P_Q_ENC: f64 = 1.5;
    /// goldens/generate_1e.py: N_crit = q_enc T/(g_s + q_enc h_s) = 60 at T 100, g_s = h_s = 1.
    P_N_CRIT: f64 = 60.0;
    /// goldens/generate_1e.py: q* = N g_s/(T - N h_s) at N 50.
    P_Q_STAR_50: f64 = 1.0;
    /// goldens/generate_1e.py: q* = N g_s/(T - N h_s) at N 60.
    P_Q_STAR_60: f64 = 1.5;
    /// goldens/generate_1e.py: q* = N g_s/(T - N h_s) at N 80.
    P_Q_STAR_80: f64 = 4.0;
    /// goldens/generate_1e.py: SSRN D.3: kappa(q = 10/9, N 50) = 20/19.
    P_KAPPA_D3: f64 = 1.0526315789473684211;
    /// goldens/generate_1e.py: N-vi (1.5, 0.1, 1): the take caps at h q_enc = 1.4.
    P_TAKE_CAP: f64 = 1.4;

    // Q: the race, check_enclosure N-iii and SSRN D.3 inside equilibria (docs/unit-1e.md section 4.9)
    /// goldens/generate_1e.py: x*.
    Q1_X_STAR: f64 = 0.70579667000425371499;
    /// goldens/generate_1e.py: v, the pool's wage.
    Q1_V: f64 = 1.2650754686042984246;
    /// goldens/generate_1e.py: P_s.
    Q1_P_S: f64 = 1.9354049316010742152;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    Q1_Y: f64 = 44.196148236528373027;
    /// goldens/generate_1e.py: N_a, hours worked.
    Q1_N_A: f64 = 15.689297662407120614;
    /// goldens/generate_1e.py: N_a/N.
    Q1_PARTICIPATION: f64 = 0.31378595324814241228;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    Q1_INCOME: f64 = 85.537443254749132593;
    /// goldens/generate_1e.py: q = r/p_g.
    Q1_Q: f64 = 1.0690557278636135043;
    /// goldens/generate_1e.py: coverage kappa = q T/(N (1 + q)).
    Q1_KAPPA: f64 = 1.0333754799030553077;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    Q1_T_P: f64 = 34.310702337592879386;
    /// goldens/generate_1e.py: provider baskets (r T_m + interest)/P_s - nu.
    Q1_PROVIDER_BASKETS: f64 = -16.059145251807696265;
    /// goldens/generate_1e.py: the good's price, the exit good.
    Q1_P_G: f64 = 0.93540493160107421517;
    /// goldens/generate_1e.py: x*.
    Q2_X_STAR: f64 = 0.57643645580848169258;
    /// goldens/generate_1e.py: v, the pool's wage.
    Q2_V: f64 = 0.83440886282971784135;
    /// goldens/generate_1e.py: P_s.
    Q2_P_S: f64 = 1.6666666666666666667;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    Q2_Y: f64 = 53.055826834488699395;
    /// goldens/generate_1e.py: N_a, hours worked.
    Q2_N_A: f64 = 24.35371364352931986;
    /// goldens/generate_1e.py: N_a/N.
    Q2_PARTICIPATION: f64 = 0.40589522739215533101;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    Q2_INCOME: f64 = 88.426378057481165659;
    /// goldens/generate_1e.py: q = r/p_g.
    Q2_Q: f64 = 1.5;
    /// goldens/generate_1e.py: coverage kappa = q T/(N (1 + q)).
    Q2_KAPPA: f64 = 1.0;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    Q2_T_P: f64 = 31.894576449496718503;
    /// goldens/generate_1e.py: provider baskets (r T_m + interest)/P_s - nu.
    Q2_PROVIDER_BASKETS: f64 = -19.136745869698031102;
    /// goldens/generate_1e.py: the good's price, the exit good.
    Q2_P_G: f64 = 0.66666666666666666667;
    /// goldens/generate_1e.py: x*.
    Q3_X_STAR: f64 = 0.57643645580848169258;
    /// goldens/generate_1e.py: v, the pool's wage.
    Q3_V: f64 = 0.83440886282971784135;
    /// goldens/generate_1e.py: P_s.
    Q3_P_S: f64 = 1.6666666666666666667;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    Q3_Y: f64 = 70.741102445984932527;
    /// goldens/generate_1e.py: N_a, hours worked.
    Q3_N_A: f64 = 32.471618191372426481;
    /// goldens/generate_1e.py: N_a/N.
    Q3_PARTICIPATION: f64 = 0.40589522739215533101;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    Q3_INCOME: f64 = 117.90183740997488755;
    /// goldens/generate_1e.py: q = r/p_g.
    Q3_Q: f64 = 1.5;
    /// goldens/generate_1e.py: coverage kappa = q T/(N (1 + q)).
    Q3_KAPPA: f64 = 0.75;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    Q3_T_P: f64 = 9.1927685993289580039;
    /// goldens/generate_1e.py: provider baskets (r T_m + interest)/P_s - nu.
    Q3_PROVIDER_BASKETS: f64 = -25.515661159597374802;
    /// goldens/generate_1e.py: the good's price, the exit good.
    Q3_P_G: f64 = 0.66666666666666666667;
    /// goldens/generate_1e.py: x*.
    Q4_X_STAR: f64 = 0.73337119933110234855;
    /// goldens/generate_1e.py: v, the pool's wage.
    Q4_V: f64 = 0.47630601193448258737;
    /// goldens/generate_1e.py: P_s.
    Q4_P_S: f64 = 1.3460534470801926986;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    Q4_Y: f64 = 82.867398902794997659;
    /// goldens/generate_1e.py: N_a, hours worked.
    Q4_N_A: f64 = 24.236410321153980672;
    /// goldens/generate_1e.py: N_a/N.
    Q4_PARTICIPATION: f64 = 0.3029551290144247584;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    Q4_INCOME: f64 = 111.54394794367658488;
    /// goldens/generate_1e.py: q = r/p_g.
    Q4_Q: f64 = 2.8897270304267912443;
    /// goldens/generate_1e.py: coverage kappa = q T/(N (1 + q)).
    Q4_KAPPA: f64 = 0.92864068860820635539;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    Q4_T_P: f64 = 0.0;
    /// goldens/generate_1e.py: provider baskets (r T_m + interest)/P_s - nu.
    Q4_PROVIDER_BASKETS: f64 = -5.708744911343491569;
    /// goldens/generate_1e.py: the good's price, the exit good.
    Q4_P_G: f64 = 0.34605344708019269859;
    /// goldens/generate_1e.py: x*.
    Q5_X_STAR: f64 = 0.79095995667054840485;
    /// goldens/generate_1e.py: v, the pool's wage.
    Q5_V: f64 = 0.34757985454775912373;
    /// goldens/generate_1e.py: P_s.
    Q5_P_S: f64 = 1.2431319882017187786;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    Q5_Y: f64 = 85.956760819946089544;
    /// goldens/generate_1e.py: N_a, hours worked.
    Q5_N_A: f64 = 19.723809903767576589;
    /// goldens/generate_1e.py: N_a/N.
    Q5_PARTICIPATION: f64 = 0.24654762379709470736;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    Q5_INCOME: f64 = 106.85559897747918515;
    /// goldens/generate_1e.py: q = r/p_g.
    Q5_Q: f64 = 4.1129923190951419075;
    /// goldens/generate_1e.py: coverage kappa = q T/(N (1 + q)).
    Q5_KAPPA: f64 = 1.0055247647582589377;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    Q5_T_P: f64 = 0.0;
    /// goldens/generate_1e.py: provider baskets (r T_m + interest)/P_s - nu.
    Q5_PROVIDER_BASKETS: f64 = 0.44198118066071501502;
    /// goldens/generate_1e.py: the good's price, the exit good.
    Q5_P_G: f64 = 0.24313198820171877857;
    /// goldens/generate_1e.py: psi, the renting share of exiters at the enclosure tie.
    Q2_SHARE: f64 = 0.89475173179455385939;
    /// goldens/generate_1e.py: f at the enclosure point, the type on its floor.
    Q2_F_BELOW: f64 = 11.405132530402671205;
    /// goldens/generate_1e.py: f at the enclosure point, the type renting.
    Q2_F_ABOVE: f64 = -1.3415681745269859419;
    /// goldens/generate_1e.py: psi, the renting share of exiters at the enclosure tie.
    Q3_SHARE: f64 = 0.19341640193734186373;
    /// goldens/generate_1e.py: f at the enclosure point, the type on its floor.
    Q3_F_BELOW: f64 = 3.2872279825595645849;
    /// goldens/generate_1e.py: f at the enclosure point, the type renting.
    Q3_F_ABOVE: f64 = -13.708372957346644944;
    /// goldens/generate_1e.py: v, on the wall at the enclosure point.
    Q6_V: f64 = 2.3703703703703703704;
    /// goldens/generate_1e.py: P_s.
    Q6_P_S: f64 = 1.6666666666666666667;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    Q6_Y: f64 = 64.221606267381293639;
    /// goldens/generate_1e.py: N_a, hours worked.
    Q6_N_A: f64 = 4.1285318314745117339;
    /// goldens/generate_1e.py: coverage kappa = q T/(N (1 + q)).
    Q6_KAPPA: f64 = 4.2857142857142857143;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    Q6_T_P: f64 = 2.7501390808226124895;
    /// goldens/generate_1e.py: psi, the renting share of exiters at the enclosure tie on the wall.
    Q6_SHARE: f64 = 0.27859473726423449519;
    /// goldens/generate_1e.py: f at the wall's enclosure point, the type on its floor.
    Q6_F_BELOW: f64 = 0.11675118739341279437;
    /// goldens/generate_1e.py: f at the wall's enclosure point, the type renting.
    Q6_F_ABOVE: f64 = -0.30232057447795227353;

    // K: the commons, G1's economy with exit (0.5, 0, 0.1) (docs/unit-1e.md section 3.3)
    /// goldens/generate_1e.py: x*.
    K1_X_STAR: f64 = 0.9205384426937588319;
    /// goldens/generate_1e.py: v, the pool's wage.
    K1_V: f64 = 0.57346088900036187046;
    /// goldens/generate_1e.py: P_s.
    K1_P_S: f64 = 1.365887351684820653;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    K1_Y: f64 = 7.6988590039587485682;
    /// goldens/generate_1e.py: N_a, hours worked.
    K1_N_A: f64 = 0.89940595044089532838;
    /// goldens/generate_1e.py: N_a/N.
    K1_PARTICIPATION: f64 = 0.2248514876102238321;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    K1_INCOME: f64 = 10.515774135912051245;
    /// goldens/generate_1e.py: q.
    K1_Q: f64 = 2.7330816312595874696;
    /// goldens/generate_1e.py: kappa.
    K1_KAPPA: f64 = 1.8303119923588519609;
    /// goldens/generate_1e.py: p_g.
    K1_P_G: f64 = 0.36588735168482065299;
    /// goldens/generate_1e.py: r_o, the plot rent.
    K1_PLOT_RENT: f64 = 0.0;
    /// goldens/generate_1e.py: the commons occupied.
    K1_T_OC: f64 = 0.31005940495591046716;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    K1_T_P: f64 = 0.0;
    /// goldens/generate_1e.py: the market's land.
    K1_T_M: f64 = 10.0;
    /// goldens/generate_1e.py: x*.
    K2_X_STAR: f64 = 0.90729619707291098295;
    /// goldens/generate_1e.py: v, the pool's wage.
    K2_V: f64 = 0.56651394336138253463;
    /// goldens/generate_1e.py: P_s.
    K2_P_S: f64 = 1.3650328667538438818;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    K2_Y: f64 = 7.7408494701591972669;
    /// goldens/generate_1e.py: N_a, hours worked.
    K2_N_A: f64 = 1.0;
    /// goldens/generate_1e.py: N_a/N.
    K2_PARTICIPATION: f64 = 0.25;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    K2_INCOME: f64 = 10.566513943361382535;
    /// goldens/generate_1e.py: q.
    K2_Q: f64 = 2.7394793485112111898;
    /// goldens/generate_1e.py: kappa.
    K2_KAPPA: f64 = 1.8314577332816884667;
    /// goldens/generate_1e.py: p_g.
    K2_P_G: f64 = 0.36503286675384388182;
    /// goldens/generate_1e.py: r_o, the plot rent.
    K2_PLOT_RENT: f64 = 0.43259131873953323891;
    /// goldens/generate_1e.py: the commons occupied.
    K2_T_OC: f64 = 0.3;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    K2_T_P: f64 = 0.0;
    /// goldens/generate_1e.py: the market's land.
    K2_T_M: f64 = 10.0;
    /// goldens/generate_1e.py: x*.
    K3_X_STAR: f64 = 0.88563381350455919867;
    /// goldens/generate_1e.py: v, the pool's wage.
    K3_V: f64 = 0.55517399235015002504;
    /// goldens/generate_1e.py: P_s.
    K3_P_S: f64 = 1.3634532149772006247;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    K3_Y: f64 = 7.5856235387047286096;
    /// goldens/generate_1e.py: N_a, hours worked.
    K3_N_A: f64 = 1.1335047027580957161;
    /// goldens/generate_1e.py: N_a/N.
    K3_PARTICIPATION: f64 = 0.28337617568952392902;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    K3_INCOME: f64 = 10.342642801453691681;
    /// goldens/generate_1e.py: q.
    K3_Q: f64 = 2.7513857596849979067;
    /// goldens/generate_1e.py: kappa.
    K3_KAPPA: f64 = 1.8335795996064334847;
    /// goldens/generate_1e.py: p_g.
    K3_P_G: f64 = 0.36345321497720062473;
    /// goldens/generate_1e.py: r_o, the plot rent.
    K3_PLOT_RENT: f64 = 1.0;
    /// goldens/generate_1e.py: the commons occupied.
    K3_T_OC: f64 = 0.0;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    K3_T_P: f64 = 0.28664952972419042839;
    /// goldens/generate_1e.py: the market's land.
    K3_T_M: f64 = 9.7133504702758095716;
    /// goldens/generate_1e.py: x*.
    K4_X_STAR: f64 = 0.89830323382720585152;
    /// goldens/generate_1e.py: v, the pool's wage.
    K4_V: f64 = 0.56180260690877409395;
    /// goldens/generate_1e.py: P_s.
    K4_P_S: f64 = 1.3644044502663948906;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    K4_Y: f64 = 8.3244976566862513306;
    /// goldens/generate_1e.py: N_a, hours worked.
    K4_N_A: f64 = 1.1453288958098360554;
    /// goldens/generate_1e.py: N_a/N.
    K4_PARTICIPATION: f64 = 0.28633222395245901386;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    K4_INCOME: f64 = 11.357981649014897211;
    /// goldens/generate_1e.py: q.
    K4_Q: f64 = 2.7442035882628716126;
    /// goldens/generate_1e.py: kappa.
    K4_KAPPA: f64 = 2.01553139134299418;
    /// goldens/generate_1e.py: p_g.
    K4_P_G: f64 = 0.36440445026639489056;
    /// goldens/generate_1e.py: r_o, the plot rent.
    K4_PLOT_RENT: f64 = 1.0;
    /// goldens/generate_1e.py: the commons occupied.
    K4_T_OC: f64 = 0.0;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    K4_T_P: f64 = 0.28546711041901639446;
    /// goldens/generate_1e.py: the market's land.
    K4_T_M: f64 = 10.714532889580983606;
    /// goldens/generate_1e.py: the share of WASTE's services occupied.
    K1_WASTE_USED: f64 = 0.31005940495591046716;
    /// goldens/generate_1e.py: e = p_g s0 - r_o h at the shadow rent.
    K2_EXIT_VALUE: f64 = 0.13925730150296861702;
    /// goldens/generate_1e.py: s(q) = s0 - q h at the market rent.
    K3_EXIT_GOODS: f64 = 0.22486142403150020933;

    // D: a dead exit (1, 0, 1), q_enc = 1 below q: G1 (docs/unit-1e.md section 3.3)
    /// goldens/generate_1e.py: q at G1, above q_enc = 1: every exiter on the floor.
    D_Q: f64 = 2.7656715745563858098;

    // I: idle land at zero rent, prices per unit of the pool's wage (docs/unit-1e.md section 4.6)
    /// goldens/generate_1e.py: Y.
    I1_Y: f64 = 5.8333333333333333333;
    /// goldens/generate_1e.py: the market's land in use.
    I1_T_M: f64 = 7.8333333333333333333;
    /// goldens/generate_1e.py: T_idle.
    I1_IDLE: f64 = 2.1666666666666666667;
    /// goldens/generate_1e.py: N_a.
    I1_N_A: f64 = 0.25;
    /// goldens/generate_1e.py: P_s in pool wages.
    I1_P_S: f64 = 0.042857142857142857143;
    /// goldens/generate_1e.py: v/P_s, 1d's ceiling.
    I1_REAL_WAGE: f64 = 23.333333333333333333;
    /// goldens/generate_1e.py: f_inf, 1d's excess.
    I1_F_END: f64 = 0.069148936170212765957;
    /// goldens/generate_1e.py: Y.
    I2_Y: f64 = 2.7997929961450740717;
    /// goldens/generate_1e.py: the market's land in use.
    I2_T_M: f64 = 3.7597220233948137534;
    /// goldens/generate_1e.py: T_idle.
    I2_IDLE: f64 = 6.2402779766051862466;
    /// goldens/generate_1e.py: N_a.
    I2_N_A: f64 = 1.4398935408746095226;
    /// goldens/generate_1e.py: P_s in pool wages.
    I2_P_S: f64 = 0.51428571428571428571;
    /// goldens/generate_1e.py: v/P_s, 1d's ceiling.
    I2_REAL_WAGE: f64 = 1.9444444444444444444;
    /// goldens/generate_1e.py: f_inf, 1d's excess.
    I2_F_END: f64 = 2.3898936931679436689;
    /// goldens/generate_1e.py: Y = 25/6: the trained's reserved demand 0.12 Y = N_T.
    I3_Y: f64 = 4.1666666666666666667;
    /// goldens/generate_1e.py: T_m = 20/3.
    I3_T_M: f64 = 6.6666666666666666667;
    /// goldens/generate_1e.py: kappa_T, the trained's real wage per support basket.
    I3_TRAINED_CLEARING: f64 = 6.5281716214787166668;
    /// goldens/generate_1e.py: the trained's wage, in pool wages.
    I3_TRAINED_WAGE: f64 = 42.473262323990767457;
    /// goldens/generate_1e.py: P_s.
    I3_P_S: f64 = 5.4217914788788920949;
    /// goldens/generate_1e.py: the entrant's hours, 65/48.
    I3_ENTRANT_HOURS: f64 = 1.3541666666666666667;
    /// goldens/generate_1e.py: participation.
    I3_PARTICIPATION: f64 = 0.21813725490196078431;
    /// goldens/generate_1e.py: Y.
    I4_Y: f64 = 1.7485871603090923406;
    /// goldens/generate_1e.py: T_m.
    I4_T_M: f64 = 2.3481027581293525717;
    /// goldens/generate_1e.py: free plots on idle land.
    I4_T_P: f64 = 1.5503633016348048267;
    /// goldens/generate_1e.py: N_a.
    I4_N_A: f64 = 0.89927339673039034661;
    /// goldens/generate_1e.py: participation.
    I4_PARTICIPATION: f64 = 0.22481834918259758665;
    /// goldens/generate_1e.py: FIELDS' share of its 7.5 services used; HEATH idles.
    I4_FIELDS_USED: f64 = 0.51979547463522098645;
    /// goldens/generate_1e.py: e = p_g s0 in pool wages, the plot free.
    I4_EXIT_VALUE: f64 = 0.25714285714285714286;
    /// goldens/generate_1e.py: NoMarket: I4 with exit (2, 0, 0.5), f at the end of the wall.
    I5_F_END: f64 = 3.0638297872340425532;

    // L: an exit good made of land alone, free at r = 0 (docs/unit-1e.md section 12 item 16)
    /// goldens/generate_1e.py: v, on the wall.
    L1_V: f64 = 10.086350438305713002;
    /// goldens/generate_1e.py: P_s.
    L1_P_S: f64 = 6.5301230825572238297;
    /// goldens/generate_1e.py: Y.
    L1_Y: f64 = 2.3972602739726027397;
    /// goldens/generate_1e.py: N_a.
    L1_N_A: f64 = 1.2328767123287671233;
    /// goldens/generate_1e.py: enclosed land in rented plots.
    L1_T_P: f64 = 6.7808219178082191781;
    /// goldens/generate_1e.py: f_inf with the plots on idle land: the wall's limit.
    L1_F_END: f64 = -0.17612887491407710185;

    // T: two priced types share a commons (docs/unit-1e.md section 3.3)
    /// goldens/generate_1e.py: x*.
    T_X_STAR: f64 = 0.86506676292016438128;
    /// goldens/generate_1e.py: v.
    T_V: f64 = 0.54443518165663000746;
    /// goldens/generate_1e.py: P_s.
    T_P_S: f64 = 1.361745269288688487;
    /// goldens/generate_1e.py: Y.
    T_Y: f64 = 7.8745487927760029095;
    /// goldens/generate_1e.py: the pool's efficiency hours.
    T_N_POOL: f64 = 1.3282197600553773998;
    /// goldens/generate_1e.py: the entrant's hours.
    T_ENTRANT_HOURS: f64 = 0.9858901199723113001;
    /// goldens/generate_1e.py: the trained's hours.
    T_TRAINED_HOURS: f64 = 0.22821976005537739981;
    /// goldens/generate_1e.py: the entrant's exiters.
    T_ENTRANT_EXITERS: f64 = 3.0141098800276886999;
    /// goldens/generate_1e.py: the trained's exiters.
    T_TRAINED_EXITERS: f64 = 0.77178023994462260019;
    /// goldens/generate_1e.py: r_o, one shadow rent for both.
    T_PLOT_RENT: f64 = 0.5283717704572644768;

    // F: 1c's M4 with a priced exit in food (0.1, 0, 0.05), rho 0.04 (docs/unit-1e.md section 3.3)
    /// goldens/generate_1e.py: x*.
    F1_X_STAR: f64 = 0.92359857105587254217;
    /// goldens/generate_1e.py: v, the pool's wage.
    F1_V: f64 = 0.14300589383721726198;
    /// goldens/generate_1e.py: P_s.
    F1_P_S: f64 = 1.5513223353284310685;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    F1_Y: f64 = 6.4987458559929469679;
    /// goldens/generate_1e.py: N_a, hours worked.
    F1_N_A: f64 = 0.18404732476679986349;
    /// goldens/generate_1e.py: N_a/N.
    F1_PARTICIPATION: f64 = 0.046011831191699965874;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    F1_INCOME: f64 = 10.081649598024942279;
    /// goldens/generate_1e.py: p_food, through the chain.
    F1_P_FOOD: f64 = 0.66813076665541326948;
    /// goldens/generate_1e.py: the commons occupied.
    F1_T_OC: f64 = 0.19079763376166000683;
    /// goldens/generate_1e.py: rented plots.
    F1_T_P: f64 = 0.0;
    /// goldens/generate_1e.py: x*.
    F2_X_STAR: f64 = 0.8404787252281769818;
    /// goldens/generate_1e.py: v, the pool's wage.
    F2_V: f64 = 0.13267354180711575492;
    /// goldens/generate_1e.py: P_s.
    F2_P_S: f64 = 1.5509189905174605692;
    /// goldens/generate_1e.py: Y = T_m/B^q.
    F2_Y: f64 = 6.3866171288999906867;
    /// goldens/generate_1e.py: N_a, hours worked.
    F2_N_A: f64 = 0.2852262490184478048;
    /// goldens/generate_1e.py: N_a/N.
    F2_PARTICIPATION: f64 = 0.071306562254611951199;
    /// goldens/generate_1e.py: I = v N_a + r T_m + interest.
    F2_INCOME: f64 = 9.9051257903750959014;
    /// goldens/generate_1e.py: p_food, through the chain.
    F2_P_FOOD: f64 = 0.66802617267694203727;
    /// goldens/generate_1e.py: the commons occupied.
    F2_T_OC: f64 = 0.0;
    /// goldens/generate_1e.py: rented plots.
    F2_T_P: f64 = 0.18573868754907760976;

    // M: three equilibria, two inside one region of the line (docs/unit-1e.md section 3.3)
    /// goldens/generate_1e.py: the all-human corner's equilibrium wage.
    M_V_CORNER: f64 = 0.2029756936648907802;
    /// goldens/generate_1e.py: the lower equilibrium on the line.
    M_X_LOW: f64 = 0.0044736729880859003628;
    /// goldens/generate_1e.py: its wage.
    M_V_LOW: f64 = 0.47752981945473418015;
    /// goldens/generate_1e.py: the upper equilibrium on the line.
    M_X_HIGH: f64 = 0.36831293358199533671;
    /// goldens/generate_1e.py: its wage.
    M_V_HIGH: f64 = 1.332677398275829529;
    /// goldens/generate_1e.py: f on the line at 1.
    M_F_LINE_1: f64 = -5.4802545807212104398;
    /// goldens/generate_1e.py: f on the line at 0, -2/31: the line's ends have the same sign.
    M_F_LINE_0: f64 = -0.064516129032258064516;

    // A: the wrong units of docs/unit-1e.md section 3.3
    /// goldens/generate_1e.py: x* with the market rent charged on commons plots (K1).
    A1_X_STAR: f64 = 0.88948157717185110772;
    /// goldens/generate_1e.py: its v.
    A1_V: f64 = 0.55718604886354229323;
    /// goldens/generate_1e.py: its participation.
    A1_PARTICIPATION: f64 = 0.28427070795822870941;
    /// goldens/generate_1e.py: Y with rented plots left in production (K3).
    A2_Y: f64 = 7.7972976856809805896;
    /// goldens/generate_1e.py: x* of Q1 with rented plots left in production.
    A3_X_STAR: f64 = 0.81354993934237555025;
    /// goldens/generate_1e.py: its Y.
    A3_Y: f64 = 62.086713726091331904;
    /// goldens/generate_1e.py: Q3 with q = r/P_s: the wall, 3/224.
    A4_PARTICIPATION: f64 = 0.013392857142857142857;
    /// goldens/generate_1e.py: its Y.
    A4_Y: f64 = 12.5;
}
