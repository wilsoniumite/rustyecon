//! The golden numbers of unit 1g, one constant each.
//!
//! Every constant equals its line in `goldens/goldens_1g.txt` rounded to 20 significant
//! digits (half up), and `h9_goldens_file::constants_match_goldens_1g_txt` enforces that.
//! `goldens_1g.txt` is written by `goldens/generate_1g.py`, which solves each instance with
//! mpmath at 70 digits from the equations of docs/unit-1g.md §4. Each constant's doc comment
//! is the generator's note.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE_1G` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE_1G: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // D-G10: productivity and the chain to land on the per-period recipes A^op + Delta A^I (docs/unit-1g.md section 2.1); spectral radii by mp.eig
    /// goldens/generate_1g.py: A0 as one type: spectral radius of A^op + A^I (1c's rule).
    A0_RADIUS_PHYSICAL: f64 = 148.2131073181570876;
    /// goldens/generate_1g.py: A0 as one type: spectral radius of A^op + Delta A^I (D-G10's).
    A0_RADIUS_PER_PERIOD: f64 = 0.3;
    /// goldens/generate_1g.py: A0's chain (fodder, the horse, horse-days): spectral radius of A^op + A^I (1c's rule).
    A0_CHAIN_RADIUS_PHYSICAL: f64 = 12.174280566758640609;
    /// goldens/generate_1g.py: A0's chain (fodder, the horse, horse-days): spectral radius of A^op + Delta A^I (D-G10's).
    A0_CHAIN_RADIUS_PER_PERIOD: f64 = 0.54772255750516611346;
    /// goldens/generate_1g.py: CHAIN's horse at weekly periods: spectral radius of A^op + A^I (1c's rule).
    HORSE_RADIUS_PHYSICAL: f64 = 1.5008632641098051053;
    /// goldens/generate_1g.py: CHAIN's horse at weekly periods: spectral radius of A^op + Delta A^I (D-G10's).
    HORSE_RADIUS_PER_PERIOD: f64 = 0.23968076641154208475;
    /// goldens/generate_1g.py: S1's chain: spectral radius of A^op + A^I (1c's rule).
    S1_RADIUS_PHYSICAL: f64 = 0.30575266521078003997;
    /// goldens/generate_1g.py: S1's chain: spectral radius of A^op + Delta A^I (D-G10's).
    S1_RADIUS_PER_PERIOD: f64 = 0.22360679774997896964;
    /// goldens/generate_1g.py: A0's delta per tick, 1 - 0.9^(1/52) (Clock::fraction).
    A0_DELTA: f64 = 0.00202411247850039517;
    /// goldens/generate_1g.py: A0R's rho per tick, 1.05^(1/52) - 1 (Clock::compound).
    A0_RHO_TICK: f64 = 0.00093871270311172379084;
    /// goldens/generate_1g.py: the horse's delta per tick, 1 - 0.92^(1/52).
    HORSE_DELTA: f64 = 0.0016022075724025086666;
    /// goldens/generate_1g.py: the plant's delta per tick, 1 - 0.9^(1/52).
    PLANT_DELTA: f64 = 0.00202411247850039517;
    // S1: the steam chain (coal, iron, the engine good, engine-hours) calibrated to 1c's M3, rho 0.05 (docs/unit-1g.md section 3.3; ORACLE-GOODS section 1.6)
    /// goldens/generate_1g.py: S1's solved coefficient lab_op (a fraction: 855257/7735400).
    S1_LAB_OP: f64 = 0.11056403030224681335;
    /// goldens/generate_1g.py: S1's solved coefficient lab_I (a fraction: 14560827/37903460).
    S1_LAB_I: f64 = 0.38415561534487880526;
    /// goldens/generate_1g.py: S1's solved coefficient seams (a fraction: 14836821/15470800).
    S1_SEAMS: f64 = 0.95902092975153191819;
    /// goldens/generate_1g.py: S1's solved coefficient site (a fraction: 186348663/3032276800).
    S1_SITE: f64 = 0.061455030424663078252;
    /// goldens/generate_1g.py: S1: x*.
    S1_X_STAR: f64 = 0.91057468799335480604;
    /// goldens/generate_1g.py: S1: 1 - x*.
    S1_ONE_MINUS_X_STAR: f64 = 0.089425312006645193958;
    /// goldens/generate_1g.py: S1: v = w/r.
    S1_V: f64 = 5.3088781572501366285;
    /// goldens/generate_1g.py: S1: P_s.
    S1_P_S: f64 = 4.7485791815332800258;
    /// goldens/generate_1g.py: S1: Y.
    S1_Y: f64 = 5.8234637271311258276;
    /// goldens/generate_1g.py: S1: N_a.
    S1_N_A: f64 = 3.001875717907908079;
    /// goldens/generate_1g.py: S1: I = Y P_s.
    S1_INCOME: f64 = 27.653178619069065849;
    /// goldens/generate_1g.py: S1: interest = sum (u - delta) V X.
    S1_INTEREST: f64 = 1.7165861894881998391;
    /// goldens/generate_1g.py: S1: coal's price.
    S1_COAL_PRICE: f64 = 2.2089779528867596331;
    /// goldens/generate_1g.py: S1: coal made a period.
    S1_COAL_OUTPUT: f64 = 4.2763280346654769631;
    /// goldens/generate_1g.py: S1: iron's price.
    S1_IRON_PRICE: f64 = 3.8589280550684481308;
    /// goldens/generate_1g.py: S1: iron made a period.
    S1_IRON_OUTPUT: f64 = 0.10558834653495004847;
    /// goldens/generate_1g.py: S1: the engine good's price, per unit of stock.
    S1_ENGINE_PRICE: f64 = 2.4867831912209203928;
    /// goldens/generate_1g.py: S1: units of the engine good made a period, delta X/kappa.
    S1_ENGINE_OUTPUT: f64 = 1.0558834653495004847;
    /// goldens/generate_1g.py: S1: units of the engine good installed, X/kappa.
    S1_ENGINE_STOCK: f64 = 10.558834653495004847;
    /// goldens/generate_1g.py: S1: an hour's price p = O + uV.
    S1_ENGINE_HOURS_PRICE: f64 = 1.8818139168520038924;
    /// goldens/generate_1g.py: S1: an hour's operating cost O.
    S1_ENGINE_HOURS_OPERATING: f64 = 1.4705621466038441825;
    /// goldens/generate_1g.py: S1: V per unit of capacity, the good's price/kappa.
    S1_ENGINE_HOURS_BUILD: f64 = 2.4867831912209203928;
    /// goldens/generate_1g.py: S1: hours X a period.
    S1_ENGINE_HOURS_HOURS: f64 = 10.558834653495004847;
    /// goldens/generate_1g.py: S1 folded to one type: operating: engine-hours per engine-hour.
    S1_FOLD_A: f64 = 0.04;
    /// goldens/generate_1g.py: S1 folded to one type: operating: labour.
    S1_FOLD_LAMBDA: f64 = 0.19056403030224681335;
    /// goldens/generate_1g.py: S1 folded to one type: operating: land.
    S1_FOLD_B: f64 = 0.38360837190061276728;
    /// goldens/generate_1g.py: S1 folded to one type: build: engine-hours per unit of capacity.
    S1_FOLD_A_I: f64 = 0.005;
    /// goldens/generate_1g.py: S1 folded to one type: build: labour.
    S1_FOLD_LAMBDA_I: f64 = 0.44415561534487880526;
    /// goldens/generate_1g.py: S1 folded to one type: build: land.
    S1_FOLD_B_I: f64 = 0.11940607691223967416;
    // S1Z: S1 at rho 0, whose five numbers are 1c's M3z's
    /// goldens/generate_1g.py: S1Z: coal's price.
    S1Z_COAL_PRICE: f64 = 1.9034918179819222748;
    /// goldens/generate_1g.py: S1Z: coal made a period.
    S1Z_COAL_OUTPUT: f64 = 4.3918298730598399373;
    /// goldens/generate_1g.py: S1Z: iron's price.
    S1Z_IRON_PRICE: f64 = 3.0635219782313760227;
    /// goldens/generate_1g.py: S1Z: iron made a period.
    S1Z_IRON_OUTPUT: f64 = 0.10844024377925530709;
    /// goldens/generate_1g.py: S1Z: the engine good's price, per unit of stock.
    S1Z_ENGINE_PRICE: f64 = 1.9134773758781068611;
    /// goldens/generate_1g.py: S1Z: units of the engine good made a period, delta X/kappa.
    S1Z_ENGINE_OUTPUT: f64 = 1.0844024377925530709;
    /// goldens/generate_1g.py: S1Z: units of the engine good installed, X/kappa.
    S1Z_ENGINE_STOCK: f64 = 10.844024377925530709;
    /// goldens/generate_1g.py: S1Z: an hour's price p = O + uV.
    S1Z_ENGINE_HOURS_PRICE: f64 = 1.3976046053422440254;
    /// goldens/generate_1g.py: S1Z: an hour's operating cost O.
    S1Z_ENGINE_HOURS_OPERATING: f64 = 1.2062568677544333393;
    /// goldens/generate_1g.py: S1Z: V per unit of capacity, the good's price/kappa.
    S1Z_ENGINE_HOURS_BUILD: f64 = 1.9134773758781068611;
    /// goldens/generate_1g.py: S1Z: hours X a period.
    S1Z_ENGINE_HOURS_HOURS: f64 = 10.844024377925530709;
    // S2, horse and engine, the engine at the margin (docs/unit-1g.md section 3.3; ORACLE-GOODS section 1.6), rho 0.05
    /// goldens/generate_1g.py: S2: gamma at the switch horse -> engine, 1c section 4.3.
    S2_SWITCH_GAMMA: f64 = 1.0422282165706983922;
    /// goldens/generate_1g.py: S2: x at the switch.
    S2_SWITCH_X: f64 = 0.36148547771379892814;
    /// goldens/generate_1g.py: S2: x*.
    S2_X_STAR: f64 = 0.66836997221771948458;
    /// goldens/generate_1g.py: S2: 1 - x*.
    S2_ONE_MINUS_X_STAR: f64 = 0.33163002778228051542;
    /// goldens/generate_1g.py: S2: v = w/r.
    S2_V: f64 = 0.71282600394783768684;
    /// goldens/generate_1g.py: S2: P_s.
    S2_P_S: f64 = 1.5538804837102414387;
    /// goldens/generate_1g.py: S2: Y.
    S2_Y: f64 = 8.2493376387291514382;
    /// goldens/generate_1g.py: S2: N_a.
    S2_N_A: f64 = 3.7757255126562787945;
    /// goldens/generate_1g.py: S2: I = Y P_s.
    S2_INCOME: f64 = 12.818484760357554776;
    /// goldens/generate_1g.py: S2: interest = sum (u - delta) V X.
    S2_INTEREST: f64 = 0.12704943116687871426;
    /// goldens/generate_1g.py: S2: coal's price.
    S2_COAL_PRICE: f64 = 1.1632593359035336671;
    /// goldens/generate_1g.py: S2: coal made a period.
    S2_COAL_OUTPUT: f64 = 1.7924916834479586474;
    /// goldens/generate_1g.py: S2: iron's price.
    S2_IRON_PRICE: f64 = 1.038042669925685677;
    /// goldens/generate_1g.py: S2: iron made a period.
    S2_IRON_OUTPUT: f64 = 0.044259053912295275244;
    /// goldens/generate_1g.py: S2: fodder's price.
    S2_FODDER_PRICE: f64 = 0.71384780118435130605;
    /// goldens/generate_1g.py: S2: fodder made a period.
    S2_FODDER_OUTPUT: f64 = 0.0;
    /// goldens/generate_1g.py: S2: the horse good's price, per unit of stock.
    S2_HORSE_PRICE: f64 = 2.4969346082904591424;
    /// goldens/generate_1g.py: S2: units of the horse good made a period, delta X/kappa.
    S2_HORSE_OUTPUT: f64 = 0.0;
    /// goldens/generate_1g.py: S2: units of the horse good installed, X/kappa.
    S2_HORSE_STOCK: f64 = 0.0;
    /// goldens/generate_1g.py: S2: an hour's price p = O + uV.
    S2_HORSE_HOURS_PRICE: f64 = 0.55407549108050234304;
    /// goldens/generate_1g.py: S2: an hour's operating cost O.
    S2_HORSE_HOURS_OPERATING: f64 = 0.17830868071061078363;
    /// goldens/generate_1g.py: S2: V per unit of capacity, the good's price/kappa.
    S2_HORSE_HOURS_BUILD: f64 = 2.4969346082904591424;
    /// goldens/generate_1g.py: S2: hours X a period.
    S2_HORSE_HOURS_HOURS: f64 = 0.0;
    /// goldens/generate_1g.py: S2: the engine good's price, per unit of stock.
    S2_ENGINE_PRICE: f64 = 0.43909540959764424106;
    /// goldens/generate_1g.py: S2: units of the engine good made a period, delta X/kappa.
    S2_ENGINE_OUTPUT: f64 = 0.44259053912295275244;
    /// goldens/generate_1g.py: S2: units of the engine good installed, X/kappa.
    S2_ENGINE_STOCK: f64 = 4.4259053912295275244;
    /// goldens/generate_1g.py: S2: an hour's price p = O + uV.
    S2_ENGINE_HOURS_PRICE: f64 = 0.61673205362434211583;
    /// goldens/generate_1g.py: S2: an hour's operating cost O.
    S2_ENGINE_HOURS_OPERATING: f64 = 0.54411665026213169947;
    /// goldens/generate_1g.py: S2: V per unit of capacity, the good's price/kappa.
    S2_ENGINE_HOURS_BUILD: f64 = 0.43909540959764424106;
    /// goldens/generate_1g.py: S2: hours X a period.
    S2_ENGINE_HOURS_HOURS: f64 = 4.4259053912295275244;
    // S2H, horse and engine, the horse at the margin (docs/unit-1g.md section 3.3; ORACLE-GOODS section 1.6), rho 0.05
    /// goldens/generate_1g.py: S2H: gamma at the switch horse -> engine, 1c section 4.3.
    S2H_SWITCH_GAMMA: f64 = 1.1729205115633965451;
    /// goldens/generate_1g.py: S2H: x at the switch.
    S2H_SWITCH_X: f64 = 0.67292051156339654511;
    /// goldens/generate_1g.py: S2H: x*.
    S2H_X_STAR: f64 = 0.64555993278226703026;
    /// goldens/generate_1g.py: S2H: 1 - x*.
    S2H_ONE_MINUS_X_STAR: f64 = 0.35444006721773296974;
    /// goldens/generate_1g.py: S2H: v = w/r.
    S2H_V: f64 = 0.53194257753558310662;
    /// goldens/generate_1g.py: S2H: P_s.
    S2H_P_S: f64 = 1.4351838657278224612;
    /// goldens/generate_1g.py: S2H: Y.
    S2H_Y: f64 = 9.3540995095559854471;
    /// goldens/generate_1g.py: S2H: N_a.
    S2H_N_A: f64 = 5.0444935871939023676;
    /// goldens/generate_1g.py: S2H: I = Y P_s.
    S2H_INCOME: f64 = 13.424852694527287355;
    /// goldens/generate_1g.py: S2H: interest = sum (u - delta) V X.
    S2H_INTEREST: f64 = 0.74147177339364318409;
    /// goldens/generate_1g.py: S2H: coal's price.
    S2H_COAL_PRICE: f64 = 1.1221037614683954634;
    /// goldens/generate_1g.py: S2H: coal made a period.
    S2H_COAL_OUTPUT: f64 = 0.0;
    /// goldens/generate_1g.py: S2H: iron's price.
    S2H_IRON_PRICE: f64 = 0.92702316950198928501;
    /// goldens/generate_1g.py: S2H: iron made a period.
    S2H_IRON_OUTPUT: f64 = 0.0;
    /// goldens/generate_1g.py: S2H: fodder's price.
    S2H_FODDER_PRICE: f64 = 0.65958277326067493199;
    /// goldens/generate_1g.py: S2H: fodder made a period.
    S2H_FODDER_OUTPUT: f64 = 1.2918009808880291058;
    /// goldens/generate_1g.py: S2H: the horse good's price, per unit of stock.
    S2H_HORSE_PRICE: f64 = 2.1170794128247245239;
    /// goldens/generate_1g.py: S2H: units of the horse good made a period, delta X/kappa.
    S2H_HORSE_OUTPUT: f64 = 0.39747722488862434023;
    /// goldens/generate_1g.py: S2H: units of the horse good installed, X/kappa.
    S2H_HORSE_STOCK: f64 = 4.9684653111078042529;
    /// goldens/generate_1g.py: S2H: an hour's price p = O + uV.
    S2H_HORSE_HOURS_PRICE: f64 = 0.4643515911416637837;
    /// goldens/generate_1g.py: S2H: an hour's operating cost O.
    S2H_HORSE_HOURS_OPERATING: f64 = 0.14574966395640495919;
    /// goldens/generate_1g.py: S2H: V per unit of capacity, the good's price/kappa.
    S2H_HORSE_HOURS_BUILD: f64 = 2.1170794128247245239;
    /// goldens/generate_1g.py: S2H: hours X a period.
    S2H_HORSE_HOURS_HOURS: f64 = 4.9684653111078042529;
    /// goldens/generate_1g.py: S2H: the engine good's price, per unit of stock.
    S2H_ENGINE_PRICE: f64 = 0.35850607557618484007;
    /// goldens/generate_1g.py: S2H: units of the engine good made a period, delta X/kappa.
    S2H_ENGINE_OUTPUT: f64 = 0.0;
    /// goldens/generate_1g.py: S2H: units of the engine good installed, X/kappa.
    S2H_ENGINE_STOCK: f64 = 0.0;
    /// goldens/generate_1g.py: S2H: an hour's price p = O + uV.
    S2H_ENGINE_HOURS_PRICE: f64 = 0.5669431620974692389;
    /// goldens/generate_1g.py: S2H: an hour's operating cost O.
    S2H_ENGINE_HOURS_OPERATING: f64 = 0.50765521984905767098;
    /// goldens/generate_1g.py: S2H: V per unit of capacity, the good's price/kappa.
    S2H_ENGINE_HOURS_BUILD: f64 = 0.35850607557618484007;
    /// goldens/generate_1g.py: S2H: hours X a period.
    S2H_ENGINE_HOURS_HOURS: f64 = 0.0;
    // A0: Appendix B's flow machine as a durable good at 52 ticks a year, delta 10% a year, omega 1/2, rho 0 (docs/unit-1g.md section 3.3): Appendix B's equilibrium exactly
    /// goldens/generate_1g.py: A0: fodder's price.
    A0_FODDER_PRICE: f64 = 0.2;
    /// goldens/generate_1g.py: A0: fodder made a period.
    A0_FODDER_OUTPUT: f64 = 5.2984861875677308081;
    /// goldens/generate_1g.py: A0: the horse good's price, per unit of stock.
    A0_HORSE_PRICE: f64 = 202.67916438517006167;
    /// goldens/generate_1g.py: A0: units of the horse good made a period, delta X/kappa.
    A0_HORSE_OUTPUT: f64 = 0.010724732009417829295;
    /// goldens/generate_1g.py: A0: units of the horse good installed, X/kappa.
    A0_HORSE_STOCK: f64 = 5.2984861875677308081;
    /// goldens/generate_1g.py: A0: an hour's price p = O + uV.
    A0_HORSE_DAYS_PRICE: f64 = 0.61024542576405559489;
    /// goldens/generate_1g.py: A0: an hour's operating cost O.
    A0_HORSE_DAYS_OPERATING: f64 = 0.2;
    /// goldens/generate_1g.py: A0: V per unit of capacity, the good's price/kappa.
    A0_HORSE_DAYS_BUILD: f64 = 202.67916438517006167;
    /// goldens/generate_1g.py: A0: hours X a period.
    A0_HORSE_DAYS_HOURS: f64 = 5.2984861875677308081;
    /// goldens/generate_1g.py: A0 as one type: V, (p_m - omega b)/delta (AGENTS-GOODS section 1).
    A0_ONE_BUILD: f64 = 202.67916438517006167;
    /// goldens/generate_1g.py: A0 as one type: W = V X (J 1).
    A0_ONE_WEALTH: f64 = 1073.8927530025931252;
    // A0R: A0 at rho 5% a year, 1.05^(1/52) - 1 a tick: unit 1a's durable point, which 1a's a < 1 rejects (a/delta = 148)
    /// goldens/generate_1g.py: A0R: x*.
    A0R_X_STAR: f64 = 0.81738052839262692215;
    /// goldens/generate_1g.py: A0R: 1 - x*.
    A0R_ONE_MINUS_X_STAR: f64 = 0.18261947160737307785;
    /// goldens/generate_1g.py: A0R: v = w/r.
    A0R_V: f64 = 0.8442723553053997883;
    /// goldens/generate_1g.py: A0R: P_s.
    A0R_P_S: f64 = 1.5800425085223375615;
    /// goldens/generate_1g.py: A0R: Y.
    A0R_Y: f64 = 8.0248707640064784596;
    /// goldens/generate_1g.py: A0R: N_a.
    A0R_N_A: f64 = 1.7123888131395095847;
    /// goldens/generate_1g.py: A0R: I = Y P_s.
    A0R_INCOME: f64 = 12.67963693252836378;
    /// goldens/generate_1g.py: A0R: interest = sum (u - delta) V X.
    A0R_INTEREST: f64 = 1.2339143960604518983;
    /// goldens/generate_1g.py: A0R: fodder's price.
    A0R_FODDER_PRICE: f64 = 0.2;
    /// goldens/generate_1g.py: A0R: fodder made a period.
    A0R_FODDER_OUTPUT: f64 = 4.937823089983803851;
    /// goldens/generate_1g.py: A0R: the horse good's price, per unit of stock.
    A0R_HORSE_PRICE: f64 = 266.20536878386680905;
    /// goldens/generate_1g.py: A0R: units of the horse good made a period, delta X/kappa.
    A0R_HORSE_OUTPUT: f64 = 0.0099947093330635970171;
    /// goldens/generate_1g.py: A0R: units of the horse good installed, X/kappa.
    A0R_HORSE_STOCK: f64 = 4.937823089983803851;
    /// goldens/generate_1g.py: A0R: an hour's price p = O + uV.
    A0R_HORSE_DAYS_PRICE: f64 = 0.9887199701131812821;
    /// goldens/generate_1g.py: A0R: an hour's operating cost O.
    A0R_HORSE_DAYS_OPERATING: f64 = 0.2;
    /// goldens/generate_1g.py: A0R: V per unit of capacity, the good's price/kappa.
    A0R_HORSE_DAYS_BUILD: f64 = 266.20536878386680905;
    /// goldens/generate_1g.py: A0R: hours X a period.
    A0R_HORSE_DAYS_HOURS: f64 = 4.937823089983803851;
    // A0F: A0's durable machine in 1b's fork economy C3 at rho 0: 1b's C3 exactly
    /// goldens/generate_1g.py: A0F: the horse's V per unit of capacity.
    A0F_BUILD: f64 = 199.90388517634383393;
    // HORSE: CHAIN's horse at weekly periods (CHAIN section 1.3-1.4): fodder, the horse (head), horse-days; delta 8% a year, J 156 ticks, rho 0, the good and space, N 12, T 10, chi 1/20, gamma = 0.2 + 0.8x (docs/unit-1g.md section 3.3)
    /// goldens/generate_1g.py: HORSE: x*.
    HORSE_X_STAR: f64 = 0.62424404534133174456;
    /// goldens/generate_1g.py: HORSE: 1 - x*.
    HORSE_ONE_MINUS_X_STAR: f64 = 0.37575595465866825544;
    /// goldens/generate_1g.py: HORSE: v = w/r.
    HORSE_V: f64 = 0.019278964656802269677;
    /// goldens/generate_1g.py: HORSE: P_s.
    HORSE_P_S: f64 = 1.0149823159707379272;
    /// goldens/generate_1g.py: HORSE: Y.
    HORSE_Y: f64 = 9.9381651174203159807;
    /// goldens/generate_1g.py: HORSE: N_a.
    HORSE_N_A: f64 = 4.5158984898161836425;
    /// goldens/generate_1g.py: HORSE: I = Y P_s.
    HORSE_INCOME: f64 = 10.087061847378872949;
    /// goldens/generate_1g.py: HORSE: fodder's price.
    HORSE_FODDER_PRICE: f64 = 1.2093621032133959202;
    /// goldens/generate_1g.py: HORSE: fodder made a period.
    HORSE_FODDER_OUTPUT: f64 = 0.058927821578862533818;
    /// goldens/generate_1g.py: HORSE: the horse good's price, per unit of stock.
    HORSE_HORSE_PRICE: f64 = 13.060476118843212755;
    /// goldens/generate_1g.py: HORSE: units of the horse good made a period, delta X/kappa.
    HORSE_HORSE_OUTPUT: f64 = 0.00096902033360716181267;
    /// goldens/generate_1g.py: HORSE: units of the horse good installed, X/kappa.
    HORSE_HORSE_STOCK: f64 = 0.60480324166369828284;
    /// goldens/generate_1g.py: HORSE: an hour's price p = O + uV.
    HORSE_HORSE_DAYS_PRICE: f64 = 0.027565192979488881377;
    /// goldens/generate_1g.py: HORSE: an hour's operating cost O.
    HORSE_HORSE_DAYS_OPERATING: f64 = 0.023212669482235995163;
    /// goldens/generate_1g.py: HORSE: V per unit of capacity, the good's price/kappa.
    HORSE_HORSE_DAYS_BUILD: f64 = 2.716579032719388253;
    /// goldens/generate_1g.py: HORSE: hours X a period.
    HORSE_HORSE_DAYS_HOURS: f64 = 2.9077078926139340521;
    /// goldens/generate_1g.py: HORSE: labour per horse-day, all through (CHAIN's 0.268).
    HORSE_LAMBDA_TILDE: f64 = 0.28014880311915418251;
    /// goldens/generate_1g.py: HORSE: land per horse-day, all through (CHAIN's 0.020 + 0.001).
    HORSE_B_TILDE: f64 = 0.022164214105509250446;
    /// goldens/generate_1g.py: HORSE: wealth factor 1 + delta(J - 1) at rho 0.
    HORSE_OMEGA: f64 = 1.2483421737223888433;
    // P1: the markets probe's L2 (engine and power buying each other's service, flow) with a plant on both, theta 0.8, delta 10% a year a tick, J 1, rho 0, built from s1 bundles of the desk's own recipe: L2's flow economy exactly (docs/unit-1g.md section 4.5; CAPACITY.md)
    /// goldens/generate_1g.py: L2's flow economy: v (CAPACITY.md's oracle 0.42570).
    L2_V: f64 = 0.42570092667810366689;
    /// goldens/generate_1g.py: L2's flow economy: x*.
    L2_X_STAR: f64 = 0.91480039929112051662;
    /// goldens/generate_1g.py: s1 = theta^(theta/(1 - theta)) (1 - theta)/delta, bundles a plant unit.
    P1_S1: f64 = 40.472059171678095387;
    /// goldens/generate_1g.py: P1: engine's kappa, plant units per unit of capacity.
    P1_ENGINE_KAPPA: f64 = 2.44140625;
    /// goldens/generate_1g.py: P1: engine's plant K* = kappa X = theta^(-theta/(1-theta)) y.
    P1_ENGINE_STOCK: f64 = 6.1284434141399148904;
    /// goldens/generate_1g.py: P1: engine's plant price P_K = s1 c_f.
    P1_ENGINE_PLANT_PRICE: f64 = 36.978423737748777123;
    /// goldens/generate_1g.py: P1: engine's V = kappa P_K.
    P1_ENGINE_BUILD: f64 = 90.279354828488225398;
    /// goldens/generate_1g.py: P1: power's kappa, plant units per unit of capacity.
    P1_POWER_KAPPA: f64 = 2.44140625;
    /// goldens/generate_1g.py: P1: power's plant K* = kappa X = theta^(-theta/(1-theta)) y.
    P1_POWER_STOCK: f64 = 3.0642217070699574452;
    /// goldens/generate_1g.py: P1: power's plant price P_K = s1 c_f.
    P1_POWER_PLANT_PRICE: f64 = 30.048557048056264134;
    /// goldens/generate_1g.py: P1: power's V = kappa P_K.
    P1_POWER_BUILD: f64 = 73.360734980606113608;
    // P1S: P1 at s = 1 bundle a plant unit: the flow economy with every planted recipe times m = theta^-theta (1 - theta)^-(1 - theta) (delta s)^(1 - theta)
    /// goldens/generate_1g.py: P1S: m at s = 1.
    P1S_M: f64 = 0.47705553571010563622;
    /// goldens/generate_1g.py: P1S: x*.
    P1S_X_STAR: f64 = 0.96934545277918438509;
    /// goldens/generate_1g.py: P1S: 1 - x*.
    P1S_ONE_MINUS_X_STAR: f64 = 0.030654547220815614913;
    /// goldens/generate_1g.py: P1S: v = w/r.
    P1S_V: f64 = 0.13943139644608053975;
    /// goldens/generate_1g.py: P1S: P_s.
    P1S_P_S: f64 = 1.0857083078191900654;
    /// goldens/generate_1g.py: P1S: Y.
    P1S_Y: f64 = 9.272643130477510384;
    /// goldens/generate_1g.py: P1S: N_a.
    P1S_N_A: f64 = 0.48328915810603477253;
    /// goldens/generate_1g.py: P1S: I = Y P_s.
    P1S_INCOME: f64 = 10.067385682201975033;
    // P1R: P1 at rho 5% a year a tick: the bundle plant's ratio is still price-free, but the long run is no longer L2's (u > delta)
    /// goldens/generate_1g.py: P1R: x*.
    P1R_X_STAR: f64 = 0.90211329973015549186;
    /// goldens/generate_1g.py: P1R: 1 - x*.
    P1R_ONE_MINUS_X_STAR: f64 = 0.097886700269844508136;
    /// goldens/generate_1g.py: P1R: v = w/r.
    P1R_V: f64 = 0.48512753202383114607;
    /// goldens/generate_1g.py: P1R: P_s.
    P1R_P_S: f64 = 1.313789841790933087;
    /// goldens/generate_1g.py: P1R: Y.
    P1R_Y: f64 = 8.2501788495893315777;
    /// goldens/generate_1g.py: P1R: N_a.
    P1R_N_A: f64 = 1.2570762217781082843;
    /// goldens/generate_1g.py: P1R: I = Y P_s.
    P1R_INCOME: f64 = 10.839001165548870275;
    /// goldens/generate_1g.py: P1R: interest = sum (u - delta) V X.
    P1R_INTEREST: f64 = 0.22915888051181438419;
    /// goldens/generate_1g.py: P1R: engine's price p = O + uV.
    P1R_ENGINE_PRICE: f64 = 1.0526905907115564414;
    /// goldens/generate_1g.py: P1R: engine's plant K* = kappa X.
    P1R_ENGINE_STOCK: f64 = 4.430873263843629052;
    /// goldens/generate_1g.py: P1R: power's price p = O + uV.
    P1R_POWER_PRICE: f64 = 0.82393779246573642631;
    /// goldens/generate_1g.py: P1R: power's plant K* = kappa X.
    P1R_POWER_STOCK: f64 = 2.2393578682822313639;
    // P2: L2 with a plant built from labour 0.5 and land 0.5 a unit on both desks: the ratio moves with prices, a fixed point (CAPACITY.md check 2c: v 0.13869, zeta/kappa engine 0.3783/48.8149, power 0.3723/52.0700)
    /// goldens/generate_1g.py: P2: x*.
    P2_X_STAR: f64 = 0.98268463578459683797;
    /// goldens/generate_1g.py: P2: 1 - x*.
    P2_ONE_MINUS_X_STAR: f64 = 0.017315364215403162027;
    /// goldens/generate_1g.py: P2: v = w/r.
    P2_V: f64 = 0.13868993703845577967;
    /// goldens/generate_1g.py: P2: P_s.
    P2_P_S: f64 = 1.084365991612069664;
    /// goldens/generate_1g.py: P2: Y.
    P2_Y: f64 = 9.2835531155816371272;
    /// goldens/generate_1g.py: P2: N_a.
    P2_N_A: f64 = 0.48142843876615947226;
    /// goldens/generate_1g.py: P2: I = Y P_s.
    P2_INCOME: f64 = 10.066769279861000721;
    /// goldens/generate_1g.py: P2: engine's kappa/zeta at the fixed point.
    P2_ENGINE_RATIO: f64 = 129.03010594613441754;
    /// goldens/generate_1g.py: P2: engine's zeta, bundles per unit of service.
    P2_ENGINE_ZETA: f64 = 0.37832216752801874966;
    /// goldens/generate_1g.py: P2: engine's kappa, plant units per unit of capacity.
    P2_ENGINE_KAPPA: f64 = 48.814949357911473326;
    /// goldens/generate_1g.py: P2: engine's price p = O + uV.
    P2_ENGINE_PRICE: f64 = 0.28127619387050365633;
    /// goldens/generate_1g.py: P2: engine's plant K* = kappa X.
    P2_ENGINE_STOCK: f64 = 138.26089597279050522;
    /// goldens/generate_1g.py: P2: engine's plant price P_K = 0.5 v + 0.5.
    P2_ENGINE_PLANT_PRICE: f64 = 0.56934496851922788984;
    /// goldens/generate_1g.py: P2: power's kappa/zeta at the fixed point.
    P2_POWER_RATIO: f64 = 139.87335085107186726;
    /// goldens/generate_1g.py: P2: power's zeta, bundles per unit of service.
    P2_POWER_ZETA: f64 = 0.37226568401874200253;
    /// goldens/generate_1g.py: P2: power's kappa, plant units per unit of capacity.
    P2_POWER_KAPPA: f64 = 52.070048630567757493;
    /// goldens/generate_1g.py: P2: power's price p = O + uV.
    P2_POWER_PRICE: f64 = 0.30003237299445096851;
    /// goldens/generate_1g.py: P2: power's plant K* = kappa X.
    P2_POWER_STOCK: f64 = 27.897565162096776921;
    /// goldens/generate_1g.py: P2: power's plant price P_K = 0.5 v + 0.5.
    P2_POWER_PLANT_PRICE: f64 = 0.56934496851922788984;
}
