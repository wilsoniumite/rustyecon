"""generate.py: the golden numbers for oracle unit 1a.

Dated 2026-09-25. Every golden that crates/oracle's tests use is computed here with mpmath
at 70 digits, from the equations. The sources are SSRN 7226858 (posted 2026-09-23):
Appendix B, eqs (21)-(25), pp.28-30; Appendix A.4, pp.27-28; and the user cost
u = (rho + delta)(1 + rho)^(J_b - 1) of laborformal dynamics/checks/check_dynamics.py:45
(derived in U3, :47-64) at 31b3482. Nothing is imported from laborformal or from the oracle.
The equations are restated in docs/unit-1a.md section 3.

Adapted from three scratch scripts of 2026-09-25 (golden.py part B, golden2.py and
regimes.py), which checked the same numbers against laborformal's macro.py at 31b3482.
Those scripts are not kept in the repository; this file is the kept source of every
golden. (The round-1 review of the same day re-solved 40 goldens independently at 60
digits and matched them within 4.5e-30.)

Run with any Python that has mpmath (1.3.0 was used):

    python goldens/generate.py           # writes goldens/goldens.txt beside this file
    python goldens/generate.py --check   # exits 1 if goldens.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in
tests/gate/goldens.rs are these values rounded to 20 significant digits, and the test
goldens_file::constants_match_goldens_txt enforces that.

The header of goldens.txt records two FNV-1a 64-bit digests: one of this file and one of
the goldens that follow the header, both with CRLF read as LF. The gate test
goldens_file::goldens_txt_is_from_generate_py recomputes both, so a goldens.txt that was
edited by hand, or that this file no longer writes, fails the gate even where mpmath is
not installed. Only --check proves that the values are this file's output; run it after
every change to either file.
"""

import argparse
import os
import sys
from fractions import Fraction

import mpmath as mp

DPS = 70
"""Working precision in decimal digits. 70, not 50, so that 1 - x* keeps 49 digits at the
deepest point of the automation path, where 1 - x* is about 5e-21."""
SIG_OUT = 30
"""Significant digits written per value; the Rust constants keep 20 of them."""
BISECTIONS = 250
"""Halvings of [1e-12, 1]: 2^-250 < 1e-75, below the working precision."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""
BRACKET_LO = "1e-12"
"""Left end of the root bracket, as in the oracle (BRACKET_LO) and macro.py:102."""
CURVATURE_CEIL = 1024
"""The oracle's CURVATURE_CEIL (src/schedule.rs): the largest k it accepts. G8 solves at it."""

mp.mp.dps = DPS

# The SSRN Appendix B instance (SSRN p.30). Decimal strings, so 0.3 is 0.3 to every digit.
BASE = dict(workers="4", land="10", space="1", a="0.3", lam="0.05", b="0.4",
            eta="1", g0="0.2", g1="0.8", k="1", chi_max="1",
            rho="0", delta="1", build_lag=1)


class Economy:
    """One category, one machine type, one land input (docs/unit-1a.md section 3)."""

    def __init__(self, **changes):
        unknown = set(changes) - set(BASE)
        assert not unknown, f"unknown parameters {unknown}"
        p = dict(BASE)
        p.update(changes)
        num = lambda key: mp.mpf(p[key])  # noqa: E731
        self.N, self.T, self.h = num("workers"), num("land"), num("space")
        self.a, self.lam, self.b = num("a"), num("lam"), num("b")
        self.eta, self.g0, self.g1, self.k = num("eta"), num("g0"), num("g1"), num("k")
        self.chi_max, self.rho, self.delta = num("chi_max"), num("rho"), num("delta")
        self.build_lag = int(p["build_lag"])
        # section 3.0: the user-cost factor
        self.u = (self.rho + self.delta) * (1 + self.rho) ** (self.build_lag - 1)

    def gamma(self, x):
        return self.eta * (self.g0 + self.g1 * x ** self.k)

    def J(self, x):
        return self.eta * (self.g0 * x + self.g1 * x ** (self.k + 1) / (self.k + 1))

    def at(self, x):
        """Sections 3.1 and 3.2 at a candidate threshold x."""
        x = mp.mpf(x)
        g, J = self.gamma(x), self.J(x)
        D = 1 - self.u * (self.a + self.lam * g)
        Vm = self.b / D
        pm = self.u * Vm
        v = g * pm
        p = v * (1 - x) + pm * J
        Ps = p + self.h
        sK = 1 - self.a * self.delta
        Y = self.T / (self.h + self.b * self.delta * J / sK)
        K = Y * J / sK
        final_hours = Y * (1 - x)
        machine_hours = self.lam * self.delta * K
        nD = final_hours + machine_hours
        z = mp.log1p(v / Ps)  # log(1 + z) would lose digits when z = v/Ps is tiny
        nS = self.N * min(max(z / self.chi_max, 0), 1)
        return dict(x=x, gamma=g, J=J, D=D, Vm=Vm, pm=pm, v=v, p=p, Ps=Ps, Y=Y, K=K,
                    final_hours=final_hours, machine_hours=machine_hours, nD=nD, nS=nS,
                    f=nD - nS)

    def solve(self):
        """Section 4: classify, then bisect n_D - n_S on [1e-12, 1]. Returns (regime, values)."""
        # viability first: at D(1) = 0 exactly, at(1) would divide by zero
        d_at_1 = 1 - self.u * (self.a + self.lam * self.gamma(mp.mpf(1)))
        if d_at_1 <= 0:
            return "NotViable", dict(d_at_1=d_at_1)
        one = self.at(1)
        if one["f"] >= 0:
            return "BoundaryNoMargin", dict(f_at_1=one["f"])
        lo, hi = mp.mpf(BRACKET_LO), mp.mpf(1)
        zero = self.at(lo)
        if zero["f"] <= 0:
            return "NoInteriorAtZero", dict(f_at_0=zero["f"])
        for _ in range(BISECTIONS):
            mid = (lo + hi) / 2
            if self.at(mid)["f"] > 0:
                lo = mid
            else:
                hi = mid
        q = self.at((lo + hi) / 2)
        q["one_minus_x"] = 1 - q["x"]
        # section 3.4
        n = q["nD"]
        interest = (self.u - self.delta) * q["Vm"] * q["K"]
        income = q["v"] * n + self.T + interest
        provider_baskets = (self.T + interest) / q["Ps"] - self.N
        q.update(
            n=n, participation=n / self.N, interest=interest, income=income,
            labor_share=q["v"] * n / income, capital_share=interest / income,
            real_wage=q["v"] / q["Ps"], support_cost=self.N * q["Ps"],
            worker_baskets=self.N + q["v"] * n / q["Ps"],
            provider_baskets=provider_baskets,
            funded=provider_baskets > 0,  # T + interest > N P_s, as the oracle computes it
            lemma_b1=one["nS"] > one["nD"] and self.T > self.N * one["Ps"],
            at_one=one, at_lo=zero)
        # section 3.5, the cost-system view, at u = 1
        lam_m, b_m = self.lam / (1 - self.a), self.b / (1 - self.a)
        q.update(lambda_tilde=(1 - q["x"] + q["J"] * lam_m, lam_m, mp.mpf(0)),
                 b_tilde=(q["J"] * b_m, b_m, mp.mpf(1)),
                 phi_w=self.lam * q["gamma"] / (1 - self.a))
        q["L_s"] = q["lambda_tilde"][0] + self.h * q["lambda_tilde"][2]
        q["B_s"] = q["b_tilde"][0] + self.h * q["b_tilde"][2]
        # section 4 step 5: the residuals must vanish at the working precision
        resid = dict(
            labor=abs(q["f"]),
            income=abs(q["Y"] * q["Ps"] - income) / income,
            land=abs(self.h * q["Y"] + self.b * self.delta * q["K"] - self.T) / self.T,
            services=abs(q["K"] - q["Y"] * q["J"] - self.a * self.delta * q["K"]) / q["K"],
            user_cost=abs(q["pm"] - self.u * (self.a * q["pm"] + self.lam * q["v"] + self.b)) / q["pm"])
        bad = {k: v for k, v in resid.items() if v > IDENTITY_TOL}
        assert not bad, f"identities fail at {DPS} digits: {bad}"
        return "Interior", q


def closure(a, lam, gamma_star, b, r, u):
    """Section 3.6 in exact rationals: p_m = u b r / (1 - u(a + lam gamma*)), w = gamma* p_m."""
    a, lam, gamma_star, b, r, u = (Fraction(z) for z in (a, lam, gamma_star, b, r, u))
    d = 1 - u * (a + lam * gamma_star)
    pm = u * b * r / d
    phi_w = lam * gamma_star / (1 - a)
    return dict(d=d, pm=pm, w=gamma_star * pm, phi_w=phi_w, phi_r=1 - phi_w)


def interior(**changes):
    regime, q = Economy(**changes).solve()
    assert regime == "Interior", (changes, regime, q)
    return q


def regime_of(**changes):
    return Economy(**changes).solve()


# ------------------------------------------------------------------------------ output
LINES = []


def section(title):
    LINES.append("")
    LINES.append(f"# {title}")


def put(key, value, note):
    """One golden. mpf and Fraction values are printed to SIG_OUT digits; str values
    (transcribed published numbers) and bools are printed as given."""
    if isinstance(value, bool):
        text = "true" if value else "false"
    elif isinstance(value, str):
        text = value
    else:
        if isinstance(value, Fraction):
            value = mp.mpf(value.numerator) / value.denominator
        text = mp.nstr(value, SIG_OUT, min_fixed=-mp.inf, max_fixed=mp.inf)
    LINES.append(f"{key} = {text}  # {note}")


def published(key, text, computed, tol, note):
    """A number transcribed from the paper; asserted against the 70-digit solve."""
    assert abs(mp.mpf(text) - computed) < mp.mpf(tol), (key, text, computed)
    put(key, text, note)


def build():
    # ------------------------------------------------------------------ G1
    section("G1: the SSRN Appendix B instance (SSRN p.30): N 4, T 10, h 1, a 0.3, b 0.4, "
            "lambda 0.05, gamma = 0.2 + 0.8x, chi ~ U[0, 1], rho 0, delta 1, J_b 1")
    q = interior()
    pub = "published, SSRN p.30; laborformal paths/checks/check_macro.py:31 at 31b3482"
    published("PUB_G1_X_STAR", "0.86315", q["x"], "5e-6", pub)
    published("PUB_G1_V", "0.54344", q["v"], "5e-6", pub)
    published("PUB_G1_Y", "7.88061", q["Y"], "5e-6", pub)
    published("PUB_G1_N_A", "1.34338", q["n"], "5e-6", pub)
    published("PUB_G1_FINAL_HOURS", "1.07846", q["final_hours"], "5e-6", pub)
    published("PUB_G1_MACHINE_HOURS", "0.26492", q["machine_hours"], "5e-6", pub)
    published("PUB_G1_SUPPORT_COST", "5.44630", q["support_cost"], "5e-6", pub)
    put("G1_X_STAR", q["x"], "x*, the root of n_D = n_S (SSRN eq 24, Lemma B.1, p.29)")
    put("G1_GAMMA_STAR", q["gamma"], "gamma(x*)")
    put("G1_J_STAR", q["J"], "J(x*), SSRN eq 21")
    put("G1_V", q["v"], "v = w/r, SSRN eq 21")
    put("G1_P_M", q["pm"], "p_m = V_m at u = 1, SSRN eq 21")
    put("G1_P", q["p"], "p, SSRN eq 22")
    put("G1_P_S", q["Ps"], "P_s, SSRN eq 22")
    put("G1_Y", q["Y"], "Y, SSRN eq 23")
    put("G1_K", q["K"], "K (the SSRN's X), SSRN eq 23")
    put("G1_FINAL_HOURS", q["final_hours"], "Y (1 - x*)")
    put("G1_MACHINE_HOURS", q["machine_hours"], "lambda delta K")
    put("G1_N_A", q["n"], "N_a = n_D(x*), SSRN eq 24")
    put("G1_PARTICIPATION", q["participation"], "N_a / N")
    put("G1_SUPPORT_COST", q["support_cost"], "N P_s")
    put("G1_INCOME", q["income"], "I = v N_a + T, SSRN eq 15 and p.30")
    put("G1_LABOR_SHARE", q["labor_share"], "v N_a / I")
    put("G1_REAL_WAGE", q["real_wage"], "w / P_s")
    put("G1_WORKER_BASKETS", q["worker_baskets"], "N + v N_a / P_s, SSRN p.30")
    put("G1_PROVIDER_BASKETS", q["provider_baskets"], "T / P_s - N, SSRN p.30")
    put("G1_COVERAGE", Economy().T / q["support_cost"], "T / (N P_s)")
    put("G1_FUNDED", q["funded"], "T + interest > N P_s(x*), macro.py:113 at 31b3482")
    put("G1_LEMMA_B1", q["lemma_b1"], "n_S(1) > n_D(1) and T > N P_s(1), SSRN eq 25")
    put("G1_PHI_W", q["phi_w"], "lambda gamma(x*) / (1 - a), check_three_taxes.py:48 at 31b3482")
    put("G1_PHI_R", 1 - q["phi_w"], "1 - phi_w")
    put("G1_LAMBDA_TILDE_GOOD", q["lambda_tilde"][0], "first row of (I - A)^-1 lambda, SSRN eq 4, p.29")
    put("G1_LAMBDA_TILDE_MACHINE", q["lambda_tilde"][1], "lambda / (1 - a) = 1/14")
    put("G1_B_TILDE_GOOD", q["b_tilde"][0], "first row of (I - A)^-1 b, SSRN eq 4, p.29")
    put("G1_B_TILDE_MACHINE", q["b_tilde"][1], "b / (1 - a) = 4/7")
    put("G1_L_S", q["L_s"], "z' lambda-tilde, SSRN p.29")
    put("G1_B_S", q["B_s"], "z' b-tilde, SSRN p.29")
    lo, one = q["at_lo"], q["at_one"]
    put("G1_F_AT_LO", lo["f"], "n_D - n_S at x = 1e-12, the bracket's left end")
    put("G1_N_D_AT_1", one["nD"], "n_D(1)")
    put("G1_N_S_AT_1", one["nS"], "n_S(1)")
    put("G1_F_AT_1", one["f"], "n_D(1) - n_S(1)")
    put("G1_P_S_AT_1", one["Ps"], "P_s(1) = 89/65")
    put("G1_SUPPORT_COST_AT_1", Economy().N * one["Ps"], "N P_s(1)")
    put("G1_V_AT_1", one["v"], "v(1) = 8/13")
    put("G1_D_AT_1", one["D"], "D(1) = 1 - a - lambda gamma(1) = 0.65")
    # the cost system reproduces the solve (section 3.5)
    assert abs(q["v"] * q["L_s"] + q["B_s"] - q["Ps"]) < IDENTITY_TOL
    assert abs(Economy().T / q["B_s"] - q["Y"]) < IDENTITY_TOL
    assert abs(Economy().T * q["L_s"] / q["B_s"] - q["n"]) < IDENTITY_TOL
    base = q

    # ------------------------------------------------------------------ G2
    section("G2: SSRN Figure 3 (SSRN p.12). Captions are published to 2 decimals")
    cap = "published caption, SSRN p.12 (Figure 3)"
    lam0 = interior(lam="0")
    half = interior(eta="0.5")
    published("PUB_G2_BASE_REAL_WAGE", "0.40", base["real_wage"], "5e-3", cap)
    published("PUB_G2_BASE_PARTICIPATION", "0.34", base["participation"], "5e-3", cap)
    published("PUB_G2_LAM0_REAL_WAGE", "0.37", lam0["real_wage"], "5e-3", cap)
    published("PUB_G2_LAM0_PARTICIPATION", "0.32", lam0["participation"], "5e-3", cap)
    published("PUB_G2_HALF_REAL_WAGE", "0.24", half["real_wage"], "5e-3", cap)
    published("PUB_G2_HALF_PARTICIPATION", "0.21", half["participation"], "5e-3", cap)
    for tag, qq, what in (("LAM0", lam0, "lambda = 0 (recursive automation)"),
                          ("HALF", half, "eta = 0.5 (task automation halves gamma)")):
        put(f"G2_{tag}_X_STAR", qq["x"], f"x*, {what}")
        put(f"G2_{tag}_V", qq["v"], f"v, {what}")
        put(f"G2_{tag}_Y", qq["Y"], f"Y, {what}")
        put(f"G2_{tag}_N_A", qq["n"], f"N_a, {what}")
        put(f"G2_{tag}_REAL_WAGE", qq["real_wage"], f"w / P_s, {what}")
        put(f"G2_{tag}_PARTICIPATION", qq["participation"], f"N_a / N, {what}")

    # ------------------------------------------------------------------ G3
    section("G3: the automation path (SSRN p.30; check_macro.py:35-43 at 31b3482): "
            "lambda 0, gamma = eta (1 + x)")
    path = []
    for tag, eta in (("1", "1"), ("0_3", "0.3"), ("0_1", "0.1"), ("0_03", "0.03"), ("0_01", "0.01")):
        qq = interior(lam="0", g0=eta, g1=eta)
        assert qq["v"] <= 2 * mp.mpf("0.4") * mp.mpf(eta) / (1 - mp.mpf("0.3"))
        assert qq["lemma_b1"] and qq["funded"]
        path.append(qq["participation"])
        put(f"G3_ETA_{tag}_PARTICIPATION", qq["participation"], f"N_a / N at eta = {eta}")
        put(f"G3_ETA_{tag}_V", qq["v"], f"v at eta = {eta}")
    assert all(later < earlier for earlier, later in zip(path, path[1:]))
    # A point evaluation, no root: deep along the path z = v/P_s is about 7e-7, where
    # log(1 + z) in f64 loses up to eps/z = 1.6e-10 relative (1.2e-10 at x = 0.25; at
    # x = 0.5 the rounding of 1 + z happens to be nearly exact) and log1p does not.
    deep = Economy(lam="0", g0="1e-6", g1="1e-6").at("0.25")
    assert mp.mpf("1e-7") < deep["v"] / deep["Ps"] < mp.mpf("1e-5")
    put("G3_ETA_1E_6_N_S_AT_QUARTER", deep["nS"],
        "n_S(0.25) at eta = 1e-6, where z = v/P_s is about 7e-7")
    # Near full automation the outputs proportional to 1 - x* need 1 - x* itself, which the
    # double x* cannot carry (docs/unit-1a.md section 4 step 4). At eta = 1e-6, 1 - x* is
    # about 4.6e-7; at eta = 1e-20 it is about 4.6e-21, and x* rounds to 1.0 in f64.
    for tag, eta in (("1E_6", "1e-6"), ("1E_20", "1e-20")):
        qq = interior(lam="0", g0=eta, g1=eta)
        assert qq["lemma_b1"] and qq["funded"] and qq["machine_hours"] == 0
        assert (float(qq["x"]) == 1.0) == (tag == "1E_20"), (tag, qq["x"])
        put(f"G3_ETA_{tag}_ONE_MINUS_X_STAR", qq["one_minus_x"], f"1 - x* at eta = {eta}")
        put(f"G3_ETA_{tag}_N_A", qq["n"], f"N_a = Y (1 - x*) at eta = {eta} (lambda = 0)")
        put(f"G3_ETA_{tag}_PARTICIPATION", qq["participation"], f"N_a / N at eta = {eta}")
        put(f"G3_ETA_{tag}_LABOR_SHARE", qq["labor_share"], f"v N_a / I at eta = {eta}")

    # ------------------------------------------------------------------ G4
    section("G4: durability and interest through u = (rho + delta)(1 + rho)^(J_b - 1) "
            "(check_dynamics.py:45 at 31b3482; SSRN A.4, pp.27-28)")
    cases = (("A", dict(rho="0.05", delta="1", build_lag=1), "(rho, delta, J_b) = (0.05, 1, 1)",
              ("U", "X_STAR", "V", "Y", "N_A", "P_M", "V_M", "INCOME", "CAPITAL_SHARE",
               "LABOR_SHARE", "REAL_WAGE")),
             ("B", dict(rho="0.05", delta="0.1", build_lag=1), "(0.05, 0.1, 1)",
              ("U", "X_STAR", "V", "Y", "N_A", "P_M", "V_M", "K", "INCOME", "CAPITAL_SHARE",
               "LABOR_SHARE", "REAL_WAGE")),
             ("C", dict(rho="0", delta="0.1", build_lag=1), "(0, 0.1, 1)",
              ("U", "X_STAR", "V", "Y", "N_A")),
             ("D", dict(rho="0.05", delta="0.1", build_lag=3), "(0.05, 0.1, 3)",
              ("U", "X_STAR", "V", "Y", "N_A", "INCOME")))
    field = dict(U="u", X_STAR="x", V="v", Y="Y", N_A="n", P_M="pm", V_M="Vm", K="K",
                 INCOME="income", CAPITAL_SHARE="capital_share", LABOR_SHARE="labor_share",
                 REAL_WAGE="real_wage")
    for tag, changes, what, keys in cases:
        qq = interior(**changes)
        qq["u"] = Economy(**changes).u
        for key in keys:
            put(f"G4_{tag}_{key}", qq[field[key]], f"{field[key]} at {what}")
    # nesting: at rho = 0 the durable economy is the flow benchmark with (a, lambda, b)
    # scaled by delta, exactly
    durable = interior(rho="0", delta="0.1")
    scaled = interior(a="0.03", lam="0.005", b="0.04")
    worst = max(abs(durable[k] - scaled[k]) for k in ("x", "v", "Y", "n", "Ps", "pm", "K"))
    assert worst < IDENTITY_TOL, worst
    # interest decides funding: T < N P_s(x*) < T + interest
    fund = interior(workers="7.3", rho="0.05")
    T = Economy().T
    assert T < fund["support_cost"] < T + fund["interest"], fund["support_cost"]
    assert fund["funded"] and fund["provider_baskets"] > 0
    what = "(N, rho, delta, J_b) = (7.3, 0.05, 1, 1)"
    put("G4_E_X_STAR", fund["x"], f"x* at {what}")
    put("G4_E_SUPPORT_COST", fund["support_cost"], f"N P_s at {what}, above T = 10")
    put("G4_E_INTEREST", fund["interest"], f"interest at {what}")
    put("G4_E_PROVIDER_BASKETS", fund["provider_baskets"], f"(T + interest)/P_s - N at {what}")
    put("G4_E_FUNDED", fund["funded"], f"funded at {what}: only through interest")
    put("G4_E_LABOR_SHARE", fund["labor_share"], f"v N_a / I at {what}")
    put("G4_E_REAL_WAGE", fund["real_wage"], f"w / P_s at {what}")
    # u = 1 with delta < 1: the price side is the flow cost system, so phi_w = lambda gamma(x*)
    # / (1 - a) with no delta in it; the clearing side scales the recipe by delta.
    what = "(rho, delta, J_b) = (0.5, 0.5, 1), where u = 1"
    one = interior(rho="0.5", delta="0.5")
    assert Economy(rho="0.5", delta="0.5").u == 1
    put("G4_F_X_STAR", one["x"], f"x* at {what}")
    put("G4_F_Y", one["Y"], f"Y at {what}")
    put("G4_F_PHI_W", one["phi_w"], f"phi_w = lambda gamma(x*) / (1 - a) at {what}")
    put("G4_F_PHI_R", 1 - one["phi_w"], f"phi_r = 1 - phi_w at {what}")
    # Near the viability edge: u a is within 2^-19 of 1 and D(x*) is about 6e-7. Every input
    # is a dyadic rational, so the f64 inputs equal these decimals exactly and the golden
    # measures only the solver's arithmetic. a = 1 - 2^-19, lambda = 2^-20.
    edge = dict(a="0.9999980926513671875", lam="0.00000095367431640625", b="0.375",
                g0="0.5", g1="1", rho="0.5", delta="0.5")
    assert Economy(**edge).u == 1 and Economy(**edge).a == 1 - mp.mpf(2) ** -19
    assert Economy(**edge).lam == mp.mpf(2) ** -20
    e = interior(**edge)
    assert e["D"] < mp.mpf("1e-6"), e["D"]
    what = "a = 1 - 2^-19, lambda = 2^-20, b = 0.375, gamma = 0.5 + x, rho = delta = 0.5"
    put("G4_G_D_STAR", e["D"], f"D(x*) near the viability edge, {what}")
    put("G4_G_X_STAR", e["x"], f"x* at {what}")
    put("G4_G_P_M", e["pm"], f"p_m = u b / D at {what}")
    put("G4_G_V", e["v"], f"v at {what}")
    put("G4_G_P_S", e["Ps"], f"P_s at {what}")
    put("G4_G_INCOME", e["income"], f"I at {what}")

    # ------------------------------------------------------------------ G5
    section("G5: two general instances, every parameter off the paper's values (constructed "
            "on 2026-09-25). Flow: N 5.2, T 12.5, h 0.7, a 0.22, lambda 0.08, b 0.55, "
            "gamma = 2.3 (0.15 + 0.9 x^4.5), chi_max 1.6, rho 0, delta 1, J_b 1. Durable: the "
            "same with k 2.5, rho 0.04, delta 0.35, J_b 2, so u = 0.39 * 1.04 = 0.4056")
    general = dict(workers="5.2", land="12.5", space="0.7", a="0.22", lam="0.08", b="0.55",
                   eta="2.3", g0="0.15", g1="0.9", chi_max="1.6")
    flow = interior(k="4.5", **general)
    durable_changes = dict(k="2.5", rho="0.04", delta="0.35", build_lag=2)
    dur = interior(**durable_changes, **general)
    dur["u"] = Economy(**durable_changes, **general).u
    assert dur["u"] == mp.mpf("0.39") * mp.mpf("1.04")
    common = (("X_STAR", "x", "x*"), ("ONE_MINUS_X_STAR", "one_minus_x", "1 - x*"),
              ("GAMMA_STAR", "gamma", "gamma(x*)"), ("J_STAR", "J", "J(x*)"), ("V", "v", "v"),
              ("P_M", "pm", "p_m"), ("V_M", "Vm", "V_m"), ("P", "p", "p"), ("P_S", "Ps", "P_s"),
              ("Y", "Y", "Y"), ("K", "K", "K"), ("FINAL_HOURS", "final_hours", "Y (1 - x*)"),
              ("MACHINE_HOURS", "machine_hours", "lambda delta K"), ("N_A", "n", "N_a"),
              ("PARTICIPATION", "participation", "N_a / N"), ("INCOME", "income", "I"),
              ("INTEREST", "interest", "interest"), ("LABOR_SHARE", "labor_share", "v N_a / I"),
              ("CAPITAL_SHARE", "capital_share", "interest / I"),
              ("REAL_WAGE", "real_wage", "w / P_s"), ("SUPPORT_COST", "support_cost", "N P_s"),
              ("WORKER_BASKETS", "worker_baskets", "N + v N_a / P_s"),
              ("PROVIDER_BASKETS", "provider_baskets", "(T + interest) / P_s - N"),
              ("FUNDED", "funded", "funded"), ("LEMMA_B1", "lemma_b1", "lemma_b1"))
    for key, name, what in common:
        put(f"G5_FLOW_{key}", flow[name], f"{what}, flow instance")
    put("G5_FLOW_PHI_W", flow["phi_w"], "phi_w = lambda gamma(x*) / (1 - a), flow instance")
    put("G5_FLOW_PHI_R", 1 - flow["phi_w"], "phi_r = 1 - phi_w, flow instance")
    put("G5_FLOW_LAMBDA_TILDE_GOOD", flow["lambda_tilde"][0], "lambda-tilde good, flow instance")
    put("G5_FLOW_LAMBDA_TILDE_MACHINE", flow["lambda_tilde"][1],
        "lambda-tilde machine = lambda / (1 - a), flow instance")
    put("G5_FLOW_B_TILDE_GOOD", flow["b_tilde"][0], "b-tilde good, flow instance")
    put("G5_FLOW_B_TILDE_MACHINE", flow["b_tilde"][1], "b-tilde machine = b / (1 - a), flow instance")
    put("G5_FLOW_L_S", flow["L_s"], "L_s, flow instance")
    put("G5_FLOW_B_S", flow["B_s"], "B_s = h + b J / (1 - a), flow instance")
    assert abs(flow["v"] * flow["L_s"] + flow["B_s"] - flow["Ps"]) < IDENTITY_TOL
    assert abs(Economy(**general).T / flow["B_s"] - flow["Y"]) < IDENTITY_TOL
    put("G5_DURABLE_U", dur["u"], "u = (rho + delta)(1 + rho), durable instance")
    for key, name, what in common:
        put(f"G5_DURABLE_{key}", dur[name], f"{what}, durable instance")

    # ------------------------------------------------------------------ G6 and G7
    section("G6: the replacement closure, price block only (main.tex:325-336, "
            "check_pinning.py:59-73 at 31b3482); exact rationals")
    c = closure("0.5", "0.1", 3, "0.2", 1, 1)
    c0 = closure("0.5", "0", 3, "0.2", 1, 1)
    cnv = closure("0.5", "0.2", 3, "0.2", 1, 1)
    assert c["pm"] == Fraction(1, 2) * c["pm"] + Fraction(1, 10) * c["w"] + Fraction(1, 5)
    put("G6_P_M", c["pm"], "p_m (main.tex c) at (a, lambda, gamma*, b, r, u) = (0.5, 0.1, 3, 0.2, 1, 1)")
    put("G6_W", c["w"], "w = gamma* p_m")
    put("G6_D", c["d"], "D = 1 - u (a + lambda gamma*)")
    put("G6_LAM0_P_M", c0["pm"], "p_m at lambda = 0")
    put("G6_LAM0_W", c0["w"], "w at lambda = 0")
    put("G6_LAM0_WAGE_CUT", 1 - c0["w"] / c["w"], "1 - w(lambda = 0) / w, the 60% cut")
    put("G6_NOT_VIABLE_D", cnv["d"], "D at (a, lambda, gamma*) = (0.5, 0.2, 3): not viable")
    section("G7: three-taxes resolution shares on the G6 instance "
            "(check_three_taxes.py:58-61 at 31b3482; SSRN p.9)")
    put("G7_PHI_W", c["phi_w"], "phi_w = lambda gamma* / (1 - a)")
    put("G7_PHI_R", c["phi_r"], "phi_r = 1 - phi_w")
    put("G7_LAM0_PHI_W", c0["phi_w"], "phi_w at lambda = 0")
    put("G7_LAM0_PHI_R", c0["phi_r"], "phi_r at lambda = 0")

    # ------------------------------------------------------------------ G8
    section("G8: regime recognition (constructed on 2026-09-25; laborformal has no instance)")
    regime, d = regime_of(workers="0.25")
    assert regime == "BoundaryNoMargin"
    put("G8_N025_F_AT_1", d["f_at_1"], "f(1) at N = 0.25: BoundaryNoMargin")
    regime, d = regime_of(lam="0.6")
    assert regime == "BoundaryNoMargin"
    put("G8_LAM06_F_AT_1", d["f_at_1"], "f(1) at lambda = 0.6: BoundaryNoMargin")
    regime, d = regime_of(lam="0.8")
    assert regime == "NotViable"
    put("G8_LAM08_D_AT_1", d["d_at_1"], "D(1) at lambda = 0.8: NotViable")
    regime, d = regime_of(rho="0.1", a="0.6", lam="0.35")
    assert regime == "NotViable" and Economy(a="0.6", lam="0.35").at(1)["D"] > 0
    put("G8_RHO01_D_AT_1", d["d_at_1"],
        "D(1) at rho = 0.1, a = 0.6, lambda = 0.35: NotViable only through u = 1.1")
    regime, d = regime_of(workers="20", chi_max="0.05")
    assert regime == "NoInteriorAtZero"
    put("G8_N20_F_AT_LO", d["f_at_0"], "f(1e-12) at N = 20, chi_max = 0.05: NoInteriorAtZero")
    q8 = interior(workers="8")
    assert not q8["lemma_b1"]
    put("G8_N8_X_STAR", q8["x"], "x* at N = 8: Interior although Lemma B.1 fails")
    put("G8_N8_SUPPORT_COST_AT_1", q8["at_one"]["Ps"] * 8, "N P_s(1) at N = 8, above T = 10")
    put("G8_N8_LEMMA_B1", q8["lemma_b1"], "lemma_b1 at N = 8")
    put("G8_N8_FUNDED", q8["funded"], "funded at x* when N = 8")
    q735 = interior(workers="7.35")
    T = Economy().T
    assert q735["support_cost"] < T < mp.mpf("7.35") * q735["at_one"]["Ps"]
    assert q735["funded"] and not q735["lemma_b1"]
    put("G8_N7_35_X_STAR", q735["x"], "x* at N = 7.35: funded although Lemma B.1 fails")
    put("G8_N7_35_SUPPORT_COST", q735["support_cost"], "N P_s(x*) at N = 7.35, below T = 10")
    put("G8_N7_35_SUPPORT_COST_AT_1", q735["at_one"]["Ps"] * mp.mpf("7.35"),
        "N P_s(1) at N = 7.35, above T = 10")
    put("G8_N7_35_LEMMA_B1", q735["lemma_b1"], "lemma_b1 at N = 7.35")
    put("G8_N7_35_FUNDED", q735["funded"], "funded at x* when N = 7.35")
    regime, d = regime_of(lam="0.7")
    assert regime == "NotViable" and d["d_at_1"] == 0
    put("G8_LAM07_D_AT_1", d["d_at_1"], "D(1) at lambda = 0.7: exactly 0, NotViable since D(1) <= 0")
    # a = 1 - 2^-53 (the double below 1), lambda = 0: s_K = 1 - a = 2^-53, so n_D falls from
    # T/h = 10 to about 0.014 between x = 0 and x = 1e-12, and f(1e-12) < 0 although
    # T/h - N = 6. A root exists in (0, 1e-12), near 3.6e-15, where the bracket does not look.
    near_one = "0.99999999999999988897769753748434595763683319091796875"
    assert mp.mpf(near_one) == 1 - mp.mpf(2) ** -53
    regime, d = regime_of(a=near_one, lam="0")
    assert regime == "NoInteriorAtZero", regime
    assert Economy(a=near_one, lam="0").at("1e-15")["f"] > 0
    put("G8_A_NEAR_1_F_AT_LO", d["f_at_0"],
        "f(1e-12) at a = 1 - 2^-53, lambda = 0: NoInteriorAtZero although T/h > N")
    # The steepest schedule the oracle accepts: k = CURVATURE_CEIL, with N = 1 so that the
    # root lies in the steep part of x^k, near 1 - 2.5/k. This is the review's economy of
    # 2026-09-25 at k = 1e20 (then accepted, and solved wrongly), brought down to the ceiling.
    steep = dict(workers="1", k=str(CURVATURE_CEIL))
    q = interior(**steep)
    assert mp.mpf("1e-4") < q["one_minus_x"] < mp.mpf("1e-2"), q["one_minus_x"]
    assert q["lemma_b1"] and q["funded"]
    what = f"N = 1, k = CURVATURE_CEIL = {CURVATURE_CEIL}"
    for key, name, label in (("X_STAR", "x", "x*"), ("ONE_MINUS_X_STAR", "one_minus_x", "1 - x*"),
                             ("GAMMA_STAR", "gamma", "gamma(x*)"), ("J_STAR", "J", "J(x*)"),
                             ("V", "v", "v"), ("P_M", "pm", "p_m"), ("P_S", "Ps", "P_s"),
                             ("Y", "Y", "Y"), ("K", "K", "K"),
                             ("FINAL_HOURS", "final_hours", "Y (1 - x*)"),
                             ("N_A", "n", "N_a"), ("INCOME", "income", "I"),
                             ("REAL_WAGE", "real_wage", "w / P_s"),
                             ("PROVIDER_BASKETS", "provider_baskets", "T / P_s - N")):
        put(f"G8_K_CEIL_{key}", q[name], f"{label} at {what}")

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens.txt: the golden numbers for oracle unit 1a.",
        f"# Written by goldens/generate.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits;",
        "# PUB_ values are transcribed from the paper and asserted against the solve.",
        "# Equations: docs/unit-1a.md section 3. laborformal references are at 31b3482.",
        f"# fnv1a64 generate.py = {fnv1a64(source_bytes()):016x}",
        f"# fnv1a64 body = {fnv1a64(body.encode('utf-8')):016x}",
    ]
    # The body is everything after the header line that carries its digest.
    return "\n".join(header) + "\n" + body


def fnv1a64(data):
    """FNV-1a, 64 bits, over the bytes with every CRLF read as LF. The Rust gate recomputes it
    (goldens_file.rs), so the digest must not depend on how git checked the files out."""
    h = 0xCBF29CE484222325
    for byte in data.replace(b"\r\n", b"\n"):
        h = ((h ^ byte) * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def source_bytes():
    with open(os.path.abspath(__file__), "rb") as fh:
        return fh.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true",
                        help="compare with goldens.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "goldens.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same = fh.read() == text
        print("goldens.txt is current" if same else "goldens.txt differs from generate.py's output")
        return 0 if same else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
