"""generate_1g.py: the golden numbers for oracle unit 1g, machines as goods.

Dated 2026-09-27. Every golden that crates/oracle's unit-1g tests use is computed here with
mpmath at 70 digits from the equations of docs/unit-1g.md section 4: a chain of goods (materials
made by fixed recipes, machine goods held as stocks, the hours of each machine, and the
categories made on the task line) as SSRN 7226858 A.1's p = Ap + lambda w + Br over every
produced good, with each stock entering its own hours' row at the user cost
u = (rho + delta)(1 + rho)^(J - 1) on the price side and at delta on the clearing side (SSRN A.4;
check_dynamics U3, L1-L2 at laborformal 31b3482); D-G10's validation, productivity and the chain
to land on the per-period recipes A^op + Delta A^I (docs/unit-1g.md section 2.1); and a plant,
y = K^(1 - theta) z^theta, at its long run, which is a unit-1c price row p = O + uV with the
operating recipe zeta times the bundle and the build recipe kappa times the plant's recipe
(docs/unit-1g.md section 4.5; D:/rustyecon-loops/capacity/CAPACITY.md, read-only).

Each chain is solved three ways, which must agree to 1e-65:
  (a) DIRECT: one linear system over every produced good, categories, materials, stocks and
      hours, with the task margin as one more row, written here from the equations;
  (b) EMBED: the unit-1c economy the oracle's mapping builds (docs/unit-1g.md section 4.2):
      each material and each machine good a flow type (theta 0, its recipe as the operating
      recipe, no build recipe, delta 1, J 1), in the chain's order, then each machine's hours a
      type built from 1/kappa of its good; solved by generate_1c.py's Economy with D-G10's
      validation in place of its own section 3.2 (Economy below);
  (c) FOLD: the goods folded into the hours rows through their Leontief inverse
      (docs/unit-1g.md section 4.3), one 1c type per machine;
and against the earlier units where the chain is one of their economies: S1 against 1c's M3
(generate_1c.py), S1 at rho 0 against M3z, A0 at rho 0 against 1a's Appendix B (generate.py),
A0's machine in the fork economy against 1b's C3 (generate_1b.py), and the plants at s1 against
the flow economy of the markets probe's L2 solved by generate_1c.py.

Nothing is imported from laborformal, from the oracle or from the design folders outside the
repository; generate_1c.py (and through it generate_1b.py and generate.py) is imported for its
Economy and for the nesting assertions.

Run with any Python that has mpmath (1.3.0 was used), from this directory or any other:

    python goldens/generate_1g.py           # writes goldens/goldens_1g.txt beside this file
    python goldens/generate_1g.py --check   # exits 1 if goldens_1g.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in tests/gate/goldens_1g.rs
are these values rounded to 20 significant digits, and the test
h9_goldens_file::constants_match_goldens_1g_txt enforces that.

The header of goldens_1g.txt records five FNV-1a 64-bit digests: of this file, of
generate_1c.py, generate_1b.py and generate.py, and of the goldens that follow the header, each
with CRLF read as LF. The gate recomputes all five.
"""

import argparse
import os
import sys
from fractions import Fraction

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import mpmath as mp  # noqa: E402

import generate_1c as g1c  # noqa: E402  (imports generate_1b.py and generate.py; 70 digits)

g1b = g1c.g1b
g1a = g1c.g1a

DPS = 70
"""Working precision in decimal digits, as in the other generators."""
SIG_OUT = 30
"""Significant digits written per value; the Rust constants keep 20 of them."""
BISECTIONS = 250
"""Halvings of [1e-12, 1] in the direct solve: 2^-250 < 1e-75, below the working precision."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""
BRACKET_LO = "1e-12"
"""Left end of the root bracket, as in the oracle and the other generators."""
TICKS_PER_YEAR = 52
"""The weekly tick of decision 121: A0, the horse and the plants are per week."""

mp.mp.dps = DPS
assert g1c.DPS == DPS and g1c.BRACKET_LO == BRACKET_LO

M = g1c.M
rel = g1c.rel


def Q(x):
    """An mpf from a decimal string, an int, a Fraction or an mpf (exactly)."""
    if isinstance(x, Fraction):
        return mp.mpf(x.numerator) / mp.mpf(x.denominator)
    return mp.mpf(x)


def fraction_per_tick(delta_year):
    """Clock::fraction: 1 - (1 - delta)^(1/ticks_per_year), the per-tick wear equivalent to an
    annual fraction (crates/core/src/clock.rs; PLAN section 3.9)."""
    return -mp.expm1(mp.log1p(-Q(delta_year)) / TICKS_PER_YEAR)


def compound_per_tick(rho_year):
    """Clock::compound: (1 + rho)^(1/ticks_per_year) - 1."""
    return mp.expm1(mp.log1p(Q(rho_year)) / TICKS_PER_YEAR)


def user_cost(rho, delta, lag):
    """u = (rho + delta)(1 + rho)^(J - 1), check_dynamics U3."""
    return (rho + delta) * (1 + rho) ** (lag - 1)


def spectral_radius(A):
    """The largest modulus of the eigenvalues of a square matrix (mp.eig)."""
    if len(A) == 1:
        return abs(A[0][0])
    values = mp.eig(mp.matrix(A), left=False, right=False)
    return max(abs(e) for e in values)


# ------------------------------------------------------------------------------ D-G10
def reaches_land(A_op, A_build, land_op, land_build):
    """docs/unit-1g.md section 2.1: type k's chain reaches land when its recipes use land, or use
    (operating or build) the service of a type whose chain does. A pattern condition: the same
    for A^op + A^I and for A^op + Delta A^I, since every delta is positive."""
    K = len(A_op)
    reach = [land_op[k] > 0 or land_build[k] > 0 for k in range(K)]
    changed = True
    while changed:
        changed = False
        for k in range(K):
            if not reach[k] and any((A_op[k][l] > 0 or A_build[k][l] > 0) and reach[l] for l in range(K)):
                reach[k] = changed = True
    return reach


class Economy(g1c.Economy):
    """generate_1c.py's Economy with D-G10's validation (docs/unit-1g.md section 2.1): the
    recipes are productive when I - A^q, A^q = A^op + Delta A^I, is a nonsingular M-matrix, and
    every type's chain reaches land as a pattern. The constructor is generate_1c.py's, line for
    line, but for its section 3.2 validation; every other method is inherited, so every solve,
    report and identity is 1c's. The nesting assertion in build() checks that every 1c instance
    gives the same object both ways."""

    def __init__(self, *, workers, land, eta, g0, g1, k, chi_max, rho, edges, categories,
                 intermediate, types):
        self.N, self.T = M(workers), M(land)
        self.eta, self.g0, self.g1, self.k = M(eta), M(g0), M(g1), M(k)
        self.chi_max, self.rho = M(chi_max), M(rho)
        self.e = [M(x) for x in edges]
        assert self.e[0] == 0 and self.e[-1] == 1
        assert all(lo < hi for lo, hi in zip(self.e, self.e[1:]))
        self.names = [c[0] for c in categories]
        self.z = [M(c[1]) for c in categories]
        self.bc = [M(c[2]) for c in categories]
        self.mu = [[M(m) for m in c[3]] for c in categories]
        C = self.C = len(categories)
        assert all(len(m) == len(self.e) - 1 for m in self.mu)
        self.Acc = [[M(a) for a in row] for row in intermediate]
        assert len(self.Acc) == C and all(len(row) == C for row in self.Acc)
        K = self.K = len(types)
        self.tn = [t["name"] for t in types]
        self.theta = [M(t["theta"]) for t in types]
        self.aop = [[M(a) for a in t["op"][0]] for t in types]
        self.lop = [M(t["op"][1]) for t in types]
        self.bop = [M(t["op"][2]) for t in types]
        self.aI = [[M(a) for a in t["build"][0]] for t in types]
        self.lI = [M(t["build"][1]) for t in types]
        self.bI = [M(t["build"][2]) for t in types]
        self.delta = [M(t["delta"]) for t in types]
        self.lag = [int(t["J"]) for t in types]
        assert all(len(a) == K for a in self.aop + self.aI)
        self.u = [(self.rho + d) * (1 + self.rho) ** (j - 1) for d, j in zip(self.delta, self.lag)]
        self.omega = [(1 + self.rho) ** (j - 1) + d * sum((1 + self.rho) ** i for i in range(j - 1))
                      for d, j in zip(self.delta, self.lag)]
        for u, d, w, j in zip(self.u, self.delta, self.omega, self.lag):
            assert rel(u - d, self.rho * w) < IDENTITY_TOL or u == d  # L1-L2
            assert rel(u * (1 + self.rho) / (self.rho + d), (1 + self.rho) ** j) < IDENTITY_TOL  # L3
        # D-G10 (docs/unit-1g.md section 2.1), in place of generate_1c.py's section 3.2 rule
        self.Aq = [[self.aop[i][l] + self.delta[i] * self.aI[i][l] for l in range(K)] for i in range(K)]
        self.lq = [self.lop[i] + self.delta[i] * self.lI[i] for i in range(K)]
        self.bq = [self.bop[i] + self.delta[i] * self.bI[i] for i in range(K)]
        assert g1c.is_m_matrix(self.Aq), "machine recipes are not productive per period"
        assert all(reaches_land(self.aop, self.aI, self.bop, self.bI)), "a machine type's chain reaches no land"
        assert any(t > 0 for t in self.theta)
        assert g1c.is_m_matrix(self.Acc), "intermediate inputs are not productive"
        self.Ahat = [[self.aop[i][l] + self.u[i] * self.aI[i][l] for l in range(K)] for i in range(K)]
        self.lhat = [self.lop[i] + self.u[i] * self.lI[i] for i in range(K)]
        self.bhat = [self.bop[i] + self.u[i] * self.bI[i] for i in range(K)]
        if g1c.is_m_matrix(self.Ahat):
            self.lt = g1c.leontief(self.Ahat, self.lhat)
            self.bt = g1c.leontief(self.Ahat, self.bhat)
        else:
            self.lt = self.bt = None
        self.ltq = g1c.leontief(self.Aq, self.lq)
        self.btq = g1c.leontief(self.Aq, self.bq)
        # in exact arithmetic the pattern is the per-period chain to land
        assert all(b > 0 for b in self.btq)
        self.yhat = g1c.leontief(g1c.transpose(self.Acc), self.z)
        self.Lbar_dir = [sum(m * (hi - lo) for m, lo, hi in zip(mu, self.e, self.e[1:])) for mu in self.mu]
        self.Lbar = g1c.leontief(self.Acc, self.Lbar_dir)
        self.bbar = g1c.leontief(self.Acc, self.bc)
        assert all(L > 0 or b > 0 for L, b in zip(self.Lbar, self.bbar))
        self.B_y = sum(y * b for y, b in zip(self.yhat, self.bc))
        self.chain_hours = sum(y * L for y, L in zip(self.yhat, self.Lbar_dir))
        assert self.B_y > 0 and self.chain_hours > 0
        assert rel(self.B_y, sum(z * b for z, b in zip(self.z, self.bbar))) < IDENTITY_TOL

    def physical(self):
        """A^op + A^I, the matrix of generate_1c.py's (and 1c's) rule."""
        return [[self.aop[i][l] + self.aI[i][l] for l in range(self.K)] for i in range(self.K)]


def economy_kwargs(econ):
    """The keyword arguments an economy was built with by make()."""
    return econ._kwargs


def make(cls, **kw):
    """cls(**kw), keeping kw so that the plants can rewrite its types."""
    e = cls(**kw)
    e._kwargs = kw
    return e


def assert_same_object(label, **kw):
    """D-G10 changes no economy 1c accepts: generate_1c.py's constructor and this one give the
    same attributes, and the same solve."""
    old = g1c.Economy(**kw)
    new = Economy(**kw)
    a, b = vars(old), vars(new)
    assert set(a) == set(b), label
    for key in a:
        assert a[key] == b[key], (label, key)
    ra, qa = old.solve()
    rb, qb = new.solve()
    assert ra == rb, label
    if ra in ("Interior", "Tie"):
        for key in ("x", "v", "Ps", "Y", "N_a", "income", "interest"):
            assert qa[key] == qb[key], (label, key)


# ------------------------------------------------------------------------------ the chain
def recipe(inputs=None, labor="0", land="0"):
    """A recipe over goods by key: {key: coefficient}, hours and land per unit made."""
    return dict(inputs={k: Q(v) for k, v in (inputs or {}).items()}, labor=Q(labor), land=Q(land))


def material(key, rec):
    return dict(key=key, recipe=rec)


def machine(key, build, hours, per_period, theta, operating, delta, lag):
    """A machine good `key` (the stock), built by `build` per unit of stock, and its service
    `hours`: `per_period` hours a period per unit of stock, task efficiency theta, the operating
    recipe per hour, wear delta and lag J per period."""
    return dict(key=key, build=build, hours=hours, kappa=Q(per_period), theta=Q(theta), op=operating,
                delta=Q(delta), J=int(lag))


class Chain:
    """A tape-level chain of goods (docs/unit-1g.md section 3.1): the categories of units 1b and
    1c on one task line, with category inputs; materials; machines. Keys are unique; recipe
    inputs name goods by key; no material, build or operating recipe uses a category (E1), and a
    category's inputs are categories (E2)."""

    def __init__(self, *, workers, land, eta, g0, g1, k, chi_max, rho, edges, categories,
                 materials, machines):
        self.scalars = dict(workers=Q(workers), land=Q(land), eta=Q(eta), g0=Q(g0), g1=Q(g1), k=Q(k),
                            chi_max=Q(chi_max))
        self.rho = Q(rho)
        self.edges = tuple(Q(e) for e in edges)
        self.cats = [dict(key=c[0], z=Q(c[1]), b=Q(c[2]), mu=tuple(Q(m) for m in c[3]),
                          inputs={k2: Q(v) for k2, v in (c[4] if len(c) > 4 else {}).items()})
                     for c in categories]
        self.materials = materials
        self.machines = machines
        keys = ([c["key"] for c in self.cats] + [f["key"] for f in materials] + [m["key"] for m in machines]
                + [m["hours"] for m in machines])
        assert len(keys) == len(set(keys)), "keys are unique"
        self.cat_keys = {c["key"] for c in self.cats}
        # the 1c types in the mapping's order: materials, machine goods, hours
        self.type_keys = [f["key"] for f in materials] + [m["key"] for m in machines] + [m["hours"] for m in machines]
        self.col = {key: i for i, key in enumerate(self.type_keys)}
        for f in materials:
            self._check_machine_side(f["recipe"])
        for m in machines:
            self._check_machine_side(m["build"])
            self._check_machine_side(m["op"])
        for c in self.cats:
            assert all(key in self.cat_keys for key in c["inputs"]), "E2: a category's inputs are categories"
        for m in machines:
            m["u"] = user_cost(self.rho, m["delta"], m["J"])

    def _check_machine_side(self, rec):
        for key in rec["inputs"]:
            assert key not in self.cat_keys, "E1: a machine-side recipe uses a category"
            assert key in self.col, f"unknown good {key}"

    # -------------------------------------------------------------- (b) the embedding
    def embed(self):
        """The unit-1c economy of docs/unit-1g.md section 4.2."""
        K = len(self.type_keys)

        def vec(inputs):
            a = [mp.mpf(0)] * K
            for key, v in inputs.items():
                a[self.col[key]] += v
            return a

        zero = ([mp.mpf(0)] * K, mp.mpf(0), mp.mpf(0))
        types = []
        for f in self.materials:
            r = f["recipe"]
            types.append(g1c.machine_type(f["key"], 0, (vec(r["inputs"]), r["labor"], r["land"]), zero, 1, 1))
        for m in self.machines:
            r = m["build"]
            types.append(g1c.machine_type(m["key"], 0, (vec(r["inputs"]), r["labor"], r["land"]), zero, 1, 1))
        for m in self.machines:
            r = m["op"]
            unit = vec({m["key"]: 1 / m["kappa"]})
            types.append(g1c.machine_type(m["hours"], m["theta"], (vec(r["inputs"]), r["labor"], r["land"]),
                                          (unit, mp.mpf(0), mp.mpf(0)), m["delta"], m["J"]))
        return make(Economy, **self.kwargs(types))

    def kwargs(self, types):
        C = len(self.cats)
        cindex = {c["key"]: j for j, c in enumerate(self.cats)}
        acc = [[mp.mpf(0)] * C for _ in range(C)]
        for j, c in enumerate(self.cats):
            for key, v in c["inputs"].items():
                acc[j][cindex[key]] += v
        cats = tuple((c["key"], c["z"], c["b"], c["mu"]) for c in self.cats)
        return dict(**self.scalars, rho=self.rho, edges=self.edges, categories=cats, intermediate=acc,
                    types=tuple(types))

    # -------------------------------------------------------------- (c) the fold
    def fold(self):
        """docs/unit-1g.md section 4.3: Phi = (I - A_GG)^-1 over the goods G (materials and
        machine goods), then each machine's operating recipe and its build (1/kappa of its good)
        folded through Phi into hours, labour and land."""
        goods = [f["key"] for f in self.materials] + [m["key"] for m in self.machines]
        hours = [m["hours"] for m in self.machines]
        G = len(goods)
        recipes = {f["key"]: f["recipe"] for f in self.materials}
        recipes.update({m["key"]: m["build"] for m in self.machines})
        A_GG = [[recipes[g]["inputs"].get(h, mp.mpf(0)) for h in goods] for g in goods]
        IA = mp.eye(G) - mp.matrix(A_GG) if G else None

        def through(rhs):
            return list(mp.lu_solve(IA, mp.matrix(rhs))) if G else []

        G_H = [through([recipes[g]["inputs"].get(h, mp.mpf(0)) for g in goods]) for h in hours]
        G_l = through([recipes[g]["labor"] for g in goods])
        G_b = through([recipes[g]["land"] for g in goods])

        def folded(inputs, labor, land):
            direct_goods = [inputs.get(g, mp.mpf(0)) for g in goods]
            a = [inputs.get(h, mp.mpf(0)) + sum(direct_goods[i] * G_H[c][i] for i in range(G))
                 for c, h in enumerate(hours)]
            return (a, labor + sum(direct_goods[i] * G_l[i] for i in range(G)),
                    land + sum(direct_goods[i] * G_b[i] for i in range(G)))

        types = []
        for m in self.machines:
            op = folded(m["op"]["inputs"], m["op"]["labor"], m["op"]["land"])
            build = folded({m["key"]: 1 / m["kappa"]}, mp.mpf(0), mp.mpf(0))
            types.append(g1c.machine_type(m["hours"], m["theta"], op, build, m["delta"], m["J"]))
        return make(Economy, **self.kwargs(types))

    # -------------------------------------------------------------- (a) the direct system
    def names(self):
        return ([c["key"] for c in self.cats] + [f["key"] for f in self.materials]
                + [m["key"] for m in self.machines] + [m["hours"] for m in self.machines])

    def gamma(self, x):
        s = self.scalars
        return s["eta"] * (s["g0"] + s["g1"] * x ** s["k"])

    def Jx(self, x):
        s = self.scalars
        return s["eta"] * (s["g0"] * x + s["g1"] * x ** (s["k"] + 1) / (s["k"] + 1))

    def tasks(self, j, x):
        H = Mm = mp.mpf(0)
        for m, lo, hi in zip(self.cats[j]["mu"], self.edges, self.edges[1:]):
            if x >= hi:
                Mm += m * (self.Jx(hi) - self.Jx(lo))
            elif x <= lo:
                H += m * (hi - lo)
            else:
                H += m * (hi - x)
                Mm += m * (self.Jx(x) - self.Jx(lo))
        return H, Mm

    def matrices(self, x, split, side):
        """(A, lambda, b) over every produced good at threshold x, the machine tasks split
        [(machine index, share)]; the stock enters its hours' row at u/kappa (side 'p') or
        delta/kappa (side 'q')."""
        names = self.names()
        idx = {n: i for i, n in enumerate(names)}
        n = len(names)
        A = [[mp.mpf(0)] * n for _ in range(n)]
        lam, b = [mp.mpf(0)] * n, [mp.mpf(0)] * n
        C = len(self.cats)
        for j, c in enumerate(self.cats):
            H, Mt = self.tasks(j, x)
            for key, v in c["inputs"].items():
                A[j][idx[key]] += v
            for m, share in split:
                mm = self.machines[m]
                A[j][idx[mm["hours"]]] += share * Mt / mm["theta"]
            lam[j], b[j] = H, c["b"]
        rows = ([(f["key"], f["recipe"], None) for f in self.materials]
                + [(m["key"], m["build"], None) for m in self.machines]
                + [(m["hours"], m["op"], m) for m in self.machines])
        for key, r, m in rows:
            i = idx[key]
            for k2, v in r["inputs"].items():
                A[i][idx[k2]] += v
            if m is not None:
                A[i][idx[m["key"]]] += (m["u"] if side == "p" else m["delta"]) / m["kappa"]
            lam[i], b[i] = r["labor"], r["land"]
        assert C == len(self.cats)
        return A, lam, b, idx

    def machine_totals(self, side):
        """lambda-tilde and b-tilde over the machine side (materials, stocks, hours): x-free
        when no machine-side row uses a category (E1)."""
        A, lam, b, idx = self.matrices(mp.mpf("0.5"), [], side)
        C = len(self.cats)
        rows = list(range(C, len(A)))
        for i in rows:
            assert all(A[i][j] == 0 for j in range(C)), "E1"
        IA = mp.matrix([[(1 if i == j else 0) - A[i][j] for j in rows] for i in rows])
        lt = list(mp.lu_solve(IA, mp.matrix([lam[i] for i in rows])))
        bt = list(mp.lu_solve(IA, mp.matrix([b[i] for i in rows])))
        names = self.names()
        return {names[i]: (lt[a], bt[a]) for a, i in enumerate(rows)}

    def closure_wage(self, m, g, tot):
        mm = self.machines[m]
        if mm["theta"] == 0:
            return None
        lt, bt = tot[mm["hours"]]
        room = mm["theta"] - g * lt
        return g * bt / room if room > 0 else None

    def technique(self, x, tot):
        g = self.gamma(x)
        best = None
        for m in range(len(self.machines)):
            w = self.closure_wage(m, g, tot)
            if w is not None and (best is None or w < best[1]):
                best = (m, w)
        return best[0]

    def at(self, x, m, split=None):
        """The price system with the task margin as one more row, and the clearing system."""
        x = Q(x)
        split = split or [(m, mp.mpf(1))]
        A, lam, b, idx = self.matrices(x, split, "p")
        n = len(A)
        mm = self.machines[m]
        G = [[(1 if i == j else 0) - A[i][j] for j in range(n)] + [-lam[i]] for i in range(n)]
        row = [mp.mpf(0)] * (n + 1)
        row[idx[mm["hours"]]] = -self.gamma(x) / mm["theta"]
        row[n] = mp.mpf(1)
        G.append(row)
        z = list(mp.lu_solve(mp.matrix(G), mp.matrix(b + [mp.mpf(0)])))
        p, v = z[:n], z[n]
        for i in range(n):
            res = p[i] - (sum(A[i][j] * p[j] for j in range(n)) + lam[i] * v + b[i])
            assert abs(res) <= IDENTITY_TOL * max(abs(p[i]), 1), ("price row", i)
        Aq, lq, bq, _ = self.matrices(x, split, "q")
        f = [c["z"] for c in self.cats] + [mp.mpf(0)] * (n - len(self.cats))
        IAT = mp.matrix([[(1 if i == j else 0) - Aq[j][i] for j in range(n)] for i in range(n)])
        yhat = list(mp.lu_solve(IAT, mp.matrix(f)))
        land = sum(bq[i] * yhat[i] for i in range(n))
        hours = sum(lq[i] * yhat[i] for i in range(n))
        Y = self.scalars["land"] / land
        Ps = sum(c["z"] * p[j] for j, c in enumerate(self.cats))
        nD = Y * hours
        nS = self.scalars["workers"] * min(max(mp.log1p(v / Ps) / self.scalars["chi_max"], 0), 1)
        return dict(x=x, m=m, p=p, v=v, Ps=Ps, Y=Y, N_a=nD, nS=nS, f=nD - nS, y=[Y * yy for yy in yhat],
                    A=A, Aq=Aq, lam=lam, lq=lq, b=b, bq=bq, idx=idx)

    def solve(self):
        """Bisection of n_D - n_S on [1e-12, 1] under the cheapest task type at each x (the
        envelope from the x-free machine totals); an interior root that is not a tie."""
        tot = self.machine_totals("p")
        lo, hi = Q(BRACKET_LO), mp.mpf(1)
        f = lambda x: self.at(x, self.technique(x, tot))["f"]  # noqa: E731
        assert f(lo) > 0 and f(hi) < 0, "not interior"
        for _ in range(BISECTIONS):
            mid = (lo + hi) / 2
            if f(mid) > 0:
                lo = mid
            else:
                hi = mid
        x = (lo + hi) / 2
        m = self.technique(x, tot)
        q = self.at(x, m)
        g = self.gamma(x)
        others = [self.closure_wage(k, g, tot) for k in range(len(self.machines)) if k != m]
        assert all(w is None or w > q["v"] * (1 + mp.mpf(10) ** -20) for w in others), "a tie"
        return self.report(q)

    def report(self, q):
        """Income and its sources, the net outputs, and every good's price and output."""
        n, Y, v, p, y = len(q["p"]), q["Y"], q["v"], q["p"], q["y"]
        T = self.scalars["land"]
        income = Y * q["Ps"]
        interest = income - v * q["N_a"] - T
        idx = q["idx"]
        src = sum((m["u"] - m["delta"]) * p[idx[m["key"]]] / m["kappa"] * y[idx[m["hours"]]] for m in self.machines)
        assert abs(interest - src) <= IDENTITY_TOL * income, ("interest", interest, src)
        f = [y[i] - sum(q["Aq"][l][i] * y[l] for l in range(n)) for i in range(n)]
        for i in range(len(self.cats), n):
            assert abs(f[i]) <= IDENTITY_TOL * max(y), "net output of an intermediate good"
        assert rel(sum(p[i] * f[i] for i in range(n)), income) < IDENTITY_TOL
        assert rel(sum(q["lq"][i] * y[i] for i in range(n)), q["N_a"]) < IDENTITY_TOL
        assert rel(sum(q["bq"][i] * y[i] for i in range(n)), T) < IDENTITY_TOL
        names = self.names()
        out = dict(x=q["x"], one_minus_x=1 - q["x"], v=v, Ps=q["Ps"], Y=Y, N_a=q["N_a"], income=income,
                   interest=interest, technique=q["m"])
        out["price"] = {names[i]: p[i] for i in range(n)}
        out["output"] = {names[i]: y[i] for i in range(n)}
        return out


# ------------------------------------------------------------------------------ 1c readouts
def goods_of(chain, qb):
    """The embedding's report read per good (docs/unit-1g.md section 4.4)."""
    types = {t["name"]: t for t in qb["types"]}
    out = dict(price={}, output={})
    for f in chain.materials:
        t = types[f["key"]]
        out["price"][f["key"]], out["output"][f["key"]] = t["p"], t["X"]
    for m in chain.machines:
        g, h = types[m["key"]], types[m["hours"]]
        out["price"][m["key"]], out["output"][m["key"]] = g["p"], g["X"]
        out["price"][m["hours"]], out["output"][m["hours"]] = h["p"], h["X"]
        # the hours type's V is the good's price per unit of capacity; its builds are the good's
        # output per unit of capacity
        assert rel(h["V"] * m["kappa"], g["p"]) < IDENTITY_TOL
        assert rel(m["delta"] * h["X"] / m["kappa"], g["X"]) < IDENTITY_TOL or m["delta"] * h["X"] == g["X"]
    for j, c in enumerate(chain.cats):
        out["price"][c["key"]], out["output"][c["key"]] = qb["cats"][j]["p"], qb["cats"][j]["gross"]
    return out


AGG = ("x", "one_minus_x", "v", "Ps", "Y", "N_a", "income", "interest")


def agg(q):
    return {k: q[k] for k in AGG}


def same(label, a, b, keys=AGG):
    """Every key to 1e-65 relative; interest, 0 at rho 0 in one form and a remainder of order
    1e-70 of income in the direct form, relative to income."""
    def err(k):
        if k == "interest":
            return abs(a[k] - b[k]) / max(abs(a["income"]), abs(b["income"]))
        return rel(a[k], b[k])
    worst = max(err(k) for k in keys)
    assert worst < IDENTITY_TOL, (label, {k: (a[k], b[k]) for k in keys if err(k) >= IDENTITY_TOL})
    return worst


def interior(econ):
    regime, q = econ.solve()
    assert regime == "Interior", (regime, {k: v for k, v in q.items() if k in ("changes", "f_at_1", "d_at_1", "f_at_0")})
    return q


def three_ways(chain, label, direct=True):
    """(b) the embedding, (c) the fold and (a) the direct system agree on every aggregate, and
    (a) and (b) on every good's price and output; the five numbers of each hours type agree."""
    eb = chain.embed()
    qb = interior(eb)
    worst = mp.mpf(0)
    qc = interior(chain.fold())
    worst = max(worst, same(f"{label}: fold against embedding", agg(qc), agg(qb)))
    goods = goods_of(chain, qb)
    tb = {t["name"]: t for t in qb["types"]}
    for c, m in enumerate(chain.machines):
        tc = qc["types"][c]
        th = tb[m["hours"]]
        for key in ("p", "O", "V", "X", "lt", "bt", "ltq", "btq", "W", "interest"):
            if th[key] is None:
                continue
            assert rel(tc[key], th[key]) < IDENTITY_TOL or tc[key] == th[key], (label, m["hours"], key)
    qa = None
    if direct:
        qa = chain.solve()
        worst = max(worst, same(f"{label}: direct against embedding", qa, agg(qb)))
        for key in qa["price"]:
            assert rel(qa["price"][key], goods["price"][key]) < IDENTITY_TOL, (label, key, "price")
            assert rel(qa["output"][key], goods["output"][key]) < IDENTITY_TOL or qa["output"][key] == goods["output"][key], (label, key, "output")
        totp, totq = chain.machine_totals("p"), chain.machine_totals("q")
        for m in chain.machines:
            th = tb[m["hours"]]
            assert rel(totp[m["hours"]][0], th["lt"]) < IDENTITY_TOL and rel(totp[m["hours"]][1], th["bt"]) < IDENTITY_TOL
            assert rel(totq[m["hours"]][0], th["ltq"]) < IDENTITY_TOL and rel(totq[m["hours"]][1], th["btq"]) < IDENTITY_TOL
    return dict(embed=eb, qb=qb, qc=qc, qa=qa, goods=goods, worst=worst)


# ------------------------------------------------------------------------------ plants
def s1_size(theta, delta):
    """s1 = theta^(theta/(1 - theta)) (1 - theta)/delta: the bundle plant's size at which its long
    run at rho = 0 is the flow economy (docs/unit-1g.md section 4.5)."""
    return theta ** (theta / (1 - theta)) * (1 - theta) / delta


def planted(econ, plants, ratios):
    """The long run's 1c types: for each plant on type k, with ratio kappa/zeta = r, the
    operating recipe zeta times k's flow recipe and the build recipe kappa times the plant's
    recipe R, zeta = r^-(1 - theta), kappa = r^theta, and the plant's delta and J."""
    kw = dict(economy_kwargs(econ))
    types = list(kw["types"])
    info = {}
    for (k, pl), r in zip(plants, ratios):
        t = types[k]
        th = pl["theta"]
        zeta, kappa = r ** (th - 1), r ** th
        a, lam, b = [Q(x) for x in t["op"][0]], Q(t["op"][1]), Q(t["op"][2])
        Ra, Rl, Rb = plant_recipe(t, pl)
        types[k] = g1c.machine_type(t["name"], t["theta"], ([zeta * x for x in a], zeta * lam, zeta * b),
                                    ([kappa * x for x in Ra], kappa * Rl, kappa * Rb), pl["delta"], pl["J"])
        info[k] = dict(zeta=zeta, kappa=kappa, ratio=r)
    kw["types"] = tuple(types)
    return make(Economy, **kw), info


def plant_recipe(t, pl):
    """R per plant unit: s bundles of the type's own flow recipe, or a fixed recipe."""
    if pl["recipe"] == "bundle":
        s = pl["size"]
        return [s * Q(x) for x in t["op"][0]], s * Q(t["op"][1]), s * Q(t["op"][2])
    Ra, Rl, Rb = pl["recipe"]
    return [Q(x) for x in Ra], Q(Rl), Q(Rb)


def new_ratios(econ, plants, prices, v):
    """(1 - theta) c_f/(theta u P_K) at the prices: c_f the bundle's cost, P_K the plant's."""
    out = []
    for k, pl in plants:
        t = economy_kwargs(econ)["types"][k]
        a, lam, b = [Q(x) for x in t["op"][0]], Q(t["op"][1]), Q(t["op"][2])
        cf = sum(a[l] * prices[l] for l in range(len(a))) + lam * v + b
        Ra, Rl, Rb = plant_recipe(t, pl)
        PK = sum(Ra[l] * prices[l] for l in range(len(Ra))) + Rl * v + Rb
        u = user_cost(Q(economy_kwargs(econ)["rho"]), Q(pl["delta"]), pl["J"])
        th = pl["theta"]
        out.append((1 - th) * cf / (th * u * PK))
    return out


def fast_root(econ, t):
    """x* for a one-technique economy: bisection to 1e-8, then the Illinois method."""
    f = lambda x: econ.at(x, t)["f"]  # noqa: E731
    lo, hi = Q(BRACKET_LO), mp.mpf(1)
    assert f(lo) > 0 and f(hi) < 0
    for _ in range(27):
        mid = (lo + hi) / 2
        if f(mid) > 0:
            lo = mid
        else:
            hi = mid
    return mp.findroot(f, (lo, hi), solver="illinois", tol=mp.mpf(10) ** -(2 * DPS - 10), maxsteps=200)


def plant_solve(base, plants):
    """The plants' long run: the bundle plant in one step (its ratio is price-free), any other
    recipe a fixed point of the ratios, found by Newton's method in ln r with the root of each
    economy by fast_root and a forward-difference Jacobian; then the full solve of generate_1c.py
    at the fixed point, with its identities, and the ratios' gap asserted below 1e-65."""
    rho = Q(economy_kwargs(base)["rho"])

    def at_ratios(ratios):
        econ, info = planted(base, plants, ratios)
        t = econ.envelope()[0]
        assert not econ.envelope()[1], "one technique"
        x = fast_root(econ, t)
        q = econ.at(x, t)
        return econ, info, new_ratios(base, plants, q["block"]["p"], q["v"])

    if all(pl["recipe"] == "bundle" for _, pl in plants):
        ratios = [(1 - pl["theta"]) / (pl["theta"] * user_cost(rho, Q(pl["delta"]), pl["J"]) * pl["size"])
                  for _, pl in plants]
        steps = 0
    else:
        # start from the flow economy's prices, then Newton in s = ln r
        q0 = interior(base)
        s = [mp.log(r) for r in new_ratios(base, plants, [t["p"] for t in q0["types"]], q0["v"])]
        # damped iteration (CAPACITY.md's solve_cap: half a step in ln r) to 1e-8, then Newton
        for _ in range(200):
            _, _, nr = at_ratios([mp.exp(x) for x in s])
            F = [mp.log(a) - b for a, b in zip(nr, s)]
            if max(abs(x) for x in F) < mp.mpf(10) ** -8:
                break
            s = [b + x / 2 for b, x in zip(s, F)]
        steps = 0
        for steps in range(1, 40):
            _, _, nr = at_ratios([mp.exp(x) for x in s])
            F = [mp.log(a) - b for a, b in zip(nr, s)]
            if max(abs(x) for x in F) < mp.mpf(10) ** -(DPS - 3):
                break
            h = mp.mpf(10) ** -30
            J = []
            for j in range(len(s)):
                sj = list(s)
                sj[j] += h
                _, _, nrj = at_ratios([mp.exp(x) for x in sj])
                J.append([((mp.log(a) - b) - Fi) / h for a, b, Fi in zip(nrj, sj, F)])
            Jm = mp.matrix([[J[j][i] for j in range(len(s))] for i in range(len(s))])
            ds = mp.lu_solve(Jm, mp.matrix([-x for x in F]))
            s = [a + ds[i] for i, a in enumerate(s)]
        ratios = [mp.exp(x) for x in s]
    econ, info = planted(base, plants, ratios)
    q = interior(econ)
    gap = max(abs(mp.log(a / b)) for a, b in zip(new_ratios(base, plants, [t["p"] for t in q["types"]], q["v"]), ratios))
    assert gap < IDENTITY_TOL, gap
    # the readouts and their identities (docs/unit-1g.md section 4.5)
    for (k, pl), r in zip(plants, ratios):
        t, th = q["types"][k], pl["theta"]
        z = info[k]
        assert rel(z["zeta"] ** th * z["kappa"] ** (1 - th), 1) < IDENTITY_TOL
        assert rel(t["u"] * t["V"], (1 - th) * t["p"]) < IDENTITY_TOL, "the user cost is 1 - theta of the price"
        assert rel(t["O"], th * t["p"]) < IDENTITY_TOL, "the bundle is theta of the price"
        z.update(stock=z["kappa"] * t["X"], bundles=z["zeta"] * t["X"], plant_price=t["V"] / z["kappa"],
                 X=t["X"], p=t["p"], V=t["V"], O=t["O"], u=t["u"])
    return econ, q, info, steps


# ------------------------------------------------------------------------------ instances
def s1_parameters(rho=Fraction(1, 20)):
    """S1 (docs/unit-1g.md section 3.3): coal from seams, labour and engine-hours; iron from
    ore, coal and labour; the engine good from iron, labour and site; an engine-hour from coal,
    labour and the engine's user cost. Seven coefficients are chosen; four (engine-hour labour,
    engine-good labour, coal's seams, the engine's site) are solved exactly, as fractions, so
    that the four totals per engine-hour are 1c's M3 machine's: operating (0.5, 0.1, 0.2), build
    (0.1, 0.2, 0.02), rho 0.05, delta 0.1, J 3 (ORACLE-GOODS section 1.6)."""
    delta, J = Fraction(1, 10), 3
    u = (rho + delta) * (1 + rho) ** (J - 1)
    a, l, b, aI, lI, bI = Fraction(1, 2), Fraction(1, 10), Fraction(1, 5), Fraction(1, 10), Fraction(1, 5), Fraction(1, 50)
    lt, bt = (l + u * lI) / (1 - a - u * aI), (b + u * bI) / (1 - a - u * aI)
    ltq, btq = (l + delta * lI) / (1 - a - delta * aI), (b + delta * bI) / (1 - a - delta * aI)
    e_c, c_i, n_I, c_o = Fraction(1, 10), Fraction(1, 2), Fraction(1, 10), Fraction(2, 5)
    lam_c, lam_i, ore = Fraction(1, 5), Fraction(1, 2), Fraction(1, 10)
    a_op, a_I = c_o * e_c, n_I * c_i * e_c
    P, Qq = lt * (1 - a_op - u * a_I), ltq * (1 - a_op - delta * a_I)
    lbar_I = (P - Qq) / (u - delta)
    lbar_op = P - u * lbar_I
    Pb, Qb = bt * (1 - a_op - u * a_I), btq * (1 - a_op - delta * a_I)
    bbar_I = (Pb - Qb) / (u - delta)
    bbar_op = Pb - u * bbar_I
    solved = dict(lab_op=lbar_op - c_o * lam_c, lab_I=lbar_I - n_I * (c_i * lam_c + lam_i),
                  seams=bbar_op / c_o, site=bbar_I - n_I * (c_i * (bbar_op / c_o) + ore))
    assert all(v > 0 for v in solved.values()), solved
    return dict(e_c=e_c, c_i=c_i, n_I=n_I, c_o=c_o, lam_c=lam_c, lam_i=lam_i, ore=ore, delta=delta, J=J,
                **solved)


GOOD_SPACE = (("GOOD", "1", "0", ("1",)), ("SPACE", "1", "1", ("0",)))
M3_HOUSEHOLD = dict(workers="4", land="10", eta="1", g0="1", g1="2", k="1", chi_max="1", edges=("0", "1"))


def s1_goods(par):
    """S1's materials and its engine."""
    materials = [material("COAL", recipe({"ENGINE_HOURS": par["e_c"]}, par["lam_c"], par["seams"])),
                 material("IRON", recipe({"COAL": par["c_i"]}, par["lam_i"], par["ore"]))]
    engine = machine("ENGINE", recipe({"IRON": par["n_I"]}, par["lab_I"], par["site"]), "ENGINE_HOURS", 1, 1,
                     recipe({"COAL": par["c_o"]}, par["lab_op"], 0), par["delta"], par["J"])
    return materials, engine


def s1_chain(rho="0.05"):
    materials, engine = s1_goods(s1_parameters())
    return Chain(**M3_HOUSEHOLD, rho=rho, categories=GOOD_SPACE, materials=materials, machines=[engine])


S2_VARIANTS = {
    "S2": dict(theta="1.3", g0="0.5", g1="1.5", workers="10"),   # the engine at the margin
    "S2H": dict(theta="1.2", g0="0.5", g1="1", workers="16"),    # the horse at the margin
}


def s2_chain(variant, rho="0.05"):
    """S1's engine beside a horse: fodder from meadow land (0.5) and labour (0.3); the horse good
    (one unit of capacity) from 2 of fodder and 1.5 of labour, delta 0.08, J 4; a horse-hour
    from 0.1 of fodder, 0.15 of a groom's labour and the horse's user cost (ORACLE-GOODS
    section 1.6)."""
    v = S2_VARIANTS[variant]
    materials, engine = s1_goods(s1_parameters())
    engine = dict(engine, theta=Q(v["theta"]))
    materials = materials + [material("FODDER", recipe({}, "0.3", "0.5"))]
    horse = machine("HORSE", recipe({"FODDER": "2"}, "1.5", 0), "HORSE_HOURS", 1, 1,
                    recipe({"FODDER": "0.1"}, "0.15", 0), "0.08", 4)
    hh = dict(M3_HOUSEHOLD, g0=v["g0"], g1=v["g1"], workers=v["workers"])
    return Chain(**hh, rho=rho, categories=GOOD_SPACE, materials=materials, machines=[horse, engine])


APPENDIX_B = dict(workers="4", land="10", eta="1", g0="0.2", g1="0.8", k="1", chi_max="1", edges=("0", "1"))
A0_OMEGA = "0.5"
"""The share of the flow machine's land that A0's horse runs on, as fodder (AGENTS-GOODS
section 1's omega; its runs used 0, 0.5, 0.85 and 1)."""


def a0_delta():
    return fraction_per_tick("0.1")


def a0_chain(rho="0", categories=GOOD_SPACE, edges=("0", "1")):
    """A0 (AGENTS-GOODS section 1; GOODS-CHAIN section 5, rule A): SSRN Appendix B's flow machine
    (a, lambda, b) = (0.3, 0.05, 0.4) as a durable good at 52 ticks a year. The horse is built
    from a/delta of its own hours, lambda/delta hours of labour and (1 - omega) b/delta of land,
    and runs on omega b of land an hour as fodder, a material made from land alone; delta is
    10% a year per tick and J one tick."""
    d = a0_delta()
    a, lam, b, w = Q("0.3"), Q("0.05"), Q("0.4"), Q(A0_OMEGA)
    materials = [material("FODDER", recipe({}, 0, w * b))]
    horse = machine("HORSE", recipe({"HORSE_DAYS": a / d}, lam / d, (1 - w) * b / d), "HORSE_DAYS", 1, 1,
                    recipe({"FODDER": 1}, 0, 0), d, 1)
    hh = dict(APPENDIX_B, edges=edges)
    return Chain(**hh, rho=rho, categories=categories, materials=materials, machines=[horse])


def a0_one_type(rho="0", categories=GOOD_SPACE, edges=("0", "1"), intermediate=(("0", "0"), ("0", "0"))):
    """A0's machine as one 1c type: operating (0; 0; omega b), build (a/delta; lambda/delta;
    (1 - omega) b/delta), delta, J 1."""
    d = a0_delta()
    a, lam, b, w = Q("0.3"), Q("0.05"), Q("0.4"), Q(A0_OMEGA)
    t = g1c.machine_type("HORSE", 1, ((0,), 0, w * b), ((a / d,), lam / d, (1 - w) * b / d), d, 1)
    hh = dict(APPENDIX_B, edges=edges)
    return make(Economy, **hh, rho=rho, categories=categories, intermediate=intermediate, types=(t,))


HORSE_HOUSEHOLD = dict(workers="12", land="10", eta="1", g0="0.2", g1="0.8", k="1", chi_max="0.05",
                       edges=("0", "1"))
"""The horse's county (docs/unit-1g.md section 3.3): the good and space on Appendix B's line,
with N 12 and chi_max 1/20, chosen so that the horse is at the margin mid-line."""


def horse_chain():
    """CHAIN's horse at weekly periods (CHAIN section 1.3-1.4): fodder (t) from farmland 1.0
    acre-year, 8 person-days and 2 horse-days; a horse (head) reared from 8 t of fodder, 3
    acre-years of pasture and 20 person-days, giving 250 horse-days a year, 250/52 a week; a
    horse-day from 0.0176 t of fodder, 0.1 of a groom's day and the horse's user cost; delta 8% a
    year and J 3 years at 52 ticks a year, rho 0. One land (farmland and pasture as one input)."""
    d = fraction_per_tick("0.08")
    materials = [material("FODDER", recipe({"HORSE_DAYS": 2}, 8, 1))]
    horse = machine("HORSE", recipe({"FODDER": 8}, 20, 3), "HORSE_DAYS", Fraction(250, 52), 1,
                    recipe({"FODDER": "0.0176"}, "0.1", 0), d, 3 * TICKS_PER_YEAR)
    return Chain(**HORSE_HOUSEHOLD, rho=0, categories=GOOD_SPACE, materials=materials, machines=[horse])


L2_TYPES = (
    g1c.machine_type("ENGINE", 2, (("0.1", "0.5"), "0.12", "0.4"), g1c.zero_recipe(2), 1, 1),
    g1c.machine_type("POWER", 0, (("0.1", "0"), "0.12", "0.6"), g1c.zero_recipe(2), 1, 1),
)
"""The markets probe's L2 (docs/probe/MARKETS-RULES.md; STATE decision 118): M4's engine and power
in the flow case, each buying the other's service, a loop."""


def l2_economy(rho="0"):
    return make(Economy, **APPENDIX_B, rho=rho, categories=GOOD_SPACE, intermediate=(("0", "0"), ("0", "0")),
                types=L2_TYPES)


PLANT_THETA = Q("0.8")
"""The plant's bundle share theta (CAPACITY.md's recommendation: y = K^0.2 z^0.8)."""


def bundle_plant(size=None, delta=None, lag=1):
    d = fraction_per_tick("0.1") if delta is None else delta
    return dict(theta=PLANT_THETA, delta=d, J=lag, recipe="bundle",
                size=s1_size(PLANT_THETA, d) if size is None else Q(size))


def fixed_plant(labor, land, machines=("0", "0"), lag=1):
    return dict(theta=PLANT_THETA, delta=fraction_per_tick("0.1"), J=lag, recipe=(machines, labor, land))


# ------------------------------------------------------------------------------ output
LINES = []


def section(title):
    LINES.append("")
    LINES.append(f"# {title}")


def put(key, value, note):
    """One golden, printed to SIG_OUT digits."""
    text = mp.nstr(value, SIG_OUT, min_fixed=-mp.inf, max_fixed=mp.inf)
    LINES.append(f"{key} = {text}  # {note}")


def put_aggregates(prefix, q, what):
    put(f"{prefix}_X_STAR", q["x"], f"{what}: x*")
    put(f"{prefix}_ONE_MINUS_X_STAR", q["one_minus_x"], f"{what}: 1 - x*")
    put(f"{prefix}_V", q["v"], f"{what}: v = w/r")
    put(f"{prefix}_P_S", q["Ps"], f"{what}: P_s")
    put(f"{prefix}_Y", q["Y"], f"{what}: Y")
    put(f"{prefix}_N_A", q["N_a"], f"{what}: N_a")
    put(f"{prefix}_INCOME", q["income"], f"{what}: I = Y P_s")
    if q["interest"] != 0:
        put(f"{prefix}_INTEREST", q["interest"], f"{what}: interest = sum (u - delta) V X")


def put_goods(prefix, chain, r, what):
    """Each material's and machine good's price and output, and each machine's hour price,
    operating cost, capacity V and hours."""
    goods, qb = r["goods"], r["qb"]
    tb = {t["name"]: t for t in qb["types"]}
    for f in chain.materials:
        k = f["key"]
        put(f"{prefix}_{k}_PRICE", goods["price"][k], f"{what}: {k.lower()}'s price")
        put(f"{prefix}_{k}_OUTPUT", goods["output"][k], f"{what}: {k.lower()} made a period")
    for m in chain.machines:
        k, h = m["key"], m["hours"]
        t = tb[h]
        put(f"{prefix}_{k}_PRICE", goods["price"][k], f"{what}: the {k.lower()} good's price, per unit of stock")
        put(f"{prefix}_{k}_OUTPUT", goods["output"][k], f"{what}: units of the {k.lower()} good made a period, delta X/kappa")
        put(f"{prefix}_{k}_STOCK", t["X"] / m["kappa"], f"{what}: units of the {k.lower()} good installed, X/kappa")
        put(f"{prefix}_{h}_PRICE", t["p"], f"{what}: an hour's price p = O + uV")
        put(f"{prefix}_{h}_OPERATING", t["O"], f"{what}: an hour's operating cost O")
        put(f"{prefix}_{h}_BUILD", t["V"], f"{what}: V per unit of capacity, the good's price/kappa")
        put(f"{prefix}_{h}_HOURS", t["X"], f"{what}: hours X a period")


def build():
    worst = mp.mpf(0)
    # ------------------------------------------------------------------ D-G10 changes nothing 1c accepts
    for label, econ in (("M3", g1c.m3()), ("M3z", g1c.m3(rho="0")), ("M4", g1c.m4()),
                        ("M4 eta 0.5", g1c.m4(eta="0.5")), ("M5 0.1", g1c.m5("0.1"))):
        kw = dict(workers=econ.N, land=econ.T, eta=econ.eta, g0=econ.g0, g1=econ.g1, k=econ.k,
                  chi_max=econ.chi_max, rho=econ.rho, edges=econ.e,
                  categories=tuple((n, z, b, mu) for n, z, b, mu in zip(econ.names, econ.z, econ.bc, econ.mu)),
                  intermediate=econ.Acc,
                  types=tuple(g1c.machine_type(n, th, (ao, lo, bo), (ai, li, bi), d, j)
                              for n, th, ao, lo, bo, ai, li, bi, d, j in
                              zip(econ.tn, econ.theta, econ.aop, econ.lop, econ.bop, econ.aI, econ.lI, econ.bI,
                                  econ.delta, econ.lag)))
        assert_same_object(label, **kw)

    # ------------------------------------------------------------------ D-G10's validation goldens
    section("D-G10: productivity and the chain to land on the per-period recipes A^op + Delta A^I "
            "(docs/unit-1g.md section 2.1); spectral radii by mp.eig")
    for tag, econ, note in (("A0", a0_one_type(), "A0 as one type"),
                            ("A0_CHAIN", a0_chain().embed(), "A0's chain (fodder, the horse, horse-days)"),
                            ("HORSE", horse_chain().embed(), "CHAIN's horse at weekly periods"),
                            ("S1", s1_chain().embed(), "S1's chain")):
        phys, per = spectral_radius(econ.physical()), spectral_radius(econ.Aq)
        assert per < 1
        put(f"{tag}_RADIUS_PHYSICAL", phys, f"{note}: spectral radius of A^op + A^I (1c's rule)")
        put(f"{tag}_RADIUS_PER_PERIOD", per, f"{note}: spectral radius of A^op + Delta A^I (D-G10's)")
        if tag != "S1":
            assert phys > 1 and not g1c.is_m_matrix(econ.physical()), tag
    # A0: the per-period radius is a (0.3), since u a^I = delta a/delta
    assert rel(spectral_radius(a0_one_type().Aq), Q("0.3")) < IDENTITY_TOL
    put("A0_DELTA", a0_delta(), "A0's delta per tick, 1 - 0.9^(1/52) (Clock::fraction)")
    put("A0_RHO_TICK", compound_per_tick("0.05"), "A0R's rho per tick, 1.05^(1/52) - 1 (Clock::compound)")
    put("HORSE_DELTA", fraction_per_tick("0.08"), "the horse's delta per tick, 1 - 0.92^(1/52)")
    put("PLANT_DELTA", fraction_per_tick("0.1"), "the plant's delta per tick, 1 - 0.9^(1/52)")

    # ------------------------------------------------------------------ S1
    section("S1: the steam chain (coal, iron, the engine good, engine-hours) calibrated to 1c's M3, "
            "rho 0.05 (docs/unit-1g.md section 3.3; ORACLE-GOODS section 1.6)")
    par = s1_parameters()
    for key in ("lab_op", "lab_I", "seams", "site"):
        put(f"S1_{key.upper()}", Q(par[key]), f"S1's solved coefficient {key} (a fraction: {par[key]})")
    ch = s1_chain()
    r = three_ways(ch, "S1")
    worst = max(worst, r["worst"])
    m3 = interior(g1c.m3())
    worst = max(worst, same("S1 against M3", agg(r["qb"]), agg(m3)))
    th = {t["name"]: t for t in r["qb"]["types"]}["ENGINE_HOURS"]
    assert rel(th["p"], m3["types"][0]["p"]) < IDENTITY_TOL
    assert rel(th["V"] * th["X"], m3["types"][0]["V"] * m3["types"][0]["X"]) < IDENTITY_TOL
    put_aggregates("S1", r["qb"], "S1")
    put_goods("S1", ch, r, "S1")
    fold = ch.fold()
    for key, val, note in (("A", fold.aop[0][0], "operating: engine-hours per engine-hour"),
                           ("LAMBDA", fold.lop[0], "operating: labour"), ("B", fold.bop[0], "operating: land"),
                           ("A_I", fold.aI[0][0], "build: engine-hours per unit of capacity"),
                           ("LAMBDA_I", fold.lI[0], "build: labour"), ("B_I", fold.bI[0], "build: land")):
        put(f"S1_FOLD_{key}", val, f"S1 folded to one type: {note}")

    section("S1Z: S1 at rho 0, whose five numbers are 1c's M3z's")
    chz = s1_chain(rho="0")
    rz = three_ways(chz, "S1Z")
    worst = max(worst, rz["worst"])
    m3z = interior(g1c.m3(rho="0"))
    worst = max(worst, same("S1Z against M3z", agg(rz["qb"]), agg(m3z)))
    put_goods("S1Z", chz, rz, "S1Z")

    # ------------------------------------------------------------------ S2, S2H
    for variant, what in (("S2", "S2, horse and engine, the engine at the margin"),
                          ("S2H", "S2H, horse and engine, the horse at the margin")):
        section(f"{what} (docs/unit-1g.md section 3.3; ORACLE-GOODS section 1.6), rho 0.05")
        c2 = s2_chain(variant)
        r2 = three_ways(c2, variant)
        worst = max(worst, r2["worst"])
        env = r2["embed"].envelope()
        first, switches = env
        names = r2["embed"].tn
        assert len(switches) == 1 and names[first] == "HORSE_HOURS" and names[switches[0][2]] == "ENGINE_HOURS"
        gs = switches[0][0]
        put(f"{variant}_SWITCH_GAMMA", gs, f"{variant}: gamma at the switch horse -> engine, 1c section 4.3")
        put(f"{variant}_SWITCH_X", r2["embed"].gamma_inv(gs), f"{variant}: x at the switch")
        put_aggregates(variant, r2["qb"], variant)
        put_goods(variant, c2, r2, variant)
        # the fold's envelope and the representative types' (five numbers only) agree
        assert rel(r2["qc"]["env"][1][0][0], gs) < IDENTITY_TOL
        rep_types = []
        tb = {t["name"]: t for t in r2["qb"]["types"]}
        for m in c2.machines:
            t = tb[m["hours"]]
            u, d = m["u"], m["delta"]
            lI, bI = (t["lt"] - t["ltq"]) / (u - d), (t["bt"] - t["btq"]) / (u - d)
            lo, bo = t["lt"] - u * lI, t["bt"] - u * bI
            assert min(lI, bI, lo, bo) >= 0
            rep_types.append(g1c.machine_type(m["hours"], m["theta"], ((0, 0), lo, bo), ((0, 0), lI, bI), d, m["J"]))
        rep = make(Economy, **c2.kwargs(rep_types))
        worst = max(worst, same(f"{variant} representative", agg(interior(rep)), agg(r2["qb"])))
        # at rho 0 the switch leaves the line and the horse is used everywhere
        c20 = s2_chain(variant, rho="0")
        e20 = c20.embed()
        assert not e20.envelope()[1] and e20.tn[e20.envelope()[0]] == "HORSE_HOURS"

    # ------------------------------------------------------------------ A0
    section("A0: Appendix B's flow machine as a durable good at 52 ticks a year, delta 10% a year, "
            "omega 1/2, rho 0 (docs/unit-1g.md section 3.3): Appendix B's equilibrium exactly")
    ra = three_ways(a0_chain(), "A0")
    worst = max(worst, ra["worst"])
    one = interior(a0_one_type())
    worst = max(worst, same("A0 one type against its chain", agg(one), agg(ra["qb"])))
    g1 = g1a.interior()
    nest = dict(x=g1["x"], one_minus_x=g1["one_minus_x"], v=g1["v"], Ps=g1["Ps"], Y=g1["Y"], N_a=g1["n"],
                income=g1["income"], interest=g1["interest"])
    worst = max(worst, same("A0 against Appendix B (generate.py)", agg(ra["qb"]), nest))
    assert rel(ra["goods"]["price"]["HORSE_DAYS"], g1["pm"]) < IDENTITY_TOL
    assert rel(ra["goods"]["output"]["HORSE_DAYS"], g1["K"]) < IDENTITY_TOL
    put_goods("A0", a0_chain(), ra, "A0")
    t = one["types"][0]
    put("A0_ONE_BUILD", t["V"], "A0 as one type: V, (p_m - omega b)/delta (AGENTS-GOODS section 1)")
    assert rel(t["V"], (g1["pm"] - Q(A0_OMEGA) * Q("0.4")) / a0_delta()) < IDENTITY_TOL
    put("A0_ONE_WEALTH", t["W"], "A0 as one type: W = V X (J 1)")

    section("A0R: A0 at rho 5% a year, 1.05^(1/52) - 1 a tick: unit 1a's durable point, which 1a's a < 1 "
            "rejects (a/delta = 148)")
    rr = three_ways(a0_chain(rho=compound_per_tick("0.05")), "A0R")
    worst = max(worst, rr["worst"])
    oner = interior(a0_one_type(rho=compound_per_tick("0.05")))
    worst = max(worst, same("A0R one type against its chain", agg(oner), agg(rr["qb"])))
    put_aggregates("A0R", rr["qb"], "A0R")
    put_goods("A0R", a0_chain(rho=compound_per_tick("0.05")), rr, "A0R")

    section("A0F: A0's durable machine in 1b's fork economy C3 at rho 0: 1b's C3 exactly")
    fork_cats = g1b.FORK
    zeros = (("0",) * 4,) * 4
    af = a0_one_type(categories=fork_cats, edges=g1b.FORK_EDGES, intermediate=zeros)
    qf = interior(af)
    c3 = g1b.interior(g1b.FORK_EDGES, g1b.FORK)
    pairs = dict(x=(qf["x"], c3["x"]), v=(qf["v"], c3["v"]), Ps=(qf["Ps"], c3["Ps"]), Y=(qf["Y"], c3["Y"]),
                 n=(qf["N_a"], c3["n"]), pm=(qf["types"][0]["p"], c3["pm"]), K=(qf["types"][0]["X"], c3["K"]))
    for j in range(4):
        pairs[f"p{j}"] = (qf["cats"][j]["p"], c3["cats"][j]["p"])
    fw = max(rel(a, b) for a, b in pairs.values())
    assert fw < IDENTITY_TOL, fw
    worst = max(worst, fw)
    put("A0F_BUILD", qf["types"][0]["V"], "A0F: the horse's V per unit of capacity")

    # ------------------------------------------------------------------ the horse
    section("HORSE: CHAIN's horse at weekly periods (CHAIN section 1.3-1.4): fodder, the horse (head), "
            "horse-days; delta 8% a year, J 156 ticks, rho 0, the good and space, N 12, T 10, chi 1/20, "
            "gamma = 0.2 + 0.8x (docs/unit-1g.md section 3.3)")
    hc = horse_chain()
    rh = three_ways(hc, "HORSE")
    worst = max(worst, rh["worst"])
    put_aggregates("HORSE", rh["qb"], "HORSE")
    put_goods("HORSE", hc, rh, "HORSE")
    tb = {t["name"]: t for t in rh["qb"]["types"]}["HORSE_DAYS"]
    put("HORSE_LAMBDA_TILDE", tb["lt"], "HORSE: labour per horse-day, all through (CHAIN's 0.268)")
    put("HORSE_B_TILDE", tb["bt"], "HORSE: land per horse-day, all through (CHAIN's 0.020 + 0.001)")
    put("HORSE_OMEGA", tb["omega"], "HORSE: wealth factor 1 + delta(J - 1) at rho 0")

    # ------------------------------------------------------------------ plants
    section("P1: the markets probe's L2 (engine and power buying each other's service, flow) with a plant on "
            "both, theta 0.8, delta 10% a year a tick, J 1, rho 0, built from s1 bundles of the desk's own "
            "recipe: L2's flow economy exactly (docs/unit-1g.md section 4.5; CAPACITY.md)")
    l2 = l2_economy()
    q2 = interior(l2)
    put("L2_V", q2["v"], "L2's flow economy: v (CAPACITY.md's oracle 0.42570)")
    put("L2_X_STAR", q2["x"], "L2's flow economy: x*")
    plants = [(0, bundle_plant()), (1, bundle_plant())]
    put("P1_S1", plants[0][1]["size"], "s1 = theta^(theta/(1 - theta)) (1 - theta)/delta, bundles a plant unit")
    e1, qp1, info1, _ = plant_solve(l2, plants)
    worst = max(worst, same("P1 against L2", agg(qp1), agg(q2)))
    for k in (0, 1):
        assert rel(qp1["types"][k]["p"], q2["types"][k]["p"]) < IDENTITY_TOL
        assert rel(qp1["types"][k]["X"], q2["types"][k]["X"]) < IDENTITY_TOL
        assert rel(info1[k]["zeta"], PLANT_THETA) < IDENTITY_TOL
        assert rel(info1[k]["stock"] / qp1["types"][k]["X"], PLANT_THETA ** (-PLANT_THETA / (1 - PLANT_THETA))) < IDENTITY_TOL
    for k, name in ((0, "ENGINE"), (1, "POWER")):
        z = info1[k]
        put(f"P1_{name}_KAPPA", z["kappa"], f"P1: {name.lower()}'s kappa, plant units per unit of capacity")
        put(f"P1_{name}_STOCK", z["stock"], f"P1: {name.lower()}'s plant K* = kappa X = theta^(-theta/(1-theta)) y")
        put(f"P1_{name}_PLANT_PRICE", z["plant_price"], f"P1: {name.lower()}'s plant price P_K = s1 c_f")
        put(f"P1_{name}_BUILD", z["V"], f"P1: {name.lower()}'s V = kappa P_K")
    section("P1S: P1 at s = 1 bundle a plant unit: the flow economy with every planted recipe times "
            "m = theta^-theta (1 - theta)^-(1 - theta) (delta s)^(1 - theta)")
    ps = [(0, bundle_plant(size="1")), (1, bundle_plant(size="1"))]
    es, qs, infos, _ = plant_solve(l2, ps)
    d = fraction_per_tick("0.1")
    m_s = PLANT_THETA ** -PLANT_THETA * (1 - PLANT_THETA) ** -(1 - PLANT_THETA) * d ** (1 - PLANT_THETA)
    scaled = []
    for t in L2_TYPES:
        a, lam, b = t["op"]
        scaled.append(g1c.machine_type(t["name"], t["theta"], ([m_s * Q(x) for x in a], m_s * Q(lam), m_s * Q(b)),
                                       g1c.zero_recipe(2), 1, 1))
    flow_m = make(Economy, **dict(economy_kwargs(l2), types=tuple(scaled)))
    worst = max(worst, same("P1S against the scaled flow economy", agg(qs), agg(interior(flow_m))))
    put("P1S_M", m_s, "P1S: m at s = 1")
    put_aggregates("P1S", qs, "P1S")
    section("P1R: P1 at rho 5% a year a tick: the bundle plant's ratio is still price-free, but the long run "
            "is no longer L2's (u > delta)")
    pr = [(0, bundle_plant()), (1, bundle_plant())]
    l2r = l2_economy(rho=compound_per_tick("0.05"))
    er, qr, infor, _ = plant_solve(l2r, pr)
    put_aggregates("P1R", qr, "P1R")
    for k, name in ((0, "ENGINE"), (1, "POWER")):
        put(f"P1R_{name}_PRICE", qr["types"][k]["p"], f"P1R: {name.lower()}'s price p = O + uV")
        put(f"P1R_{name}_STOCK", infor[k]["stock"], f"P1R: {name.lower()}'s plant K* = kappa X")
    section("P2: L2 with a plant built from labour 0.5 and land 0.5 a unit on both desks: the ratio moves with "
            "prices, a fixed point (CAPACITY.md check 2c: v 0.13869, zeta/kappa engine 0.3783/48.8149, "
            "power 0.3723/52.0700)")
    pf = [(0, fixed_plant("0.5", "0.5")), (1, fixed_plant("0.5", "0.5"))]
    ef, qf2, infof, steps = plant_solve(l2, pf)
    assert abs(qf2["v"] - Q("0.13869")) < Q("5e-6")
    for k, (zeta, kappa) in ((0, ("0.3783", "48.8149")), (1, ("0.3723", "52.0700"))):
        assert abs(infof[k]["zeta"] - Q(zeta)) < Q("5e-5") and abs(infof[k]["kappa"] - Q(kappa)) < Q("5e-5")
    put_aggregates("P2", qf2, "P2")
    for k, name in ((0, "ENGINE"), (1, "POWER")):
        z = infof[k]
        put(f"P2_{name}_RATIO", z["ratio"], f"P2: {name.lower()}'s kappa/zeta at the fixed point")
        put(f"P2_{name}_ZETA", z["zeta"], f"P2: {name.lower()}'s zeta, bundles per unit of service")
        put(f"P2_{name}_KAPPA", z["kappa"], f"P2: {name.lower()}'s kappa, plant units per unit of capacity")
        put(f"P2_{name}_PRICE", z["p"], f"P2: {name.lower()}'s price p = O + uV")
        put(f"P2_{name}_STOCK", z["stock"], f"P2: {name.lower()}'s plant K* = kappa X")
        put(f"P2_{name}_PLANT_PRICE", z["plant_price"], f"P2: {name.lower()}'s plant price P_K = 0.5 v + 0.5")
    LINES.append(f"# P2's Newton steps: {steps}")

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens_1g.txt: the golden numbers for oracle unit 1g.",
        f"# Written by goldens/generate_1g.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits.",
        "# Equations: docs/unit-1g.md section 4. laborformal references are at 31b3482.",
        f"# The three forms, and the nesting with 1a, 1b and 1c, agree within {mp.nstr(worst, 3)} (1e-65 asserted).",
        f"# fnv1a64 generate_1g.py = {fnv1a64(source_bytes(__file__)):016x}",
        f"# fnv1a64 generate_1c.py = {fnv1a64(source_bytes(g1c.__file__)):016x}",
        f"# fnv1a64 generate_1b.py = {fnv1a64(source_bytes(g1b.__file__)):016x}",
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
                        help="compare with goldens_1g.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(HERE, "goldens_1g.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same_text = fh.read() == text
        print("goldens_1g.txt is current" if same_text else "goldens_1g.txt differs from generate_1g.py's output")
        return 0 if same_text else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
