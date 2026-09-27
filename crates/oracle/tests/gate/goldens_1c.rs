//! The golden numbers of unit 1c, one constant each.
//!
//! Every constant equals its line in `goldens/goldens_1c.txt` rounded to 20 significant
//! digits (half up), and `m8_goldens_file::constants_match_goldens_1c_txt` enforces that.
//! `goldens_1c.txt` is written by `goldens/generate_1c.py`, which solves each instance with
//! mpmath at 70 digits from the equations of docs/unit-1c.md §4. Each constant's doc comment
//! is the generator's note; laborformal references are at 31b3482.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE_1C` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE_1C: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // M2: check_dynamics' machine (operating (0.5, 0.1, 0.2), build (0.1, 0.2, 0.02), rho 0.05, delta 0.1, J 3) as a price block at the targets' own margins (docs/unit-1c.md section 0.2)
    /// goldens/generate_1c.py: u = (rho + delta)(1 + rho)^(J - 1), check_dynamics U3.
    M2_U: f64 = 0.165375;
    /// goldens/generate_1c.py: omega = (1 + rho)^(J - 1) + delta sum (1 + rho)^i, L2.
    M2_OMEGA: f64 = 1.3075;
    /// goldens/generate_1c.py: lambda-tilde, price side: (lambda + u lambda_I)/(1 - a - u a_I).
    M2_LAMBDA_TILDE: f64 = 0.27525402694107609173;
    /// goldens/generate_1c.py: b-tilde, price side: (b + u b_I)/(1 - a - u a_I).
    M2_B_TILDE: f64 = 0.42052382552938438865;
    /// goldens/generate_1c.py: lambda-tilde^q = 0.12/0.49, clearing side.
    M2_LAMBDA_TILDE_Q: f64 = 0.24489795918367346939;
    /// goldens/generate_1c.py: b-tilde^q = 0.202/0.49, clearing side.
    M2_B_TILDE_Q: f64 = 0.41224489795918367347;
    /// goldens/generate_1c.py: x*, the target's double (an input).
    M2_SLOPED_X_STAR: f64 = 0.59624904823858182468;
    /// goldens/generate_1c.py: gamma* = 1 + 4x*.
    M2_SLOPED_GAMMA: f64 = 3.3849961929543272987;
    /// goldens/generate_1c.py: J(x*) = x* + 2x*^2.
    M2_SLOPED_J: f64 = 1.3072749032894111761;
    /// goldens/generate_1c.py: Den = 1 - a - lambda gamma* - u(a_I + lambda_I gamma*), R1.
    M2_SLOPED_DEN: f64 = 0.033004131622602894724;
    /// goldens/generate_1c.py: p_m = c/r = theta_c, R1.
    M2_SLOPED_P_M: f64 = 6.1600620893404983605;
    /// goldens/generate_1c.py: v = w/r = theta_w = gamma* theta_c, R2.
    M2_SLOPED_V: f64 = 20.851786720779866155;
    /// goldens/generate_1c.py: V = p_K/r = a_I theta_c + lambda_I theta_w + b_I, R3.
    M2_SLOPED_BUILD: f64 = 4.8063635530900230671;
    /// goldens/generate_1c.py: O = a p_m + lambda v + b.
    M2_SLOPED_OPERATING: f64 = 5.3652097167482357957;
    /// goldens/generate_1c.py: p_good = 1/r = v(1 - x*) + p_m J(x*).
    M2_SLOPED_P_GOOD: f64 = 16.471823306540339961;
    /// goldens/generate_1c.py: (1 - x*) + lambda-tilde^q J(x*), hours per unit of the good.
    M2_SLOPED_LABOR_PER_GOOD: f64 = 0.7238999076690290756;
    /// goldens/generate_1c.py: Y = N/labour per good, N = 1.
    M2_SLOPED_Y: f64 = 1.3814064477781441699;
    /// goldens/generate_1c.py: X/Y = J(x*)/(1 - a - delta a_I).
    M2_SLOPED_SERVICES_PER_GOOD: f64 = 2.6679079658967575023;
    /// goldens/generate_1c.py: X = Y J(x*)/(1 - a - delta a_I).
    M2_SLOPED_SERVICES: f64 = 3.6854652661684539795;
    /// goldens/generate_1c.py: c in the target's units (the good as numeraire).
    M2_SLOPED_TARGET_C: f64 = 0.37397572659090933614;
    /// goldens/generate_1c.py: w in the target's units (the good as numeraire).
    M2_SLOPED_TARGET_W: f64 = 1.2659064107675564896;
    /// goldens/generate_1c.py: r in the target's units (the good as numeraire).
    M2_SLOPED_TARGET_R: f64 = 0.060709733305780283912;
    /// goldens/generate_1c.py: Y in the target's units (the good as numeraire).
    M2_SLOPED_TARGET_Y: f64 = 1.3814064477781441699;
    /// goldens/generate_1c.py: X in the target's units (the good as numeraire).
    M2_SLOPED_TARGET_X: f64 = 3.6854652661684539795;
    /// goldens/generate_1c.py: pK in the target's units (the good as numeraire).
    M2_SLOPED_TARGET_PK: f64 = 0.29179304947871783721;
    /// goldens/generate_1c.py: the land-share closure's root at 70 digits (not 1c's closure).
    M2_SLOPED_LAND_SHARE_ROOT: f64 = 0.59624904823858193746;
    /// goldens/generate_1c.py: Den at gamma* = 3.
    M2_FLAT_DEN: f64 = 0.0842375;
    /// goldens/generate_1c.py: p_m = c/r at gamma* = 3.
    M2_FLAT_P_M: f64 = 2.4135034871642676955;
    /// goldens/generate_1c.py: v = w/r at gamma* = 3.
    M2_FLAT_V: f64 = 7.2405104614928030865;
    /// goldens/generate_1c.py: V = p_K/r at gamma* = 3.
    M2_FLAT_BUILD: f64 = 1.7094524410149873869;
    /// goldens/generate_1c.py: O at gamma* = 3.
    M2_FLAT_OPERATING: f64 = 2.1308027897314141564;
    /// goldens/generate_1c.py: m, the target's machine-task share (an input).
    M2_FLAT_M: f64 = 0.35709260151683958062;
    /// goldens/generate_1c.py: Y = N/((1 - m) + lambda-tilde^q gamma-bar m), N = 1.
    M2_FLAT_Y: f64 = 1.1046536171646545931;
    /// goldens/generate_1c.py: X = Y gamma-bar m/(1 - a - delta a_I).
    M2_FLAT_SERVICES: f64 = 2.4150834730304906089;
    /// goldens/generate_1c.py: c in the target's units (w = 1).
    M2_FLAT_TARGET_C: f64 = 0.33333333333333333333;
    /// goldens/generate_1c.py: r in the target's units (w = 1).
    M2_FLAT_TARGET_R: f64 = 0.1381118092872455107;
    /// goldens/generate_1c.py: m from the land-share closure at 70 digits (not 1c's closure).
    M2_FLAT_LAND_SHARE_M: f64 = 0.35709260151683956143;

    // M3: M2's machine in Appendix B's closure, G1's household, gamma = 1 + 2x, rho 0.05 (docs/unit-1c.md section 3.3)
    /// goldens/generate_1c.py: x*.
    M3_X_STAR: f64 = 0.91057468799335480604;
    /// goldens/generate_1c.py: 1 - x*.
    M3_ONE_MINUS_X_STAR: f64 = 0.089425312006645193958;
    /// goldens/generate_1c.py: gamma(x*).
    M3_GAMMA_STAR: f64 = 2.8211493759867096121;
    /// goldens/generate_1c.py: v = w/r.
    M3_V: f64 = 5.3088781572501366285;
    /// goldens/generate_1c.py: u.
    M3_U: f64 = 0.165375;
    /// goldens/generate_1c.py: omega.
    M3_OMEGA: f64 = 1.3075;
    /// goldens/generate_1c.py: p_m = O + u V.
    M3_P_M: f64 = 1.8818139168520038924;
    /// goldens/generate_1c.py: O, the operating cost.
    M3_OPERATING: f64 = 1.6717947741510156091;
    /// goldens/generate_1c.py: V, the build cost.
    M3_BUILD: f64 = 1.2699570231352277149;
    /// goldens/generate_1c.py: p of the good.
    M3_P_GOOD: f64 = 3.7485791815332800258;
    /// goldens/generate_1c.py: P_s.
    M3_P_S: f64 = 4.7485791815332800258;
    /// goldens/generate_1c.py: Y.
    M3_Y: f64 = 5.8234637271311258276;
    /// goldens/generate_1c.py: X = K.
    M3_SERVICES: f64 = 20.675922142915218675;
    /// goldens/generate_1c.py: N_a.
    M3_N_A: f64 = 3.001875717907908079;
    /// goldens/generate_1c.py: final hours.
    M3_FINAL_HOURS: f64 = 0.52076506075808183802;
    /// goldens/generate_1c.py: machine hours (lambda + delta lambda_I) X.
    M3_MACHINE_HOURS: f64 = 2.481110657149826241;
    /// goldens/generate_1c.py: I = v N_a + T + interest.
    M3_INCOME: f64 = 27.653178619069065849;
    /// goldens/generate_1c.py: lambda-tilde, price side.
    M3_LAMBDA_TILDE: f64 = 0.27525402694107609173;
    /// goldens/generate_1c.py: lambda-tilde^q, clearing side.
    M3_LAMBDA_TILDE_Q: f64 = 0.24489795918367346939;
    /// goldens/generate_1c.py: b-tilde, price side.
    M3_B_TILDE: f64 = 0.42052382552938438865;
    /// goldens/generate_1c.py: b-tilde^q, clearing side.
    M3_B_TILDE_Q: f64 = 0.41224489795918367347;
    /// goldens/generate_1c.py: v N_a/I.
    M3_LABOR_SHARE: f64 = 0.57630237193026765214;
    /// goldens/generate_1c.py: v/P_s.
    M3_REAL_WAGE: f64 = 1.117992973118317116;
    /// goldens/generate_1c.py: interest = rho omega V X, L2.
    M3_INTEREST: f64 = 1.7165861894881998391;
    /// goldens/generate_1c.py: W = omega V X, machine wealth.
    M3_WEALTH: f64 = 34.331723789763996783;
    /// goldens/generate_1c.py: interest/I.
    M3_CAPITAL_SHARE: f64 = 0.062075547015216440453;

    // M3Z: M2's machine in Appendix B's closure, G1's household, gamma = 1 + 2x, rho 0 (docs/unit-1c.md section 3.3)
    /// goldens/generate_1c.py: x*.
    M3Z_X_STAR: f64 = 0.93944579285911351352;
    /// goldens/generate_1c.py: 1 - x*.
    M3Z_ONE_MINUS_X_STAR: f64 = 0.06055420714088648648;
    /// goldens/generate_1c.py: gamma(x*).
    M3Z_GAMMA_STAR: f64 = 2.878891585718227027;
    /// goldens/generate_1c.py: v = w/r.
    M3Z_V: f64 = 4.0235521384808297705;
    /// goldens/generate_1c.py: u.
    M3Z_U: f64 = 0.1;
    /// goldens/generate_1c.py: omega.
    M3Z_OMEGA: f64 = 1.2;
    /// goldens/generate_1c.py: p_m = O + u V.
    M3Z_P_M: f64 = 1.3976046053422440254;
    /// goldens/generate_1c.py: O, the operating cost.
    M3Z_OPERATING: f64 = 1.3011575165192049898;
    /// goldens/generate_1c.py: V, the build cost.
    M3Z_BUILD: f64 = 0.96447088823039035664;
    /// goldens/generate_1c.py: p of the good.
    M3Z_P_GOOD: f64 = 2.7900844573429237992;
    /// goldens/generate_1c.py: P_s.
    M3Z_P_S: f64 = 3.7900844573429237992;
    /// goldens/generate_1c.py: Y.
    M3Z_Y: f64 = 5.7106572226425542111;
    /// goldens/generate_1c.py: X = K.
    M3Z_SERVICES: f64 = 21.23437018493785044;
    /// goldens/generate_1c.py: N_a.
    M3Z_N_A: f64 = 2.8939287425630387991;
    /// goldens/generate_1c.py: final hours.
    M3Z_FINAL_HOURS: f64 = 0.3458043203704967463;
    /// goldens/generate_1c.py: machine hours (lambda + delta lambda_I) X.
    M3Z_MACHINE_HOURS: f64 = 2.5481244221925420528;
    /// goldens/generate_1c.py: I = v N_a + T + interest.
    M3Z_INCOME: f64 = 21.643873180750653453;
    /// goldens/generate_1c.py: lambda-tilde, price side.
    M3Z_LAMBDA_TILDE: f64 = 0.24489795918367346939;
    /// goldens/generate_1c.py: lambda-tilde^q, clearing side.
    M3Z_LAMBDA_TILDE_Q: f64 = 0.24489795918367346939;
    /// goldens/generate_1c.py: b-tilde, price side.
    M3Z_B_TILDE: f64 = 0.41224489795918367347;
    /// goldens/generate_1c.py: b-tilde^q, clearing side.
    M3Z_B_TILDE_Q: f64 = 0.41224489795918367347;
    /// goldens/generate_1c.py: v N_a/I.
    M3Z_LABOR_SHARE: f64 = 0.53797548541849385837;
    /// goldens/generate_1c.py: v/P_s.
    M3Z_REAL_WAGE: f64 = 1.0615995985750620446;

    // M4: the three-type fork economy, eta 1, rho 0.04: loom, engine and power on 1b's fork economy with intermediate inputs (docs/unit-1c.md section 3.3)
    /// goldens/generate_1c.py: gamma at the switch loom -> engine, closed form.
    M4_SWITCH_GAMMA: f64 = 0.43150661211919930493;
    /// goldens/generate_1c.py: the switch loom -> engine on the line.
    M4_SWITCH_X: f64 = 0.28938326514899913117;
    /// goldens/generate_1c.py: x*, with the engine at the margin.
    M4_X_STAR: f64 = 0.81660019325202962523;
    /// goldens/generate_1c.py: v.
    M4_V: f64 = 0.12971114414607967078;
    /// goldens/generate_1c.py: P_s.
    M4_P_S: f64 = 1.5507722758016523373;
    /// goldens/generate_1c.py: Y.
    M4_Y: f64 = 6.5098827079475574343;
    /// goldens/generate_1c.py: N_a.
    M4_N_A: f64 = 0.3213138107587870914;
    /// goldens/generate_1c.py: I.
    M4_INCOME: f64 = 10.095345622205656911;
    /// goldens/generate_1c.py: interest.
    M4_INTEREST: f64 = 0.05366764018219771385;
    /// goldens/generate_1c.py: v/P_s.
    M4_REAL_WAGE: f64 = 0.083642934665585968587;
    /// goldens/generate_1c.py: L_s, price side.
    M4_L_S: f64 = 0.051867407402670454948;
    /// goldens/generate_1c.py: B_s, price side.
    M4_B_S: f64 = 1.5440444950435611101;
    /// goldens/generate_1c.py: L_s^q, clearing side.
    M4_L_S_Q: f64 = 0.049357849468856443131;
    /// goldens/generate_1c.py: B_s^q, clearing side.
    M4_B_S_Q: f64 = 1.5361259869999732087;
    /// goldens/generate_1c.py: u of the loom.
    M4_LOOM_U: f64 = 0.1456;
    /// goldens/generate_1c.py: omega of the loom.
    M4_LOOM_OMEGA: f64 = 1.14;
    /// goldens/generate_1c.py: lambda-tilde of the loom.
    M4_LOOM_LAMBDA_TILDE: f64 = 0.45371483821077908998;
    /// goldens/generate_1c.py: b-tilde of the loom.
    M4_LOOM_B_TILDE: f64 = 0.12107468709546726147;
    /// goldens/generate_1c.py: lambda-tilde^q of the loom.
    M4_LOOM_LAMBDA_TILDE_Q: f64 = 0.40531257155942294481;
    /// goldens/generate_1c.py: b-tilde^q of the loom.
    M4_LOOM_B_TILDE_Q: f64 = 0.10638246444291784342;
    /// goldens/generate_1c.py: p of the loom.
    M4_LOOM_P: f64 = 0.17992655787584084456;
    /// goldens/generate_1c.py: O of the loom.
    M4_LOOM_OPERATING: f64 = 0.11474088460549940179;
    /// goldens/generate_1c.py: V of the loom.
    M4_LOOM_BUILD: f64 = 0.44770379993366375524;
    /// goldens/generate_1c.py: the loom's closure wage at x*.
    M4_LOOM_CLOSURE_WAGE: f64 = 0.16857294780232336472;
    /// goldens/generate_1c.py: u of the engine.
    M4_ENGINE_U: f64 = 0.097344;
    /// goldens/generate_1c.py: omega of the engine.
    M4_ENGINE_OMEGA: f64 = 1.1836;
    /// goldens/generate_1c.py: lambda-tilde of the engine.
    M4_ENGINE_LAMBDA_TILDE: f64 = 0.045262504992904392782;
    /// goldens/generate_1c.py: b-tilde of the engine.
    M4_ENGINE_B_TILDE: f64 = 0.29815838954071774421;
    /// goldens/generate_1c.py: lambda-tilde^q of the engine.
    M4_ENGINE_LAMBDA_TILDE_Q: f64 = 0.037783375314861460957;
    /// goldens/generate_1c.py: b-tilde^q of the engine.
    M4_ENGINE_B_TILDE_Q: f64 = 0.27455919395465994962;
    /// goldens/generate_1c.py: p of the engine.
    M4_ENGINE_P: f64 = 0.30402944085026501671;
    /// goldens/generate_1c.py: O of the engine.
    M4_ENGINE_OPERATING: f64 = 0.26086963649967659899;
    /// goldens/generate_1c.py: V of the engine.
    M4_ENGINE_BUILD: f64 = 0.44337405849963446875;
    /// goldens/generate_1c.py: X of the engine.
    M4_ENGINE_SERVICES: f64 = 2.2008417237567965685;
    /// goldens/generate_1c.py: the engine's closure wage at x*.
    M4_ENGINE_CLOSURE_WAGE: f64 = 0.12971114414607967078;
    /// goldens/generate_1c.py: u of the power.
    M4_POWER_U: f64 = 0.097344;
    /// goldens/generate_1c.py: omega of the power.
    M4_POWER_OMEGA: f64 = 1.1836;
    /// goldens/generate_1c.py: lambda-tilde of the power.
    M4_POWER_LAMBDA_TILDE: f64 = 0.030175003328602928521;
    /// goldens/generate_1c.py: b-tilde of the power.
    M4_POWER_B_TILDE: f64 = 0.51263679302714516281;
    /// goldens/generate_1c.py: lambda-tilde^q of the power.
    M4_POWER_LAMBDA_TILDE_Q: f64 = 0.025188916876574307305;
    /// goldens/generate_1c.py: b-tilde^q of the power.
    M4_POWER_B_TILDE_Q: f64 = 0.50637279596977329975;
    /// goldens/generate_1c.py: p of the power.
    M4_POWER_P: f64 = 0.51655082723351001114;
    /// goldens/generate_1c.py: O of the power.
    M4_POWER_OPERATING: f64 = 0.50259422288292159342;
    /// goldens/generate_1c.py: V of the power.
    M4_POWER_BUILD: f64 = 0.14337405849963446875;
    /// goldens/generate_1c.py: X of the power.
    M4_POWER_SERVICES: f64 = 1.1004208618783982843;
    /// goldens/generate_1c.py: p of manufactures.
    M4_MANUFACTURES_P: f64 = 0.043780239482438162407;
    /// goldens/generate_1c.py: v/p of manufactures.
    M4_MANUFACTURES_REAL_WAGE: f64 = 2.9627783145889711812;
    /// goldens/generate_1c.py: p of food.
    M4_FOOD_P: f64 = 0.66799618444616177099;
    /// goldens/generate_1c.py: v/p of food.
    M4_FOOD_REAL_WAGE: f64 = 0.19417946863517444161;
    /// goldens/generate_1c.py: p of care.
    M4_CARE_P: f64 = 0.26575731319520371529;
    /// goldens/generate_1c.py: v/p of care.
    M4_CARE_REAL_WAGE: f64 = 0.48808118424498222654;
    /// goldens/generate_1c.py: p of shelter.
    M4_SHELTER_P: f64 = 1.0206131960896479681;
    /// goldens/generate_1c.py: v/p of shelter.
    M4_SHELTER_REAL_WAGE: f64 = 0.1270913845157516329;
    /// goldens/generate_1c.py: y-hat of manufactures, the basket's gross output.
    M4_MANUFACTURES_YHAT: f64 = 0.524;
    /// goldens/generate_1c.py: L-bar of manufactures through the chain.
    M4_MANUFACTURES_L_BAR: f64 = 0.8;
    /// goldens/generate_1c.py: b-bar of manufactures, land through the chain.
    M4_MANUFACTURES_B_BAR: f64 = 0.0;
    /// goldens/generate_1c.py: y-hat of food, the basket's gross output.
    M4_FOOD_YHAT: f64 = 1.04;
    /// goldens/generate_1c.py: L-bar of food through the chain.
    M4_FOOD_L_BAR: f64 = 0.805;
    /// goldens/generate_1c.py: b-bar of food, land through the chain.
    M4_FOOD_B_BAR: f64 = 0.6;
    /// goldens/generate_1c.py: y-hat of care, the basket's gross output.
    M4_CARE_YHAT: f64 = 0.2;
    /// goldens/generate_1c.py: L-bar of care through the chain.
    M4_CARE_L_BAR: f64 = 0.411;
    /// goldens/generate_1c.py: b-bar of care, land through the chain.
    M4_CARE_B_BAR: f64 = 0.22;
    /// goldens/generate_1c.py: y-hat of shelter, the basket's gross output.
    M4_SHELTER_YHAT: f64 = 0.8;
    /// goldens/generate_1c.py: L-bar of shelter through the chain.
    M4_SHELTER_L_BAR: f64 = 0.26;
    /// goldens/generate_1c.py: b-bar of shelter, land through the chain.
    M4_SHELTER_B_BAR: f64 = 1.0;

    // M4 path: the switch and the root as task automation lowers eta
    /// goldens/generate_1c.py: the switch at eta 2.
    M4_ETA2_SWITCH_X: f64 = 0.019691632574499565584;
    /// goldens/generate_1c.py: x* at eta 2.
    M4_ETA2_X_STAR: f64 = 0.74204429563077694458;
    /// goldens/generate_1c.py: v at eta 2.
    M4_ETA2_V: f64 = 0.24544595555111700983;
    /// goldens/generate_1c.py: the switch at eta 1.
    M4_ETA1_SWITCH_X: f64 = 0.28938326514899913117;
    /// goldens/generate_1c.py: x* at eta 1.
    M4_ETA1_X_STAR: f64 = 0.81660019325202962523;
    /// goldens/generate_1c.py: v at eta 1.
    M4_ETA1_V: f64 = 0.12971114414607967078;
    /// goldens/generate_1c.py: the switch at eta 0.5.
    M4_ETA05_SWITCH_X: f64 = 0.82876653029799826234;
    /// goldens/generate_1c.py: x* at eta 0.5.
    M4_ETA05_X_STAR: f64 = 0.89769898858934104979;
    /// goldens/generate_1c.py: v at eta 0.5.
    M4_ETA05_V: f64 = 0.069157733897259938505;

    // M4t: the tie at the switch, eta 0.5, N 8 (docs/unit-1c.md section 4.7)
    /// goldens/generate_1c.py: x* = the switch point.
    M4T_X_STAR: f64 = 0.82876653029799826234;
    /// goldens/generate_1c.py: gamma at the switch.
    M4T_GAMMA: f64 = 0.43150661211919930493;
    /// goldens/generate_1c.py: f at the switch under the loom.
    M4T_F_LOOM: f64 = 0.81345450186486179175;
    /// goldens/generate_1c.py: f at the switch under the engine.
    M4T_F_ENGINE: f64 = -0.067383662779535658336;
    /// goldens/generate_1c.py: sigma, the engine's share of the machine tasks.
    M4T_SHARE: f64 = 0.92300430261428115677;
    /// goldens/generate_1c.py: v.
    M4T_V: f64 = 0.064963057288499518554;
    /// goldens/generate_1c.py: P_s.
    M4T_P_S: f64 = 1.4968975100449137833;
    /// goldens/generate_1c.py: Y.
    M4T_Y: f64 = 6.7140878145988791705;
    /// goldens/generate_1c.py: N_a.
    M4T_N_A: f64 = 0.33986513945300654882;
    /// goldens/generate_1c.py: I.
    M4T_INCOME: f64 = 10.050301331895958964;
    /// goldens/generate_1c.py: interest.
    M4T_INTEREST: f64 = 0.028222653371309421724;
    /// goldens/generate_1c.py: L_s^q with the split.
    M4T_L_S_Q: f64 = 0.050619704245454699429;
    /// goldens/generate_1c.py: B_s^q with the split.
    M4T_B_S_Q: f64 = 1.4894056014960584203;
    /// goldens/generate_1c.py: X of the loom at the tie.
    M4T_LOOM_SERVICES: f64 = 0.17575741974157999639;
    /// goldens/generate_1c.py: X of the engine at the tie.
    M4T_ENGINE_SERVICES: f64 = 1.0508577148308887454;
    /// goldens/generate_1c.py: X of the power at the tie.
    M4T_POWER_SERVICES: f64 = 0.53421672840252337253;

    // M4 regimes
    /// goldens/generate_1c.py: f(1) at eta 0.25: BoundaryNoMargin.
    M4_REG_ETA025_F_AT_1: f64 = 0.39646115974723321568;
    /// goldens/generate_1c.py: d(1) at eta 50, the engine's least pivot: NotViable.
    M4_REG_ETA50_D_AT_1: f64 = -0.0123776;
    /// goldens/generate_1c.py: f(1e-12) at N 200, chi_max 0.01: NoInteriorAtZero.
    M4_REG_N200_F_AT_LO: f64 = -190.75346260388830846;

    // M5 path: interest selects the technique, G1's household on gamma = 0.2 + 0.8x with a flow and a durable task type (docs/unit-1c.md section 3.3)
    /// goldens/generate_1c.py: x* at rho 0.
    M5_RHO0_X_STAR: f64 = 0.98585935628627146194;
    /// goldens/generate_1c.py: v at rho 0.
    M5_RHO0_V: f64 = 0.036595909713248331463;
    /// goldens/generate_1c.py: N_a at rho 0.
    M5_RHO0_N_A: f64 = 0.14069985600957043599;
    /// goldens/generate_1c.py: I at rho 0.
    M5_RHO0_INCOME: f64 = 10.00514903922719328;
    /// goldens/generate_1c.py: x* at rho 0.05.
    M5_RHO0_05_X_STAR: f64 = 0.93600030634593811309;
    /// goldens/generate_1c.py: v at rho 0.05.
    M5_RHO0_05_V: f64 = 0.1910047225509255231;
    /// goldens/generate_1c.py: N_a at rho 0.05.
    M5_RHO0_05_N_A: f64 = 0.62962264010530237988;
    /// goldens/generate_1c.py: I at rho 0.05.
    M5_RHO0_05_INCOME: f64 = 10.986031694009706203;
    /// goldens/generate_1c.py: interest at rho 0.05.
    M5_RHO0_05_INTEREST: f64 = 0.8657707963246116886;
    /// goldens/generate_1c.py: x* at rho 0.1.
    M5_RHO0_1_X_STAR: f64 = 0.87478588741106422192;
    /// goldens/generate_1c.py: v at rho 0.1.
    M5_RHO0_1_V: f64 = 0.47343913004420151224;
    /// goldens/generate_1c.py: N_a at rho 0.1.
    M5_RHO0_1_N_A: f64 = 1.2321344454088485648;
    /// goldens/generate_1c.py: I at rho 0.1.
    M5_RHO0_1_INCOME: f64 = 12.894356749263873923;
    /// goldens/generate_1c.py: interest at rho 0.1.
    M5_RHO0_1_INTEREST: f64 = 2.3110160893320139581;
    /// goldens/generate_1c.py: the switch flow -> durable at rho 0.1.
    M5_RHO0_1_SWITCH_X: f64 = 0.34233278653318684224;
    /// goldens/generate_1c.py: x* at rho 0.15.
    M5_RHO0_15_X_STAR: f64 = 0.88954352648655180271;
    /// goldens/generate_1c.py: v at rho 0.15.
    M5_RHO0_15_V: f64 = 0.50153949761761870206;
    /// goldens/generate_1c.py: N_a at rho 0.15.
    M5_RHO0_15_N_A: f64 = 1.2820504100197906925;
    /// goldens/generate_1c.py: I at rho 0.15.
    M5_RHO0_15_INCOME: f64 = 10.642998918561787894;
    /// goldens/generate_1c.py: x* at rho 0.3.
    M5_RHO0_3_X_STAR: f64 = 0.88954352648655180271;
    /// goldens/generate_1c.py: v at rho 0.3.
    M5_RHO0_3_V: f64 = 0.50153949761761870206;
    /// goldens/generate_1c.py: N_a at rho 0.3.
    M5_RHO0_3_N_A: f64 = 1.2820504100197906925;
    /// goldens/generate_1c.py: I at rho 0.3.
    M5_RHO0_3_INCOME: f64 = 10.642998918561787894;

    // M5m: three equilibria at rho 0.1, N 60, h 0.2 (docs/unit-1c.md section 5.5)
    /// goldens/generate_1c.py: the switch flow -> durable.
    M5M_SWITCH_X: f64 = 0.34233278653318684224;
    /// goldens/generate_1c.py: f(1e-12), flow.
    M5M_F_AT_LO: f64 = 32.536714584059839575;
    /// goldens/generate_1c.py: f at the switch, flow.
    M5M_F_FLOW_AT_SWITCH: f64 = -1.7185567914492680325;
    /// goldens/generate_1c.py: f at the switch, durable.
    M5M_F_DURABLE_AT_SWITCH: f64 = 4.5089416135953250503;
    /// goldens/generate_1c.py: f(1), durable.
    M5M_F_AT_1: f64 = -42.190024304465227684;
    /// goldens/generate_1c.py: the equilibrium below the switch (flow), for reference.
    M5M_FLOW_ROOT: f64 = 0.32226807920428180529;
    /// goldens/generate_1c.py: sigma of the tie at the switch, for reference.
    M5M_TIE_SHARE: f64 = 0.32468517480835597758;
    /// goldens/generate_1c.py: the equilibrium above the switch (durable), for reference.
    M5M_DURABLE_ROOT: f64 = 0.4031287106909996285;
}
