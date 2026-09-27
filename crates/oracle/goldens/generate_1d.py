"""generate_1d.py: the golden numbers for oracle unit 1d.

Dated 2026-09-27. Every golden that crates/oracle's unit-1d tests use is computed here with
mpmath at 70 digits, from the equations of docs/unit-1d.md section 4: worker types sharing
the line's shape with an efficiency each, the paper's human-required set H (SSRN section 3.1,
p.6; main.tex:147-162) and reserved tasks, living costs as support baskets on one basket
(SSRN eq 8-11 per type), the walk for the basket's price with walled types, the three
stretches of the path (the all-human corner, the task line, the wall), the envelope of the
cheapest task type continued above gamma(1), and ties. The constructions are this unit's own:
laborformal has no equilibrium with a human-required set or with worker types (ADDENDUM
section 5 item 4); its limits (check_pinning.py D1 :147-178, SSRN Prop E.1, check_kset.py
P9-i :39-47) are asserted along the automation path BP.

The unit-1c generator (generate_1c.py, and through it generate_1b.py and generate.py) supplies
the machine block, the tasks on the line and the Leontief solves, which this file's Economy
extends (docs/unit-1d.md section 12), and its solves are used to assert that the one-type form
nests them. Nothing is imported from laborformal or from the oracle.

Run with any Python that has mpmath (1.3.0 was used), from this directory or any other:

    python goldens/generate_1d.py           # writes goldens/goldens_1d.txt beside this file
    python goldens/generate_1d.py --check   # exits 1 if goldens_1d.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in
tests/gate/goldens_1d.rs are these values rounded to 20 significant digits, and the test
d9_goldens_file::constants_match_goldens_1d_txt enforces that.

The header of goldens_1d.txt records five FNV-1a 64-bit digests: of this file, of
generate_1c.py, generate_1b.py and generate.py, and of the goldens that follow the header,
each with CRLF read as LF. The gate recomputes all five.
"""

import argparse
import os
import sys

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
"""Halvings of a region of the line [1e-12, 1], as generate_1c.py."""
HALVINGS = 400
"""Halvings of a corner's bracket in omega, of [0, 1e-12] and of a tie's [0, 1]: 2^-400 of
the bracket, below the working precision."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""
BRACKET_LO = "1e-12"
"""Left end of the line's bracket, as in the oracle and macro.py:102."""
PATH_GRID = 12
"""Points per stretch and technique on which f is asserted nonincreasing."""
WALL_SCAN = 200
"""Wages on the wall at which the envelope is checked against argmin p_t/theta_t."""

mp.mp.dps = DPS
assert g1c.DPS == DPS and g1c.BRACKET_LO == BRACKET_LO

M, rel, leontief, transpose = g1c.M, g1c.rel, g1c.leontief, g1c.transpose
TOL = IDENTITY_TOL
LO = M(BRACKET_LO)
INF = mp.inf


def F(z, chi):
    """The uniform work-cost cdf on [0, chi], clamped (1a's)."""
    return min(max(z / chi, 0), 1)


def worker(name, N, chi, eps="1", sig="1"):
    """A worker type: N workers, chi_max, efficiency eps, support sig (docs/unit-1d.md 3.1)."""
    return dict(name=name, N=N, chi=chi, eps=eps, sig=sig)


class Economy(g1c.Economy):
    """Unit 1c's economy with worker types, the common human-required hours L^H_j and the
    reserved hours R_ji (docs/unit-1d.md sections 2-4)."""

    def __init__(self, *, workers, land, eta, g0, g1, k, rho, edges, categories, intermediate,
                 types, required=None, reserved=None):
        super().__init__(workers=workers[0]["N"], land=land, eta=eta, g0=g0, g1=g1, k=k,
                         chi_max=workers[0]["chi"], rho=rho, edges=edges, categories=categories,
                         intermediate=intermediate, types=types)
        C = self.C
        self.wn = [w["name"] for w in workers]
        self.Nw = [M(w["N"]) for w in workers]
        self.chi = [M(w["chi"]) for w in workers]
        self.eps = [M(w["eps"]) for w in workers]
        self.sig = [M(w["sig"]) for w in workers]
        I = self.I = len(workers)
        self.LH = [M(x) for x in (required or ("0",) * C)]
        R = reserved or tuple(("0",) * I for _ in range(C))
        self.R = [[M(x) for x in row] for row in R]
        # section 3.2
        assert len(self.LH) == C and len(self.R) == C and all(len(r) == I for r in self.R)
        assert all(n > 0 and c > 0 and s > 0 and e >= 0 for n, c, s, e in zip(self.Nw, self.chi, self.sig, self.eps))
        assert any(e > 0 for e in self.eps), "no pooled type"
        self.LH_chain = leontief(self.Acc, self.LH)
        self.R_chain = [leontief(self.Acc, [self.R[j][i] for j in range(C)]) for i in range(I)]
        self.LH_y = sum(y * h for y, h in zip(self.yhat, self.LH))
        self.lR = [sum(y * self.R[j][i] for j, y in enumerate(self.yhat)) for i in range(I)]
        for i in range(I):
            assert self.eps[i] > 0 or self.lR[i] > 0, ("a type with nowhere to work", i)
        assert sum(y * (L + h) for y, L, h in zip(self.yhat, self.Lbar_dir, self.LH)) > 0
        # z'L^H-tilde = yhat'L^H and z'R-tilde = yhat'R (section 3.1)
        assert rel(self.LH_y, sum(z * h for z, h in zip(self.z, self.LH_chain))) < TOL or self.LH_y == 0
        for i in range(I):
            assert rel(self.lR[i], sum(z * r for z, r in zip(self.z, self.R_chain[i]))) < TOL or self.lR[i] == 0
        self.nu = sum(s * n for s, n in zip(self.sig, self.Nw))

    # ---------------------------------------------------------------- the envelope on the path
    def envelope_ext(self):
        """Section 5.2: 1c's walk continued above gamma(1), with no upper limit. Returns the first
        technique, the line's switches (gamma_s <= gamma(1)) and the wall's."""
        tasks = [t for t in range(self.K) if self.theta[t] > 0]
        glo, g1 = self.gamma(LO), self.gamma(M(1))
        flatter = lambda a, b: self.lt[a] * self.theta[b] < self.lt[b] * self.theta[a]  # noqa: E731
        first = None
        for t in tasks:
            v = self.closure_wage(t, glo)
            if v is None:
                continue
            if first is None or v < first[1] or (v == first[1] and flatter(t, first[0])):
                first = (t, v)
        if first is None:
            return tasks[0], [], []
        cur, g, visited, sw = first[0], glo, {first[0]}, []
        while True:
            best = None
            for l in tasks:
                if l in visited:
                    continue
                delta = self.bt[l] * self.lt[cur] - self.bt[cur] * self.lt[l]
                if not delta > 0:
                    continue
                gx = (self.bt[l] * self.theta[cur] - self.bt[cur] * self.theta[l]) / delta
                if not (g < gx and self.theta[l] - gx * self.lt[l] > 0):
                    continue
                if best is None or gx < best[1] or (gx == best[1] and flatter(l, best[0])):
                    best = (l, gx)
            if best is None:
                break
            sw.append((best[1], cur, best[0]))
            visited.add(best[0])
            cur, g = best[0], best[1]
        line = [s for s in sw if s[0] <= g1]
        wall = [s for s in sw if s[0] > g1]
        assert (first[0], line) == self.envelope(), "the line's switches are 1c's"
        return first[0], line, wall

    def cheapest(self, w):
        """The task type with the least p_t/theta_t at the wage w, ties to the lower index."""
        cands = [t for t in range(self.K) if self.theta[t] > 0]
        return min(cands, key=lambda t: ((w * self.lt[t] + self.bt[t]) / self.theta[t], t))

    # ---------------------------------------------------------------- one evaluation
    def price_side(self, x, t, w):
        """The machine block: with the margin column on the line (w None), or at the wage w
        (sections 4.2 and 5.1)."""
        if w is None:
            b = self.block(self.gamma(x), t)
            if not b["viable"]:
                return None
            return b["v"], b["p"], b["pi"], b["O"], b["V"], b["d"]
        K = self.K
        p = [w * self.lt[k] + self.bt[k] for k in range(K)]
        O = [sum(self.aop[k][l] * p[l] for l in range(K)) + self.lop[k] * w + self.bop[k] for k in range(K)]
        V = [sum(self.aI[k][l] * p[l] for l in range(K)) + self.lI[k] * w + self.bI[k] for k in range(K)]
        for k in range(K):
            assert rel(p[k], O[k] + self.u[k] * V[k]) < TOL
        return w, p, p[t] / self.theta[t], O, V, None

    def walk(self, Pbase, e, c):
        """Section 4.3: the least fixed point of P = Pbase + sum_i max(e_i, c_i P), with e_i and
        c_i per basket; (None, walled) when a denominator is <= 0."""
        order = sorted([i for i in range(self.I) if c[i] > 0], key=lambda i: (e[i] / c[i], i))
        walled = []
        P = Pbase + sum(e)
        for i in order:
            if c[i] * P > e[i]:
                walled.append(i)
                den = 1 - sum(c[j] for j in walled)
                if not den > 0:
                    return None, walled
                P = (Pbase + sum(e[j] for j in range(self.I) if j not in walled)) / den
            else:
                break
        if P is not None:
            # the fixed point, and the least: every piece's own fixed point is at most P
            assert rel(P, Pbase + sum(max(e[i], c[i] * P) for i in range(self.I))) < TOL
        return P, sorted(walled)

    def evaluate(self, x, t, w=None, split=None):
        """Section 5.1 at (x, t) on the line (w None) or at a corner's wage w, with the task
        services split as given (by default all to t)."""
        x = M(x)
        ps = self.price_side(x, t, w)
        if ps is None:
            return dict(viable=False, d=self.block(self.gamma(x), t)["d"])
        w, p, pi, O, V, d = ps
        C, I = self.C, self.I
        HM = [self.tasks(j, x) for j in range(C)]
        Mt = [m for _, m in HM]
        H = [h + lh for (h, _), lh in zip(HM, self.LH)]
        pbase = leontief(self.Acc, [w * H[j] + pi * Mt[j] + self.bc[j] for j in range(C)])
        Pbase = sum(z * q for z, q in zip(self.z, pbase))
        Hy = sum(y * h for y, h in zip(self.yhat, H))
        My = sum(y * m for y, m in zip(self.yhat, Mt))
        sp = split or [(t, 1)]
        qty = self.quantities(My, sp)
        Y = self.T / qty["land"]
        nD = Y * (Hy + qty["hours"])
        D = [Y * l for l in self.lR]
        out = dict(viable=True, x=x, t=t, w=w, p=p, pi=pi, O=O, V=V, d=d, H=H, Mt=Mt, pbase=pbase,
                   Pbase=Pbase, Hy=Hy, My=My, qty=qty, Y=Y, nD=nD, D=D, split=sp,
                   gamma=self.gamma(x), J=self.J(x), short=None)
        for i in range(I):
            if D[i] > self.Nw[i]:
                out.update(short=("reserved", i), f=INF)
                return out
        zeta = [mp.expm1(self.chi[i] * D[i] / self.Nw[i]) for i in range(I)]
        e = [self.eps[i] * w * self.lR[i] for i in range(I)]
        c = [zeta[i] * self.sig[i] * self.lR[i] for i in range(I)]
        P, walled = self.walk(Pbase, e, c)
        out.update(zeta=zeta, e=e, c=c)
        if P is None:
            out.update(short=("ceiling", walled), f=INF)
            return out
        wages = [zeta[i] * self.sig[i] * P if i in walled else self.eps[i] * w for i in range(I)]
        nS = [self.Nw[i] * F(mp.log1p(wages[i] / (self.sig[i] * P)), self.chi[i]) for i in range(I)]
        S = sum(self.eps[i] * (nS[i] - D[i]) for i in range(I) if i not in walled)
        reserved_cost = [sum(wages[i] * self.R_chain[i][j] for i in range(I)) for j in range(C)]
        out.update(Ps=P, walled=walled, wages=wages, nS=nS, S=S, f=nD - S,
                   reserved_cost=reserved_cost, pc=[pbase[j] + reserved_cost[j] for j in range(C)])
        return out

    # ---------------------------------------------------------------- the corners, in omega
    def S_of(self, omega, q):
        """Section 4.5: the pool's net supply at the real wage omega, with the corner's D, zeta."""
        S = mp.mpf(0)
        for i in range(self.I):
            if self.eps[i] == 0:
                continue
            r = self.eps[i] * omega / self.sig[i] if omega != INF else INF
            if r >= q["zeta"][i]:
                n = self.Nw[i] * (F(mp.log1p(r), self.chi[i]) if r != INF else 1)
                S += self.eps[i] * (n - q["D"][i])
        return S

    def basket_totals(self, q):
        """Price-side basket totals (L_s, B_s) at (x, t, split): the common H included."""
        cl = sum(s * self.lt[tt] / self.theta[tt] for tt, s in q["split"])
        cb = sum(s * self.bt[tt] / self.theta[tt] for tt, s in q["split"])
        lt_c = leontief(self.Acc, [h + m * cl for h, m in zip(q["H"], q["Mt"])])
        bt_c = leontief(self.Acc, [b + m * cb for b, m in zip(self.bc, q["Mt"])])
        return (sum(z * v for z, v in zip(self.z, lt_c)), sum(z * v for z, v in zip(self.z, bt_c)),
                lt_c, bt_c)

    def wage_of_omega(self, omega, q):
        """Section 4.5: v = omega B/((1 - C) - omega L) with the walled set at omega."""
        L_s, B_s, _, _ = self.basket_totals(q)
        LW, CW = L_s, mp.mpf(0)
        for i in range(self.I):
            if self.eps[i] > 0 and self.eps[i] * omega / self.sig[i] >= q["zeta"][i]:
                LW += self.eps[i] * self.lR[i]
            else:
                CW += q["zeta"][i] * self.sig[i] * self.lR[i]
        den = (1 - CW) - omega * LW
        assert den > 0
        return omega * B_s / den

    def omega_limit(self, q):
        """lim v/P_s as v -> infinity at (x, t): the walk with Pbase = L_s and e_i = eps_i lR_i;
        None when it has no fixed point."""
        L_s, _, _, _ = self.basket_totals(q)
        e = [self.eps[i] * self.lR[i] for i in range(self.I)]
        c = [q["zeta"][i] * self.sig[i] * self.lR[i] for i in range(self.I)]
        P, _ = self.walk(L_s, e, c)
        if P is None:
            return None
        return INF if P == 0 else 1 / P

    def corner_root(self, q, om_lo, om_hi):
        """The omega in [om_lo, om_hi] where n_D - S(omega) changes sign, by bisection."""
        f = lambda om: q["nD"] - self.S_of(om, q)  # noqa: E731
        assert om_lo == 0 or f(om_lo) > 0
        hi = om_hi
        if hi == INF:
            hi = max(om_lo, mp.mpf(1)) * 2
            while f(hi) > 0:
                hi *= 2
        assert f(hi) <= 0
        lo = om_lo
        for _ in range(HALVINGS):
            mid = (lo + hi) / 2
            if f(mid) > 0:
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2

    def wall_end(self, t):
        """Section 4.5: f_infinity under the wall's last technique t, and omega_infinity."""
        q = self.evaluate(1, t, w=M(1))
        if q["short"] and q["short"][0] == "reserved":
            return dict(f=INF, omega=None, reserved=q["short"][1])
        if "zeta" not in q:
            q["zeta"] = [mp.expm1(self.chi[i] * q["D"][i] / self.Nw[i]) for i in range(self.I)]
        om = self.omega_limit(q)
        if om is None:
            return dict(f=INF, omega=None, reserved=None)
        return dict(f=q["nD"] - self.S_of(om, q), omega=om, reserved=None)

    # ---------------------------------------------------------------- the solve
    def solve(self):
        """Section 5.3: the sequence along the path, its sides and the one equilibrium.
        Returns (regime, values), the regime one of NotViable, LaborShort, MultipleEquilibria,
        Contestable, Wall, AllHuman."""
        first, line_sw, wall_sw = self.envelope_ext()
        ltechs = [first] + [s[2] for s in line_sw]
        wtechs = [ltechs[-1]] + [s[2] for s in wall_sw]
        one = self.evaluate(1, ltechs[-1])
        if not one["viable"]:
            return "NotViable", dict(d_at_1=one["d"])
        at0 = self.evaluate(0, first)
        atlo = self.evaluate(LO, first)
        xs = [self.gamma_inv(g) for g, _, _ in line_sw]
        bounds = [LO] + xs + [M(1)]
        wws = [self.closure_wage(b, g) for g, b, _ in wall_sw]
        seq = [("Zero", at0["f"]), ("Lo", atlo["f"])]
        for i, x in enumerate(xs):
            seq.append((("SwitchBelow", i), self.evaluate(x, ltechs[i])["f"]))
            seq.append((("SwitchAbove", i), self.evaluate(x, ltechs[i + 1])["f"]))
        seq.append(("One", one["f"]))
        for s, (g, b, a) in enumerate(wall_sw):
            qb, qa = self.evaluate(1, b, w=wws[s]), self.evaluate(1, a, w=wws[s])
            assert rel(qb["pi"], qa["pi"]) < TOL  # the two delivered costs are equal at v_s
            seq.append((("WallBelow", s), qb["f"]))
            seq.append((("WallAbove", s), qa["f"]))
        end = self.wall_end(wtechs[-1])
        seq.append(("End", end["f"]))
        sides = [True] + [(f >= 0 if kind in ("One", "End") else f > 0) for kind, f in seq]
        changes = [i for i in range(len(sides) - 1) if sides[i] != sides[i + 1]]
        base = dict(seq=seq, changes=len(changes), ltechs=ltechs, wtechs=wtechs, line_sw=line_sw,
                    wall_sw=wall_sw, xs=xs, wws=wws, one=one, at0=at0, atlo=atlo, f_line_1=one["f"],
                    f_line_0=at0["f"], f_line_lo=atlo["f"], f_end=end["f"], omega_end=end["omega"],
                    reserved_short=end["reserved"])
        if not changes:
            return "LaborShort", base
        if len(changes) > 1:
            return "MultipleEquilibria", base
        c = changes[0]
        if c == 0:
            om0 = at0["w"] / at0["Ps"]
            om = om0 if at0["f"] == 0 else self.corner_root(at0, mp.mpf(0), om0)
            w = self.wage_of_omega(om, at0)
            return "AllHuman", self.report(0, self.cheapest(w), w=w, base=base, margin="AllHuman", omega=om)
        (before, f_before), (after, f_after) = seq[c - 1], seq[c]
        kind = before if isinstance(before, str) else before[0]
        if kind == "Zero":
            lo, hi = mp.mpf(0), LO
            for _ in range(HALVINGS):
                mid = (lo + hi) / 2
                if self.evaluate(mid, first)["f"] > 0:
                    lo = mid
                else:
                    hi = mid
            return "Contestable", self.report((lo + hi) / 2, first, base=base, margin="Contestable")
        if kind in ("Lo", "SwitchAbove"):
            r = 0 if kind == "Lo" else before[1] + 1
            t = ltechs[r]
            lo, hi = bounds[r], bounds[r + 1]
            for _ in range(BISECTIONS):
                mid = (lo + hi) / 2
                if self.evaluate(mid, t)["f"] > 0:
                    lo = mid
                else:
                    hi = mid
            return "Contestable", self.report((lo + hi) / 2, t, base=base, margin="Contestable")
        if kind == "SwitchBelow":
            i = before[1]
            below, above = ltechs[i], ltechs[i + 1]
            share = self.tie_share(xs[i], below, above, None)
            return "Contestable", self.report(xs[i], below, base=base, margin="Contestable",
                                              tie=(above, share, line_sw[i][0]))
        if kind == "WallBelow":
            s = before[1]
            below, above = wall_sw[s][1], wall_sw[s][2]
            share = self.tie_share(M(1), below, above, wws[s])
            return "Wall", self.report(1, below, w=wws[s], base=base, margin="Wall",
                                       tie=(above, share, wall_sw[s][0]))
        # a piece of the wall, from One or a WallAbove to a WallBelow or End
        piece = 0 if kind == "One" else before[1] + 1
        t = wtechs[piece]
        w_lo = one["w"] if piece == 0 else wws[piece - 1]
        q = self.evaluate(1, t, w=w_lo)
        om_lo = w_lo / q["Ps"]
        if after == "End":
            om_hi = end["omega"]
        else:
            q2 = self.evaluate(1, t, w=wws[after[1]])
            om_hi = wws[after[1]] / q2["Ps"]
        if f_before == 0:
            om = om_lo
        elif f_after == 0:
            om = om_hi
        else:
            om = self.corner_root(q, om_lo, om_hi)
        w = self.wage_of_omega(om, q)
        return "Wall", self.report(1, t, w=w, base=base, margin="Wall", omega=om)

    def tie_share(self, x, below, above, w):
        """Section 4.6: sigma at a switch of the line (w None) or of the wall (w = v_s): 1c's
        closed form when no type is walled under either pure technique, else bisection."""
        f = lambda s: self.evaluate(x, below, w=w, split=[(below, 1 - s), (above, s)])["f"]  # noqa: E731
        fa, fb = f(mp.mpf(0)), f(mp.mpf(1))
        assert fa > 0 >= fb, (fa, fb)
        qa = self.evaluate(x, below, w=w)
        qb = self.evaluate(x, below, w=w, split=[(above, 1)])
        if not qa["short"] and not qb["short"] and not qa["walled"] and not qb["walled"]:
            Ba, Bb = qa["qty"]["land"], qb["qty"]["land"]
            sig = Ba * fa / (Ba * fa - Bb * fb)
            assert abs(f(sig)) < TOL * qa["nD"]
            return sig
        lo, hi = mp.mpf(0), mp.mpf(1)
        for _ in range(HALVINGS):
            mid = (lo + hi) / 2
            if f(mid) > 0:
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2

    def closed_form_share(self, x, below, above, w):
        """1c's closed form for sigma, B_a f_a/(B_a f_a - B_b f_b) with the supply of the type
        below's evaluation on both sides, as unit 1c computes it: what a unit that ignores the
        walled wage's move with sigma would report (section 4.6)."""
        qa = self.evaluate(x, below, w=w)
        qb = self.evaluate(x, below, w=w, split=[(above, 1)])
        Ba, Bb = qa["qty"]["land"], qb["qty"]["land"]
        fa, fb = qa["nD"] - qa["S"], qb["nD"] - qa["S"]
        return Ba * fa / (Ba * fa - Bb * fb)

    # ---------------------------------------------------------------- the report
    def report(self, x, t, w=None, base=None, margin=None, tie=None, omega=None):
        """Section 4.7 at the equilibrium, every identity asserted to 1e-65."""
        split = [(t, 1)] if tie is None else [(t, 1 - tie[1]), (tie[0], tie[1])]
        q = self.evaluate(x, t, w=w, split=split)
        assert q["short"] is None
        K, C, I = self.K, self.C, self.I
        w, Y, Ps = q["w"], q["Y"], q["Ps"]
        g = w / q["pi"]
        X = [Y * xx for xx in q["qty"]["xhat"]]
        interest = sum(self.rho * om * Vk * Xk for om, Vk, Xk in zip(self.omega, q["V"], X))
        pooled = [i for i in range(I) if i not in q["walled"]]
        s_i = {i: self.eps[i] * (q["nS"][i] - q["D"][i]) for i in pooled}
        S = sum(s_i.values())
        pool = {i: (q["nD"] * s_i[i] / S if S != 0 else mp.mpf(0)) for i in pooled}
        hours = [q["D"][i] + (pool[i] / self.eps[i] if i in pooled and self.eps[i] > 0 else 0) for i in range(I)]
        wage_bill = w * q["nD"] + sum(q["wages"][i] * q["D"][i] for i in range(I))
        income = wage_bill + self.T + interest
        provider = (self.T + interest) / Ps - self.nu
        L_s, B_s, lt_c, bt_c = self.basket_totals(q)
        cql = sum(s * self.ltq[tt] / self.theta[tt] for tt, s in split)
        cqb = sum(s * self.btq[tt] / self.theta[tt] for tt, s in split)
        lq_c = leontief(self.Acc, [h + m * cql for h, m in zip(q["H"], q["Mt"])])
        bq_c = leontief(self.Acc, [b + m * cqb for b, m in zip(self.bc, q["Mt"])])
        g_margin = self.gamma(q["x"]) if margin == "Contestable" else g
        Lstar = leontief(self.Acc, [h + m / g_margin for h, m in zip(q["H"], q["Mt"])])
        pc, resv = q["pc"], q["reserved_cost"]
        cats = []
        for j in range(C):
            cats.append(dict(name=self.names[j], p=pc[j], real_wage=w / pc[j], L_star=Lstar[j],
                             Lbar=self.Lbar[j] + self.LH_chain[j], bbar=self.bbar[j], reserved_cost=resv[j],
                             lambda_tilde=lt_c[j], b_tilde=bt_c[j], lambda_q=lq_c[j], b_q=bq_c[j],
                             H=q["H"][j], M=q["Mt"][j], output=self.z[j] * Y,
                             phi_w=(w * lt_c[j] + resv[j]) / pc[j]))
        types = []
        for i in range(I):
            wage = q["wages"][i]
            types.append(dict(name=self.wn[i], wage=wage, real_wage=wage / Ps, pooled=i not in q["walled"],
                              premium=(wage / (self.eps[i] * w)) if self.eps[i] > 0 else None,
                              hours=hours[i], reserved_hours=q["D"][i], supply=q["nS"][i],
                              zeta=q["zeta"][i], chi_star=mp.log1p(wage / (self.sig[i] * Ps))))
        out = dict(base or {})
        out.update(margin=margin, x=q["x"], one_minus_x=1 - q["x"], gamma=q["gamma"], g=g, t=t, w=w,
                   pi=q["pi"], p_machine=q["p"], O=q["O"], V=q["V"], Ps=Ps, Y=Y, n_pool=q["nD"],
                   n_a=sum(hours), income=income, interest=interest, wage_bill=wage_bill,
                   labor_share=wage_bill / income, real_wage=w / Ps, provider=provider,
                   worker_baskets=self.nu + wage_bill / Ps, types=types, cats=cats, X=X, tie=tie, q=q,
                   replacement_top=self.gamma(M(1)) * q["pi"], replacement_bottom=self.gamma(M(0)) * q["pi"],
                   L_s=L_s, B_s=B_s, omega=omega, S=q["S"], My=q["My"], Hy=q["Hy"],
                   final_hours=Y * q["Hy"], required_hours=Y * self.LH_y, Lq=sum(z * c["lambda_q"] for z, c in zip(self.z, cats)),
                   Bq=sum(z * c["b_q"] for z, c in zip(self.z, cats)))
        self.assert_identities(out, q, split)
        return out

    def assert_identities(self, r, q, split):
        """Every identity of section 4.7 at 1e-65, and every bound."""
        tol, s = TOL, 1 + TOL
        I, C, K = self.I, self.C, self.K
        w, Y, Ps = r["w"], r["Y"], r["Ps"]
        ch = {}
        # the pool's clearing and each reserved market (section 4.3)
        ch["pool clears"] = rel(q["nD"], q["S"])
        omega = w / Ps
        for i in range(I):
            if i in q["walled"]:
                ch[f"reserved clears {i}"] = rel(q["nS"][i], q["D"][i])
                assert q["wages"][i] > self.eps[i] * w * (1 - tol), ("walled wage above pooled", i)
                assert self.eps[i] * omega / self.sig[i] <= q["zeta"][i] * s, ("walled below its threshold", i)
            else:
                assert q["nS"][i] >= q["D"][i] * (1 - tol), ("pooled covers reserved", i)
                assert q["wages"][i] == self.eps[i] * w
        # the walk's fixed point (section 4.3)
        ch["walk"] = rel(Ps, q["Pbase"] + sum(max(q["e"][i], q["c"][i] * Ps) for i in range(I)))
        # the basket (SSRN eq 7) with reserved costs
        ch["P_s = z p"] = rel(Ps, sum(z * p for z, p in zip(self.z, q["pc"])))
        ch["P_s totals"] = rel(Ps, w * r["L_s"] + r["B_s"] + sum(q["wages"][i] * self.lR[i] for i in range(I)))
        # income four ways (App. C) and the baskets
        ch["income Y P_s"] = rel(Y * Ps, r["income"])
        ch["income supply side"] = rel(sum(q["wages"][i] * r["types"][i]["hours"] for i in range(I))
                                       + self.T + r["interest"], r["income"])
        ch["income sum p z Y"] = rel(sum(c["p"] * c["output"] for c in r["cats"]), r["income"])
        ch["baskets"] = rel(r["worker_baskets"] + r["provider"], Y)
        # the fork in both forms with reserved costs, and the bounds (section 4.7)
        for j, c in enumerate(r["cats"]):
            ch[f"fork direct {j}"] = rel(c["p"], w * c["L_star"] + c["bbar"] + c["reserved_cost"])
            ch[f"fork totals {j}"] = rel(c["p"], w * c["lambda_tilde"] + c["b_tilde"] + c["reserved_cost"])
            assert c["bbar"] <= c["b_q"] * s and c["b_q"] <= c["b_tilde"] * s, ("chain", j)
            assert c["b_tilde"] + c["reserved_cost"] <= c["p"] * s, ("lower bound", j)
            assert c["p"] <= (w * c["Lbar"] + c["bbar"] + c["reserved_cost"]) * s, ("upper bound", j)
            assert c["L_star"] <= c["Lbar"] * s, ("L*", j)
            assert c["lambda_q"] <= c["lambda_tilde"] * s, ("lambda^q", j)
            if r["margin"] == "AllHuman":
                ch[f"upper bound attained {j}"] = rel(c["p"], w * c["Lbar"] + c["bbar"] + c["reserved_cost"])
        # the full cost system over the C + K rows (SSRN eq 2-4, A.1) and the clearing side
        n = C + K
        Ap = [[mp.mpf(0)] * n for _ in range(n)]
        Aq = [[mp.mpf(0)] * n for _ in range(n)]
        lam, lamq, bb, bbq = [mp.mpf(0)] * n, [mp.mpf(0)] * n, [mp.mpf(0)] * n, [mp.mpf(0)] * n
        R = [mp.mpf(0)] * n
        for j in range(C):
            for l in range(C):
                Ap[j][l] = Aq[j][l] = self.Acc[j][l]
            for tt, sh in split:
                Ap[j][C + tt] += sh * q["Mt"][j] / self.theta[tt]
                Aq[j][C + tt] += sh * q["Mt"][j] / self.theta[tt]
            lam[j] = lamq[j] = q["H"][j]
            bb[j] = bbq[j] = self.bc[j]
            R[j] = sum(q["wages"][i] * self.R[j][i] for i in range(I))
        for i in range(K):
            for l in range(K):
                Ap[C + i][C + l] = self.Ahat[i][l]
                Aq[C + i][C + l] = self.Aq[i][l]
            lam[C + i], lamq[C + i] = self.lhat[i], self.lq[i]
            bb[C + i], bbq[C + i] = self.bhat[i], self.bq[i]
        p = q["pc"] + q["p"]
        for i in range(n):
            ch[f"p = Ap + lv + b + R, row {i}"] = rel(p[i], sum(Ap[i][l] * p[l] for l in range(n)) + lam[i] * w + bb[i] + R[i])
        y = [Y * yy for yy in self.yhat] + r["X"]
        f = [y[i] - sum(Aq[l][i] * y[l] for l in range(n)) for i in range(n)]
        for j in range(C):
            assert abs(f[j] - Y * self.z[j]) <= tol * Y, ("f", j)
        for k in range(K):
            assert abs(f[C + k]) <= tol * max(y), ("f machines", k)
        ch["n_pool = lq'y"] = rel(sum(l * yy for l, yy in zip(lamq, y)), q["nD"])
        ch["T = bq'y"] = rel(sum(b * yy for b, yy in zip(bbq, y)), self.T)
        ch["p'f"] = rel(sum(pi * fi for pi, fi in zip(p, f)),
                        w * q["nD"] + sum(q["wages"][i] * q["D"][i] for i in range(I)) + self.T + r["interest"])
        # eq 11 for the pool on the clearing side
        ch["eq11 Y"] = rel(Y, self.T / r["Bq"])
        ch["eq11 n_pool"] = rel(q["nD"], self.T * r["Lq"] / r["Bq"])
        # the margin, or the corner's inequality (section 4.7)
        if r["margin"] == "Wall":
            assert w >= r["replacement_top"] * (1 - tol), "the wall's wage is above gamma(1) pi"
            assert r["x"] == 1
        elif r["margin"] == "AllHuman":
            assert w <= r["replacement_bottom"] * s, "the all-human wage is below gamma(0) pi"
            assert r["My"] == 0 and all(xx == 0 for xx in r["X"]) and r["interest"] == 0
        else:
            ch["margin"] = rel(w, self.gamma(r["x"]) * q["pi"])
        # the cheapest task type
        for k in range(K):
            if self.theta[k] > 0:
                assert q["p"][k] / self.theta[k] >= q["pi"] * (1 - tol), ("cheapest", k)
        bad = {k: v for k, v in ch.items() if v > tol}
        assert not bad, f"identities fail at {DPS} digits: {bad}"

    # ---------------------------------------------------------------- the path (section 5.4)
    def assert_path(self, result):
        """f nonincreasing on a grid of every stretch within each technique; every rho = 0
        switch downward; the envelope on the wall against argmin p_t/theta_t; the junctions."""
        first, line_sw, wall_sw = self.envelope_ext()
        ltechs = [first] + [s[2] for s in line_sw]
        q0 = self.evaluate(0, first)
        stretches = []
        if not q0["short"]:
            stretches.append([self.evaluate(0, first, w=q0["w"] * k / PATH_GRID)["f"] for k in range(1, PATH_GRID + 1)])
        xs = [self.gamma_inv(g) for g, _, _ in line_sw]
        bounds = [M(0)] + xs + [M(1)]
        for r, t in enumerate(ltechs):
            a, b = bounds[r], bounds[r + 1]
            stretches.append([self.evaluate(a + (b - a) * k / PATH_GRID, t)["f"] for k in range(PATH_GRID + 1)])
        one = self.evaluate(1, ltechs[-1])
        wts = [ltechs[-1]] + [s[2] for s in wall_sw]
        wws = [one["w"]] + [self.closure_wage(s[1], s[0]) for s in wall_sw] + [one["w"] * 20 + 20]
        for r, t in enumerate(wts):
            a, b = wws[r], wws[r + 1]
            stretches.append([self.evaluate(1, t, w=a + (b - a) * k / PATH_GRID)["f"] for k in range(PATH_GRID + 1)])
        for vals in stretches:
            for f0, f1 in zip(vals, vals[1:]):
                assert f1 <= f0 + TOL * (1 + abs(f0)) or f0 == INF, "f rises within a stretch"
        # at rho = 0 every switch lowers labour demand (the Proposition of section 5.4)
        if self.rho == 0:
            for (g, b, a), x in zip(line_sw, xs):
                assert self.evaluate(x, a)["nD"] < self.evaluate(x, b)["nD"]
            for g, b, a in wall_sw:
                v = self.closure_wage(b, g)
                assert self.evaluate(1, a, w=v)["nD"] < self.evaluate(1, b, w=v)["nD"]
        # the envelope on the wall is the cheapest type at each wage
        v1 = one["w"]
        for k in range(WALL_SCAN + 1):
            v = v1 * (1 + mp.mpf(k) / 10)
            t = wts[0]
            for (g, b, a), vs in zip(wall_sw, wws[1:]):
                if v >= vs:
                    t = a
            costs = [((v * self.lt[tt] + self.bt[tt]) / self.theta[tt]) for tt in range(self.K) if self.theta[tt] > 0]
            got = (v * self.lt[t] + self.bt[t]) / self.theta[t]
            assert rel(got, min(costs)) < TOL, ("wall envelope", v, t)
        # the corners' evaluation equals the line's at both junctions (Lemma 4')
        for x, t in ((0, first), (1, ltechs[-1])):
            ql = self.evaluate(x, t)
            if ql["short"]:
                continue
            qc = self.evaluate(x, t, w=ql["w"])
            assert rel(ql["f"], qc["f"]) < TOL and rel(ql["Ps"], qc["Ps"]) < TOL, ("junction", x)


# ------------------------------------------------------------------------------ instances
def goodspace(**over):
    """1a's G1 in 1d form: one type, the good and space, one flow machine (section 3.3, W)."""
    p = dict(N="4", T="10", h="1", a="0.3", lam="0.05", b="0.4", eta="1", g0="0.2", g1="0.8",
             k="1", chi="1", rho="0", delta="1", J=1)
    p.update(over)
    return Economy(workers=(worker("WORKER", p["N"], p["chi"]),), land=p["T"], eta=p["eta"], g0=p["g0"],
                   g1=p["g1"], k=p["k"], rho=p["rho"], edges=("0", "1"),
                   categories=(("GOOD", "1", "0", ("1",)), ("SPACE", p["h"], "1", ("0",))),
                   intermediate=(("0", "0"), ("0", "0")),
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), ((p["a"],), p["lam"], p["b"]),
                                           p["delta"], p["J"]),))


def baumol(workers=None, eta="1", LH="0.25", N="4", reserved=None):
    """B: services (mu 0.75, L^H 0.25), goods (mu 1) and space on one segment, Appendix B's
    machine (section 3.3)."""
    ws = workers or (worker("WORKER", N, "1"),)
    return Economy(workers=ws, land="10", eta=eta, g0="0.2", g1="0.8", k="1", rho="0", edges=("0", "1"),
                   categories=(("SERVICES", "1", "0", ("0.75",)), ("GOODS", "1", "0", ("1",)),
                               ("SPACE", "1", "1", ("0",))),
                   intermediate=(("0",) * 3,) * 3,
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), (("0.3",), "0.05", "0.4"), "1", 1),),
                   required=(LH, "0", "0"), reserved=reserved)


def entrant_trained(NE, NT, eta="1", chiE="1"):
    """E: the entrant (N_E, chi, 1, 1) and the trained (N_T, 0.8, 1.5, 1.2) with reserved hours
    0.1 per unit of services and 0.02 per unit of goods (section 3.3)."""
    ws = (worker("ENTRANT", NE, chiE), worker("TRAINED", NT, "0.8", eps="1.5", sig="1.2"))
    return baumol(workers=ws, eta=eta, reserved=(("0", "0.1"), ("0", "0.02"), ("0", "0")))


def three_types(NT, NM):
    """E7, E8: E's economy with the master (N_M, 0.6, 1.8, 1.5), reserved 0.05 per unit of goods,
    and the trained's reserved hours 0.1 per unit of services only (section 3.3)."""
    ws = (worker("ENTRANT", "8", "1"), worker("TRAINED", NT, "0.8", eps="1.5", sig="1.2"),
          worker("MASTER", NM, "0.6", eps="1.8", sig="1.5"))
    return baumol(workers=ws, reserved=(("0", "0.1", "0"), ("0", "0", "0.05"), ("0", "0", "0")))


def full(NE, NT, eta):
    """F: 1c's M4 with L^H_care 0.2, the entrant and the trained (N_T, 0.8, 1.4, 1.2) with
    reserved hours 0.05 per unit of food and 0.3 per unit of care (section 3.3)."""
    cats, acc = g1c.fork_categories()
    ws = (worker("ENTRANT", NE, "1"), worker("TRAINED", NT, "0.8", eps="1.4", sig="1.2"))
    return Economy(workers=ws, land="10", eta=eta, g0="0.2", g1="0.8", k="1", rho="0.04",
                   edges=("0", "0.4", "0.75", "1"), categories=cats, intermediate=acc, types=g1c.M4_TYPES,
                   required=("0", "0", "0.2", "0"),
                   reserved=(("0", "0"), ("0", "0.05"), ("0", "0.3"), ("0", "0")))


def as_1d(e):
    """The 1d form of a generate_1c Economy: one type, no H, no reserved hours (D0)."""
    types = []
    for k in range(e.K):
        types.append(g1c.machine_type(e.tn[k], e.theta[k], (tuple(e.aop[k]), e.lop[k], e.bop[k]),
                                      (tuple(e.aI[k]), e.lI[k], e.bI[k]), e.delta[k], e.lag[k]))
    cats = tuple((e.names[j], e.z[j], e.bc[j], tuple(e.mu[j])) for j in range(e.C))
    return Economy(workers=(worker("WORKER", e.N, e.chi_max),), land=e.T, eta=e.eta, g0=e.g0, g1=e.g1, k=e.k,
                   rho=e.rho, edges=tuple(e.e), categories=cats, intermediate=tuple(tuple(r) for r in e.Acc),
                   types=tuple(types))


def wall_switch_economy(N):
    """X: 1c's M5 at rho 0.15 and chi_max 3, at N (section 3.3)."""
    return as_1d(g1c.Economy(workers=N, land="10", eta="1", g0="0.2", g1="0.8", k="1", chi_max="3", rho="0.15",
                             edges=("0", "1"), categories=(("GOOD", "1", "0", ("1",)), ("SPACE", "1", "1", ("0",))),
                             intermediate=(("0", "0"), ("0", "0")), types=(g1c.FLOW, g1c.DURABLE)))


def solved(econ, margin=None):
    regime, r = econ.solve()
    assert regime in ("Contestable", "Wall", "AllHuman"), (regime, r.get("changes"), r.get("f_end"))
    if margin is not None:
        assert regime == margin, (regime, margin)
    econ.assert_path(r)
    return r


def assert_nests(e, label):
    """The one-type form of a generate_1c economy (section 7): its equilibrium equals 1c's to
    1e-65; its line values equal 1c's boundary diagnostics; NotViable's d(1) is 1c's."""
    reg_c, qc = e.solve()
    d = as_1d(e)
    reg_d, qd = d.solve()
    if reg_c in ("Interior", "Tie"):
        assert reg_d == "Contestable", (label, reg_d)
        assert (qd["tie"] is None) == (qc["tie"] is None), label
        pairs = dict(x=(qd["x"], qc["x"]), v=(qd["w"], qc["v"]), Ps=(qd["Ps"], qc["Ps"]), Y=(qd["Y"], qc["Y"]),
                     n=(qd["n_a"], qc["N_a"]), income=(qd["income"], qc["income"]),
                     interest=(qd["interest"], qc["interest"]))
        for j in range(e.C):
            pairs[f"p{j}"] = (qd["cats"][j]["p"], qc["cats"][j]["p"])
        worst = max(abs(a - b) / max(abs(b), mp.mpf(10) ** -300) for a, b in pairs.values())
        assert worst < TOL, (label, worst)
        return worst, reg_c, reg_d, qd
    if reg_c == "NotViable":
        assert reg_d == "NotViable" and qd["d_at_1"] == qc["d_at_1"], label
        return mp.mpf(0), reg_c, reg_d, qd
    if reg_c == "BoundaryNoMargin":
        worst = rel(qd["f_line_1"], qc["f_at_1"])
    elif reg_c == "NoInteriorAtZero":
        worst = rel(qd["f_line_lo"], qc["f_at_0"])
    else:  # MultipleEquilibria
        worst = mp.mpf(0)
    assert worst < TOL, (label, worst)
    return worst, reg_c, reg_d, qd


# ------------------------------------------------------------------------------ output
LINES = []


def section(title):
    LINES.append("")
    LINES.append(f"# {title}")


def put(key, value, note):
    """One golden, printed to SIG_OUT digits."""
    text = mp.nstr(value, SIG_OUT, min_fixed=-mp.inf, max_fixed=mp.inf)
    LINES.append(f"{key} = {text}  # {note}")


def build():
    worst = mp.mpf(0)
    # ------------------------------------------------------------------ nesting (D0, d1)
    ab_edges, ab_cats = g1b.appendix_b_form()
    for econ, label in ((g1c.m3(), "M3"), (g1c.m3(rho="0"), "M3z"), (g1c.m4(), "M4"),
                        (g1c.m4(eta="2"), "M4 eta 2"), (g1c.m4(eta="0.5"), "M4 eta 0.5"),
                        (g1c.m4(eta="0.5", workers="8"), "M4t"),
                        (g1c.one_type_form(ab_edges, ab_cats), "G1"),
                        (g1c.one_type_form(ab_edges, ab_cats, rho="0.05", delta="0.1", build_lag=3), "G4 D"),
                        (g1c.one_type_form(g1b.FORK_EDGES, g1b.FORK), "C3"),
                        (g1c.one_type_form(g1b.GAP_EDGES, g1b.GAP, workers="5"), "gap")):
        w, reg_c, reg_d, _ = assert_nests(econ, label)
        worst = max(worst, w)
    for rho in ("0", "0.05", "0.1", "0.15", "0.3"):
        w, _, _, _ = assert_nests(g1c.m5(rho), f"M5 rho {rho}")
        worst = max(worst, w)
    # 1c's boundary rows, solved; M5m still three equilibria
    section("D0: unit 1c's boundary rows in one-type form, solved (docs/unit-1d.md section 3.3)")
    w, reg_c, reg_d, qd = assert_nests(g1c.m4(eta="0.25"), "M4 eta 0.25")
    assert reg_c == "BoundaryNoMargin" and reg_d == "Wall"
    worst = max(worst, w)
    put("D0_M4_ETA025_V", qd["w"], "M4 at eta 0.25 (1c's BoundaryNoMargin): the wall's wage")
    put("D0_M4_ETA025_F_LINE_1", qd["f_line_1"], "its f on the line at 1, 1c's f_at_1")
    w, reg_c, reg_d, qd = assert_nests(g1c.m4(workers="200", chi_max="0.01"), "M4 N 200")
    assert reg_c == "NoInteriorAtZero" and reg_d == "AllHuman"
    worst = max(worst, w)
    put("D0_M4_N200_V", qd["w"], "M4 at N 200, chi_max 0.01 (1c's NoInteriorAtZero): the all-human wage")
    put("D0_M4_N200_F_LINE_LO", qd["f_line_lo"], "its f on the line at 1e-12, 1c's f_at_0")
    w, reg_c, reg_d, qd = assert_nests(g1c.m4(eta="50"), "M4 eta 50")
    assert reg_c == reg_d == "NotViable"
    w, reg_c, reg_d, qd = assert_nests(g1c.m5("0.1", workers="60", space="0.2"), "M5m")
    assert reg_c == reg_d == "MultipleEquilibria" and qd["changes"] == 3
    # M5b (unit-1c.md section 3.3, H4 and H1): 1c counts three, 1d two, its wall short to the end
    for chi in ("1", "0.01"):
        m5b = g1c.Economy(workers="15", land="10", eta="3", g0="0.2", g1="0.8", k="1", chi_max=chi, rho="0.1",
                          edges=("0", "1"), categories=(("GOOD", "1", "0", ("1",)), ("SPACE", "0.05", "1", ("0",))),
                          intermediate=(("0", "0"), ("0", "0")),
                          types=(g1c.machine_type("FLOW", "1", (("0", "0"), "0.3", "1"), g1c.zero_recipe(2), "1", 1),
                                 g1c.machine_type("LAB", "1", (("0", "0"), "0.25", "0"), (("0", "0"), "0", "9"), "0.01", 3)))
        rc, qc = m5b.solve()
        rd, qd = as_1d(m5b).solve()
        assert rd == "MultipleEquilibria" and qd["changes"] == 2 and qd["f_end"] > 0
        # 1c's count (unit-1c.md section 5.3 after P1.6; generate_1c.py stops at f(1) >= 0 and
        # says BoundaryNoMargin): the line's values with f(1) >= 0 positive and the boundary one more
        assert rc == "BoundaryNoMargin" and rel(qc["f_at_1"], qd["f_line_1"]) < TOL
        line = [f for kind, f in qd["seq"] if kind not in ("Zero", "End") and not (isinstance(kind, tuple) and kind[0].startswith("Wall"))]
        sides = [True] + [f > 0 for f in line[:-1]] + [line[-1] >= 0, False]
        assert sum(1 for a, b in zip(sides, sides[1:]) if a != b) == 3
        if chi == "1":
            put("D0_M5B_F_END", qd["f_end"], "M5b (H4): f at the end of the wall, labour-short: two sign changes, not 1c's three")

    # ------------------------------------------------------------------ W
    section("W: the corners in Appendix B's closure, 1a's G8 rows in one-type form (docs/unit-1d.md section 3.3)")
    r = solved(goodspace(lam="0.6"), "Wall")
    L_s, B_s = r["L_s"], r["B_s"]
    zeta = mp.expm1(mp.mpf(1) * r["n_pool"] / 4)
    assert rel(r["w"], zeta * B_s / (1 - zeta * L_s)) < TOL and rel(r["real_wage"], zeta) < TOL
    assert rel(r["Y"], mp.mpf(350) / 47) < TOL and rel(r["n_a"], mp.mpf(180) / 47) < TOL
    _, g8 = g1a.Economy(lam="0.6").solve()
    assert rel(r["f_line_1"], g8["f_at_1"]) < TOL
    assert rel(r["p_machine"][0], (mp.mpf("0.6") * r["w"] + mp.mpf("0.4")) / mp.mpf("0.7")) < TOL
    assert r["provider"] < 0
    for key, val, note in (("V", r["w"], "the wall's wage, zeta B_s/(1 - zeta L_s)"),
                           ("G", r["g"], "g = v/pi, above gamma(1) = 1"),
                           ("P_M", r["p_machine"][0], "p_m = pi = (lambda v + b)/(1 - a) at the wall's own wage"),
                           ("REPLACEMENT_TOP", r["replacement_top"], "gamma(1) pi"),
                           ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y = 350/47"), ("N_A", r["n_a"], "N_a = 180/47"),
                           ("INCOME", r["income"], "I"), ("LABOR_SHARE", r["labor_share"], "v N_a/I"),
                           ("REAL_WAGE", r["real_wage"], "v/P_s = zeta = expm1(45/47)"),
                           ("PROVIDER_BASKETS", r["provider"], "provider baskets: not funded"),
                           ("F_LINE_1", r["f_line_1"], "f on the line at 1, 1a's G8 golden"),
                           ("F_END", r["f_end"], "f at the end of the wall = -8/47")):
        put(f"W1_{key}", val, note)
    regime, r = goodspace(N="0.25").solve()
    assert regime == "LaborShort" and rel(r["f_end"], mp.mpf(13) / 188) < TOL and r["reserved_short"] is None
    _, g8 = g1a.Economy(workers="0.25").solve()
    assert rel(r["f_line_1"], g8["f_at_1"]) < TOL
    put("W2_EXCESS", r["f_end"], "LaborShort: f_infinity = n_D(1) - N = 13/188")
    put("W2_F_LINE_1", r["f_line_1"], "f on the line at 1, 1a's G8 golden")
    regime, r = goodspace(lam="0.6", chi="3").solve()
    assert regime == "LaborShort" and rel(r["omega_end"], mp.mpf(35) / 18) < TOL
    put("W3_EXCESS", r["f_end"], "LaborShort at the real-wage ceiling: f_infinity")
    put("W3_OMEGA_END", r["omega_end"], "omega_infinity = 1/L_s = 35/18")
    r = solved(goodspace(N="20", chi="0.05"), "AllHuman")
    om = mp.expm1(mp.mpf(1) / 40)
    assert rel(r["w"], om / (1 - om)) < TOL and r["w"] < r["replacement_bottom"]
    _, g8 = g1a.Economy(workers="20", chi_max="0.05").solve()
    assert rel(r["f_line_lo"], g8["f_at_0"]) < TOL
    for key, val, note in (("V", r["w"], "the all-human wage omega/(1 - omega), omega = expm1(1/40)"),
                           ("REPLACEMENT_BOTTOM", r["replacement_bottom"], "gamma(0) pi"),
                           ("PI", r["pi"], "pi, the delivered machine-task price at the wage"),
                           ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y = T/h"), ("N_A", r["n_a"], "N_a"),
                           ("INCOME", r["income"], "I"),
                           ("F_LINE_0", r["f_line_0"], "f on the line at x = 0"),
                           ("F_LINE_LO", r["f_line_lo"], "f on the line at 1e-12, 1a's G8 golden")):
        put(f"W4_{key}", val, note)
    w5 = goodspace(N="10", T=float("10.000000000005"), chi=float("0.001"))
    r = solved(w5, "Contestable")
    assert r["x"] < LO
    for key, val, note in (("X_STAR", r["x"], "x*, a root below 1e-12 (T and chi_max the doubles)"),
                           ("V", r["w"], "v"), ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"),
                           ("F_LINE_0", r["f_line_0"], "f on the line at x = 0"),
                           ("F_LINE_LO", r["f_line_lo"], "f on the line at 1e-12")):
        put(f"W5_{key}", val, note)
    r_dec = solved(goodspace(N="10", T="10.000000000005", chi="0.001"), "Contestable")
    assert rel(r_dec["x"], r["x"]) > mp.mpf("8e-5")
    put("W5_X_STAR_DECIMAL", r_dec["x"], "x* with the decimal inputs, for reference: 8.9e-5 away")
    near_one = "0.99999999999999988897769753748434595763683319091796875"
    assert M(near_one) == 1 - mp.mpf(2) ** -53
    r = solved(goodspace(a=near_one, lam="0"), "Contestable")
    assert r["x"] < LO
    _, g8 = g1a.Economy(a=near_one, lam="0").solve()
    assert rel(r["f_line_lo"], g8["f_at_0"]) < TOL
    for key, val, note in (("X_STAR", r["x"], "x*, a root below 1e-12 at a = 1 - 2^-53, lambda 0"),
                           ("V", r["w"], "v"), ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"),
                           ("F_LINE_0", r["f_line_0"], "f on the line at x = 0"),
                           ("F_LINE_LO", r["f_line_lo"], "f on the line at 1e-12")):
        put(f"W6_{key}", val, note)

    # ------------------------------------------------------------------ B
    section("B1: the human-required economy, N 8, eta 1: a contestable margin with the tail")
    r = solved(baumol(N="8"), "Contestable")
    assert rel(r["required_hours"], r["Y"] / 4) < TOL
    for key, val, note in (("X_STAR", r["x"], "x*"), ("ONE_MINUS_X_STAR", r["one_minus_x"], "1 - x*"),
                           ("V", r["w"], "v"), ("P_M", r["p_machine"][0], "p_m"), ("P_S", r["Ps"], "P_s"),
                           ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"),
                           ("FINAL_HOURS", r["final_hours"], "Y (H_y + L^H_y), with the tail"),
                           ("REQUIRED_HOURS", r["required_hours"], "Y L^H_y = Y/4"), ("INCOME", r["income"], "I"),
                           ("P_SERVICES", r["cats"][0]["p"], "p of services"), ("P_GOODS", r["cats"][1]["p"], "p of goods"),
                           ("F_LINE_1", r["f_line_1"], "f on the line at 1")):
        put(f"B1_{key}", val, note)
    wrong = solved(baumol(N="8", LH="0"), "Contestable")
    assert abs(wrong["x"] - r["x"]) > mp.mpf("0.1")

    section("BP: check_pinning D1's automation path, N 8, every point at the wall (docs/unit-1d.md section 4.8)")
    last = None
    for tag, eta in (("ETA0_3", "0.3"), ("ETA0_1", "0.1"), ("ETA0_01", "0.01"), ("ETA0_001", "0.001"), ("ETA1EM6", "1e-6")):
        r = solved(baumol(N="8", eta=eta), "Wall")
        services, goods = r["cats"][0], r["cats"][1]
        pm = r["p_machine"][0]
        assert rel(pm, (mp.mpf("0.05") * r["w"] + mp.mpf("0.4")) / mp.mpf("0.7")) < TOL
        ratio = services["p"] / goods["p"]
        share = g1b.ces_share("0.3", "0.5", ratio)
        vals = dict(V=r["w"], P_M=pm, P_SERVICES_OVER_V=services["p"] / r["w"], PHI_W_SERVICES=services["phi_w"],
                    P_GOODS=goods["p"], RELATIVE_PRICE=ratio, CES_SHARE=share,
                    V_OVER_REPLACEMENT=r["w"] / r["replacement_top"])
        if last is not None:
            assert vals["P_SERVICES_OVER_V"] < last["P_SERVICES_OVER_V"] and vals["PHI_W_SERVICES"] > last["PHI_W_SERVICES"]
            assert vals["CES_SHARE"] > last["CES_SHARE"] and vals["P_GOODS"] < last["P_GOODS"]
            assert vals["V_OVER_REPLACEMENT"] > last["V_OVER_REPLACEMENT"] and vals["V"] < last["V"]
        last = vals
        notes = dict(V="the wall's wage", P_M="p_m = (lambda v + b)/(1 - a)", P_SERVICES_OVER_V="p_services/v -> 1/4",
                     PHI_W_SERVICES="labour's share of services' price -> 1", P_GOODS="p_goods -> 0",
                     RELATIVE_PRICE="p_services/p_goods, diverging",
                     CES_SHARE="ces_share(0.3, 0.5, p_services/p_goods) -> 1",
                     V_OVER_REPLACEMENT="v/(gamma(1) pi), growing as 1/eta: the machine no longer prices the hour")
        for key, val in vals.items():
            put(f"BP_{tag}_{key}", val, f"{notes[key]} at eta {eta}")
    zeta = mp.expm1(mp.mpf(5) / 16)
    v_inf = zeta / (1 - zeta / 4)
    assert abs(last["V"] - v_inf) < mp.mpf("1e-6")
    section("BLIM: the limit of BP as eta -> 0")
    put("BLIM_V_INF", v_inf, "v_infinity = zeta/(1 - zeta/4), zeta = expm1(5/16)")

    section("B2: N 4, the tail decides the regime")
    r = solved(baumol(N="4", LH="0"), "Contestable")
    assert r["provider"] > 0
    for key, val, note in (("X_STAR", r["x"], "x* without the tail"), ("V", r["w"], "v"), ("P_S", r["Ps"], "P_s"),
                           ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"), ("PROVIDER_BASKETS", r["provider"], "provider baskets")):
        put(f"B2_NO_TAIL_{key}", val, note)
    r = solved(baumol(N="4"), "Wall")
    L_s, B_s = r["L_s"], r["B_s"]
    zeta = mp.expm1(r["n_pool"] / 4)
    assert rel(r["w"], zeta * B_s / (1 - zeta * L_s)) < TOL and r["provider"] > 0
    assert rel(r["Y"], mp.mpf("6.25")) < TOL and rel(r["n_a"], mp.mpf(65) / 32) < TOL
    for key, val, note in (("V", r["w"], "the wall's wage with the tail"), ("G", r["g"], "g = v/pi"),
                           ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y = 6.25"), ("N_A", r["n_a"], "N_a = 65/32"),
                           ("PROVIDER_BASKETS", r["provider"], "provider baskets"),
                           ("F_LINE_1", r["f_line_1"], "f on the line at 1")):
        put(f"B2_TAIL_{key}", val, note)

    # ------------------------------------------------------------------ E
    section("E: entrant and trained, on B's economy (docs/unit-1d.md section 3.3)")
    for tag, (NE, NT, eta, chiE), margin, walled in (
            ("E1", ("8", "3", "1", "1"), "Contestable", False), ("E2", ("8", "1", "1", "1"), "Contestable", True),
            ("E3", ("8", "2", "0.3", "1"), "Wall", True), ("E4", ("8", "3", "0.3", "1"), "Wall", False),
            ("E5", ("40", "2", "1", "0.05"), "AllHuman", True)):
        r = solved(entrant_trained(NE, NT, eta, chiE), margin)
        trained = r["types"][1]
        assert trained["pooled"] == (not walled)
        if not walled:
            assert trained["premium"] == 1
        vals = [("V", r["w"], "v, the pool's wage"), ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"),
                ("N_A", r["n_a"], "N_a, hours of both types"), ("N_POOL", r["n_pool"], "the pool's efficiency hours"),
                ("INCOME", r["income"], "I"),
                ("ENTRANT_HOURS", r["types"][0]["hours"], "the entrant's hours"),
                ("TRAINED_WAGE", trained["wage"], "the trained's wage"),
                ("TRAINED_PREMIUM", trained["premium"], "the trained's premium v_T/(1.5 v)"),
                ("TRAINED_HOURS", trained["hours"], "the trained's hours"),
                ("TRAINED_RESERVED_HOURS", trained["reserved_hours"], "the trained's reserved hours D_T")]
        if margin == "Contestable":
            vals.insert(0, ("X_STAR", r["x"], "x*"))
        if margin == "AllHuman":
            vals.append(("REPLACEMENT_BOTTOM", r["replacement_bottom"], "gamma(0) pi"))
        for key, val, note in vals:
            put(f"{tag}_{key}", val, note)
    regime, r = entrant_trained("8", "0.5").solve()
    assert regime == "LaborShort" and r["reserved_short"] == 1 and r["f_end"] == INF
    assert all(f == INF for _, f in r["seq"])
    econ = entrant_trained("8", "0.5")
    put("E6_TRAINED_DEMAND_AT_1", econ.evaluate(1, 0)["D"][1], "the trained's reserved demand at x = 1, above N_T 0.5")

    section("E7 and E8: three types and the walk")
    r = solved(three_types("3", "0.6"), "Contestable")
    ty = r["types"]
    assert ty[1]["pooled"] and not ty[2]["pooled"]
    q = r["q"]
    thresholds = [q["e"][i] / q["c"][i] if q["c"][i] > 0 else None for i in range(3)]
    assert thresholds[2] < thresholds[1]
    for key, val, note in (("X_STAR", r["x"], "x*"), ("V", r["w"], "v"), ("P_S", r["Ps"], "P_s"),
                           ("TRAINED_WAGE", ty[1]["wage"], "the trained's wage, pooled"),
                           ("MASTER_WAGE", ty[2]["wage"], "the master's wage, at its wall"),
                           ("MASTER_PREMIUM", ty[2]["premium"], "the master's premium"),
                           ("MASTER_HOURS", ty[2]["hours"], "the master's hours"),
                           ("TRAINED_THRESHOLD", thresholds[1], "the P_s above which the trained is walled"),
                           ("MASTER_THRESHOLD", thresholds[2], "the P_s above which the master is walled, below the trained's")):
        put(f"E7_{key}", val, note)
    r = solved(three_types("1.53", "0.5"), "Contestable")
    ty = r["types"]
    assert not ty[1]["pooled"] and not ty[2]["pooled"]
    q = r["q"]
    e, c = q["e"], q["c"]
    p_all = q["Pbase"] + sum(e)
    p_master = (q["Pbase"] + e[0] + e[1]) / (1 - c[2])
    t_trained = e[1] / c[1]
    assert p_all < t_trained < p_master and e[2] / c[2] < p_all
    for key, val, note in (("X_STAR", r["x"], "x*"), ("V", r["w"], "v"), ("P_S", r["Ps"], "P_s, both walled"),
                           ("P_S_MASTER_WALLED", p_master, "P after walling the master alone, above the trained's threshold"),
                           ("TRAINED_THRESHOLD", t_trained, "the trained's threshold"),
                           ("TRAINED_WAGE", ty[1]["wage"], "the trained's wage"),
                           ("TRAINED_PREMIUM", ty[1]["premium"], "the trained's premium"),
                           ("MASTER_WAGE", ty[2]["wage"], "the master's wage"),
                           ("MASTER_PREMIUM", ty[2]["premium"], "the master's premium")):
        put(f"E8_{key}", val, note)

    # ------------------------------------------------------------------ F
    section("F: 1c's M4 with L^H_care 0.2, the entrant and the trained (docs/unit-1d.md section 3.3)")
    econ = full("4", "3", "1")
    r = solved(econ, "Contestable")
    ty = r["types"][1]
    assert not ty["pooled"] and r["t"] == 1
    direct_care = ty["wage"] * mp.mpf("0.3")
    assert r["cats"][2]["reserved_cost"] > direct_care * (1 + mp.mpf("0.01"))
    for key, val, note in (("X_STAR", r["x"], "x*, with the engine"), ("V", r["w"], "v"), ("P_S", r["Ps"], "P_s"),
                           ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"), ("N_POOL", r["n_pool"], "the pool's hours"),
                           ("INCOME", r["income"], "I"), ("INTEREST", r["interest"], "interest"),
                           ("TRAINED_WAGE", ty["wage"], "the trained's wage, at its wall"),
                           ("TRAINED_PREMIUM", ty["premium"], "the trained's premium"),
                           ("TRAINED_HOURS", ty["hours"], "the trained's hours")):
        put(f"F1_{key}", val, note)
    for j, name in enumerate(econ.names):
        put(f"F1_{name}_P", r["cats"][j]["p"], f"p of {name.lower()}")
        put(f"F1_{name}_RESERVED_COST", r["cats"][j]["reserved_cost"], f"reserved cost of {name.lower()} through the chain")
    r = solved(full("4", "3", "0.25"), "Wall")
    (gs, below, above), = r["wall_sw"]
    assert (below, above) == (0, 1) and r["t"] == 1 and r["ltechs"] == [0]
    for key, val, note in (("V", r["w"], "the wall's wage, with the engine"), ("G", r["g"], "g = v/pi"),
                           ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"),
                           ("WALL_SWITCH_GAMMA", gs, "gamma_s of the switch loom -> engine on the wall"),
                           ("WALL_SWITCH_V", r["wws"][0], "v_s, the loom's closure wage at gamma_s"),
                           ("TRAINED_PREMIUM", r["types"][1]["premium"], "the trained's premium")):
        put(f"F2_{key}", val, note)
    econ = full("16", "1.5", "0.5")
    r = solved(econ, "Contestable")
    assert r["tie"] is not None and not r["types"][1]["pooled"]
    closed = econ.closed_form_share(r["x"], r["t"], r["tie"][0], None)
    assert abs(closed - r["tie"][1]) > mp.mpf("1e-4") * r["tie"][1]
    xsw = r["x"]
    fs = [econ.evaluate(xsw, r["t"], split=[(r["t"], 1 - M(k) / 50), (r["tie"][0], M(k) / 50)])["f"] for k in range(51)]
    assert all(b < a for a, b in zip(fs, fs[1:]))
    for key, val, note in (("X_STAR", xsw, "x* = the switch loom -> engine on the line"),
                           ("V", r["w"], "v"), ("SHARE", r["tie"][1], "sigma, the engine's share, by bisection"),
                           ("SHARE_CLOSED_FORM", closed, "1c's closed form at the same switch, for reference"),
                           ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"),
                           ("TRAINED_PREMIUM", r["types"][1]["premium"], "the trained's premium")):
        put(f"F3_{key}", val, note)

    # ------------------------------------------------------------------ X
    section("X: switches at the wall, 1c's M5 at rho 0.15, chi_max 3 (docs/unit-1d.md section 3.3)")
    r = solved(wall_switch_economy("0.5"), "Wall")
    (gs, below, above), = r["wall_sw"]
    assert (below, above) == (0, 1) and r["tie"] is not None
    put("X_WALL_SWITCH_GAMMA", gs, "gamma_s of the switch flow -> durable, above gamma(1)")
    put("X_WALL_SWITCH_V", r["wws"][0], "v_s, the flow type's closure wage at gamma_s")
    for key, val, note in (("SHARE", r["tie"][1], "sigma toward the durable type"), ("P_S", r["Ps"], "P_s"),
                           ("Y", r["Y"], "Y"), ("N_A", r["n_a"], "N_a"), ("INTEREST", r["interest"], "interest")):
        put(f"X1_{key}", val, note)
    r = solved(wall_switch_economy("1"), "Wall")
    assert r["t"] == 0 and r["tie"] is None
    for key, val, note in (("V", r["w"], "v, the flow type"), ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y = 100/13"),
                           ("N_A", r["n_a"], "N_a = 6/13")):
        put(f"X2_{key}", val, note)
    r = solved(wall_switch_economy("0.003"), "Wall")
    assert r["t"] == 1 and r["tie"] is None
    for key, val, note in (("V", r["w"], "v, the durable type"), ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"),
                           ("N_A", r["n_a"], "N_a"), ("INTEREST", r["interest"], "interest")):
        put(f"X3_{key}", val, note)

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens_1d.txt: the golden numbers for oracle unit 1d.",
        f"# Written by goldens/generate_1d.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits.",
        "# Equations: docs/unit-1d.md section 4. laborformal references are at 31b3482.",
        f"# The one-type form nests generate_1c.py's solves within {mp.nstr(worst, 3)} (1e-65 asserted).",
        f"# fnv1a64 generate_1d.py = {fnv1a64(source_bytes(__file__)):016x}",
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
                        help="compare with goldens_1d.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(HERE, "goldens_1d.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same = fh.read() == text
        print("goldens_1d.txt is current" if same else "goldens_1d.txt differs from generate_1d.py's output")
        return 0 if same else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
