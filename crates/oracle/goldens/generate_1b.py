"""generate_1b.py: the golden numbers for oracle unit 1b.

Dated 2026-09-27. Every golden that crates/oracle's unit-1b tests use is computed here with
mpmath at 70 digits, from the equations of docs/unit-1b.md section 4: the category form of
SSRN 7226858's cost system (eq 2-4, 7, 10-13, 19-20, 26 and Appendix C) and main.tex's
section 6 (main.tex:441-511 at laborformal 31b3482). Nothing is imported from laborformal
or from the oracle. The unit-1a generator, generate.py, is imported only to assert that the
category form nests it (docs/unit-1b.md section 7).

Run with any Python that has mpmath (1.3.0 was used), from this directory or any other:

    python goldens/generate_1b.py           # writes goldens/goldens_1b.txt beside this file
    python goldens/generate_1b.py --check   # exits 1 if goldens_1b.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in
tests/gate/goldens_1b.rs are these values rounded to 20 significant digits, and the test
goldens_file::constants_match_goldens_1b_txt enforces that.

The header of goldens_1b.txt records three FNV-1a 64-bit digests: of this file, of
generate.py (whose solve the nesting assertions use) and of the goldens that follow the
header, each with CRLF read as LF. The gate recomputes all three.
"""

import argparse
import os
import sys
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import mpmath as mp  # noqa: E402

import generate as g1a  # noqa: E402  (sets mp.mp.dps to 70 as well)

DPS = 70
"""Working precision in decimal digits, as in generate.py."""
SIG_OUT = 30
"""Significant digits written per value; the Rust constants keep 20 of them."""
BISECTIONS = 250
"""Halvings of [1e-12, 1]: 2^-250 < 1e-75, below the working precision."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""
QUADRATURE_TOL = mp.mpf(10) ** -60
"""mpmath's quadrature of main.tex's integrand must match the closed form to 1e-60."""
BRACKET_LO = "1e-12"
"""Left end of the root bracket, as in the oracle and macro.py:102."""
GRID = 100
"""Intervals of the grid on which monotonicity and single crossing are asserted."""

mp.mp.dps = DPS
assert g1a.DPS == DPS and g1a.BRACKET_LO == BRACKET_LO

# 1a's scalars at the SSRN Appendix B instance (SSRN p.30), without space: space is a category.
SCALARS = dict(workers="4", land="10", a="0.3", lam="0.05", b="0.4",
               eta="1", g0="0.2", g1="0.8", k="1", chi_max="1",
               rho="0", delta="1", build_lag=1)

# The fork economy (docs/unit-1b.md section 3.3): edges and (name, z, b, densities).
FORK_EDGES = ("0", "0.4", "0.75", "1")
FORK = (("MANUFACTURES", "0.3", "0", ("2", "0", "0")),
        ("FOOD", "1", "0.6", ("0.5", "1.5", "0")),
        ("CARE", "0.2", "0.1", ("0", "0", "1")),
        ("SHELTER", "0.8", "1", ("0", "0.4", "0")))

# The gap economy: N 5, the middle segment [0.4, 0.6) used by no category.
GAP_EDGES = ("0", "0.4", "0.6", "1")
GAP = (("MANUFACTURES", "0.3", "0", ("2", "0", "0")),
       ("FOOD", "1", "0.6", ("0.5", "0", "0.2")),
       ("CARE", "0.2", "0.1", ("0", "0", "0.3")),
       ("SHELTER", "0.8", "1", ("0", "0", "0.1")))


def appendix_b_form(space="1"):
    """1a's economy as two categories on one segment: the good (z 1, b 0, mu 1) and space
    (z = h, b 1, mu 0), docs/unit-1b.md section 3.3."""
    return ("0", "1"), (("GOOD", "1", "0", ("1",)), ("SPACE", space, "1", ("0",)))


class Economy:
    """Categories on one task line (docs/unit-1b.md sections 2-4)."""

    def __init__(self, edges, categories, **changes):
        unknown = set(changes) - set(SCALARS)
        assert not unknown, f"unknown parameters {unknown}"
        p = dict(SCALARS)
        p.update(changes)
        num = lambda key: mp.mpf(p[key])  # noqa: E731
        self.N, self.T = num("workers"), num("land")
        self.a, self.lam, self.b = num("a"), num("lam"), num("b")
        self.eta, self.g0, self.g1, self.k = num("eta"), num("g0"), num("g1"), num("k")
        self.chi_max, self.rho, self.delta = num("chi_max"), num("rho"), num("delta")
        self.build_lag = int(p["build_lag"])
        self.u = (self.rho + self.delta) * (1 + self.rho) ** (self.build_lag - 1)
        self.e = [mp.mpf(x) for x in edges]
        assert self.e[0] == 0 and self.e[-1] == 1
        assert all(lo < hi for lo, hi in zip(self.e, self.e[1:]))
        self.names = [c[0] for c in categories]
        self.z = [mp.mpf(c[1]) for c in categories]
        self.bj = [mp.mpf(c[2]) for c in categories]
        self.mu = [[mp.mpf(m) for m in c[3]] for c in categories]
        assert all(len(m) == len(self.e) - 1 for m in self.mu)
        # section 4.1: the all-human hours, and section 3.2's basket conditions
        self.Lbar = [sum(m * (hi - lo) for m, lo, hi in zip(mu, self.e, self.e[1:]))
                     for mu in self.mu]
        assert all(L > 0 or b > 0 for L, b in zip(self.Lbar, self.bj))
        self.Bd = sum(z * b for z, b in zip(self.z, self.bj))
        self.Lbar_s = sum(z * L for z, L in zip(self.z, self.Lbar))
        assert self.Bd > 0 and self.Lbar_s > 0

    def gamma(self, x):
        return self.eta * (self.g0 + self.g1 * x ** self.k)

    def J(self, x):
        return self.eta * (self.g0 * x + self.g1 * x ** (self.k + 1) / (self.k + 1))

    def tasks(self, j, x):
        """(H_j, M_j) at threshold x: section 4.1."""
        H = M = mp.mpf(0)
        for m, lo, hi in zip(self.mu[j], self.e, self.e[1:]):
            if x >= hi:
                M += m * (self.J(hi) - self.J(lo))
            elif x <= lo:
                H += m * (hi - lo)
            else:
                H += m * (hi - x)
                M += m * (self.J(x) - self.J(lo))
        return H, M

    def density_at(self, x):
        """mu_s(x) = sum_j z_j mu_{j, s(x)}, with s(x) the segment [e_{s-1}, e_s) holding x
        (the last segment closed at 1)."""
        s = max(i for i in range(len(self.e) - 1) if self.e[i] <= x)
        return sum(z * mu[s] for z, mu in zip(self.z, self.mu))

    def at(self, x):
        """Sections 4.1-4.3 at a candidate threshold x."""
        x = mp.mpf(x)
        g, J = self.gamma(x), self.J(x)
        D = 1 - self.u * (self.a + self.lam * g)
        Vm = self.b / D
        pm = self.u * Vm
        v = g * pm
        H, M = zip(*(self.tasks(j, x) for j in range(len(self.z))))
        p = [v * h + pm * m + b for h, m, b in zip(H, M, self.bj)]
        Ps = sum(z * pj for z, pj in zip(self.z, p))
        Hs = sum(z * h for z, h in zip(self.z, H))
        Ms = sum(z * m for z, m in zip(self.z, M))
        sK = 1 - self.a * self.delta
        Y = self.T / (self.Bd + self.b * self.delta * Ms / sK)
        K = Y * Ms / sK
        final_hours = Y * Hs
        machine_hours = self.lam * self.delta * K
        nD = final_hours + machine_hours
        nS = self.N * min(max(mp.log1p(v / Ps) / self.chi_max, 0), 1)
        return dict(x=x, gamma=g, J=J, D=D, Vm=Vm, pm=pm, v=v, H=list(H), M=list(M), p=p,
                    Ps=Ps, Hs=Hs, Ms=Ms, Y=Y, K=K, final_hours=final_hours,
                    machine_hours=machine_hours, nD=nD, nS=nS, f=nD - nS)

    def solve(self):
        """Section 5.1: classify, then bisect n_D - n_S on [1e-12, 1]."""
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
        q = self.report(self.at((lo + hi) / 2))
        q.update(at_one=one, at_lo=zero,
                 lemma_b1=one["nS"] > one["nD"] and self.T > self.N * one["Ps"])
        return "Interior", q

    def report(self, q):
        """Sections 4.2-4.4 at x*: every output, and every identity and bound asserted."""
        v, pm, g, Y = q["v"], q["pm"], q["gamma"], q["Y"]
        u, a, lam, b, delta = self.u, self.a, self.lam, self.b, self.delta
        sK = 1 - a * delta
        q["one_minus_x"] = 1 - q["x"]
        n = q["nD"]
        interest = (u - delta) * q["Vm"] * q["K"]
        income = v * n + self.T + interest
        provider_baskets = (self.T + interest) / q["Ps"] - self.N
        q.update(n=n, participation=n / self.N, interest=interest, income=income,
                 labor_share=v * n / income, capital_share=interest / income,
                 real_wage=v / q["Ps"], support_cost=self.N * q["Ps"],
                 worker_baskets=self.N + v * n / q["Ps"], provider_baskets=provider_baskets,
                 funded=provider_baskets > 0)
        # section 4.2: the machine row's totals, price side (scaled by u) and clearing side
        lt_m, bt_m = u * lam / (1 - u * a), u * b / (1 - u * a)
        lq_m, bq_m = lam * delta / sK, b * delta / sK
        cats = []
        for j in range(len(self.z)):
            H, M, p, bj, z = q["H"][j], q["M"][j], q["p"][j], self.bj[j], self.z[j]
            c = dict(p=p, real_wage=v / p, H=H, M=M, Lbar=self.Lbar[j], L_star=H + M / g,
                     lambda_tilde=H + M * lt_m, b_tilde=bj + M * bt_m,
                     lambda_q=H + M * lq_m, b_q=bj + M * bq_m, share=z * p / q["Ps"],
                     y=z * Y, wage_floor=1 / (self.Lbar[j] + bj / v))
            c["wage_ceiling"] = v / c["b_tilde"] if c["b_tilde"] > 0 else None
            c["phi_w"] = v * c["lambda_tilde"] / p
            c["phi_r"] = c["b_tilde"] / p
            cats.append(c)
        q["cats"] = cats
        dot = lambda key: sum(z * c[key] for z, c in zip(self.z, cats))  # noqa: E731
        q.update(L_star_s=dot("L_star"), L_s=dot("lambda_tilde"), B_s=dot("b_tilde"),
                 L_q=dot("lambda_q"), B_q=dot("b_q"), Bd=self.Bd,
                 lambda_tilde_machine=lt_m, b_tilde_machine=bt_m)
        q["rent_ceiling"] = v / q["B_s"]
        q["margin_active"] = self.density_at(q["x"]) > 0
        self.assert_identities(q)
        return q

    def assert_identities(self, q):
        """Every identity of section 4 to 1e-65, and every bound of section 4.2."""
        v, pm, Y, K = q["v"], q["pm"], q["Y"], q["K"]
        u, a, lam, b, delta = self.u, self.a, self.lam, self.b, self.delta
        tol = IDENTITY_TOL
        rel = lambda x, y: abs(x - y) / max(abs(x), abs(y))  # noqa: E731
        checks = dict(
            margin=rel(v, q["gamma"] * pm),
            user_cost=rel(pm, u * (a * pm + lam * v + b)),
            machine_row=rel(pm, v * q["lambda_tilde_machine"] + q["b_tilde_machine"]),
            basket_totals=rel(q["Ps"], v * q["L_s"] + q["B_s"]),
            basket_direct=rel(q["Ps"], v * q["L_star_s"] + q["Bd"]),
            eq11_Y=rel(Y, self.T / q["B_q"]),
            eq11_nD=rel(q["n"], self.T * q["L_q"] / q["B_q"]),
            income=rel(Y * q["Ps"], q["income"]),
            expenditure=rel(sum(c["p"] * c["y"] for c in q["cats"]), q["income"]),
            land=rel(q["Bd"] * Y + b * delta * K, self.T),
            baskets=rel(q["worker_baskets"] + q["provider_baskets"], Y),
        )
        if K > 0:
            checks["services"] = rel(K, Y * q["Ms"] + a * delta * K)
        for j, c in enumerate(q["cats"]):
            bj = self.bj[j]
            checks[f"fork_direct_{j}"] = rel(c["p"], v * c["L_star"] + bj)
            checks[f"fork_totals_{j}"] = rel(c["p"], v * c["lambda_tilde"] + c["b_tilde"])
            if c["L_star"] > 0:
                checks[f"forms_agree_{j}"] = rel(
                    c["L_star"], c["lambda_tilde"] + (c["b_tilde"] - bj) / v)
            # the bounds, with a relative slack at the working precision
            slack = 1 + tol
            assert bj <= c["b_q"] * slack and c["b_q"] <= c["b_tilde"] * slack, (j, c)
            assert c["b_tilde"] <= c["p"] * slack, (j, c)
            assert c["p"] <= (v * c["Lbar"] + bj) * slack, (j, c)
            assert c["L_star"] <= c["Lbar"] * slack, (j, c)
            assert c["wage_floor"] <= c["real_wage"] * slack, (j, c)
            if c["wage_ceiling"] is not None:
                assert c["real_wage"] <= c["wage_ceiling"] * slack, (j, c)
            if bj == 0:
                assert c["real_wage"] * slack >= 1 / c["Lbar"], (j, c)
        assert q["real_wage"] <= q["rent_ceiling"] * (1 + tol)
        bad = {k: x for k, x in checks.items() if x > tol}
        assert not bad, f"identities fail at {DPS} digits: {bad}"

    def quadrature_L_star(self, j, x):
        """main.tex eq effective-hours (:450) by mpmath quadrature: the integral over the task
        line of mu_j(t) min{1, gamma(t)/gamma(x)}, split at the edges and at x."""
        x = mp.mpf(x)
        gx = self.gamma(x)
        total = mp.mpf(0)
        for m, lo, hi in zip(self.mu[j], self.e, self.e[1:]):
            if m == 0:
                continue
            points = [lo] + ([x] if lo < x < hi else []) + [hi]
            total += m * mp.quad(lambda t: min(1, self.gamma(t) / gx), points)
        return total

    def assert_single_crossing(self):
        """n_D nonincreasing, v/P_s strictly increasing and f single-crossing on the grid
        {1e-12, 1/GRID, ..., 1} (section 5.3)."""
        grid = [self.at(BRACKET_LO)] + [self.at(mp.mpf(i) / GRID) for i in range(1, GRID + 1)]
        for lo, hi in zip(grid, grid[1:]):
            assert hi["nD"] <= lo["nD"], (lo["x"], hi["x"])
            assert hi["v"] / hi["Ps"] > lo["v"] / lo["Ps"], (lo["x"], hi["x"])
        signs = [q["f"] > 0 for q in grid]
        assert signs[0] and not signs[-1]
        assert sum(1 for s, t in zip(signs, signs[1:]) if s != t) == 1


def interior(edges, categories, **changes):
    regime, q = Economy(edges, categories, **changes).solve()
    assert regime == "Interior", (changes, regime, q)
    return q


def as_doubles(edges, categories, **changes):
    """The economy the oracle solves when it reads these decimals: every input the double
    nearest its decimal, held exactly (mpf of a Python float is exact). Elsewhere the
    difference is about 1e-16 relative and the goldens take the decimals; near an interior
    edge it is not: the double 0.4 is 0.4 + 2.2e-17, which is 2.2e-8 of a 1e-9 sliver."""
    p = dict(SCALARS)
    p.update(changes)
    scalars = {k: (v if k == "build_lag" else float(v)) for k, v in p.items()}
    cats = [(n, float(z), float(b), tuple(float(m) for m in mu)) for n, z, b, mu in categories]
    return Economy([float(e) for e in edges], cats, **scalars)


def workers_near(economy, x):
    """The double N nearest n_D(x)/F(x): with it the root lies at x, to within what one
    rounding of N moves it."""
    q = economy.at(x)
    return float(q["nD"] / min(max(mp.log1p(q["v"] / q["Ps"]) / economy.chi_max, 0), 1))


def ces_share(alpha, sigma, q):
    """SSRN eq 26 (p.31): the expenditure share of a directly rented service with CES weight
    alpha, elasticity sigma and relative price q = r/p."""
    alpha, sigma, q = mp.mpf(alpha), mp.mpf(sigma), mp.mpf(q)
    first = alpha ** sigma * q ** (1 - sigma) / ((1 - alpha) ** sigma + alpha ** sigma * q ** (1 - sigma))
    second = 1 / (1 + ((1 - alpha) / alpha) ** sigma * q ** (sigma - 1))
    assert abs(first - second) < IDENTITY_TOL * max(first, mp.mpf(10) ** -300), (first, second)
    return second


# ------------------------------------------------------------------------------ output
LINES = []


def section(title):
    LINES.append("")
    LINES.append(f"# {title}")


def put(key, value, note):
    """One golden. mpf and Fraction values are printed to SIG_OUT digits; bools as given."""
    if isinstance(value, bool):
        text = "true" if value else "false"
    else:
        if isinstance(value, Fraction):
            value = mp.mpf(value.numerator) / value.denominator
        text = mp.nstr(value, SIG_OUT, min_fixed=-mp.inf, max_fixed=mp.inf)
    LINES.append(f"{key} = {text}  # {note}")


def assert_nests(changes, label):
    """The category form of a 1a instance equals generate.py's 1a solve to 1e-65."""
    space = changes.pop("space", "1")
    edges, cats = appendix_b_form(space)
    q = interior(edges, cats, **changes)
    regime, r = g1a.Economy(space=space, **changes).solve()
    assert regime == "Interior", (label, regime)
    good, space = q["cats"]
    pairs = dict(x=(q["x"], r["x"]), v=(q["v"], r["v"]), pm=(q["pm"], r["pm"]),
                 Vm=(q["Vm"], r["Vm"]), J=(q["Ms"], r["J"]), p=(good["p"], r["p"]),
                 Ps=(q["Ps"], r["Ps"]), Y=(q["Y"], r["Y"]), K=(q["K"], r["K"]),
                 final_hours=(q["final_hours"], r["final_hours"]),
                 machine_hours=(q["machine_hours"], r["machine_hours"]), n=(q["n"], r["n"]),
                 income=(q["income"], r["income"]), interest=(q["interest"], r["interest"]),
                 real_wage=(q["real_wage"], r["real_wage"]),
                 provider_baskets=(q["provider_baskets"], r["provider_baskets"]))
    worst = max(abs(x - y) / max(abs(y), mp.mpf(10) ** -300) for x, y in pairs.values())
    assert worst < IDENTITY_TOL, (label, worst)
    assert q["lemma_b1"] == r["lemma_b1"] and q["funded"] == r["funded"], label
    # space is priced at r = 1, and the good's totals are 1a's cost-system rows (at u = 1)
    assert space["p"] == 1 and space["real_wage"] == q["v"], label
    if q["cats"] and Economy(edges, cats, **changes).u == 1:
        assert abs(good["lambda_tilde"] - r["lambda_tilde"][0]) < IDENTITY_TOL, label
        assert abs(good["b_tilde"] - r["b_tilde"][0]) < IDENTITY_TOL, label
        assert abs(q["L_s"] - r["L_s"]) < IDENTITY_TOL and abs(q["B_s"] - r["B_s"]) < IDENTITY_TOL
    return worst


def build():
    # ------------------------------------------------------------------ nesting (C1)
    worst = mp.mpf(0)
    for changes, label in ((dict(), "G1"), (dict(lam="0"), "lambda 0"), (dict(eta="0.5"), "eta 0.5"),
                           (dict(rho="0.05", delta="0.1", build_lag=1), "G4 B"),
                           (dict(rho="0.05", delta="0.1", build_lag=3), "G4 D"),
                           (dict(workers="5.2", land="12.5", space="0.7", a="0.22", lam="0.08",
                                 b="0.55", eta="2.3", g0="0.15", g1="0.9", chi_max="1.6",
                                 k="2.5", rho="0.04", delta="0.35", build_lag=2), "G5 durable")):
        worst = max(worst, assert_nests(dict(changes), label))

    # ------------------------------------------------------------------ C1
    section("C1: the fork at G1, SSRN Appendix B in category form (docs/unit-1b.md section 3.3): "
            "the good (z 1, b 0, mu 1) and space (z = h = 1, b 1, mu 0) on one segment")
    edges, cats = appendix_b_form()
    q = interior(edges, cats)
    good, space = q["cats"]
    assert good["Lbar"] == 1 and good["wage_floor"] < good["real_wage"] < good["wage_ceiling"]
    put("C1_GOOD_L_STAR", good["L_star"], "L* = (1 - x*) + J(x*)/gamma(x*), main.tex eq effective-hours")
    put("C1_GOOD_LAMBDA_TILDE", good["lambda_tilde"], "lambda-tilde of the good = 1a's L_s, SSRN eq 4")
    put("C1_GOOD_B_TILDE", good["b_tilde"], "b-tilde of the good, SSRN eq 4")
    put("C1_GOOD_P", good["p"], "p of the good = 1a's p, SSRN eq 12")
    put("C1_GOOD_REAL_WAGE", good["real_wage"], "v/p of the good, SSRN eq 12")
    put("C1_GOOD_WAGE_CEILING", good["wage_ceiling"], "v/b-tilde of the good, SSRN eq 13's ceiling")
    put("C1_RENT_CEILING", q["rent_ceiling"], "v/B_s, the basket's rent ceiling (check_macro C3)")
    put("C1_REAL_WAGE", q["real_wage"], "v/P_s")
    put("C1_GOOD_SHARE", good["share"], "p/P_s, the good's expenditure share")

    # ------------------------------------------------------------------ C2
    section("C2: the fork along G3's path (lambda 0, gamma = eta (1 + x)), with the CES share of "
            "SSRN eq 26 at alpha 0.3 (parameters constructed)")
    alpha = "0.3"
    last_v, last_wage = mp.inf, mp.mpf(0)
    for tag, eta in (("1", "1"), ("0_3", "0.3"), ("0_1", "0.1"), ("0_03", "0.03"), ("0_01", "0.01")):
        qq = interior(edges, cats, lam="0", g0=eta, g1=eta)
        gd = qq["cats"][0]
        q_rel = 1 / gd["p"]
        assert gd["real_wage"] >= 1 and q_rel >= 1 / (qq["v"] * gd["Lbar"])
        assert qq["v"] < last_v and gd["real_wage"] > last_wage
        last_v, last_wage = qq["v"], gd["real_wage"]
        put(f"C2_ETA_{tag}_GOOD_REAL_WAGE", gd["real_wage"], f"v/p of the good at eta = {eta}")
        put(f"C2_ETA_{tag}_V", qq["v"], f"v at eta = {eta} (1a's G3)")
        put(f"C2_ETA_{tag}_Q", q_rel, f"q = r/p of the good at eta = {eta}")
        put(f"C2_ETA_{tag}_CES_HALF", ces_share(alpha, "0.5", q_rel), f"alpha(q), sigma 0.5, at eta = {eta}")
        put(f"C2_ETA_{tag}_CES_TWO", ces_share(alpha, "2", q_rel), f"alpha(q), sigma 2, at eta = {eta}")
    limit = mp.mpf(4) / 3
    assert abs(limit - 2 / (1 - 1 + mp.mpf("1.5"))) < IDENTITY_TOL
    assert last_wage < limit
    put("C2_LIMIT_GOOD_REAL_WAGE", limit, "gamma(1)/J(1) = 2/(3/2), the limit of v/p as eta -> 0")
    put("C2_Q1_CES_ZERO", ces_share(alpha, "0", 1), "alpha(1) at sigma 0 = 1/2")
    put("C2_Q1_CES_HALF", ces_share(alpha, "0.5", 1), "alpha(1) at sigma 0.5 = 1/(1 + sqrt(7/3))")
    put("C2_Q1_CES_ONE", ces_share(alpha, "1", 1), "alpha(1) at sigma 1 = alpha")
    put("C2_Q1_CES_TWO", ces_share(alpha, "2", 1), "alpha(1) at sigma 2 = 9/58")
    assert ces_share(alpha, "2", 1) == mp.mpf(9) / 58 or abs(ces_share(alpha, "2", 1) - mp.mpf(9) / 58) < IDENTITY_TOL

    # ------------------------------------------------------------------ C3
    section("C3: the fork economy (constructed 2026-09-27): G1's scalars, edges (0, 0.4, 0.75, 1); "
            "manufactures (z 0.3, b 0, mu (2, 0, 0)), food (1, 0.6, (0.5, 1.5, 0)), care (0.2, 0.1, "
            "(0, 0, 1)), shelter (0.8, 1, (0, 0.4, 0))")
    econ = Economy(FORK_EDGES, FORK)
    assert abs(econ.Bd - mp.mpf("1.42")) < IDENTITY_TOL
    assert abs(econ.Lbar_s - mp.mpf("1.127")) < IDENTITY_TOL
    econ.assert_single_crossing()
    regime, q = econ.solve()
    assert regime == "Interior" and q["margin_active"]
    assert mp.mpf("0.4") < q["x"] < mp.mpf("0.75")
    assert q["lemma_b1"] and q["funded"]
    for j in range(4):
        quad = econ.quadrature_L_star(j, q["x"])
        assert abs(quad - q["cats"][j]["L_star"]) < QUADRATURE_TOL, (j, quad, q["cats"][j]["L_star"])
    what = "C3 flow"
    for key, name, label in (("X_STAR", "x", "x*"), ("ONE_MINUS_X_STAR", "one_minus_x", "1 - x*"),
                             ("GAMMA_STAR", "gamma", "gamma(x*)"), ("V", "v", "v"), ("P_M", "pm", "p_m"),
                             ("P_S", "Ps", "P_s"), ("Y", "Y", "Y"), ("K", "K", "K"),
                             ("M_S", "Ms", "M_s, machine services per basket"),
                             ("H_S", "Hs", "H_s, hours at final tasks per basket"),
                             ("FINAL_HOURS", "final_hours", "Y H_s"),
                             ("MACHINE_HOURS", "machine_hours", "lambda delta K"), ("N_A", "n", "N_a"),
                             ("PARTICIPATION", "participation", "N_a / N"),
                             ("INCOME", "income", "I"), ("LABOR_SHARE", "labor_share", "v N_a / I"),
                             ("REAL_WAGE", "real_wage", "v / P_s"),
                             ("SUPPORT_COST", "support_cost", "N P_s"),
                             ("WORKER_BASKETS", "worker_baskets", "N + v N_a / P_s"),
                             ("PROVIDER_BASKETS", "provider_baskets", "T / P_s - N"),
                             ("L_STAR_S", "L_star_s", "L_s* = sum z_j L_j*"),
                             ("L_S", "L_s", "L_s = sum z_j lambda-tilde_j"),
                             ("B_S", "B_s", "B_s = sum z_j b-tilde_j"),
                             ("RENT_CEILING", "rent_ceiling", "v / B_s"),
                             ("LAMBDA_TILDE_MACHINE", "lambda_tilde_machine", "u lambda / (1 - u a)"),
                             ("B_TILDE_MACHINE", "b_tilde_machine", "u b / (1 - u a)")):
        put(f"C3_{key}", q[name], f"{label}, {what}")
    put("C3_FUNDED", q["funded"], f"funded, {what}")
    put("C3_LEMMA_B1", q["lemma_b1"], f"lemma_b1, {what}")
    lo, one = q["at_lo"], q["at_one"]
    put("C3_F_AT_LO", lo["f"], "n_D - n_S at x = 1e-12, C3 flow")
    put("C3_F_AT_1", one["f"], "n_D(1) - n_S(1), C3 flow")
    put("C3_N_S_AT_1", one["nS"], "n_S(1), C3 flow")
    put("C3_N_D_AT_1", one["nD"], "n_D(1), C3 flow")
    put("C3_P_S_AT_1", one["Ps"], "P_s(1), C3 flow")
    for j, name in enumerate(econ.names):
        c = q["cats"][j]
        for key, field, label in (("P", "p", "p_j"), ("REAL_WAGE", "real_wage", "v/p_j"),
                                  ("L_STAR", "L_star", "L_j*"), ("H", "H", "H_j"), ("M", "M", "M_j"),
                                  ("LAMBDA_TILDE", "lambda_tilde", "lambda-tilde_j"),
                                  ("B_TILDE", "b_tilde", "b-tilde_j"), ("SHARE", "share", "z_j p_j / P_s"),
                                  ("WAGE_FLOOR", "wage_floor", "1/(Lbar_j + b_j/v)"),
                                  ("PHI_W", "phi_w", "v lambda-tilde_j / p_j"),
                                  ("PHI_R", "phi_r", "b-tilde_j / p_j")):
            put(f"C3_{name}_{key}", c[field], f"{label} of {name.lower()}, {what}")
    care = q["cats"][2]
    assert care["L_star"] == care["Lbar"] == mp.mpf("0.25") and care["M"] == 0
    assert q["cats"][0]["H"] == 0 and q["cats"][0]["M"] == 2 * econ.J(mp.mpf("0.4"))
    flow = q

    # C3d: durable, (rho, delta, J_b) = (0.04, 0.35, 2)
    dur = interior(FORK_EDGES, FORK, rho="0.04", delta="0.35", build_lag=2)
    assert Economy(FORK_EDGES, FORK, rho="0.04", delta="0.35", build_lag=2).u == mp.mpf("0.39") * mp.mpf("1.04")
    assert dur["x"] > mp.mpf("0.75"), "C3d's root lies in the top segment"
    what = "C3d, (rho, delta, J_b) = (0.04, 0.35, 2)"
    for key, name, label in (("X_STAR", "x", "x*"), ("ONE_MINUS_X_STAR", "one_minus_x", "1 - x*"),
                             ("V", "v", "v"), ("P_M", "pm", "p_m"), ("P_S", "Ps", "P_s"), ("Y", "Y", "Y"),
                             ("K", "K", "K"), ("N_A", "n", "N_a"), ("INCOME", "income", "I"),
                             ("INTEREST", "interest", "interest"),
                             ("LABOR_SHARE", "labor_share", "v N_a / I"),
                             ("CAPITAL_SHARE", "capital_share", "interest / I"),
                             ("REAL_WAGE", "real_wage", "v / P_s"),
                             ("PROVIDER_BASKETS", "provider_baskets", "(T + interest) / P_s - N"),
                             ("L_S", "L_s", "L_s, price side"), ("B_S", "B_s", "B_s, price side"),
                             ("L_S_Q", "L_q", "L_s^q, clearing side"),
                             ("B_S_Q", "B_q", "B_s^q, clearing side")):
        put(f"C3D_{key}", dur[name], f"{label}, {what}")
    assert abs(dur["L_s"] - dur["L_q"]) > mp.mpf("1e-3") and abs(dur["B_s"] - dur["B_q"]) > mp.mpf("1e-3")
    for j, name in enumerate(econ.names):
        c = dur["cats"][j]
        for key, field, label in (("P", "p", "p_j"), ("REAL_WAGE", "real_wage", "v/p_j"),
                                  ("LAMBDA_TILDE", "lambda_tilde", "lambda-tilde_j"),
                                  ("B_TILDE", "b_tilde", "b-tilde_j"),
                                  ("LAMBDA_Q", "lambda_q", "lambda-tilde_j^q"),
                                  ("B_Q", "b_q", "b-tilde_j^q")):
            put(f"C3D_{name}_{key}", c[field], f"{label} of {name.lower()}, {what}")

    # C3z: rho = 0, delta = 0.1, equal to the flow economy with (a, lambda, b) scaled by delta
    z0 = interior(FORK_EDGES, FORK, delta="0.1")
    scaled = interior(FORK_EDGES, FORK, a="0.03", lam="0.005", b="0.04")
    keys = ("x", "v", "pm", "Ps", "Y", "K", "n", "income", "L_s", "B_s", "L_q", "B_q")
    gap = max(abs(z0[k] - scaled[k]) for k in keys)
    gap = max([gap] + [abs(z0["cats"][j][f] - scaled["cats"][j][f]) for j in range(4)
                       for f in ("p", "L_star", "lambda_tilde", "b_tilde", "lambda_q", "b_q")])
    assert gap < IDENTITY_TOL, gap
    what = "C3z, (rho, delta, J_b) = (0, 0.1, 1)"
    for key, name, label in (("X_STAR", "x", "x*"), ("V", "v", "v"), ("Y", "Y", "Y"), ("N_A", "n", "N_a"),
                             ("P_S", "Ps", "P_s"), ("REAL_WAGE", "real_wage", "v / P_s")):
        put(f"C3Z_{key}", z0[name], f"{label}, {what}")
    for j, name in enumerate(econ.names):
        put(f"C3Z_{name}_REAL_WAGE", z0["cats"][j]["real_wage"], f"v/p_j of {name.lower()}, {what}")

    # ------------------------------------------------------------------ C4
    section("C4: the fork economy along the paths: task automation (eta) and recursive "
            "automation (lambda)")
    rows = []
    for tag, eta in (("1", "1"), ("0_5", "0.5"), ("0_25", "0.25"), ("0_1", "0.1"), ("0_03", "0.03")):
        qq = interior(FORK_EDGES, FORK, eta=eta)
        rows.append(qq)
        what = f"task automation, eta = {eta}"
        put(f"C4_ETA_{tag}_X_STAR", qq["x"], f"x*, {what}")
        put(f"C4_ETA_{tag}_V", qq["v"], f"v, {what}")
        put(f"C4_ETA_{tag}_REAL_WAGE", qq["real_wage"], f"v/P_s, {what}")
        for j, name in enumerate(econ.names):
            put(f"C4_ETA_{tag}_{name}_REAL_WAGE", qq["cats"][j]["real_wage"], f"v/p of {name.lower()}, {what}")
    for earlier, later in zip(rows, rows[1:]):
        assert later["v"] < earlier["v"]
        assert later["cats"][3]["real_wage"] < earlier["cats"][3]["real_wage"]
        assert later["cats"][0]["real_wage"] > earlier["cats"][0]["real_wage"]
    rise = rows[-1]["cats"][0]["real_wage"] / rows[0]["cats"][0]["real_wage"] - 1
    fall = 1 - rows[-1]["cats"][3]["real_wage"] / rows[0]["cats"][3]["real_wage"]
    assert mp.mpf("0.26") < rise < mp.mpf("0.28") and mp.mpf("0.95") < fall < mp.mpf("0.97")
    lam_rows = [flow]
    for tag, lam in (("0_025", "0.025"), ("0", "0")):
        qq = interior(FORK_EDGES, FORK, lam=lam)
        lam_rows.append(qq)
        what = f"recursive automation, lambda = {lam}"
        put(f"C4_LAM_{tag}_X_STAR", qq["x"], f"x*, {what}")
        put(f"C4_LAM_{tag}_V", qq["v"], f"v, {what}")
        put(f"C4_LAM_{tag}_REAL_WAGE", qq["real_wage"], f"v/P_s, {what}")
        for j, name in enumerate(econ.names):
            put(f"C4_LAM_{tag}_{name}_REAL_WAGE", qq["cats"][j]["real_wage"], f"v/p of {name.lower()}, {what}")
    for earlier, later in zip(lam_rows, lam_rows[1:]):
        assert later["x"] < earlier["x"] and later["v"] < earlier["v"]

    # ------------------------------------------------------------------ C6
    section("C6: check_interior.py's parity instance (:50-58 at 31b3482): (a, lambda, b) = (0.2, 0.1, "
            "0.4), relative capability 0.35 everywhere, human productivity 0.2 + i 0.008125, i = 0..320")
    a, lam, b, gbar = Fraction("0.2"), Fraction("0.1"), Fraction("0.4"), Fraction("0.35")
    v = b * gbar / (1 - a - lam * gbar)
    pm = (lam * v + b) / (1 - a)
    assert v == Fraction(14, 100) / Fraction(765, 1000) and v / pm == gbar
    human = [Fraction("0.2") + i * Fraction("0.008125") for i in range(321)]
    assert human[-1] == Fraction("2.8")
    lbar = sum(1 / h for h in human) / 321
    for land in ("0", "0.1", "0.5", "2"):
        cost = Fraction(land) + sum(min(v / h, pm / (h / gbar)) for h in human) / 321
        assert cost == Fraction(land) + v * lbar
    put("C6_PARITY_V", v, "v = b gbar / (1 - a - lambda gbar) = 0.14/0.765")
    put("C6_PARITY_P_M", pm, "p_m = (lambda v + b)/(1 - a)")
    put("C6_PARITY_L_BAR", lbar, "Lbar = mean of 1/gamma_L over the 321 cells")
    put("C6_PARITY_TASK_COST", v * lbar, "v Lbar, the task cost of every land level")

    # ------------------------------------------------------------------ C7
    section("C7: the gap economy (constructed 2026-09-27): C3 with N 5 and edges (0, 0.4, 0.6, 1), "
            "the middle segment used by no category: manufactures (0.3, 0, (2, 0, 0)), food (1, 0.6, "
            "(0.5, 0, 0.2)), care (0.2, 0.1, (0, 0, 0.3)), shelter (0.8, 1, (0, 0, 0.1))")
    gecon = Economy(GAP_EDGES, GAP, workers="5")
    gecon.assert_single_crossing()
    regime, gq = gecon.solve()
    assert regime == "Interior" and not gq["margin_active"]
    assert mp.mpf("0.4") < gq["x"] < mp.mpf("0.6")
    lq, bq = Fraction("1.0312") / 7, Fraction("10.5736") / 7
    assert abs(gq["L_q"] - mp.mpf(lq.numerator) / lq.denominator) < IDENTITY_TOL
    assert abs(gq["B_q"] - mp.mpf(bq.numerator) / bq.denominator) < IDENTITY_TOL
    what = "the gap economy, N 5"
    for key, name, label in (("X_STAR", "x", "x*"), ("GAMMA_STAR", "gamma", "gamma(x*) = w/p_m"),
                             ("V", "v", "v"), ("P_S", "Ps", "P_s"), ("Y", "Y", "Y"), ("K", "K", "K"),
                             ("N_A", "n", "N_a"), ("INCOME", "income", "I"),
                             ("H_S", "Hs", "H_s"), ("M_S", "Ms", "M_s")):
        put(f"C7_GAP_{key}", gq[name], f"{label}, {what}")
    put("C7_GAP_L_S_Q", lq, f"L_s^q = 1.0312/7, {what}")
    put("C7_GAP_B_S_Q", bq, f"B_s^q = 10.5736/7, {what}")
    for j, name in enumerate(gecon.names):
        put(f"C7_GAP_{name}_P", gq["cats"][j]["p"], f"p_j of {name.lower()}, {what}")
        put(f"C7_GAP_{name}_REAL_WAGE", gq["cats"][j]["real_wage"], f"v/p_j of {name.lower()}, {what}")
    g45 = interior(GAP_EDGES, GAP, workers="4.5")
    assert not g45["margin_active"] and mp.mpf("0.4") < g45["x"] < mp.mpf("0.6")
    for k in ("Y", "K", "n", "Hs", "Ms"):
        assert abs(g45[k] - gq[k]) < IDENTITY_TOL, k
    put("C7_GAP_N4_5_X_STAR", g45["x"], "x*, the gap economy at N 4.5: N_a, Y and K as at N 5")
    left = interior(GAP_EDGES, GAP, workers="5.4")
    assert left["x"] < mp.mpf("0.4") and left["margin_active"]

    section("C7: roots near an interior edge (constructed 2026-09-27; docs/unit-1b.md section 5.4), every "
            "input the double the oracle reads: the gap economy with x* 1e-9 below 0.4 and 1e-9 above 0.6, "
            "and the sliver economy (edges (0, 0.5, 1), a service (1, 0.5, (0, 1)) and a site (1, 1, (0, 0)), "
            "rho 0.05, delta 0.2, J_b 2) with x* 1e-6 above 0.5")
    sliver_edges = ("0", "0.5", "1")
    sliver = (("SERVICE", "1", "0.5", ("0", "1")), ("SITE", "1", "1", ("0", "0")))
    sliver_scalars = dict(rho="0.05", delta="0.2", build_lag=2)
    for label, what, edges, cats, scalars, edge, offset, outputs in (
            ("EDGE_BELOW", "the gap economy, x* 1e-9 below 0.4", GAP_EDGES, GAP, dict(), "0.4", "-1e-9",
             (("MANUFACTURES_H", lambda q: q["H"][0], "H of manufactures, all of it in the sliver"),)),
            ("EDGE_ABOVE", "the gap economy, x* 1e-9 above 0.6", GAP_EDGES, GAP, dict(), "0.6", "1e-9",
             (("CARE_M", lambda q: q["M"][2], "M of care, all of it in the sliver"),
              ("SHELTER_M", lambda q: q["M"][3], "M of shelter, all of it in the sliver"))),
            ("SLIVER", "the sliver economy, x* 1e-6 above 0.5", sliver_edges, sliver, sliver_scalars,
             "0.5", "1e-6",
             (("K", lambda q: q["K"], "K, all machine use in the sliver"),
              ("M_S", lambda q: q["Ms"], "M_s, all of it in the sliver"),
              ("INTEREST", lambda q: q["interest"], "interest, rho W_K with K in the sliver")))):
        e = mp.mpf(float(edge))
        target = e + mp.mpf(offset)
        workers = workers_near(as_doubles(edges, cats, workers="1", **scalars), target)
        econ = as_doubles(edges, cats, workers=workers, **scalars)
        regime, q = econ.solve()
        assert regime == "Interior" and q["margin_active"], label
        assert abs(q["x"] - target) < abs(mp.mpf(offset)) / 1000, (label, q["x"])
        put(f"C7_{label}_N", mp.mpf(workers), f"N, a double: {what}")
        for key, name, text in (("X_STAR", "x", "x*"), ("V", "v", "v"), ("P_S", "Ps", "P_s"),
                                ("Y", "Y", "Y"), ("N_A", "n", "N_a")):
            put(f"C7_{label}_{key}", q[name], f"{text}, {what}")
        for key, get, text in outputs:
            put(f"C7_{label}_{key}", get(q), f"{text}, {what}")

    section("C7: regimes on the fork economy")
    regime, d = Economy(FORK_EDGES, FORK, workers="0.2").solve()
    assert regime == "BoundaryNoMargin"
    put("C7_N0_2_F_AT_1", d["f_at_1"], "f(1) at N = 0.2: BoundaryNoMargin")
    regime, d = Economy(FORK_EDGES, FORK, lam="0.6").solve()
    assert regime == "BoundaryNoMargin"
    put("C7_LAM0_6_F_AT_1", d["f_at_1"], "f(1) at lambda = 0.6: BoundaryNoMargin")
    regime, d = Economy(FORK_EDGES, FORK, lam="0.8").solve()
    assert regime == "NotViable"
    put("C7_LAM0_8_D_AT_1", d["d_at_1"], "D(1) at lambda = 0.8: NotViable")
    regime, d = Economy(FORK_EDGES, FORK, workers="20", chi_max="0.05").solve()
    assert regime == "NoInteriorAtZero"
    put("C7_N20_F_AT_LO", d["f_at_0"], "f(1e-12) at N = 20, chi_max = 0.05: NoInteriorAtZero")
    q6 = interior(FORK_EDGES, FORK, workers="6")
    assert not q6["funded"] and not q6["lemma_b1"]
    put("C7_N6_X_STAR", q6["x"], "x* at N = 6: Interior, with funded and lemma_b1 both false")
    put("C7_N6_FUNDED", q6["funded"], "funded at N = 6")
    put("C7_N6_LEMMA_B1", q6["lemma_b1"], "lemma_b1 at N = 6")
    # Viability at the top of the line with no top-segment tasks, at lambda 0.7 (the gap
    # economy without its top segment's tasks): D(1) is 0 exactly for the decimal inputs, and
    # +1.7e-17 for the doubles the oracle reads, where gamma(1) = 0.2 + 0.8 is 1 + 5.6e-17.
    # In f64 D(1) rounds to 0.0, and the regime is NotViable by the convention at zero.
    no_top = tuple((n, z, b, (m[0], m[1], "0")) for n, z, b, m in GAP)
    regime, d = Economy(GAP_EDGES, no_top, workers="5", lam="0.7").solve()
    assert regime == "NotViable" and d["d_at_1"] == 0
    doubles = as_doubles(GAP_EDGES, no_top, workers="5", lam="0.7")
    d_at_1 = 1 - doubles.u * (doubles.a + doubles.lam * doubles.gamma(mp.mpf(1)))
    assert mp.mpf("1.6e-17") < d_at_1 < mp.mpf("1.7e-17"), d_at_1

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens_1b.txt: the golden numbers for oracle unit 1b.",
        f"# Written by goldens/generate_1b.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits.",
        "# Equations: docs/unit-1b.md section 4. laborformal references are at 31b3482.",
        f"# The category form nests generate.py's 1a solves within {mp.nstr(worst, 3)} (1e-65 asserted).",
        f"# fnv1a64 generate_1b.py = {fnv1a64(source_bytes(__file__)):016x}",
        f"# fnv1a64 generate.py = {fnv1a64(source_bytes(g1a.__file__)):016x}",
        f"# fnv1a64 body = {fnv1a64(body.encode('utf-8')):016x}",
    ]
    return "\n".join(header) + "\n" + body


def fnv1a64(data):
    """FNV-1a, 64 bits, over the bytes with every CRLF read as LF (generate.py's)."""
    return g1a.fnv1a64(data)


def source_bytes(path):
    with open(os.path.abspath(path), "rb") as fh:
        return fh.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true",
                        help="compare with goldens_1b.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(HERE, "goldens_1b.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same = fh.read() == text
        print("goldens_1b.txt is current" if same else "goldens_1b.txt differs from generate_1b.py's output")
        return 0 if same else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
