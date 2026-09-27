//! The golden numbers of unit 1b, one constant each.
//!
//! Every constant equals its line in `goldens/goldens_1b.txt` rounded to 20 significant
//! digits (half up), and `c8_goldens_file::constants_match_goldens_1b_txt` enforces that.
//! `goldens_1b.txt` is written by `goldens/generate_1b.py`, which solves each instance with
//! mpmath at 70 digits from the equations of docs/unit-1b.md §4. Each constant's doc comment
//! is the generator's note; laborformal references are at 31b3482.

// A literal keeps 20 significant digits so that it records the golden, not the double
// nearest to it. The compiler rounds it to the nearest f64.
#![allow(clippy::excessive_precision)]

/// Declares each golden as a `pub const`, and `TABLE_1B` with every name and literal as
/// written.
macro_rules! goldens {
    ($( $(#[$attr:meta])* $name:ident : $ty:ty = $value:literal; )*) => {
        $( $(#[$attr])* pub const $name: $ty = $value; )*

        /// (name, literal as written) for every constant above.
        pub const TABLE_1B: &[(&str, &str)] = &[$((stringify!($name), stringify!($value))),*];
    };
}

goldens! {
    // C1: the fork at G1, SSRN Appendix B in category form (docs/unit-1b.md section 3.3): the good (z 1, b 0, mu 1) and space (z = h = 1, b 1, mu 0) on one segment
    /// goldens/generate_1b.py: L* = (1 - x*) + J(x*)/gamma(x*), main.tex eq effective-hours.
    C1_GOOD_L_STAR: f64 = 0.66535131631003383848;
    /// goldens/generate_1b.py: lambda-tilde of the good = 1a's L_s, SSRN eq 4.
    C1_GOOD_LAMBDA_TILDE: f64 = 0.17046683479342606591;
    /// goldens/generate_1b.py: b-tilde of the good, SSRN eq 4.
    C1_GOOD_B_TILDE: f64 = 0.26893802364690478262;
    /// goldens/generate_1b.py: p of the good = 1a's p, SSRN eq 12.
    C1_GOOD_P: f64 = 0.36157583177980927464;
    /// goldens/generate_1b.py: v/p of the good, SSRN eq 12.
    C1_GOOD_REAL_WAGE: f64 = 1.5029653890908211134;
    /// goldens/generate_1b.py: v/b-tilde of the good, SSRN eq 13's ceiling.
    C1_GOOD_WAGE_CEILING: f64 = 2.0206735861577852136;
    /// goldens/generate_1b.py: v/B_s, the basket's rent ceiling (check_macro C3).
    C1_RENT_CEILING: f64 = 0.42826044343359912019;
    /// goldens/generate_1b.py: v/P_s.
    C1_REAL_WAGE: f64 = 0.3991228016925181936;
    /// goldens/generate_1b.py: p/P_s, the good's expenditure share.
    C1_GOOD_SHARE: f64 = 0.2655568814754655817;

    // C2: the fork along G3's path (lambda 0, gamma = eta (1 + x)), with the CES share of SSRN eq 26 at alpha 0.3 (parameters constructed)
    /// goldens/generate_1b.py: v/p of the good at eta = 1.
    C2_ETA_1_GOOD_REAL_WAGE: f64 = 1.1819187877999820791;
    /// goldens/generate_1b.py: v at eta = 1 (1a's G3).
    C2_ETA_1_V: f64 = 0.98840122029213958952;
    /// goldens/generate_1b.py: q = r/p of the good at eta = 1.
    C2_ETA_1_Q: f64 = 1.1957884748975167441;
    /// goldens/generate_1b.py: alpha(q), sigma 0.5, at eta = 1.
    C2_ETA_1_CES_HALF: f64 = 0.41720785618326636565;
    /// goldens/generate_1b.py: alpha(q), sigma 2, at eta = 1.
    C2_ETA_1_CES_TWO: f64 = 0.13314863099244117043;
    /// goldens/generate_1b.py: v/p of the good at eta = 0.3.
    C2_ETA_0_3_GOOD_REAL_WAGE: f64 = 1.264023705823887722;
    /// goldens/generate_1b.py: v at eta = 0.3 (1a's G3).
    C2_ETA_0_3_V: f64 = 0.32367862903339933844;
    /// goldens/generate_1b.py: q = r/p of the good at eta = 0.3.
    C2_ETA_0_3_Q: f64 = 3.90518122743734574;
    /// goldens/generate_1b.py: alpha(q), sigma 0.5, at eta = 0.3.
    C2_ETA_0_3_CES_HALF: f64 = 0.56402241581444845787;
    /// goldens/generate_1b.py: alpha(q), sigma 2, at eta = 0.3.
    C2_ETA_0_3_CES_TWO: f64 = 0.044920517603712652238;
    /// goldens/generate_1b.py: v/p of the good at eta = 0.1.
    C2_ETA_0_1_GOOD_REAL_WAGE: f64 = 1.3058486238553142227;
    /// goldens/generate_1b.py: v at eta = 0.1 (1a's G3).
    C2_ETA_0_1_V: f64 = 0.11186280479118877161;
    /// goldens/generate_1b.py: q = r/p of the good at eta = 0.1.
    C2_ETA_0_1_Q: f64 = 11.673662450113833805;
    /// goldens/generate_1b.py: alpha(q), sigma 0.5, at eta = 0.1.
    C2_ETA_0_1_CES_HALF: f64 = 0.69104700183614749627;
    /// goldens/generate_1b.py: alpha(q), sigma 2, at eta = 0.1.
    C2_ETA_0_1_CES_TWO: f64 = 0.015490281344367582874;
    /// goldens/generate_1b.py: v/p of the good at eta = 0.03.
    C2_ETA_0_03_GOOD_REAL_WAGE: f64 = 1.3244824123062040581;
    /// goldens/generate_1b.py: v at eta = 0.03 (1a's G3).
    C2_ETA_0_03_V: f64 = 0.034056082139179303478;
    /// goldens/generate_1b.py: q = r/p of the good at eta = 0.03.
    C2_ETA_0_03_Q: f64 = 38.89121499335571966;
    /// goldens/generate_1b.py: alpha(q), sigma 0.5, at eta = 0.03.
    C2_ETA_0_03_CES_HALF: f64 = 0.80325049843134940416;
    /// goldens/generate_1b.py: alpha(q), sigma 2, at eta = 0.03.
    C2_ETA_0_03_CES_TWO: f64 = 0.0047005500620400046454;
    /// goldens/generate_1b.py: v/p of the good at eta = 0.01.
    C2_ETA_0_01_GOOD_REAL_WAGE: f64 = 1.3303189612820227402;
    /// goldens/generate_1b.py: v at eta = 0.01 (1a's G3).
    C2_ETA_0_01_V: f64 = 0.011402655775197234064;
    /// goldens/generate_1b.py: q = r/p of the good at eta = 0.01.
    C2_ETA_0_01_Q: f64 = 116.66746655421262523;
    /// goldens/generate_1b.py: alpha(q), sigma 0.5, at eta = 0.01.
    C2_ETA_0_01_CES_HALF: f64 = 0.8761010290122206556;
    /// goldens/generate_1b.py: alpha(q), sigma 2, at eta = 0.01.
    C2_ETA_0_01_CES_TWO: f64 = 0.001571858600187029765;
    /// goldens/generate_1b.py: gamma(1)/J(1) = 2/(3/2), the limit of v/p as eta -> 0.
    C2_LIMIT_GOOD_REAL_WAGE: f64 = 1.3333333333333333333;
    /// goldens/generate_1b.py: alpha(1) at sigma 0 = 1/2.
    C2_Q1_CES_ZERO: f64 = 0.5;
    /// goldens/generate_1b.py: alpha(1) at sigma 0.5 = 1/(1 + sqrt(7/3)).
    C2_Q1_CES_HALF: f64 = 0.39564392373896000165;
    /// goldens/generate_1b.py: alpha(1) at sigma 1 = alpha.
    C2_Q1_CES_ONE: f64 = 0.3;
    /// goldens/generate_1b.py: alpha(1) at sigma 2 = 9/58.
    C2_Q1_CES_TWO: f64 = 0.15517241379310344828;

    // C3: the fork economy (constructed 2026-09-27): G1's scalars, edges (0, 0.4, 0.75, 1); manufactures (z 0.3, b 0, mu (2, 0, 0)), food (1, 0.6, (0.5, 1.5, 0)), care (0.2, 0.1, (0, 0, 1)), shelter (0.8, 1, (0, 0.4, 0))
    /// goldens/generate_1b.py: x*, C3 flow.
    C3_X_STAR: f64 = 0.71090347785320059226;
    /// goldens/generate_1b.py: 1 - x*, C3 flow.
    C3_ONE_MINUS_X_STAR: f64 = 0.28909652214679940774;
    /// goldens/generate_1b.py: gamma(x*), C3 flow.
    C3_GAMMA_STAR: f64 = 0.76872278228256047381;
    /// goldens/generate_1b.py: v, C3 flow.
    C3_V: f64 = 0.4647912788060681259;
    /// goldens/generate_1b.py: p_m, C3 flow.
    C3_P_M: f64 = 0.60462794848614772328;
    /// goldens/generate_1b.py: P_s, C3 flow.
    C3_P_S: f64 = 1.7925374977968492847;
    /// goldens/generate_1b.py: Y, C3 flow.
    C3_Y: f64 = 5.8178032129320888237;
    /// goldens/generate_1b.py: K, C3 flow.
    C3_K: f64 = 4.3467985940910846758;
    /// goldens/generate_1b.py: M_s, machine services per basket, C3 flow.
    C3_M_S: f64 = 0.52300823945027399087;
    /// goldens/generate_1b.py: H_s, hours at final tasks per basket, C3 flow.
    C3_H_S: f64 = 0.12115567030717492209;
    /// goldens/generate_1b.py: Y H_s, C3 flow.
    C3_FINAL_HOURS: f64 = 0.70485984797802313461;
    /// goldens/generate_1b.py: lambda delta K, C3 flow.
    C3_MACHINE_HOURS: f64 = 0.21733992970455423379;
    /// goldens/generate_1b.py: N_a, C3 flow.
    C3_N_A: f64 = 0.9221997776825773684;
    /// goldens/generate_1b.py: N_a / N, C3 flow.
    C3_PARTICIPATION: f64 = 0.2305499444206443421;
    /// goldens/generate_1b.py: I, C3 flow.
    C3_INCOME: f64 = 10.42863041398375686;
    /// goldens/generate_1b.py: v N_a / I, C3 flow.
    C3_LABOR_SHARE: f64 = 0.041101314071788954885;
    /// goldens/generate_1b.py: v / P_s, C3 flow.
    C3_REAL_WAGE: f64 = 0.25929236034243538841;
    /// goldens/generate_1b.py: N P_s, C3 flow.
    C3_SUPPORT_COST: f64 = 7.1701499911873971389;
    /// goldens/generate_1b.py: N + v N_a / P_s, C3 flow.
    C3_WORKER_BASKETS: f64 = 4.2391193570625846558;
    /// goldens/generate_1b.py: T / P_s - N, C3 flow.
    C3_PROVIDER_BASKETS: f64 = 1.5786838558695041679;
    /// goldens/generate_1b.py: L_s* = sum z_j L_j*, C3 flow.
    C3_L_STAR_S: f64 = 0.80151567979890759228;
    /// goldens/generate_1b.py: L_s = sum z_j lambda-tilde_j, C3 flow.
    C3_L_S: f64 = 0.15851340169648020715;
    /// goldens/generate_1b.py: B_s = sum z_j b-tilde_j, C3 flow.
    C3_B_S: f64 = 1.7188618511144422805;
    /// goldens/generate_1b.py: v / B_s, C3 flow.
    C3_RENT_CEILING: f64 = 0.27040641951807574243;
    /// goldens/generate_1b.py: u lambda / (1 - u a), C3 flow.
    C3_LAMBDA_TILDE_MACHINE: f64 = 0.071428571428571428571;
    /// goldens/generate_1b.py: u b / (1 - u a), C3 flow.
    C3_B_TILDE_MACHINE: f64 = 0.57142857142857142857;
    /// goldens/generate_1b.py: funded, C3 flow.
    C3_FUNDED: bool = true;
    /// goldens/generate_1b.py: lemma_b1, C3 flow.
    C3_LEMMA_B1: bool = true;
    /// goldens/generate_1b.py: n_D - n_S at x = 1e-12, C3 flow.
    C3_F_AT_LO: f64 = 7.6481961400442555623;
    /// goldens/generate_1b.py: n_D(1) - n_S(1), C3 flow.
    C3_F_AT_1: f64 = -0.92326297508086722453;
    /// goldens/generate_1b.py: n_S(1), C3 flow.
    C3_N_S_AT_1: f64 = 1.1740896553003717311;
    /// goldens/generate_1b.py: n_D(1), C3 flow.
    C3_N_D_AT_1: f64 = 0.25082668021950450653;
    /// goldens/generate_1b.py: P_s(1), C3 flow.
    C3_P_S_AT_1: f64 = 1.8038892307692307692;
    /// goldens/generate_1b.py: p_j of manufactures, C3 flow.
    C3_MANUFACTURES_P: f64 = 0.1741328491640105443;
    /// goldens/generate_1b.py: v/p_j of manufactures, C3 flow.
    C3_MANUFACTURES_REAL_WAGE: f64 = 2.6691763273700016452;
    /// goldens/generate_1b.py: L_j* of manufactures, C3 flow.
    C3_MANUFACTURES_L_STAR: f64 = 0.37464741079332217592;
    /// goldens/generate_1b.py: H_j of manufactures, C3 flow.
    C3_MANUFACTURES_H: f64 = 0.0;
    /// goldens/generate_1b.py: M_j of manufactures, C3 flow.
    C3_MANUFACTURES_M: f64 = 0.288;
    /// goldens/generate_1b.py: lambda-tilde_j of manufactures, C3 flow.
    C3_MANUFACTURES_LAMBDA_TILDE: f64 = 0.020571428571428571429;
    /// goldens/generate_1b.py: b-tilde_j of manufactures, C3 flow.
    C3_MANUFACTURES_B_TILDE: f64 = 0.16457142857142857143;
    /// goldens/generate_1b.py: z_j p_j / P_s of manufactures, C3 flow.
    C3_MANUFACTURES_SHARE: f64 = 0.02914296343223475326;
    /// goldens/generate_1b.py: 1/(Lbar_j + b_j/v) of manufactures, C3 flow.
    C3_MANUFACTURES_WAGE_FLOOR: f64 = 1.25;
    /// goldens/generate_1b.py: v lambda-tilde_j / p_j of manufactures, C3 flow.
    C3_MANUFACTURES_PHI_W: f64 = 0.054908770163040033843;
    /// goldens/generate_1b.py: b-tilde_j / p_j of manufactures, C3 flow.
    C3_MANUFACTURES_PHI_R: f64 = 0.94509122983695996616;
    /// goldens/generate_1b.py: p_j of food, C3 flow.
    C3_FOOD_P: f64 = 0.85248227834842577815;
    /// goldens/generate_1b.py: v/p_j of food, C3 flow.
    C3_FOOD_REAL_WAGE: f64 = 0.54522104518881160714;
    /// goldens/generate_1b.py: L_j* of food, C3 flow.
    C3_FOOD_L_STAR: f64 = 0.54321647126639130952;
    /// goldens/generate_1b.py: H_j of food, C3 flow.
    C3_FOOD_H: f64 = 0.058644783220199111614;
    /// goldens/generate_1b.py: M_j of food, C3 flow.
    C3_FOOD_M: f64 = 0.37250129625022581665;
    /// goldens/generate_1b.py: lambda-tilde_j of food, C3 flow.
    C3_FOOD_LAMBDA_TILDE: f64 = 0.085252018666643812803;
    /// goldens/generate_1b.py: b-tilde_j of food, C3 flow.
    C3_FOOD_B_TILDE: f64 = 0.81285788357155760952;
    /// goldens/generate_1b.py: z_j p_j / P_s of food, C3 flow.
    C3_FOOD_SHARE: f64 = 0.47557291236372158474;
    /// goldens/generate_1b.py: 1/(Lbar_j + b_j/v) of food, C3 flow.
    C3_FOOD_WAGE_FLOOR: f64 = 0.49605585530168370024;
    /// goldens/generate_1b.py: v lambda-tilde_j / p_j of food, C3 flow.
    C3_FOOD_PHI_W: f64 = 0.046481194721883616915;
    /// goldens/generate_1b.py: b-tilde_j / p_j of food, C3 flow.
    C3_FOOD_PHI_R: f64 = 0.95351880527811638308;
    /// goldens/generate_1b.py: p_j of care, C3 flow.
    C3_CARE_P: f64 = 0.21619781970151703148;
    /// goldens/generate_1b.py: v/p_j of care, C3 flow.
    C3_CARE_REAL_WAGE: f64 = 2.1498425814273220769;
    /// goldens/generate_1b.py: L_j* of care, C3 flow.
    C3_CARE_L_STAR: f64 = 0.25;
    /// goldens/generate_1b.py: H_j of care, C3 flow.
    C3_CARE_H: f64 = 0.25;
    /// goldens/generate_1b.py: M_j of care, C3 flow.
    C3_CARE_M: f64 = 0.0;
    /// goldens/generate_1b.py: lambda-tilde_j of care, C3 flow.
    C3_CARE_LAMBDA_TILDE: f64 = 0.25;
    /// goldens/generate_1b.py: b-tilde_j of care, C3 flow.
    C3_CARE_B_TILDE: f64 = 0.1;
    /// goldens/generate_1b.py: z_j p_j / P_s of care, C3 flow.
    C3_CARE_SHARE: f64 = 0.024121985728860777756;
    /// goldens/generate_1b.py: 1/(Lbar_j + b_j/v) of care, C3 flow.
    C3_CARE_WAGE_FLOOR: f64 = 2.1498425814273220769;
    /// goldens/generate_1b.py: v lambda-tilde_j / p_j of care, C3 flow.
    C3_CARE_PHI_W: f64 = 0.53746064535683051923;
    /// goldens/generate_1b.py: b-tilde_j / p_j of care, C3 flow.
    C3_CARE_PHI_R: f64 = 0.46253935464316948077;
    /// goldens/generate_1b.py: p_j of shelter, C3 flow.
    C3_SHELTER_P: f64 = 1.0557197509486461712;
    /// goldens/generate_1b.py: v/p_j of shelter, C3 flow.
    C3_SHELTER_REAL_WAGE: f64 = 0.44026009590937090225;
    /// goldens/generate_1b.py: L_j* of shelter, C3 flow.
    C3_SHELTER_L_STAR: f64 = 0.11988123161814953748;
    /// goldens/generate_1b.py: H_j of shelter, C3 flow.
    C3_SHELTER_H: f64 = 0.015638608858719763097;
    /// goldens/generate_1b.py: M_j of shelter, C3 flow.
    C3_SHELTER_M: f64 = 0.080133679000060217774;
    /// goldens/generate_1b.py: lambda-tilde_j of shelter, C3 flow.
    C3_SHELTER_LAMBDA_TILDE: f64 = 0.021362443073009778652;
    /// goldens/generate_1b.py: b-tilde_j of shelter, C3 flow.
    C3_SHELTER_B_TILDE: f64 = 1.0457906737143201244;
    /// goldens/generate_1b.py: z_j p_j / P_s of shelter, C3 flow.
    C3_SHELTER_SHARE: f64 = 0.47116213847518288425;
    /// goldens/generate_1b.py: 1/(Lbar_j + b_j/v) of shelter, C3 flow.
    C3_SHELTER_WAGE_FLOOR: f64 = 0.43639473352947256164;
    /// goldens/generate_1b.py: v lambda-tilde_j / p_j of shelter, C3 flow.
    C3_SHELTER_PHI_W: f64 = 0.0094050312361817612168;
    /// goldens/generate_1b.py: b-tilde_j / p_j of shelter, C3 flow.
    C3_SHELTER_PHI_R: f64 = 0.99059496876381823878;
    /// goldens/generate_1b.py: x*, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_X_STAR: f64 = 0.76916199916626330199;
    /// goldens/generate_1b.py: 1 - x*, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_ONE_MINUS_X_STAR: f64 = 0.23083800083373669801;
    /// goldens/generate_1b.py: v, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_V: f64 = 0.15349426647317235432;
    /// goldens/generate_1b.py: p_m, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_P_M: f64 = 0.18826038769933046651;
    /// goldens/generate_1b.py: P_s, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_P_S: f64 = 1.5366380608625373639;
    /// goldens/generate_1b.py: Y, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_Y: f64 = 6.6180204392511955284;
    /// goldens/generate_1b.py: K, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_K: f64 = 4.3029355447378739266;
    /// goldens/generate_1b.py: N_a, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_N_A: f64 = 0.38083949356762359007;
    /// goldens/generate_1b.py: I, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_INCOME: f64 = 10.169502094519594853;
    /// goldens/generate_1b.py: interest, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_INTEREST: f64 = 0.11104541581041802903;
    /// goldens/generate_1b.py: v N_a / I, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_LABOR_SHARE: f64 = 0.0057482340989613912222;
    /// goldens/generate_1b.py: interest / I, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CAPITAL_SHARE: f64 = 0.01091945453949619263;
    /// goldens/generate_1b.py: v / P_s, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_REAL_WAGE: f64 = 0.099889668479911126888;
    /// goldens/generate_1b.py: (T + interest) / P_s - N, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_PROVIDER_BASKETS: f64 = 2.5799785084946683619;
    /// goldens/generate_1b.py: L_s, price side, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_L_S: f64 = 0.05960375347228008124;
    /// goldens/generate_1b.py: B_s, price side, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_B_S: f64 = 1.5274892264442619331;
    /// goldens/generate_1b.py: L_s^q, clearing side, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_L_S_Q: f64 = 0.057545832181007009178;
    /// goldens/generate_1b.py: B_s^q, clearing side, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_B_S_Q: f64 = 1.5110258561140773566;
    /// goldens/generate_1b.py: p_j of manufactures, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_MANUFACTURES_P: f64 = 0.054218991657407174355;
    /// goldens/generate_1b.py: v/p_j of manufactures, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_MANUFACTURES_REAL_WAGE: f64 = 2.8310055532396202833;
    /// goldens/generate_1b.py: lambda-tilde_j of manufactures, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_MANUFACTURES_LAMBDA_TILDE: f64 = 0.0066497859550050095637;
    /// goldens/generate_1b.py: b-tilde_j of manufactures, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_MANUFACTURES_B_TILDE: f64 = 0.05319828764004007651;
    /// goldens/generate_1b.py: lambda-tilde_j^q of manufactures, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_MANUFACTURES_LAMBDA_Q: f64 = 0.0056312849162011173184;
    /// goldens/generate_1b.py: b-tilde_j^q of manufactures, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_MANUFACTURES_B_Q: f64 = 0.045050279329608938547;
    /// goldens/generate_1b.py: p_j of food, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_FOOD_P: f64 = 0.67878697225216980023;
    /// goldens/generate_1b.py: v/p_j of food, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_FOOD_REAL_WAGE: f64 = 0.22613024814528870307;
    /// goldens/generate_1b.py: lambda-tilde_j of food, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_FOOD_LAMBDA_TILDE: f64 = 0.0096629702158666545223;
    /// goldens/generate_1b.py: b-tilde_j of food, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_FOOD_B_TILDE: f64 = 0.67730376172693323618;
    /// goldens/generate_1b.py: lambda-tilde_j^q of food, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_FOOD_LAMBDA_Q: f64 = 0.0081829608938547486034;
    /// goldens/generate_1b.py: b-tilde_j^q of food, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_FOOD_B_Q: f64 = 0.66546368715083798883;
    /// goldens/generate_1b.py: p_j of care, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CARE_P: f64 = 0.13834591627205451626;
    /// goldens/generate_1b.py: v/p_j of care, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CARE_REAL_WAGE: f64 = 1.1094961861492818199;
    /// goldens/generate_1b.py: lambda-tilde_j of care, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CARE_LAMBDA_TILDE: f64 = 0.23119534537396985697;
    /// goldens/generate_1b.py: b-tilde_j of care, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CARE_B_TILDE: f64 = 0.10285875632186527168;
    /// goldens/generate_1b.py: lambda-tilde_j^q of care, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CARE_LAMBDA_Q: f64 = 0.23114061341900152634;
    /// goldens/generate_1b.py: b-tilde_j^q of care, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_CARE_B_Q: f64 = 0.10242090068211862665;
    /// goldens/generate_1b.py: p_j of shelter, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_SHELTER_P: f64 = 1.0173952598234181351;
    /// goldens/generate_1b.py: v/p_j of shelter, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_SHELTER_REAL_WAGE: f64 = 0.15086984629731146294;
    /// goldens/generate_1b.py: lambda-tilde_j of shelter, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_SHELTER_LAMBDA_TILDE: f64 = 0.0021334729938974405684;
    /// goldens/generate_1b.py: b-tilde_j of shelter, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_SHELTER_B_TILDE: f64 = 1.0170677839511795245;
    /// goldens/generate_1b.py: lambda-tilde_j^q of shelter, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_SHELTER_LAMBDA_Q: f64 = 0.0018067039106145251397;
    /// goldens/generate_1b.py: b-tilde_j^q of shelter, C3d, (rho, delta, J_b) = (0.04, 0.35, 2).
    C3D_SHELTER_B_Q: f64 = 1.0144536312849162011;
    /// goldens/generate_1b.py: x*, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_X_STAR: f64 = 0.93807870421979404719;
    /// goldens/generate_1b.py: v, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_V: f64 = 0.039387319046324126847;
    /// goldens/generate_1b.py: Y, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_Y: f64 = 6.9193306663031228905;
    /// goldens/generate_1b.py: N_a, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_N_A: f64 = 0.10750959088903672838;
    /// goldens/generate_1b.py: P_s, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_P_S: f64 = 1.4458384773078020048;
    /// goldens/generate_1b.py: v / P_s, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_REAL_WAGE: f64 = 0.027241852851823800181;
    /// goldens/generate_1b.py: v/p_j of manufactures, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_MANUFACTURES_REAL_WAGE: f64 = 3.3002186228327612422;
    /// goldens/generate_1b.py: v/p_j of food, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_FOOD_REAL_WAGE: f64 = 0.063801384732327754173;
    /// goldens/generate_1b.py: v/p_j of care, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_CARE_REAL_WAGE: f64 = 0.36049009533485673741;
    /// goldens/generate_1b.py: v/p_j of shelter, C3z, (rho, delta, J_b) = (0, 0.1, 1).
    C3Z_SHELTER_REAL_WAGE: f64 = 0.039237077568494895079;

    // C4: the fork economy along the paths: task automation (eta) and recursive automation (lambda)
    /// goldens/generate_1b.py: x*, task automation, eta = 1.
    C4_ETA_1_X_STAR: f64 = 0.71090347785320059226;
    /// goldens/generate_1b.py: v, task automation, eta = 1.
    C4_ETA_1_V: f64 = 0.4647912788060681259;
    /// goldens/generate_1b.py: v/P_s, task automation, eta = 1.
    C4_ETA_1_REAL_WAGE: f64 = 0.25929236034243538841;
    /// goldens/generate_1b.py: v/p of manufactures, task automation, eta = 1.
    C4_ETA_1_MANUFACTURES_REAL_WAGE: f64 = 2.6691763273700016452;
    /// goldens/generate_1b.py: v/p of food, task automation, eta = 1.
    C4_ETA_1_FOOD_REAL_WAGE: f64 = 0.54522104518881160714;
    /// goldens/generate_1b.py: v/p of care, task automation, eta = 1.
    C4_ETA_1_CARE_REAL_WAGE: f64 = 2.1498425814273220769;
    /// goldens/generate_1b.py: v/p of shelter, task automation, eta = 1.
    C4_ETA_1_SHELTER_REAL_WAGE: f64 = 0.44026009590937090225;
    /// goldens/generate_1b.py: x*, task automation, eta = 0.5.
    C4_ETA_0_5_X_STAR: f64 = 0.74133547685661099243;
    /// goldens/generate_1b.py: v, task automation, eta = 0.5.
    C4_ETA_0_5_V: f64 = 0.23319597890872611844;
    /// goldens/generate_1b.py: v/P_s, task automation, eta = 0.5.
    C4_ETA_0_5_REAL_WAGE: f64 = 0.14557993052074431759;
    /// goldens/generate_1b.py: v/p of manufactures, task automation, eta = 0.5.
    C4_ETA_0_5_MANUFACTURES_REAL_WAGE: f64 = 2.7537096579350305345;
    /// goldens/generate_1b.py: v/p of food, task automation, eta = 0.5.
    C4_ETA_0_5_FOOD_REAL_WAGE: f64 = 0.32251992691158850188;
    /// goldens/generate_1b.py: v/p of care, task automation, eta = 0.5.
    C4_ETA_0_5_CARE_REAL_WAGE: f64 = 1.4731361959096763823;
    /// goldens/generate_1b.py: v/p of shelter, task automation, eta = 0.5.
    C4_ETA_0_5_SHELTER_REAL_WAGE: f64 = 0.22702851882045977152;
    /// goldens/generate_1b.py: x*, task automation, eta = 0.25.
    C4_ETA_0_25_X_STAR: f64 = 0.81580469774130705867;
    /// goldens/generate_1b.py: v, task automation, eta = 0.25.
    C4_ETA_0_25_V: f64 = 0.12368952077476531915;
    /// goldens/generate_1b.py: v/P_s, task automation, eta = 0.25.
    C4_ETA_0_25_REAL_WAGE: f64 = 0.081908097335280514043;
    /// goldens/generate_1b.py: v/p of manufactures, task automation, eta = 0.25.
    C4_ETA_0_25_MANUFACTURES_REAL_WAGE: f64 = 2.9605686048369640518;
    /// goldens/generate_1b.py: v/p of food, task automation, eta = 0.25.
    C4_ETA_0_25_FOOD_REAL_WAGE: f64 = 0.18720695492334621575;
    /// goldens/generate_1b.py: v/p of care, task automation, eta = 0.25.
    C4_ETA_0_25_CARE_REAL_WAGE: f64 = 0.94657127911672200244;
    /// goldens/generate_1b.py: v/p of shelter, task automation, eta = 0.25.
    C4_ETA_0_25_SHELTER_REAL_WAGE: f64 = 0.12205350488999499074;
    /// goldens/generate_1b.py: x*, task automation, eta = 0.1.
    C4_ETA_0_1_X_STAR: f64 = 0.91636122404346763349;
    /// goldens/generate_1b.py: v, task automation, eta = 0.1.
    C4_ETA_0_1_V: f64 = 0.053677124048319267321;
    /// goldens/generate_1b.py: v/P_s, task automation, eta = 0.1.
    C4_ETA_0_1_REAL_WAGE: f64 = 0.036869857767907098279;
    /// goldens/generate_1b.py: v/p of manufactures, task automation, eta = 0.1.
    C4_ETA_0_1_MANUFACTURES_REAL_WAGE: f64 = 3.2398922890096323152;
    /// goldens/generate_1b.py: v/p of food, task automation, eta = 0.1.
    C4_ETA_0_1_FOOD_REAL_WAGE: f64 = 0.086010729866266939192;
    /// goldens/generate_1b.py: v/p of care, task automation, eta = 0.1.
    C4_ETA_0_1_CARE_REAL_WAGE: f64 = 0.4759351154494526326;
    /// goldens/generate_1b.py: v/p of shelter, task automation, eta = 0.1.
    C4_ETA_0_1_SHELTER_REAL_WAGE: f64 = 0.053393315781235546928;
    /// goldens/generate_1b.py: x*, task automation, eta = 0.03.
    C4_ETA_0_03_X_STAR: f64 = 0.97320434605992943451;
    /// goldens/generate_1b.py: v, task automation, eta = 0.03.
    C4_ETA_0_03_V: f64 = 0.016810624452634077947;
    /// goldens/generate_1b.py: v/P_s, task automation, eta = 0.03.
    C4_ETA_0_03_REAL_WAGE: f64 = 0.011749802080120372712;
    /// goldens/generate_1b.py: v/p of manufactures, task automation, eta = 0.03.
    C4_ETA_0_03_MANUFACTURES_REAL_WAGE: f64 = 3.3977898501664706514;
    /// goldens/generate_1b.py: v/p of food, task automation, eta = 0.03.
    C4_ETA_0_03_FOOD_REAL_WAGE: f64 = 0.027685966727487033955;
    /// goldens/generate_1b.py: v/p of care, task automation, eta = 0.03.
    C4_ETA_0_03_CARE_REAL_WAGE: f64 = 0.16185801659777499978;
    /// goldens/generate_1b.py: v/p of shelter, task automation, eta = 0.03.
    C4_ETA_0_03_SHELTER_REAL_WAGE: f64 = 0.016783982758397044791;
    /// goldens/generate_1b.py: x*, recursive automation, lambda = 0.025.
    C4_LAM_0_025_X_STAR: f64 = 0.70310159858561335066;
    /// goldens/generate_1b.py: v, recursive automation, lambda = 0.025.
    C4_LAM_0_025_V: f64 = 0.44790058106251196756;
    /// goldens/generate_1b.py: v/P_s, recursive automation, lambda = 0.025.
    C4_LAM_0_025_REAL_WAGE: f64 = 0.25142221680166862341;
    /// goldens/generate_1b.py: v/p of manufactures, recursive automation, lambda = 0.025.
    C4_LAM_0_025_MANUFACTURES_REAL_WAGE: f64 = 2.6475044405155926407;
    /// goldens/generate_1b.py: v/p of food, recursive automation, lambda = 0.025.
    C4_LAM_0_025_FOOD_REAL_WAGE: f64 = 0.53002086949055100003;
    /// goldens/generate_1b.py: v/p of care, recursive automation, lambda = 0.025.
    C4_LAM_0_025_CARE_REAL_WAGE: f64 = 2.1129863149816156787;
    /// goldens/generate_1b.py: v/p of shelter, recursive automation, lambda = 0.025.
    C4_LAM_0_025_SHELTER_REAL_WAGE: f64 = 0.42492434830200469621;
    /// goldens/generate_1b.py: x*, recursive automation, lambda = 0.
    C4_LAM_0_X_STAR: f64 = 0.69555988217240181918;
    /// goldens/generate_1b.py: v, recursive automation, lambda = 0.
    C4_LAM_0_V: f64 = 0.43225594613595511734;
    /// goldens/generate_1b.py: v/P_s, recursive automation, lambda = 0.
    C4_LAM_0_REAL_WAGE: f64 = 0.24405601380617598307;
    /// goldens/generate_1b.py: v/p of manufactures, recursive automation, lambda = 0.
    C4_LAM_0_MANUFACTURES_REAL_WAGE: f64 = 2.62655522825667172;
    /// goldens/generate_1b.py: v/p of food, recursive automation, lambda = 0.
    C4_LAM_0_FOOD_REAL_WAGE: f64 = 0.51574056118281099226;
    /// goldens/generate_1b.py: v/p of care, recursive automation, lambda = 0.
    C4_LAM_0_CARE_REAL_WAGE: f64 = 2.0775144864646863617;
    /// goldens/generate_1b.py: v/p of shelter, recursive automation, lambda = 0.
    C4_LAM_0_SHELTER_REAL_WAGE: f64 = 0.4106831579149892248;

    // C6: check_interior.py's parity instance (:50-58 at 31b3482): (a, lambda, b) = (0.2, 0.1, 0.4), relative capability 0.35 everywhere, human productivity 0.2 + i 0.008125, i = 0..320
    /// goldens/generate_1b.py: v = b gbar / (1 - a - lambda gbar) = 0.14/0.765.
    C6_PARITY_V: f64 = 0.1830065359477124183;
    /// goldens/generate_1b.py: p_m = (lambda v + b)/(1 - a).
    C6_PARITY_P_M: f64 = 0.52287581699346405229;
    /// goldens/generate_1b.py: Lbar = mean of 1/gamma_L over the 321 cells.
    C6_PARITY_L_BAR: f64 = 1.020256901278867244;
    /// goldens/generate_1b.py: v Lbar, the task cost of every land level.
    C6_PARITY_TASK_COST: f64 = 0.18671368127979269825;

    // C7: the gap economy (constructed 2026-09-27): C3 with N 5 and edges (0, 0.4, 0.6, 1), the middle segment used by no category: manufactures (0.3, 0, (2, 0, 0)), food (1, 0.6, (0.5, 0, 0.2)), care (0.2, 0.1, (0, 0, 0.3)), shelter (0.8, 1, (0, 0, 0.1))
    /// goldens/generate_1b.py: x*, the gap economy, N 5.
    C7_GAP_X_STAR: f64 = 0.45534506624697703036;
    /// goldens/generate_1b.py: gamma(x*) = w/p_m, the gap economy, N 5.
    C7_GAP_GAMMA_STAR: f64 = 0.56427605299758162429;
    /// goldens/generate_1b.py: v, the gap economy, N 5.
    C7_GAP_V: f64 = 0.33598549968628947264;
    /// goldens/generate_1b.py: P_s, the gap economy, N 5.
    C7_GAP_P_S: f64 = 1.5600097496109288149;
    /// goldens/generate_1b.py: Y, the gap economy, N 5.
    C7_GAP_Y: f64 = 6.6202617840659756374;
    /// goldens/generate_1b.py: K, the gap economy, N 5.
    C7_GAP_K: f64 = 1.4980706665657864871;
    /// goldens/generate_1b.py: N_a, the gap economy, N 5.
    C7_GAP_N_A: f64 = 0.97525913596126201105;
    /// goldens/generate_1b.py: I, the gap economy, N 5.
    C7_GAP_INCOME: f64 = 10.32767292811956354;
    /// goldens/generate_1b.py: H_s, the gap economy, N 5.
    C7_GAP_H_S: f64 = 0.136;
    /// goldens/generate_1b.py: M_s, the gap economy, N 5.
    C7_GAP_M_S: f64 = 0.1584;
    /// goldens/generate_1b.py: L_s^q = 1.0312/7, the gap economy, N 5.
    C7_GAP_L_S_Q: f64 = 0.14731428571428571429;
    /// goldens/generate_1b.py: B_s^q = 10.5736/7, the gap economy, N 5.
    C7_GAP_B_S_Q: f64 = 1.5105142857142857143;
    /// goldens/generate_1b.py: p_j of manufactures, the gap economy, N 5.
    C7_GAP_MANUFACTURES_P: f64 = 0.17148313027926081201;
    /// goldens/generate_1b.py: v/p_j of manufactures, the gap economy, N 5.
    C7_GAP_MANUFACTURES_REAL_WAGE: f64 = 1.9592918506860473066;
    /// goldens/generate_1b.py: p_j of food, the gap economy, N 5.
    C7_GAP_FOOD_P: f64 = 0.66974962254471836081;
    /// goldens/generate_1b.py: v/p_j of food, the gap economy, N 5.
    C7_GAP_FOOD_REAL_WAGE: f64 = 0.50165836362805285931;
    /// goldens/generate_1b.py: p_j of care, the gap economy, N 5.
    C7_GAP_CARE_P: f64 = 0.14031825996235473672;
    /// goldens/generate_1b.py: v/p_j of care, the gap economy, N 5.
    C7_GAP_CARE_REAL_WAGE: f64 = 2.3944531508331794588;
    /// goldens/generate_1b.py: p_j of shelter, the gap economy, N 5.
    C7_GAP_SHELTER_P: f64 = 1.0134394199874515789;
    /// goldens/generate_1b.py: v/p_j of shelter, the gap economy, N 5.
    C7_GAP_SHELTER_REAL_WAGE: f64 = 0.33152992972234063069;
    /// goldens/generate_1b.py: x*, the gap economy at N 4.5: N_a, Y and K as at N 5.
    C7_GAP_N4_5_X_STAR: f64 = 0.54168278552109559196;

    // C7: roots near an interior edge (constructed 2026-09-27; docs/unit-1b.md section 5.4), every input the double the oracle reads: the gap economy with x* 1e-9 below 0.4 and 1e-9 above 0.6, and the sliver economy (edges (0, 0.5, 1), a service (1, 0.5, (0, 1)) and a site (1, 1, (0, 0)), rho 0.05, delta 0.2, J_b 2) with x* 1e-6 above 0.5
    /// goldens/generate_1b.py: N, a double: the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_N: f64 = 5.3901519305605711807;
    /// goldens/generate_1b.py: x*, the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_X_STAR: f64 = 0.39999999900000002966;
    /// goldens/generate_1b.py: v, the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_V: f64 = 0.30860534075319854122;
    /// goldens/generate_1b.py: P_s, the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_P_S: f64 = 1.5559762610549569488;
    /// goldens/generate_1b.py: Y, the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_Y: f64 = 6.6202617854985205661;
    /// goldens/generate_1b.py: N_a, the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_N_A: f64 = 0.9752591431840993919;
    /// goldens/generate_1b.py: H of manufactures, all of it in the sliver, the gap economy, x* 1e-9 below 0.4.
    C7_EDGE_BELOW_MANUFACTURES_H: f64 = 1.9999999850911775771e-9;
    /// goldens/generate_1b.py: N, a double: the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_N: f64 = 4.2195653430021993202;
    /// goldens/generate_1b.py: x*, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_X_STAR: f64 = 0.60000000099999997125;
    /// goldens/generate_1b.py: v, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_V: f64 = 0.40840840891341794879;
    /// goldens/generate_1b.py: P_s, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_P_S: f64 = 1.570678678753073844;
    /// goldens/generate_1b.py: Y, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_Y: f64 = 6.6202617834869467501;
    /// goldens/generate_1b.py: N_a, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_N_A: f64 = 0.97525913373440280385;
    /// goldens/generate_1b.py: M of care, all of it in the sliver, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_CARE_M: f64 = 2.0399999878451169096e-10;
    /// goldens/generate_1b.py: M of shelter, all of it in the sliver, the gap economy, x* 1e-9 above 0.6.
    C7_EDGE_ABOVE_SHELTER_M: f64 = 6.799999959483723661e-11;
    /// goldens/generate_1b.py: N, a double: the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_N: f64 = 75.810948777333209136;
    /// goldens/generate_1b.py: x*, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_X_STAR: f64 = 0.50000099999999998617;
    /// goldens/generate_1b.py: v, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_V: f64 = 0.068975048281634615191;
    /// goldens/generate_1b.py: P_s, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_P_S: f64 = 1.5344875241407713243;
    /// goldens/generate_1b.py: Y, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_Y: f64 = 6.6666664397161684856;
    /// goldens/generate_1b.py: N_a, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_N_A: f64 = 3.3333265957448630278;
    /// goldens/generate_1b.py: K, all machine use in the sliver, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_K: f64 = 4.2553218408946403243e-6;
    /// goldens/generate_1b.py: M_s, all of it in the sliver, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_M_S: f64 = 6.0000039999170273806e-7;
    /// goldens/generate_1b.py: interest, rho W_K with K in the sliver, the sliver economy, x* 1e-6 above 0.5.
    C7_SLIVER_INTEREST: f64 = 1.1647247542939867354e-7;

    // C7: regimes on the fork economy
    /// goldens/generate_1b.py: f(1) at N = 0.2: BoundaryNoMargin.
    C7_N0_2_F_AT_1: f64 = 0.19212219745448591998;
    /// goldens/generate_1b.py: f(1) at lambda = 0.6: BoundaryNoMargin.
    C7_LAM0_6_F_AT_1: f64 = 0.19428727505460366806;
    /// goldens/generate_1b.py: D(1) at lambda = 0.8: NotViable.
    C7_LAM0_8_D_AT_1: f64 = -0.1;
    /// goldens/generate_1b.py: f(1e-12) at N = 20, chi_max = 0.05: NoInteriorAtZero.
    C7_N20_F_AT_LO: f64 = -12.063380281698479298;
    /// goldens/generate_1b.py: x* at N = 6: Interior, with funded and lemma_b1 both false.
    C7_N6_X_STAR: f64 = 0.67223405154979632125;
    /// goldens/generate_1b.py: funded at N = 6.
    C7_N6_FUNDED: bool = false;
    /// goldens/generate_1b.py: lemma_b1 at N = 6.
    C7_N6_LEMMA_B1: bool = false;
}
