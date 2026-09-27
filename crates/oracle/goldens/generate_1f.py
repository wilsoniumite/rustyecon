"""generate_1f.py: the golden numbers for oracle unit 1f.

Dated 2026-09-27. Every golden that crates/oracle's unit-1f tests use is computed here with
mpmath at 70 digits, from the equations of docs/unit-1f.md section 4: SSRN A.1's government (a
payroll tax on gross wages, a uniform tax on final purchases, a tax on market rent, a uniform
transfer and a transfer program), its budget closed by the owners' levy or by the uniform
transfer (SSRN section 7, Prop 6, eq 16; A.1), one participation rule with the support, the exit
life and the government (SSRN eq 8-9, p.16, D.1 eq 27), the households' accounts and the income
identity with government (App. C), three-taxes' ledger, legs and circular flow at the equilibrium
(three-taxes/checks/check_three_taxes.py T1 :27-61, T6 :63-85, T5 :105-134), each tax's incidence
as the paper states it (SSRN D.2, D.5; main.tex:819-833), and a CES basket over the categories
(SSRN eq 26; main.tex:806-811) whose unit-elasticity case is check_pinning.py's A-joint household
(:225-267). laborformal has no equilibrium with a government (docs/unit-1f.md section 0.1):
every instance is constructed.

The unit-1e generator (generate_1e.py, and through it generate_1d.py, generate_1c.py,
generate_1b.py and generate.py) supplies the machine block, the tasks, the Leontief solves, the
worker types, the walk, the corners, the exit sub-problem, the idle stretch and the path, which
this file's Economy extends (docs/unit-1f.md section 7), and its solves are used to assert that
the household form nests them. Nothing is imported from laborformal or from the oracle.

Run with any Python that has mpmath (1.3.0 was used), from this directory or any other:

    python goldens/generate_1f.py           # writes goldens/goldens_1f.txt beside this file
    python goldens/generate_1f.py --check   # exits 1 if goldens_1f.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in tests/gate/goldens_1f.rs
are these values rounded to 20 significant digits, and the test
f11_goldens_file::constants_match_goldens_1f_txt enforces that.

The header of goldens_1f.txt records seven FNV-1a 64-bit digests: of this file, of
generate_1e.py, generate_1d.py, generate_1c.py, generate_1b.py and generate.py, and of the
goldens that follow the header, each with CRLF read as LF. The gate recomputes all seven.
"""

import argparse
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import mpmath as mp  # noqa: E402

import generate_1e as g1e  # noqa: E402  (imports generate_1d.py and the rest; 70 digits)

g1d = g1e.g1d
g1c = g1d.g1c
g1b = g1c.g1b
g1a = g1c.g1a

DPS = 70
"""Working precision in decimal digits, as in the other generators."""
SIG_OUT = 30
"""Significant digits written per value; the Rust constants keep 20 of them."""
BISECTIONS = g1e.BISECTIONS
HALVINGS = g1e.HALVINGS
SCAN = g1e.SCAN
"""Interior points per piece in the count: four times the oracle's EXIT_SCAN (section 7)."""
GRID = 16
"""Points per piece at which f is asserted nonincreasing and Lemma F1 is checked."""
DERIVATIVE = mp.mpf(10) ** -30
"""The step of every exact derivative: a symmetric difference at 70 digits, good to 1e-35."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""

mp.mp.dps = DPS
assert g1e.DPS == DPS

M, rel, leontief, transpose = g1c.M, g1c.rel, g1c.leontief, g1c.transpose
TOL = IDENTITY_TOL
LO = g1e.LO
INF = mp.inf
F = g1d.F
worker = g1d.worker
priced = g1e.priced


def gov(tw="0", tc="0", budget=("rent", "0"), mw="0", me="0", mode="S"):
    """A government (docs/unit-1f.md section 3.1): budget ("rent", d-hat) for RentRate or
    ("dividend", tau_R) for Dividend; mode "S" (Supplement) or "R" (Replace)."""
    return dict(tw=M(tw), tc=M(tc), budget=(budget[0], M(budget[1])), mw=M(mw), me=M(me), mode=mode)


NONE = gov()


class Economy(g1e.Economy):
    """Unit 1e's economy with a basket and a government (docs/unit-1f.md sections 2-4). basket
    is None for the fixed basket z, or sigma for the CES over the categories with weights z."""

    def __init__(self, *, basket=None, government=None, **kw):
        super().__init__(**kw)
        g = government or NONE
        self.tw, self.tc, self.mw, self.me = g["tw"], g["tc"], g["mw"], g["me"]
        self.budget = g["budget"]
        self.replace = g["mode"] == "R"
        self.people = sum(self.Nw)
        self.reserved = any(l > 0 for l in self.lR)
        self.sigma = None if basket is None else M(basket)
        if self.sigma is not None:
            assert self.exit_free and not self.reserved, "section 2.11"
            self.Z = sum(self.z)
            self.sbar = [z / self.Z for z in self.z]
        if self.budget[0] == "dividend":
            assert not self.reserved and not self.takers and self.mw == self.me, "section 2.6"
        if self.reserved:
            assert self.mw <= self.me, "section 2.8"
        self._d = mp.mpf(0)

    # ---------------------------------------------------------------- the rule (section 4.3-4.5)
    def pc(self, P):
        return (1 + self.tc) * P

    def moving(self):
        """Whether d moves with T_m on the idle stretch (R = 0 there): a payroll or a consumption
        tax under the Dividend closure."""
        return self.budget[0] == "dividend" and (self.tw > 0 or self.tc > 0)

    def transfer(self, P, W, R, C):
        """d at a point: d-hat P^c, or (tau_w W + tau_R R + t_c C - mu P^c N)/N."""
        if self.budget[0] == "rent":
            return self.budget[1] * self.pc(P)
        return (self.tw * W + self.budget[1] * R + self.tc * C - self.mw * self.pc(P) * self.people) / self.people

    def unearned(self, i, Pc, d):
        own = self.sig[i] * Pc
        return max(own, d) if self.replace else own + d

    def own_support(self, i, Pc, d):
        own = self.sig[i] * Pc
        return max(own - d, 0) if self.replace else own

    def supply(self, i, wage, P, m, d):
        """Section 4.4's point form: N_i F_i(ln1p(num/den))."""
        z = self.chi_star(i, wage, P, m, d)
        return self.Nw[i] * (F(z, self.chi[i]) if z != INF else 1)

    def chi_star(self, i, wage, P, m, d):
        """ln1p(num/den); +inf where a free basket leaves den = 0 at a positive num (on idle
        land with a composite that costs nothing in pool wages)."""
        Pc = self.pc(P)
        num = (self.mw - self.me) * Pc + ((1 - self.tw) * wage - (1 + self.tc) * m)
        den = (self.unearned(i, Pc, d) + self.me * Pc) + (1 + self.tc) * m
        if den == 0:
            assert num > 0
            return INF
        return mp.log1p(num / den)

    def ahat(self, i, delta):
        return max(self.sig[i], delta) if self.replace else self.sig[i] + delta

    def ratio(self, i, omega, delta):
        """Section 4.4's corner form."""
        if omega == INF:
            return INF
        return ((self.mw - self.me) + (1 - self.tw) * self.eps[i] * omega / (1 + self.tc)) / (self.ahat(i, delta) + self.me)

    def fixed_delta(self):
        return self.budget[1] if self.budget[0] == "rent" else mp.mpf(0)

    def walled_rate(self, i, zeta):
        """Section 4.5: c_i = ((a_i + mu_e) zeta_i - (mu_w - mu_e)) (1 + t_c)/(1 - tau_w)."""
        return ((self.ahat(i, self.fixed_delta()) + self.me) * zeta - (self.mw - self.me)) * (1 + self.tc) / (1 - self.tw)

    def corner_delta(self, q, omega):
        """Section 4.3's delta(omega) at a corner of a fixed-basket economy without exit values."""
        if self.budget[0] == "rent":
            return self.budget[1]
        if omega == INF:
            return mp.mpf(0)
        L_s, B_s, _, _ = self.basket_totals(q)
        wages = self.tw * omega * q["nD"]
        rent = self.budget[1] * self.T * (1 - omega * L_s) / B_s
        return (wages + rent + self.tc * q["Y"]) / (self.people * (1 + self.tc)) - self.mw

    # ---------------------------------------------------------------- the basket (section 4.2)
    def ces_at(self, p):
        """(P, c): P = Z M and c_j = z_j (p_j/M)^-sigma at the category prices p."""
        s = self.sigma
        w = [j for j in range(self.C) if self.z[j] > 0]
        if s == 1:
            M_ = mp.exp(sum(self.sbar[j] * mp.log(p[j]) for j in w))
        else:
            M_ = sum(self.sbar[j] * p[j] ** (1 - s) for j in w) ** (1 / (1 - s))
        c = [self.z[j] * (p[j] / M_) ** (-s) if self.z[j] > 0 else mp.mpf(0) for j in range(self.C)]
        return self.Z * M_, c

    def ces_price(self, p):
        return self.ces_at(p)[0]

    def quantities_c(self, My, split, By):
        """g1c's quantities with the basket's chain land B_y."""
        task = [mp.mpf(0)] * self.K
        for t, share in split:
            task[t] += share * My / self.theta[t]
        xhat = leontief(transpose(self.Aq), task)
        land = By + sum(b * x for b, x in zip(self.bq, xhat))
        hours = sum(l * x for l, x in zip(self.lq, xhat))
        return dict(task=task, xhat=xhat, land=land, hours=hours)

    # ---------------------------------------------------------------- one evaluation
    def ev(self, x, t, w=None, r=1, land=None, force=None, split=None, edge=None, halv=HALVINGS):
        """Section 5.1 at (x, t) on the line (w None), at a corner's wage w, or on the idle
        stretch (r = 0, w = 1, land given): the producer side, the basket at the point's prices,
        the consumer side, the exit sub-problem, the walk with section 4.5's c_i and supply by
        section 4.4's rule."""
        x = M(x)
        ps = self.prices_at(x, t, w, r)
        if ps is None:
            return dict(viable=False, d=self.block(self.gamma(x), t)["d"])
        w, p, pi, O, V = ps
        C, I = self.C, self.I
        HM = [self.tasks(j, x) for j in range(C)]
        Mt = [m for _, m in HM]
        H = [h + lh for (h, _), lh in zip(HM, self.LH)]
        pbase = leontief(self.Acc, [w * H[j] + pi * Mt[j] + r * self.bc[j] for j in range(C)])
        Pz = sum(z * q for z, q in zip(self.z, pbase))
        if self.sigma is None:
            P0, cz, yh, By = Pz, list(self.z), self.yhat, self.B_y
        else:
            P0, cz = self.ces_at(pbase)
            yh = leontief(transpose(self.Acc), cz)
            By = sum(y * b for y, b in zip(yh, self.bc))
        Hy = sum(y * h for y, h in zip(yh, H))
        My = sum(y * m for y, m in zip(yh, Mt))
        sp = split or [(t, 1)]
        qty = self.quantities_c(My, sp, By)
        out = dict(viable=True, x=x, t=t, w=w, r=M(r), p=p, pi=pi, O=O, V=V, H=H, Mt=Mt, pbase=pbase, Pbase=P0,
                   Pz=Pz, cz=cz, yh=yh, By=By, Hy=Hy, My=My, qty=qty, split=sp, short=None, gamma=self.gamma(x),
                   J=self.J(x), edge=edge)
        if self.exit_free:
            sub = dict(regime="Unused", rc=mp.mpf(0), Toc=mp.mpf(0), spill=mp.mpf(0), G=mp.mpf(0),
                       branch=["D" if e is None else "F" for e in self.ex], m=[mp.mpf(0)] * I,
                       s=[mp.mpf(0)] * I, hh=[mp.mpf(0)] * I, plots=[mp.mpf(0)] * I)
        else:
            pg = pbase[self.g]
            wall = self.wall_price(t) if r == 0 and pg == 0 else None
            # the transfer on the whole endowment: the Dividend closure has no plot takers
            T0 = self.T if land is None else M(land)
            Y0 = T0 / qty["land"]
            self._d = self.transfer(P0, w * Y0 * (Hy + qty["hours"]), M(r) * T0, Y0 * P0)
            sub = self.exit_state(w, P0, pg, r, force, halv, wall=wall)
            out.update(wall=wall)
        Tland = (self.T - sub["spill"]) if land is None else M(land)
        Y = Tland / qty["land"]
        nD = Y * (Hy + qty["hours"])
        D = [Y * l for l in self.lR]
        out.update(Y=Y, nD=nD, D=D, Tland=Tland, sub=sub, spill=sub["spill"])
        for i in range(I):
            if D[i] > self.Nw[i]:
                out.update(short=("reserved", i), f=INF)
                return out
        zeta = [mp.expm1(self.chi[i] * D[i] / self.Nw[i]) for i in range(I)]
        if edge is not None:
            zeta[edge[0]] = edge[1]
        e = [self.eps[i] * w * self.lR[i] for i in range(I)]
        c = [self.walled_rate(i, zeta[i]) * self.lR[i] for i in range(I)]
        P, walled = self.walk(P0, e, c)
        out.update(zeta=zeta, e=e, c=c)
        if P is None:
            out.update(short=("ceiling", walled), f=INF)
            return out
        wages = [self.walled_rate(i, zeta[i]) * P if i in walled else self.eps[i] * w for i in range(I)]
        W = w * nD + sum(wages[i] * D[i] for i in range(I))
        d = self.transfer(P, W, M(r) * Tland, Y * P)
        m = sub["m"]
        nS = [self.supply(i, wages[i], P, m[i], d) for i in range(I)]
        S = sum(self.eps[i] * (nS[i] - D[i]) for i in range(I) if i not in walled)
        reserved_cost = [sum(wages[i] * self.R_chain[i][j] for i in range(I)) for j in range(C)]
        pc = [pbase[j] + reserved_cost[j] for j in range(C)]
        out.update(Ps=P, walled=walled, wages=wages, nS=nS, S=S, f=nD - S, reserved_cost=reserved_cost, pc=pc,
                   pg=pc[self.g], dtr=d, Pc=self.pc(P), W=W)
        return out

    def evaluate(self, x, t, w=None, split=None, edge=None):
        """generate_1d.py's evaluation, which its solve and corners read: this unit's on the whole
        endowment at r = 1."""
        return self.ev(x, t, w=w, split=split, edge=edge)

    def trial(self, rc, v, P, pg, force, pin=None, wall=None):
        """generate_1e.py's trial with section 4.4's supply at the point's transfer."""
        price, money = (pg, rc) if wall is None else (wall, 0)
        out = dict(G=mp.mpf(0), branch=[], m=[], s=[], nS=[], hh=[], plots=[])
        for i in range(self.I):
            e = self.ex[i]
            wage = self.eps[i] * v
            share = 0
            if e is None:
                br, m, s = "D", mp.mpf(0), mp.mpf(0)
            elif force is not None and force[0] == "share" and force[1] == i:
                br, m, s, share = "F", pg * e["sf"], e["sf"], force[2]
            else:
                if pin is not None and pin[0] == i:
                    plot = pin[1] == "P"
                else:
                    plot = (force is not None and force[0] == "above" and force[1] == i) or \
                        rc * e["h"] < price * (e["s0"] - e["sf"])
                if plot:
                    br, m, s, share = "P", pg * e["s0"] - money * e["h"], \
                        e["s0"] - (rc / price) * e["h"] if rc else e["s0"], 1
                else:
                    br, m, s = "F", pg * e["sf"], e["sf"]
            n = self.supply(i, wage, P, m, self._d)
            hh = share * (self.Nw[i] - n)
            pl = (e["h"] * hh) if e is not None else mp.mpf(0)
            out["G"] += pl
            for key, val in (("branch", br), ("m", m), ("s", s), ("nS", n), ("hh", hh), ("plots", pl)):
                out[key].append(val)
        return out

    # ---------------------------------------------------------------- the corners (1d's, section 4.4)
    def S_of(self, omega, q):
        delta = self.corner_delta(q, omega)
        S = mp.mpf(0)
        for i in range(self.I):
            if self.eps[i] == 0:
                continue
            r = self.ratio(i, omega, delta)
            if r >= q["zeta"][i]:
                n = self.Nw[i] * (F(mp.log1p(r), self.chi[i]) if r != INF else 1)
                S += self.eps[i] * (n - q["D"][i])
        return S

    def wage_of_omega(self, omega, q):
        L_s, B_s, _, _ = self.basket_totals(q)
        delta = self.corner_delta(q, omega)
        LW, CW = L_s, mp.mpf(0)
        for i in range(self.I):
            if self.eps[i] > 0 and self.ratio(i, omega, delta) >= q["zeta"][i]:
                LW += self.eps[i] * self.lR[i]
            else:
                CW += self.walled_rate(i, q["zeta"][i]) * self.lR[i]
        den = (1 - CW) - omega * LW
        assert den > 0
        return omega * B_s / den

    def omega_limit(self, q):
        L_s, _, _, _ = self.basket_totals(q)
        e = [self.eps[i] * self.lR[i] for i in range(self.I)]
        c = [self.walled_rate(i, q["zeta"][i]) * self.lR[i] for i in range(self.I)]
        P, _ = self.walk(L_s, e, c)
        if P is None:
            return None
        return INF if P == 0 else 1 / P

    def report(self, x, t, w=None, base=None, margin=None, tie=None, omega=None, edge=None):
        """generate_1d.py's solve reports here: where, for report_1f."""
        out = dict(base or {})
        out.update(x=M(x), t=t, w=w, tie=tie, edge=edge, margin=margin)
        return out

    def report_1e(self, path, x, t, w=None, r=1, land=None, margin=None, tie=None, force=None, edge=None,
                  enclosure=None):
        return self.report_1f(path, x, t, w=w, r=r, land=land, margin=margin, tie=tie, force=force, edge=edge,
                              enclosure=enclosure)

    # ---------------------------------------------------------------- the start (section 2.9)
    def start_value(self):
        """f_0 = n_D(x = 0) - S(v = 0): by the corner form without exit values and a fixed basket,
        else at the point v = 0; +inf under a CES basket weighing a category free at v = 0."""
        first = self.envelope_ext()[0]
        if self.exit_free and self.sigma is None:
            q = self.evaluate(0, first)
            if "zeta" not in q:
                # a short corner: D_i and zeta_i are still the corner's (the oracle's corner form)
                q["zeta"] = [mp.expm1(self.chi[i] * q["D"][i] / self.Nw[i]) for i in range(self.I)]
            return q["nD"] - self.S_of(mp.mpf(0), q)
        if self.sigma is not None and any(self.z[j] > 0 and self.bbar[j] == 0 for j in range(self.C)):
            return INF
        return self.ev(0, first, w=mp.mpf(0))["f"]

    @staticmethod
    def positive(kind, f):
        """Section 5.3 step 3, the start evaluated."""
        if kind in ("One", "Rest"):
            return f >= 0
        return f > 0

    # ---------------------------------------------------------------- the solve (section 5.3)
    def solve(self, scan=SCAN):
        """Returns (regime, report): an equilibrium's stretch (Contestable, Wall, AllHuman, Idle),
        or NotViable, NoMarket, SurplusLabour, MultipleEquilibria."""
        if self.exit_free and self.sigma is None:
            f0 = self.start_value()
            if f0 > 0:
                regime, r = self.solve_exit_free()
                if isinstance(r, dict):
                    r["f_start"] = f0
                return regime, r
            regime, r = g1d.Economy.solve(self)
            if regime == "NotViable":
                return regime, None
            sides = [False] + [(f >= 0 if kind == "One" else f > 0) for kind, f in r["seq"]]
            rest = self.ev(1, r["wtechs"][-1], w=M(1), r=0, land=0)
            sides.append(rest["f"] >= 0)
            changes = sum(1 for a, b in zip(sides, sides[1:]) if a != b)
            assert changes == 0, "an economy with a negative start and a change of side"
            return "SurplusLabour", dict(f_start=f0)
        path = self.path()
        if path is None:
            return "NotViable", None
        changes = self.count(path, scan)
        if not changes:
            f0 = path["stations"][0][1]
            if not f0 > 0:
                return "SurplusLabour", dict(f_start=f0)
            return "NoMarket", dict(f_end=path["f_end"], path=path)
        if len(changes) > 1:
            return "MultipleEquilibria", dict(changes=changes, path=path)
        st = path["stations"]
        k = next(k for k in range(1, len(st)) if self.positive(self.name(st[k - 1][0]), st[k - 1][1])
                 and not self.positive(self.name(st[k][0]), st[k][1]))
        regime, r = self.locate(path, k)
        r["f_start"] = path["stations"][0][1]
        return regime, r

    def free_at_the_end(self, t):
        if self.sigma is None:
            return False
        q = self.ev(1, t, w=M(1))
        _, _, lt_c, _ = self.basket_totals(q)
        return any(self.z[j] > 0 and lt_c[j] == 0 for j in range(self.C))

    def supply_at_the_end(self, t):
        """Section 5.3 step 3: S at omega_inf = 1/(Z M(lambda-tilde)) for sigma < 1, +inf
        otherwise, with the transfer's limit d-hat (RentRate) or -mu (Dividend)."""
        q = self.ev(1, t, w=M(1))
        _, _, lt_c, _ = self.basket_totals(q)
        omega = INF
        wtd = [j for j in range(self.C) if self.z[j] > 0]
        if self.sigma < 1 and any(lt_c[j] > 0 for j in wtd):
            M_ = sum(self.sbar[j] * lt_c[j] ** (1 - self.sigma) for j in wtd) ** (1 / (1 - self.sigma))
            omega = 1 / (self.Z * M_)
        delta = self.budget[1] if self.budget[0] == "rent" else -self.mw
        S = mp.mpf(0)
        for i in range(self.I):
            r = self.ratio(i, omega, delta)
            S += self.eps[i] * self.Nw[i] * (F(mp.log1p(r), self.chi[i]) if r != INF else 1)
        return S

    def path(self):
        """generate_1e.py's path with the start evaluated, the corners in omega = v/P_z, and a
        CES basket's end: with a free weighted category the wall's end is the path's end, its
        value -S_inf (section 2.12)."""
        first, line_sw, wall_sw = self.envelope_ext()
        ltechs = [first] + [s[2] for s in line_sw]
        wtechs = [ltechs[-1]] + [s[2] for s in wall_sw]
        one = self.ev(1, ltechs[-1])
        if not one["viable"]:
            return None
        at0, atlo = self.ev(0, first), self.ev(LO, first)
        xs = [self.gamma_inv(g) for g, _, _ in line_sw]
        bounds = [LO] + xs + [M(1)]
        f0 = self.start_value()
        path = dict(stations=[("Start", f0)], pieces=[None], enclosures=[], one=one, at0=at0, atlo=atlo,
                    xs=xs, ltechs=ltechs, wtechs=wtechs, line_sw=line_sw, wall_sw=wall_sw)
        span = self.corner_span("AllHuman", 0, first)
        self.push_piece(path, span, mp.mpf(0), at0["w"] / at0["Pz"], ("Zero", at0["f"]))
        path["stations"].append(("Lo", atlo["f"]))
        path["pieces"].append((("Bottom", first), mp.mpf(0), LO))
        for r_, t in enumerate(ltechs):
            if r_ < len(xs):
                end = (("SwitchBelow", r_), self.ev(xs[r_], t)["f"])
            else:
                end = ("One", one["f"])
            self.push_piece(path, ("Line", t), bounds[r_], bounds[r_ + 1], end)
            if r_ < len(xs):
                path["stations"].append((("SwitchAbove", r_), self.ev(xs[r_], ltechs[r_ + 1])["f"]))
                path["pieces"].append(None)
        wws = [self.closure_wage(b, g) for g, b, _ in wall_sw]
        om_lo = one["w"] / one["Pz"]
        for s, t in enumerate(wtechs):
            span = self.corner_span("Wall", 1, t)
            if s < len(wall_sw):
                qb = self.ev(1, wall_sw[s][1], w=wws[s])
                self.push_piece(path, span, om_lo, wws[s] / qb["Pz"], (("WallBelow", s), qb["f"]))
                qa = self.ev(1, wall_sw[s][2], w=wws[s])
                path["stations"].append((("WallAbove", s), qa["f"]))
                path["pieces"].append(None)
                om_lo = wws[s] / qa["Pz"]
            else:
                om_end = 1 / span[2] if span[2] > 0 else INF
                te = t
                if self.free_at_the_end(te):
                    f_end = -self.supply_at_the_end(te)
                    path.update(om_end=om_end, f_end=f_end, t_inf=self.T, te=te, free_end=True)
                    self.push_piece(path, span, om_lo, om_end, ("Rest", f_end), to_the_end=True)
                    path.update(wws=wws, S_inf=-f_end)
                    return path
                t_inf = self.T - self.ev(1, te, w=M(1), r=0, land=self.T)["spill"]
                f_end = self.ev(1, te, w=M(1), r=0, land=t_inf)["f"]
                path.update(om_end=om_end, f_end=f_end, t_inf=t_inf, te=te, free_end=False,
                            last_start=path["stations"][-1][1])
                self.push_piece(path, span, om_lo, om_end, ("End", f_end), to_the_end=True)
        rest = self.ev(1, path["te"], w=M(1), r=0, land=0)
        path["stations"].append(("Rest", rest["f"]))
        path["pieces"].append((("Idle", path["te"]), mp.mpf(0), path["t_inf"]))
        path.update(wws=wws, S_inf=-rest["f"])
        return path

    def count(self, path, scan):
        """Section 5.3 step 3 with the scan where a type takes plots, and at rho > 0 under a CES
        basket."""
        pts = []
        scanning = scan > 0 and (self.takers or (self.rho > 0 and self.sigma is not None))
        for k, (kind, f) in enumerate(path["stations"]):
            if scanning and path["pieces"][k] is not None:
                for s, fs in self.scan_values(path["pieces"][k], scan):
                    pts.append(("scan", k, s, fs, fs > 0))
            name = kind if isinstance(kind, str) else kind[0]
            pts.append(("station", k, None, f, self.positive(name, f)))
        return [(pts[j], pts[j + 1]) for j in range(len(pts) - 1) if pts[j][4] != pts[j + 1][4]]

    def locate(self, path, k):
        """generate_1e.py's locate, with the start's value evaluated."""
        st, pieces = path["stations"], path["pieces"]
        (before, f_before), (after, f_after) = st[k - 1], st[k]
        piece = pieces[k]
        if piece is None:
            kind = self.name(before)
            if kind == "EncBelow":
                return self.enclosure_tie(path, path["enclosures"][before[1]])
            if kind == "SwitchBelow":
                i = before[1]
                b, a = path["ltechs"][i], path["ltechs"][i + 1]
                share = self.tie_share_1e(path["xs"][i], b, a, None)
                return "Contestable", self.report_1f(path, path["xs"][i], b, tie=(a, share, path["line_sw"][i][0]),
                                                    margin="Contestable")
            s = before[1]
            g, b, a = path["wall_sw"][s]
            share = self.tie_share_1e(1, b, a, path["wws"][s])
            return "Wall", self.report_1f(path, 1, b, w=path["wws"][s], tie=(a, share, g), margin="Wall")
        span, lo, hi = piece
        if span[0] == "Idle":
            return self.idle(path, junction=False)
        if self.name(after) == "End" and f_after == 0 and f_before != 0:
            return self.idle(path, junction=True)
        f = lambda s: self.at_span(span, s)["f"]  # noqa: E731
        w = None
        if f_before == 0:
            s = lo
        elif f_after == 0:
            s = hi
        elif self.sigma is not None and span[0] in ("AllHuman", "Wall"):
            # A CES basket's corner in the pool's wage itself (section 5.3 step 2): near
            # omega_z = 1/L_z, omega_z resolves v only to v L_z/B_z of the working precision.
            x = 0 if span[0] == "AllHuman" else 1
            g = lambda v: self.ev(x, span[1], w=v)["f"]  # noqa: E731
            a = self.corner_wage(span, lo)
            b = INF if hi * span[2] >= 1 else self.corner_wage(span, hi)
            if b == INF:
                b = max(a, 1) * 2
                while g(b) > 0:
                    b *= 2
            for _ in range(HALVINGS):
                mid = (a + b) / 2
                if g(mid) > 0:
                    a = mid
                else:
                    b = mid
            w = (a + b) / 2
        else:
            n = BISECTIONS if span[0] == "Line" else HALVINGS
            a, b = lo, hi
            if b == INF:
                b = max(lo, 1) * 2
                while f(b) > 0:
                    b *= 2
            for _ in range(n):
                mid = (a + b) / 2
                if f(mid) > 0:
                    a = mid
                else:
                    b = mid
            s = (a + b) / 2
        if span[0] in ("Line", "Bottom"):
            return "Contestable", self.report_1f(path, s, span[1], margin="Contestable")
        if w is None:
            w = self.corner_wage(span, s)
        if span[0] == "Wall":
            return "Wall", self.report_1f(path, 1, span[1], w=w, margin="Wall")
        return "AllHuman", self.report_1f(path, 0, self.cheapest(w), w=w, margin="AllHuman")

    def idle(self, path, junction):
        """generate_1e.py's idle stretch; the Dividend closure's moving transfer bisects on T_m,
        as reserved hours do (section 5.3 step 3)."""
        if not self.moving():
            return g1e.Economy.idle(self, path, junction)
        te, t_inf = path["te"], path["t_inf"]
        ev = lambda t: self.ev(1, te, w=M(1), r=0, land=t)  # noqa: E731
        if junction:
            return "Idle", self.report_1f(path, 1, te, w=M(1), r=0, land=t_inf, margin="Wall")
        lo, hi = mp.mpf(0), t_inf
        for _ in range(HALVINGS):
            mid = (lo + hi) / 2
            if ev(mid)["f"] < 0:
                lo = mid
            else:
                hi = mid
        return "Idle", self.report_1f(path, 1, te, w=M(1), r=0, land=(lo + hi) / 2, margin="Wall")

    # ---------------------------------------------------------------- the report (section 4.6-4.9)
    def totals_c(self, q):
        """The basket's totals with the point's content: price side (L_s, B_s, lt_c, bt_c) and
        quantity side (Lq, Bq)."""
        split = q["split"]
        cl = sum(s * self.lt[tt] / self.theta[tt] for tt, s in split)
        cb = sum(s * self.bt[tt] / self.theta[tt] for tt, s in split)
        cql = sum(s * self.ltq[tt] / self.theta[tt] for tt, s in split)
        cqb = sum(s * self.btq[tt] / self.theta[tt] for tt, s in split)
        lt_c = leontief(self.Acc, [h + m * cl for h, m in zip(q["H"], q["Mt"])])
        bt_c = leontief(self.Acc, [b + m * cb for b, m in zip(self.bc, q["Mt"])])
        lq_c = leontief(self.Acc, [h + m * cql for h, m in zip(q["H"], q["Mt"])])
        bq_c = leontief(self.Acc, [b + m * cqb for b, m in zip(self.bc, q["Mt"])])
        c = q["cz"]
        return (sum(z * v for z, v in zip(c, lt_c)), sum(z * v for z, v in zip(c, bt_c)), lt_c, bt_c,
                sum(z * v for z, v in zip(c, lq_c)), sum(z * v for z, v in zip(c, bq_c)))

    def report_1f(self, path, x, t, w=None, r=1, land=None, margin=None, tie=None, force=None, edge=None,
                  enclosure=None):
        """Sections 4.6-4.9 at the equilibrium, every identity asserted to 1e-65."""
        split = [(t, 1)] if tie is None else [(t, 1 - tie[1]), (tie[0], tie[1])]
        q = self.ev(x, t, w=w, r=r, land=land, split=split, force=force, edge=edge)
        assert q["short"] is None
        I, C = self.I, self.C
        w, Y, P, rr = q["w"], q["Y"], q["Ps"], q["r"]
        sub = q["sub"]
        X = [Y * xx for xx in q["qty"]["xhat"]]
        interest = sum(self.rho * om * Vk * Xk for om, Vk, Xk in zip(self.omega, q["V"], X))
        pooled = [i for i in range(I) if i not in q["walled"]]
        s_i = {i: self.eps[i] * (q["nS"][i] - q["D"][i]) for i in pooled}
        S = sum(s_i.values())
        pool = {i: (q["nD"] * s_i[i] / S if S != 0 else mp.mpf(0)) for i in pooled}
        hours = [q["D"][i] + (pool[i] / self.eps[i] if i in pooled and self.eps[i] > 0 else 0) for i in range(I)]
        W = q["W"]
        Tm = q["Tland"]
        R = rr * Tm
        income = W + R + interest
        Cc = Y * P
        E = [self.Nw[i] - q["nS"][i] for i in range(I)]
        Pc, d = q["Pc"], q["dtr"]
        m_w, m_e = self.mw * Pc, self.me * Pc
        Mprog = m_w * sum(q["nS"]) + m_e * sum(E)
        G_w, G_c = self.tw * W, self.tc * Cc
        if self.budget[0] == "rent":
            levy = self.people * d + Mprog - G_w - G_c
            G_R, tauR = levy, (levy / R if R > 0 else None)
        else:
            levy, G_R, tauR = None, self.budget[1] * R, self.budget[1]
        A = [self.unearned(i, Pc, d) for i in range(I)]
        own = [self.own_support(i, Pc, d) for i in range(I)]
        net = [(1 - self.tw) * q["wages"][i] for i in range(I)]
        spend = [q["nS"][i] * (A[i] + m_w + net[i]) + E[i] * (A[i] + m_e) for i in range(I)]
        provider = R - G_R + interest - sum(self.Nw[i] * own[i] for i in range(I))
        L_s, B_s, lt_c, bt_c, Lq, Bq = self.totals_c(q)
        reserved_bill = sum(q["wages"][i] * self.lR[i] for i in range(I))
        kappa = rr * self.T / (self.people * P)
        phi_q = rr * Bq / P
        out = dict(margin=margin, x=q["x"], t=t, w=w, r=rr, Ps=P, Pc=Pc, d=d, Y=Y, n_pool=q["nD"], n_a=sum(hours),
                   hours=hours, wages=q["wages"], nS=q["nS"], E=E, income=income, interest=interest, W=W, R=R,
                   C=Cc, T_m=Tm, T_p=q["spill"], idle=(self.T - Tm - q["spill"] if rr == 0 else mp.mpf(0)),
                   kappa=kappa, q=self.q_of(rr, q["pg"], t), pg=q["pg"], p=q["pc"], p_m=q["p"], pi=q["pi"],
                   content=q["cz"], shares=[q["cz"][j] * q["pc"][j] / P for j in range(C)], L_s=L_s, B_s=B_s,
                   Lq=Lq, Bq=Bq, lt_c=lt_c, bt_c=bt_c, G_w=G_w, G_c=G_c, G_R=G_R, levy=levy, tauR=tauR, M=Mprog,
                   A=A, own=own, net=net, spend=spend, provider=provider, phi_w_s=(w * L_s + reserved_bill) / P,
                   phi_r_s=rr * B_s / P, phi_q=phi_q, R0=phi_q * (Cc - G_R),
                   mult=(1 / (1 - phi_q * G_R / R) if R > 0 else None),
                   omega_net=(1 - self.tw) / (1 + self.tc) * w / P,
                   tie=tie, edge=edge, enclosure=enclosure, q_ev=q, regime=sub["regime"], walled=q["walled"],
                   f_end=path.get("f_end") if path else None, X=X, zeta=q["zeta"], gamma=q["gamma"],
                   phi_w_m=w * self.lt[t] / q["p"][t], phi_r_m=rr * self.bt[t] / q["p"][t])
        self.assert_identities_1f(out, q)
        return out

    def assert_identities_1f(self, r, q):
        """Every identity of sections 4.8-4.9 at 1e-65, each supply from the rule, and the CES's
        Euler, Shephard and eq 26."""
        ch = {}
        I, C = self.I, self.C
        w, Y, P, rr = r["w"], r["Y"], r["Ps"], r["r"]
        # the pool's clearing carries 1 - x*'s cancellation near x = 1, 70 digits less log10 of
        # 1/(1 - x*) (the automation path at eta 2^-16 keeps 65)
        amp = 1 / (1 - r["x"]) if r["margin"] == "Contestable" else 1
        assert rel(q["nD"], q["S"]) < TOL * max(amp, 1), ("pool clears", rel(q["nD"], q["S"]))
        for i in range(I):
            if i in q["walled"]:
                ch[f"reserved clears {i}"] = rel(q["nS"][i], q["D"][i])
                ch[f"walled wage {i}"] = rel(q["wages"][i], self.walled_rate(i, q["zeta"][i]) * P)
            want = self.supply(i, q["wages"][i], P, q["sub"]["m"][i], r["d"])
            ch[f"supply {i}"] = rel(q["nS"][i], want) if want != 0 else abs(q["nS"][i])
        # (I1) income three ways, and T_m = Y B^q
        ch["I = Y P"] = rel(r["income"], Y * P)
        ch["I = sum p c Y"] = rel(sum(p * c * Y for p, c in zip(r["p"], r["content"])), r["income"])
        ch["T_m = Y B^q"] = rel(Y * r["Bq"], r["T_m"])
        ch["n_pool = Y L^q + reserved"] = rel(q["nD"], Y * r["Lq"])
        # (I2) the budget; (I3) spending; (I4) composites
        ch["budget"] = abs(r["G_w"] + r["G_R"] + r["G_c"] - self.people * r["d"] - r["M"]) / r["C"]
        spent = sum(r["spend"]) + r["provider"]
        ch["spending"] = rel(spent, (1 + self.tc) * r["C"])
        ch["composites"] = rel(spent / r["Pc"], Y)
        # receipts (Prop 6 (iii); main.tex:541-545)
        ch["receipts"] = rel((1 - self.tw) * r["W"] + (r["R"] - r["G_R"]) + r["interest"] + self.people * r["d"]
                             + r["M"], r["W"] + r["R"] + r["interest"] + r["G_c"])
        # coverage, the rent share (T5), the basket's totals
        ch["R = phi_q C"] = rel(r["R"], r["phi_q"] * r["C"]) if r["R"] > 0 else abs(r["phi_q"])
        ch["P = v L + r B + reserved"] = rel(P, w * r["L_s"] + rr * r["B_s"]
                                             + sum(q["wages"][i] * self.lR[i] for i in range(I)))
        ch["Euler"] = rel(P, sum(c * p for c, p in zip(r["content"], r["p"])))
        if self.sigma is not None:
            # the shares and Shephard's lemma at the point's prices
            p = r["p"]
            den = sum(self.z[k] * p[k] ** (1 - self.sigma) for k in range(C) if self.z[k] > 0)
            for j in range(C):
                if self.z[j] == 0:
                    continue
                ch[f"share {j}"] = rel(r["shares"][j], self.z[j] * p[j] ** (1 - self.sigma) / den)
                h = p[j] * DERIVATIVE
                up = [pk + (h if k == j else 0) for k, pk in enumerate(p)]
                dn = [pk - (h if k == j else 0) for k, pk in enumerate(p)]
                shep = (self.ces_price(up) - self.ces_price(dn)) / (2 * h)
                assert rel(shep, r["content"][j]) < mp.mpf(10) ** -35, ("Shephard", j)
        if rr == 1:
            # D.3's identity with the price-side totals
            if not self.reserved:
                ch["D.3"] = rel(r["kappa"], self.T / (self.people * (r["L_s"] * w + r["B_s"])))
            # the corollary's bound (SSRN p.31)
            assert r["W"] / (self.people * P) <= w * (r["n_pool"] / self.people) / r["B_s"] * (1 + TOL) \
                or self.reserved
        if r["margin"] == "Contestable":
            ch["margin"] = rel(w, self.gamma(r["x"]) * q["pi"])
        bad = {k: v for k, v in ch.items() if v > TOL}
        assert not bad, f"identities fail at {DPS} digits: {bad}"


# ------------------------------------------------------------------------------ instances
def econ(N="4", T="10", h="1", a="0.3", lam="0.05", b="0.4", eta="1", g0="0.2", g1="0.8", chi="1", sig="1",
         z=None, basket=None, government=None, exits=(None,), parcels=None, required=None, rho="0"):
    """1a's G1 in household form (docs/unit-1f.md section 3.3): the good and space, one flow
    machine, one type (N, chi, 1, sig), with weights z (default (1, h)), a basket and a
    government."""
    zg, zs = z or ("1", h)
    return Economy(parcels=parcels or (("LAND", T, "1", "E"),), exits=exits, exit_good=0, basket=basket,
                   government=government, workers=(worker("WORKER", N, chi, sig=sig),), eta=eta, g0=g0, g1=g1,
                   k="1", rho=rho, edges=("0", "1"), categories=(("GOOD", zg, "0", ("1",)), ("SPACE", zs, "1", ("0",))),
                   intermediate=(("0", "0"), ("0", "0")), required=required,
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), ((a,), lam, b), "1", 1),))


def tx(lam="0.1", T="8", government=None):
    """TX: N 4, T 8, h 1, a 0.5, lam 0.1, b 0.2, gamma = 2 + 2x, chi_max 1/8, support 1/4."""
    return econ(T=T, a="0.5", lam=lam, b="0.2", g0="2", g1="2", chi="0.125", sig="0.25", government=government)


def ces_z(alpha, sigma):
    """z = ((1 - alpha)^sigma, alpha^sigma): space's share is eq 26's alpha(q)."""
    alpha, sigma = M(alpha), M(sigma)
    return ((1 - alpha) ** sigma, alpha ** sigma)


def race(N, eta, government):
    """1e's Q: (LAND, 100, 1), exit (1.5, 0, 1)."""
    return econ(N=N, T="100", eta=eta, exits=(priced("1.5", "0", "1"),), government=government)


def k3(government):
    """1e's K3: (FIELDS, 10, 1), exit (0.5, 0, 0.1)."""
    return econ(parcels=(("FIELDS", "10", "1", "E"),), exits=(priced("0.5", "0", "0.1"),), government=government)


def entrant_trained(government):
    """1d's E2: the entrant (8, 1, 1, 1) and the trained (1, 0.8, 1.5, 1.2) with reserved hours on
    B's economy."""
    ws = (worker("ENTRANT", "8", "1"), worker("TRAINED", "1", "0.8", eps="1.5", sig="1.2"))
    return Economy(parcels=(("LAND", "10", "1", "E"),), exits=(None, None), exit_good=0, government=government,
                   workers=ws, eta="1", g0="0.2", g1="0.8", k="1", rho="0", edges=("0", "1"),
                   categories=(("SERVICES", "1", "0", ("0.75",)), ("GOODS", "1", "0", ("1",)),
                               ("SPACE", "1", "1", ("0",))),
                   intermediate=(("0",) * 3,) * 3,
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), (("0.3",), "0.05", "0.4"), "1", 1),),
                   required=("0.25", "0", "0"), reserved=(("0", "0.1"), ("0", "0.02"), ("0", "0")))


def a_joint(g1="4", lam="0.1"):
    """check_pinning's A-joint household: N 1, T 10, a 0.5, lam, b 0.2, gamma = 1 + g1 x, the
    land share 0.3 as Ces { sigma: 1 } over (good, space), saturated supply (chi_max 1/64)."""
    return econ(N="1", a="0.5", lam=lam, b="0.2", g0="1", g1=g1, chi="0.015625", z=("0.7", "0.3"), basket="1")


def solved(e, stretch=None):
    regime, r = e.solve()
    assert regime in ("Contestable", "Wall", "AllHuman", "Idle"), (regime, r)
    if stretch is not None:
        assert regime == stretch, (regime, stretch)
    r["stretch"] = regime
    return r


def same_allocation(a, b, what, tol=TOL):
    for key in ("x", "w", "Ps", "Y", "n_a", "T_m", "T_p"):
        assert rel(a[key], b[key]) < tol or a[key] == b[key], (what, key, a[key], b[key])
    for i in range(len(a["nS"])):
        assert rel(a["nS"][i], b["nS"][i]) < tol, (what, "nS", i)


def assert_f_nonincreasing(e):
    """Proposition F at rho = 0: f nonincreasing on a grid of the line and of each corner."""
    first, line_sw, wall_sw = e.envelope_ext()
    assert not line_sw and not wall_sw
    fs = [e.ev(M(k) / GRID, first)["f"] for k in range(1, GRID + 1)]
    one = e.ev(1, first)
    ws = [one["w"] * (1 + M(k) / 4) for k in range(GRID)]
    fs += [e.ev(1, first, w=v)["f"] for v in ws]
    for f0, f1 in zip(fs, fs[1:]):
        assert f1 <= f0 + TOL * (1 + abs(f0)), "f rises along the path"
    return len(fs)


def lemma_f1(e, x, w=None):
    """Lemma F1 at a point: the composition's effect on L_s/B_s at fixed totals and r as v rises,
    against its closed form; returns the effect (<= 0)."""
    q = e.ev(x, e.envelope_ext()[0], w=w)
    _, _, lt_c, bt_c, _, _ = e.totals_c(q)
    v, r = q["w"], q["r"]

    def ratio(vv):
        p = [vv * l + r * b for l, b in zip(lt_c, bt_c)]
        _, c = e.ces_at(p)
        return sum(cj * l for cj, l in zip(c, lt_c)) / sum(cj * b for cj, b in zip(c, bt_c))

    h = v * DERIVATIVE
    fd = (ratio(v + h) - ratio(v - h)) / (2 * h) * v
    p = [v * l + r * b for l, b in zip(lt_c, bt_c)]
    _, c = e.ces_at(p)
    L = sum(cj * l for cj, l in zip(c, lt_c))
    B = sum(cj * b for cj, b in zip(c, bt_c))
    phi = lambda l, b: v * l / (v * l + r * b)  # noqa: E731
    phis = v * L / (v * L + r * B)
    closed = -e.sigma * sum(c[j] * (lt_c[j] * B - bt_c[j] * L) / B ** 2 * (phi(lt_c[j], bt_c[j]) - phis)
                           for j in range(e.C))
    assert rel(fd, closed) < mp.mpf(10) ** -30, ("Lemma F1", fd, closed)
    assert closed <= 0
    return closed


# ------------------------------------------------------------------------------ output
LINES = []
AJ_GAP = []
START = time.time()


def section(title):
    if os.environ.get("GENERATE_1F_PROGRESS"):
        print(f"{time.time() - START:7.1f} s  {title}", file=sys.stderr, flush=True)
    LINES.append("")
    LINES.append(f"# {title}")


def put(key, value, note):
    """One golden, printed to SIG_OUT digits."""
    text = mp.nstr(M(value) if not isinstance(value, mp.mpf) else value, SIG_OUT, min_fixed=-mp.inf,
                   max_fixed=mp.inf)
    LINES.append(f"{key} = {text}  # {note}")


def put_all(prefix, r, keys):
    for key, field, note in keys:
        put(f"{prefix}_{key}", field(r) if callable(field) else r[field], note)


ALLOC = (("X_STAR", "x", "x*"), ("V", "w", "v, the pool's wage"), ("P", "Ps", "P, the composite's producer price"),
         ("N_A", "n_a", "N_a, hours worked"))


def build():
    grid_points = 0
    nest_worst = mp.mpf(0)

    # ------------------------------------------------------------------ H0: the nesting
    section("H0: unit 1e's instances in household form equal generate_1e.py's solve (docs/unit-1f.md section 9)")
    for label, a, b in (
            ("G1", g1e.goodspace(), econ()),
            ("W1", g1e.goodspace(lam="0.6"), econ(lam="0.6")),
            ("W3", g1e.goodspace(lam="0.6", chi="3"), econ(lam="0.6", chi="3")),
            ("W4", g1e.goodspace(N="20", chi="0.05"), econ(N="20", chi="0.05")),
            ("Q1", g1e.goodspace(N="50", T="100", exits=(priced("1.5", "0", "1"),), eta="2.5"),
             race("50", "2.5", None)),
            ("K3", g1e.goodspace(parcels=g1e.fields(), exits=(priced("0.5", "0", "0.1"),)), k3(None))):
        ra, qa = a.solve()
        rb, qb = b.solve()
        assert ra == rb, (label, ra, rb)
        for key_a, key_b in (("x", "x"), ("w", "w"), ("Ps", "Ps"), ("Y", "Y"), ("n_a", "n_a"),
                             ("income", "income")):
            err = rel(qa[key_a], qb[key_b])
            nest_worst = max(nest_worst, err)
            assert err < TOL, (label, key_a, err)

    # ------------------------------------------------------------------ TX
    section("TX: three-taxes' worked instance inside the closure (check_three_taxes.py T1 :27-61, T6 :63-85, T5 :105-134)")
    t0 = solved(tx(), "Contestable")
    assert rel(t0["x"], M("0.5")) < TOL and rel(t0["gamma"], 3) < TOL and rel(t0["p_m"][0], 1) < TOL
    assert rel(t0["w"], 3) < TOL and rel(t0["Ps"], M("3.75")) < TOL and rel(t0["n_a"], 4) < TOL
    lam_m, b_m = M("0.1") / (1 - M("0.5")), M("0.2") / (1 - M("0.5"))
    assert rel(t0["phi_w_m"], M("0.6")) < TOL and rel(t0["phi_w_s"], M("0.6")) < TOL
    assert rel(t0["phi_r_m"], M("0.4")) < TOL and rel(t0["phi_r_s"], M("0.4")) < TOL
    for key, val, note in (("X_STAR", t0["x"], "x* = 1/2 exactly: labour demand T L_s/B_s = N at x = 1/2"),
                           ("GAMMA_STAR", t0["gamma"], "gamma(x*) = 3"), ("P_M", t0["p_m"][0], "p_m = 1"),
                           ("V", t0["w"], "v = gamma* p_m = 3"), ("P_GOOD", t0["p"][0], "the good's price 11/4"),
                           ("P", t0["Ps"], "P = 15/4"), ("L_S", t0["L_s"], "L_s = 3/4"), ("B_S", t0["B_s"], "B_s = 3/2"),
                           ("Y", t0["Y"], "Y = 16/3"), ("N_A", t0["n_a"], "N_a = N = 4, supply saturated"),
                           ("W", t0["W"], "the wage bill 12"), ("R", t0["R"], "rent 8"), ("I", t0["income"], "I = 20"),
                           ("LAMBDA_M", lam_m, "lambda-tilde_m = lambda/(1 - a) = 0.2"),
                           ("B_M", b_m, "b-tilde_m = b/(1 - a) = 0.4"),
                           ("PHI_W", t0["phi_w_m"], "phi_w = lambda gamma*/(1 - a) = 0.6, the machine service and the basket"),
                           ("PHI_R", t0["phi_r_m"], "phi_r = 0.4"), ("KAPPA", t0["kappa"], "kappa = 8/15"),
                           ("OMEGA_NET", t0["omega_net"], "the net real wage v/P = 0.8")):
        put(f"TX_{key}", val, note)
    # every tax on the grid leaves the allocation (supply saturated)
    for tw in ("0", "0.125", "0.25", "0.5"):
        for tc in ("0", "0.25"):
            for dh in ("0", "1"):
                r = solved(tx(government=gov(tw=tw, tc=tc, budget=("rent", dh))), "Contestable")
                same_allocation(r, t0, ("TX grid", tw, tc, dh))
                assert rel(r["omega_net"], (1 - M(tw)) / (1 + M(tc)) * M("0.8")) < TOL
                assert rel(r["phi_w_s"], M("0.6")) < TOL
    r1 = solved(tx(government=gov(tc="0.25", budget=("dividend", "0"))), "Contestable")
    same_allocation(r1, t0, "TX1")
    assert rel(r1["G_c"], 5) < TOL and rel(r1["d"], M("1.25")) < TOL
    tc = M("0.25")
    for key, val, note in (("G_C", r1["G_c"], "the consumption tax's revenue t_c Y P = 5"),
                           ("D", r1["d"], "d = G_c/N = 5/4, returned uniformly"),
                           ("LEG_WAGE_MACHINE", tc * t0["w"] * lam_m, "per unit of machine service: t v lambda-tilde = 0.15"),
                           ("LEG_RENT_MACHINE", tc * b_m, "per unit of machine service: t r b-tilde = 0.10"),
                           ("LEG_WAGE_BASKET", tc * t0["w"] * t0["L_s"], "per composite: t v L_s = 0.5625"),
                           ("LEG_RENT_BASKET", tc * t0["B_s"], "per composite: t r B_s = 0.375"),
                           ("WAGE_LEG", tc * r1["W"], "in aggregate, t_c W = 3"), ("RENT_LEG", tc * r1["R"], "t_c R = 2")):
        put(f"TX1_{key}", val, note)
    r2 = solved(tx(government=gov(budget=("dividend", "1"))), "Contestable")
    same_allocation(r2, t0, "TX2")
    assert rel(r2["d"] * 4, r2["R"]) < TOL and rel(r2["R0"] * r2["mult"], r2["R"]) < TOL
    for key, val, note in (("D", r2["d"], "d = tau_R R/N = 2: (1 - 1) 8 + 4 2 = 8 (Prop 6 (iii))"),
                           ("RENT_SHARE", r2["phi_q"], "phi^q_r = r B^q_s/P = 0.4"),
                           ("R0", r2["R0"], "R_0 = phi^q_r (C - G_R) = 4.8"),
                           ("MULTIPLIER", r2["mult"], "1/(1 - phi^q_r tau_R) = 5/3")):
        put(f"TX2_{key}", val, note)
    r3 = solved(tx(lam="0", T="12"), "Contestable")
    assert rel(r3["x"], M("0.5")) < TOL and rel(r3["p_m"][0], M("0.4")) < TOL and r3["phi_w_m"] == 0
    for key, val, note in (("X_STAR", r3["x"], "x* = 1/2 at T 12"), ("P_M", r3["p_m"][0], "p_m = b/(1 - a) = 0.4"),
                           ("V", r3["w"], "v = 3 p_m = 1.2"), ("P", r3["Ps"], "P = 2.1"), ("Y", r3["Y"], "Y = 8"),
                           ("N_A", r3["n_a"], "N_a = 4"),
                           ("TAX_PER_UNIT", M("0.3") * r3["p_m"][0], "a 30% consumption tax and a 30% rent tax each take 0.12 per unit")):
        put(f"TX3_{key}", val, note)
    rd = solved(tx(government=gov(budget=("rent", "1"))), "Contestable")
    same_allocation(rd, t0, "TXD")
    assert rel(rd["tauR"], M(15) / 8) < TOL and not (0 <= rd["levy"] <= rd["R"])
    put("TXD_RENT_TAX", rd["tauR"], "tau_R = N P/R = 15/8, outside the rent: kappa = 8/15 < 1")

    # ------------------------------------------------------------------ G
    section("G: Appendix B's economy with a government (docs/unit-1f.md section 3.3)")
    g1 = solved(econ(), "Contestable")
    ga = solved(econ(government=gov(budget=("rent", "1"), mode="R")), "Contestable")
    same_allocation(ga, g1, "GA")
    assert rel(ga["tauR"], 1 / g1["kappa"]) < TOL
    put("GA_RENT_TAX", ga["tauR"], "GA, a replacing transfer of one composite: tau_R = N P/R = 1/kappa (eq 16)")
    put("G1_KAPPA", g1["kappa"], "G1's coverage kappa = T/(N P)")
    gres = {}
    for tag, gv in (("GB", gov(budget=("rent", "1"))), ("GC", gov(budget=("dividend", "0.5"))),
                    ("GP", gov(tw="0.1")), ("GT", gov(tc="0.25")), ("GTP", gov(tw="0.2")),
                    ("GW", gov(mw="0.2")), ("GE", gov(me="0.2")), ("GU", gov(mw="0.2", me="0.2")),
                    ("GS", gov(budget=("rent", "0.2")))):
        r = solved(econ(government=gv), "Contestable")
        gres[tag] = r
    same_allocation(gres["GT"], gres["GTP"], "GT = GT'")
    same_allocation(gres["GU"], gres["GS"], "GU = GS")
    assert rel(gres["GC"]["d"], M("1.25")) < TOL
    # the reservation wage (e^chi* - 1)(P + d) = v at GB, and eq 27 at GW and GE
    for tag in ("GB", "GW", "GE"):
        r = gres[tag]
        chi_star = r["n_a"] / 4
        want = mp.expm1(chi_star) * (r["Ps"] + r["d"] + e_me(tag) * r["Ps"]) - (e_mw(tag) - e_me(tag)) * r["Ps"]
        assert rel(want, r["w"]) < TOL, (tag, want, r["w"])
    assert gres["GW"]["n_a"] > g1["n_a"] > gres["GE"]["n_a"] and gres["GW"]["w"] < g1["w"] < gres["GE"]["w"]
    assert gres["GB"]["x"] > g1["x"] and gres["GB"]["n_a"] < g1["n_a"]
    for tag in ("GB", "GC", "GP", "GT", "GW", "GE", "GU"):
        put_all(tag, gres[tag], ALLOC)
    put("GB_RENT_TAX", gres["GB"]["tauR"], "the levy's rate, (N d)/R")
    put("GB_KAPPA", gres["GB"]["kappa"], "kappa at GB")
    put("GC_D", gres["GC"]["d"], "d = tau_R R/N = 5/4")
    assert rel(gres["GT"]["omega_net"], gres["GTP"]["omega_net"]) < TOL
    put("GT_OMEGA_NET", gres["GT"]["omega_net"], "the net wage in producer composites, kappa_w v/P, as at tau_w = t/(1 + t)")

    # ------------------------------------------------------------------ INC
    section("INC: payroll incidence at G1, workers bear eps_D/(eps_D + eps_S) (SSRN D.2 p.32; main.tex:829-833)")
    h = DERIVATIVE
    up = solved(econ(government=gov(tw=h)), "Contestable")
    dn = solved(econ(government=gov(tw=-h)), "Contestable")
    share = -(mp.log(up["omega_net"]) - mp.log(dn["omega_net"])) / (2 * h)
    e0 = econ()
    xs = g1["x"]
    pu, pd = e0.ev(xs + h, 0), e0.ev(xs - h, 0)
    dlo = mp.log(pu["w"] / pu["Ps"]) - mp.log(pd["w"] / pd["Ps"])
    eps_d = -(mp.log(pu["nD"]) - mp.log(pd["nD"])) / dlo
    eps_s = (mp.log(pu["S"]) - mp.log(pd["S"])) / dlo
    assert rel(share, eps_d / (eps_d + eps_s)) < mp.mpf(10) ** -35, (share, eps_d, eps_s)
    put("INC_SHARE", share, "-d ln omega_net/d tau_w at tau_w = 0 = eps_D/(eps_D + eps_S)")
    put("INC_EPS_D", eps_d, "eps_D = -d ln n_D/d ln omega along the line at x*")
    put("INC_EPS_S", eps_s, "eps_S = d ln S/d ln omega_net at x*")

    # ------------------------------------------------------------------ W
    section("W: the wall, where labour demand does not move with the wage (docs/unit-1f.md section 4.10 (e))")
    wres = {}
    for tag, tw in (("0", "0"), ("005", "0.05"), ("01", "0.1")):
        r = solved(econ(lam="0.6", government=gov(tw=tw)), "Wall")
        wres[tag] = r
        put(f"W1_V_{tag}", r["w"], f"the wall's gross wage at tau_w {tw}")
    for tag in ("005", "01"):
        assert rel(wres[tag]["omega_net"], wres["0"]["omega_net"]) < TOL
        same_allocation(dict(wres[tag], w=wres["0"]["w"], Ps=wres["0"]["Ps"]), wres["0"], "W1 quantities")
    put("W1_OMEGA_NET", wres["0"]["omega_net"], "the net real wage, the same at every tau_w on the wall")
    wi = solved(econ(lam="0.6", government=gov(tw="0.2")), "Idle")
    assert rel(wi["omega_net"], M(14) / 9) < TOL
    for key, val, note in (("OMEGA_NET", wi["omega_net"], "(1 - 0.2) 35/18 = 14/9 on idle land"),
                           ("T_M", wi["T_m"], "the market's land in use"), ("N_A", wi["n_a"], "N_a"),
                           ("Y", wi["Y"], "Y")):
        put(f"WI_{key}", val, note)
    regime, res = econ(N="20", chi="0.05", government=gov(mw="0.1")).solve()
    assert regime == "SurplusLabour" and rel(res["f_start"], -10) < TOL
    put("SL_F_START", res["f_start"], "SurplusLabour: f_0 = n_D(0) - S(0) = 10 - 20")

    # ------------------------------------------------------------------ AP
    section("AP: SSRN's automation path with a payroll tax, the corollary and eq 17 (SSRN p.16, p.31)")
    prev = None
    for k in (0, 4, 8, 12, 16, 20):
        eta = mp.mpf(2) ** -k
        r = solved(econ(lam="0", g0="1", g1="1", eta=eta, government=gov(tw="0.5")), "Contestable")
        bound = M("0.5") * r["w"] / r["B_s"]
        gw = r["G_w"] / (4 * r["Ps"])
        assert gw <= bound and r["kappa"] <= 10 / (4 * r["B_s"]) * (1 + TOL)
        if prev is not None:
            assert gw < prev["gw"] and r["kappa"] > prev["kappa"] and r["R"] / r["income"] > prev["RI"]
        prev = dict(gw=gw, kappa=r["kappa"], RI=r["R"] / r["income"])
        for key, val, note in (("X_STAR", r["x"], "x*"), ("V", r["w"], "v/r"),
                               ("LABOR_SHARE", r["W"] / r["income"], "the labour share W/I"),
                               ("KAPPA", r["kappa"], "kappa = r T/(N P)"), ("T_NB", 10 / (4 * r["B_s"]), "T/(N B_s)"),
                               ("GW_NP", gw, "payroll revenue per person over P, G_w/(N P)"),
                               ("BOUND", bound, "the corollary's bound tau_w (v/r)/B_s"),
                               ("C_R", r["C"] / r["R"], "C/R, the consumption base over the rent base")):
            put(f"AP{k}_{key}", val, f"{note} at eta 2^-{k}")
    e_full = econ(lam="0", g0="1", g1="1", T="12")
    q1 = e_full.ev(1, 0)
    L_s, B_s, _, _, _, _ = e_full.totals_c(q1)
    assert L_s == 0 and rel(q1["Ps"], B_s) < TOL

    # ------------------------------------------------------------------ K
    section("K: coverage with parcels, RentRate d-hat 1 in Replace mode (SSRN eq 16; docs/unit-1f.md section 3.3)")
    RR1 = gov(budget=("rent", "1"), mode="R")
    for tag, N, eta in (("KR4", "80", "1"), ("KR5", "80", "0.7"), ("KR1", "50", "2.5")):
        r = solved(race(N, eta, RR1), "Contestable")
        base = solved(race(N, eta, None), "Contestable")
        same_allocation(r, base, tag)
        if tag == "KR1":
            assert r["T_p"] > 0 and r["tauR"] > 1 / r["kappa"]
            assert rel(r["tauR"], M(N) * r["Ps"] / r["T_m"]) < TOL
        else:
            assert rel(r["tauR"], 1 / r["kappa"]) < TOL
        put(f"{tag}_KAPPA", r["kappa"], "kappa = r T/(N P)")
        put(f"{tag}_RENT_TAX", r["tauR"], "the levy's rate at d-hat 1: 1/kappa with T_m = T, N P/(r T_m) with rented plots")
    kt = solved(k3(gov(tc="0.25")), "Contestable")
    ktp = solved(k3(gov(tw="0.2")), "Contestable")
    same_allocation(kt, ktp, "KT")
    put_all("KT", kt, ALLOC + (("T_P", "T_p", "enclosed land in rented plots"),))

    # ------------------------------------------------------------------ E
    section("E: a walled type under a government (docs/unit-1f.md section 4.5)")
    ER = gov(tw="0.1", tc="0.1", budget=("rent", "0.5"), me="0.1")
    er = solved(entrant_trained(ER), "Wall")
    erp = solved(entrant_trained(gov(tw=1 - M("0.9") / M("1.1"), budget=("rent", "0.5"), me="0.1")), "Wall")
    same_allocation(er, erp, "ER")
    assert er["walled"] == [1]
    econ_er = entrant_trained(ER)
    c_t = econ_er.walled_rate(1, er["zeta"][1])
    assert rel(er["wages"][1], c_t * er["Ps"]) < TOL
    put_all("ER", er, (("V", "w", "the pool's wage, on the wall"), ("P", "Ps", "P"), ("N_A", "n_a", "N_a")))
    put("ER_TRAINED_WAGE", er["wages"][1], "the trained's wage c_T P at its wall")
    put("ER_TRAINED_RATE", c_t, "c_T = ((a_T + mu_e) zeta_T - (mu_w - mu_e)) (1 + t_c)/(1 - tau_w)")

    # ------------------------------------------------------------------ C
    section("C: the CES household on G1's economy, alpha 0.3 (SSRN eq 26; docs/unit-1f.md section 3.3)")
    cres = {}
    lemma = 0
    for tag, sigma in (("C1", "0.5"), ("C2", "1"), ("C3", "2")):
        e = econ(z=ces_z("0.3", sigma), basket=sigma)
        r = solved(e, "Contestable")
        cres[tag] = r
        q_ = 1 / r["p"][0]
        assert rel(r["shares"][1], g1b.ces_share("0.3", sigma, q_)) < TOL
        grid_points += assert_f_nonincreasing(e)
        for k in range(1, 5):
            lemma_f1(e, M(k) / 5)
            lemma += 1
        put_all(tag, r, ALLOC + (("SPACE_SHARE", lambda rr: rr["shares"][1], "space's share = eq 26's alpha(q)"),
                                 ("Q", "q", "q = r/p_g")))
    assert rel(cres["C2"]["shares"][1], M("0.3")) < TOL
    section("CP: C1-C3's baskets on AP's path, space's share toward eq 26's limits (SSRN App. C p.31)")
    for tag, sigma in (("05", "0.5"), ("1", "1"), ("2", "2")):
        for k in (0, 8, 16):
            eta = mp.mpf(2) ** -k
            e = econ(lam="0", g0="1", g1="1", eta=eta, z=ces_z("0.3", sigma), basket=sigma)
            r = solved(e)
            put(f"CP{tag}_{k}_SHARE", r["shares"][1], f"space's share at sigma {sigma}, eta 2^-{k}")
            put(f"CP{tag}_{k}_Q", r["q"], f"q = r/p_g at sigma {sigma}, eta 2^-{k}")
    section("CW: W3 under CES, a free category at the wall's end: no idle stretch (docs/unit-1f.md section 2.12)")
    for tag, sigma in (("05", "0.5"), ("2", "2")):
        e = econ(lam="0.6", chi="3", z=ces_z("0.3", sigma), basket=sigma)
        r = solved(e, "Wall")
        path = e.path()
        assert path["free_end"] and path["f_end"] < 0
        lemma_f1(e, 1, w=r["w"])
        lemma += 1
        put_all(f"CW{tag}", r, (("V", "w", "v, on the wall with land scarce"), ("P", "Ps", "P"),
                                ("N_A", "n_a", "N_a")))
        put(f"CW{tag}_F_END", path["f_end"], "f_inf = -S_inf, the path's end")
    section("CI: W3 with space given one human-required hour, C1's basket: the idle stretch (docs/unit-1f.md section 2.12)")
    e = econ(lam="0.6", chi="3", z=ces_z("0.3", "0.5"), basket="0.5", required=("0", "1"))
    r = solved(e, "Idle")
    assert not e.path()["free_end"]
    put_all("CI", r, (("T_M", "T_m", "the market's land in use"), ("Y", "Y", "Y"), ("N_A", "n_a", "N_a"),
                      ("P", "Ps", "P in pool wages"), ("IDLE", "idle", "T_idle")))
    put("CI_SPACE_SHARE", r["shares"][1], "space's share at the idle prices")
    section("CA: C3's basket at the all-human corner, the good free as v -> 0 (docs/unit-1f.md section 5.3 step 2)")
    # N 40, T 3, gamma = 1.5 + x, chi_max 0.02: space's content vanishes as v -> 0 (sigma 2),
    # so Y = T/B_s grows without bound there and f_0 = +inf
    e = econ(N="40", T="3", g0="1.5", g1="1", chi="0.02", z=ces_z("0.3", "2"), basket="2")
    assert e.start_value() == INF
    r = solved(e, "AllHuman")
    put_all("CA", r, (("V", "w", "v, at the all-human corner"), ("P", "Ps", "P"), ("N_A", "n_a", "N_a"),
                      ("SPACE_SHARE", lambda rr: rr["shares"][1], "space's share = eq 26's alpha(q)")))
    section("CF: C1's basket on W3 with few workers, the wall far out (docs/unit-1f.md section 5.5)")
    for tag, n in (("CF2", "0.01"), ("CF3", "0.001")):
        e = econ(N=n, lam="0.6", chi="3", z=ces_z("0.3", "0.5"), basket="0.5")
        r = solved(e, "Wall")
        assert e.path()["free_end"]
        put_all(tag, r, (("V", "w", f"v, on the wall with land scarce, N {n}"), ("P", "Ps", "P"),
                         ("N_A", "n_a", "N_a"), ("Y", "Y", "Y")))
    section("CS: W1 under a steep CES (sigma 20) with eq 26's weights, space's weight 0.3^20 (docs/unit-1f.md section 5.5)")
    e = econ(lam="0.6", z=ces_z("0.3", "20"), basket="20")
    r = solved(e, "Contestable")
    put_all("CS", r, ALLOC + (("Y", "Y", "Y"), ("SPACE_SHARE", lambda rr: rr["shares"][1], "space's share"),
                              ("GOOD_CONTENT", lambda rr: rr["content"][0], "c_good, units of the good per composite"),
                              ("SPACE_CONTENT", lambda rr: rr["content"][1], "c_space, units of space per composite")))

    # ------------------------------------------------------------------ AJ
    section("AJ: check_pinning's A-joint household, Ces { sigma: 1 } over (good, space) (check_pinning.py:225-267)")
    # A-joint itself, in the good numeraire, from its own equations: land clearing in x*
    aJ, lamJ, bJ, TJ, alpha = M("0.5"), M("0.1"), M("0.2"), M(10), M("0.3")

    def joint(xs):
        g = 1 + 4 * xs
        Ig = xs + 2 * xs * xs
        cJ = 1 / (g * (1 - xs) + Ig)
        wJ, rJ = g * cJ, cJ * (1 - aJ - lamJ * g) / bJ
        YJ = 1 / ((1 - xs) + lamJ * Ig / (1 - aJ))
        XJ = YJ * Ig / (1 - aJ)
        return bJ * XJ + alpha * (wJ + rJ * TJ) / rJ - TJ, wJ, cJ, rJ, YJ, XJ

    lo_, hi_ = M("0.001"), M("0.989")
    for _ in range(BISECTIONS):
        mid = (lo_ + hi_) / 2
        if (joint(lo_)[0] > 0) == (joint(mid)[0] > 0):
            lo_ = mid
        else:
            hi_ = mid
    xj = (lo_ + hi_) / 2
    _, wj, cj, rj, yj, xxj = joint(xj)
    printed = dict(x=0.8752410404997317, w=1.5160527572576763, c=0.33682844446030447, r=0.08404473252192293,
                   Y=1.6495500577338336, X=7.942038511535193)
    for key, val in (("x", xj), ("w", wj), ("c", cj), ("r", rj), ("Y", yj), ("X", xxj)):
        assert rel(val, M(printed[key])) < 3e-15, (key, val)
    # the household form reproduces it: in r = 1 units, divide by the good's price
    aj = a_joint()
    regime, _ = aj.solve()
    assert regime == "NotViable" and 1 - M("0.5") - M("0.1") * 5 == 0
    tweak = econ(N="1", a="0.5", lam="0.1", b="0.2", g0="1", g1=M("3.99999999999999999999999999999"),
                 chi="0.015625", z=("0.7", "0.3"), basket="1")
    rt = solved(tweak, "Contestable")
    assert rel(rt["x"], xj) < mp.mpf(10) ** -20 and rel(rt["w"] / rt["p"][0], wj) < mp.mpf(10) ** -20
    aj_gap = max(rel(val, M(printed[key])) for key, val in (("x", xj), ("w", wj), ("c", cj), ("r", rj),
                                                              ("Y", yj), ("X", xxj)))
    AJ_GAP.append(aj_gap)
    r = solved(a_joint(g1="3.9"), "Contestable")
    pg = r["p"][0]
    assert rel(r["Ps"], pg ** M("0.7") * 1 ** M("0.3")) < TOL  # check_dynamics :405, r = 1
    for key, val, note in (("X_STAR", r["x"], "x*"), ("V", r["w"], "v"), ("P", r["Ps"], "P = p^0.7 r^0.3"),
                           ("W_P", r["w"] / pg, "w/p"), ("R_P", 1 / pg, "r/p"),
                           ("GOODS", r["content"][0] * r["Y"], "goods, the good's final output")):
        put(f"AJ1_{key}", val, f"AJ1 (gamma = 1 + 3.9x): {note}")
    r = solved(a_joint(lam="0.08"), "Wall")
    pg = r["p"][0]
    goods = r["content"][0] * r["Y"]
    assert rel(goods, M(25) / 12) < TOL and rel(1 / pg, M(5) / 42) < TOL and rel(r["w"], 15) < TOL
    for key, val, note in (("GOODS", goods, "goods 25/12 = N/lambda-tilde_g"), ("R_P", 1 / pg, "r/p = 5/42"),
                           ("P_GOOD", pg, "p = 8.4"), ("V", r["w"], "v = 15"), ("W_P", r["w"] / pg, "w/p = 25/14"),
                           ("HOUSING", r["content"][1] * r["Y"], "housing 7.5 = T - 1.2 goods")):
        put(f"AJW_{key}", val, f"AJW (lambda 0.08), the wall in closed form: {note}")

    # ------------------------------------------------------------------ X
    section("X: the wrong units of docs/unit-1f.md section 3.3")
    for tag, sigma in (("X1", "0.5"), ("X3", "2")):
        # C1's (C3's) composite frozen at its reference content per good: one good and
        # (3/7)^sigma space, a fixed basket
        zg, zs = ces_z("0.3", sigma)
        r = solved(econ(z=("1", zs / zg)), "Contestable")
        assert rel(r["x"], cres["C" + tag[1]]["x"]) > mp.mpf(10) ** -3
        put(f"{tag}_X_STAR", r["x"], f"x* with C{tag[1]}'s composite frozen at one good and (3/7)^{sigma} space")
        put(f"{tag}_SPACE_SHARE", r["shares"][1], "space's share with the composite frozen")

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens_1f.txt: the golden numbers for oracle unit 1f.",
        f"# Written by goldens/generate_1f.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits.",
        "# Equations: docs/unit-1f.md section 4. laborformal references are at 31b3482.",
        f"# The household form nests generate_1e.py's solves within {mp.nstr(nest_worst, 3)} (1e-65 asserted).",
        f"# f nonincreasing at {grid_points} grid points; Lemma F1 at {lemma} points against a derivative.",
        f"# A-joint's root (check_pinning.py:225-267) within {mp.nstr(AJ_GAP[0], 3)} of its printed values, and the",
        "# household form's with gamma(1) 1e-29 below A-joint's within 1e-20 of it.",
        f"# fnv1a64 generate_1f.py = {fnv1a64(source_bytes(__file__)):016x}",
        f"# fnv1a64 generate_1e.py = {fnv1a64(source_bytes(g1e.__file__)):016x}",
        f"# fnv1a64 generate_1d.py = {fnv1a64(source_bytes(g1d.__file__)):016x}",
        f"# fnv1a64 generate_1c.py = {fnv1a64(source_bytes(g1c.__file__)):016x}",
        f"# fnv1a64 generate_1b.py = {fnv1a64(source_bytes(g1b.__file__)):016x}",
        f"# fnv1a64 generate.py = {fnv1a64(source_bytes(g1a.__file__)):016x}",
        f"# fnv1a64 body = {fnv1a64(body.encode('utf-8')):016x}",
    ]
    return "\n".join(header) + "\n" + body


def e_mw(tag):
    return M("0.2") if tag == "GW" else mp.mpf(0)


def e_me(tag):
    return M("0.2") if tag == "GE" else mp.mpf(0)


def fnv1a64(data):
    """FNV-1a, 64 bits, over the bytes with every CRLF read as LF (generate.py's)."""
    return g1a.fnv1a64(data)


def source_bytes(path):
    with open(os.path.abspath(path), "rb") as fh:
        return fh.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--check", action="store_true",
                        help="compare with goldens_1f.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(HERE, "goldens_1f.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same = fh.read() == text
        print("goldens_1f.txt is current" if same else "goldens_1f.txt differs from generate_1f.py's output")
        return 0 if same else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
