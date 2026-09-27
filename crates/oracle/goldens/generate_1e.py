"""generate_1e.py: the golden numbers for oracle unit 1e.

Dated 2026-09-27. Every golden that crates/oracle's unit-1e tests use is computed here with
mpmath at 70 digits, from the equations of docs/unit-1e.md section 4: parcels of an acreage, a
quality and an access (enclosed, or open: a commons), the rent schedule over parcels and which
of them idle (main.tex:140-145, :692-698, :804); main.tex's priced exit s(q) = max(s0 - q h, s_)
beside SSRN's dependence form, both under SSRN eq 8 with the exit life's value (main.tex:369-381;
SSRN pp.10-11); exit plots on the commons, at its shadow rent, and rented on enclosed land; idle
enclosed land at zero rent with the pool's wage as numeraire (SSRN A.1, App. C); enclosure points
and ties; and coverage, q* and N_crit (main.tex:835-864; SSRN eq 16, D.3; check_enclosure.py
N-i to N-vi :42-126; check_pinning.py P3 :131-145). laborformal has no equilibrium with parcels
or a priced exit (ADDENDUM section 5 item 4): every instance is constructed.

The unit-1d generator (generate_1d.py, and through it generate_1c.py, generate_1b.py and
generate.py) supplies the machine block, the tasks, the Leontief solves, the worker types, the
walk and the corners, which this file's Economy extends (docs/unit-1e.md section 7), and its
solves are used to assert that the parcel form nests them. Nothing is imported from laborformal
or from the oracle.

Run with any Python that has mpmath (1.3.0 was used), from this directory or any other:

    python goldens/generate_1e.py           # writes goldens/goldens_1e.txt beside this file
    python goldens/generate_1e.py --check   # exits 1 if goldens_1e.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in tests/gate/goldens_1e.rs
are these values rounded to 20 significant digits, and the test
e10_goldens_file::constants_match_goldens_1e_txt enforces that.

The header of goldens_1e.txt records six FNV-1a 64-bit digests: of this file, of
generate_1d.py, generate_1c.py, generate_1b.py and generate.py, and of the goldens that follow
the header, each with CRLF read as LF. The gate recomputes all six.
"""

import argparse
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import mpmath as mp  # noqa: E402

import generate_1d as g1d  # noqa: E402  (imports generate_1c.py and the rest; 70 digits)

g1c = g1d.g1c
g1b = g1c.g1b
g1a = g1c.g1a

DPS = 70
"""Working precision in decimal digits, as in the other generators."""
SIG_OUT = 30
"""Significant digits written per value; the Rust constants keep 20 of them."""
BISECTIONS = 250
"""Halvings of a region of the line, as generate_1c.py and generate_1d.py."""
HALVINGS = 400
"""Halvings of a corner's bracket in omega, of [0, 1e-12], of an enclosure point's bracket, of
the idle stretch and of a crowded commons' plot rent in [0, r]: 2^-400 of the bracket."""
SCAN = 1024
"""Interior points per piece in the count: four times the oracle's EXIT_SCAN (section 7)."""
SCAN_HALVINGS = 100
"""Halvings of a crowded commons' plot rent at a scan point, which decides only a sign."""
SCAN_DPS = 30
"""Working precision at a scan point: 30 digits decide a sign far below the oracle's rounding."""
GRID = 8
"""Points per piece at which Lemma 5's sign is checked against a finite difference."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""

mp.mp.dps = DPS
assert g1d.DPS == DPS

M, rel, leontief = g1c.M, g1c.rel, g1c.leontief
TOL = IDENTITY_TOL
LO = g1d.LO
INF = mp.inf
F = g1d.F
worker = g1d.worker


def priced(s0, sf, h):
    """A priced exit form (s0, s_, h), in units of the exit good (section 3.1)."""
    return dict(s0=M(s0), sf=M(sf), h=M(h))


def s_of_q(q, e):
    """main.tex:370: s(q) = max(s0 - q h, s_)."""
    return max(e["s0"] - q * e["h"], e["sf"])


class Economy(g1d.Economy):
    """Unit 1d's economy with parcels, exit forms and an exit good (docs/unit-1e.md sections
    2-4). ALT names a wrong unit of section 3.3 for the A instances: 'commons_rent' charges the
    market rent on commons plots, 'no_spill' leaves rented plots in production, 'basket_q'
    deflates the rent by P_s."""

    def __init__(self, *, parcels, exits, exit_good=0, alt=None, **kw):
        self.parcels = [(n, M(a), M(q), s) for n, a, q, s in parcels]
        T = sum((a * q for _, a, q, s in self.parcels if s == "E"), mp.mpf(0))
        To = sum((a * q for _, a, q, s in self.parcels if s == "O"), mp.mpf(0))
        super().__init__(land=T, **kw)
        self.To = To
        self.ex = list(exits)
        self.g = exit_good
        self.alt = alt
        I = self.I
        assert len(self.ex) == I and 0 <= exit_good < self.C
        assert T > 0 and all(a > 0 and q >= 0 for _, a, q, _ in self.parcels)
        self.priced = any(e is not None for e in self.ex)
        self.exit_free = all(e is None or (e["s0"] == 0 and e["sf"] == 0) for e in self.ex)
        if self.priced:
            assert all(r == 0 for row in self.R for r in row), "section 2.9: no reserved hours"
        self.takers = [i for i, e in enumerate(self.ex) if e is not None and e["h"] > 0 and e["s0"] > e["sf"]]
        need = sum((self.ex[i]["h"] * self.Nw[i] for i in self.takers), mp.mpf(0))
        assert T + To > need, "section 3.2: land for every plot"
        self.q_enc = [((e["s0"] - e["sf"]) / e["h"]) if i in self.takers else None for i, e in enumerate(self.ex)]
        self.target = [(e["h"] / (e["s0"] - e["sf"])) if i in self.takers else None for i, e in enumerate(self.ex)]
        # section 5.2: the certification's totals
        self.bbar_g = self.bbar[self.g]
        self.ell0 = sum(y * (L + h) for y, L, h in zip(self.yhat, self.Lbar_dir, self.LH)) / self.B_y
        self.certified = self.rho == 0 and (len(self.takers) <= 1 or To == 0) and all(
            self.ex[i]["h"] <= self.ex[i]["s0"] * self.bbar_g and self.ex[i]["h"] * self.ell0 <= self.eps[i]
            for i in self.takers)

    # ---------------------------------------------------------------- prices at the rent r
    def prices_at(self, x, t, w, r):
        """The machine block on the line (w None, r = 1: the closure wage in closed form) or at
        the wage w and rent r (section 5.1 step 1): p = w lt + r bt, O and V from the recipes."""
        K = self.K
        if w is None:
            g = self.gamma(x)
            w = self.closure_wage(t, g)
            if w is None:
                return None
        p = [w * self.lt[k] + r * self.bt[k] for k in range(K)]
        O = [sum(self.aop[k][l] * p[l] for l in range(K)) + self.lop[k] * w + r * self.bop[k] for k in range(K)]
        V = [sum(self.aI[k][l] * p[l] for l in range(K)) + self.lI[k] * w + r * self.bI[k] for k in range(K)]
        return w, p, p[t] / self.theta[t], O, V

    # ---------------------------------------------------------------- the exit sub-problem
    def trial(self, rc, v, P, pg, force, pin=None, wall=None):
        """Section 5.1 step 2 at the plot rent rc: each type's branch, exit value and supply,
        and G, the plot land asked for. pin = (i, "P" or "F") fixes type i's branch, for G's
        two limits at the rent where type i's plot demand drops. wall is the exit good's price
        per unit of rent at the wall's end when it is free at r = 0 (section 12 item 16): the
        branch and the exit goods are then decided there, rc a rent in [0, 1] of the wall's
        units, and the exit value in money is p_g s = 0."""
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
            n = self.Nw[i] * F(mp.log1p((wage - m) / (self.sig[i] * P + m)), self.chi[i])
            hh = share * (self.Nw[i] - n)
            pl = (e["h"] * hh) if e is not None else mp.mpf(0)
            out["G"] += pl
            for key, val in (("branch", br), ("m", m), ("s", s), ("nS", n), ("hh", hh), ("plots", pl)):
                out[key].append(val)
        return out

    def exit_state(self, v, P, pg, r, force, halv, wall=None):
        """Section 4.4: the regime and the plot rent at the point's prices, at the rent r. At
        r = 0 the plots past the commons stand free on idle enclosed land (Idle, not
        Enclosed). With a free exit good at r = 0 (wall given, section 12 item 16) the regime
        is decided in the wall's end's units, rents in [0, 1], and every rent in money is 0."""
        To = self.To
        top = M(r) if wall is None else M(1)
        price = pg if wall is None else wall

        def money(x):
            return x if wall is None else mp.mpf(0)

        def trial(rc, pin=None):
            return self.trial(rc, v, P, pg, force, pin=pin, wall=wall)

        if force is not None and force[0] == "share":
            t = trial(top)
            return dict(t, regime="Enclosed", rc=money(M(r)), Toc=min(t["G"], To), spill=max(t["G"] - To, 0))
        if self.alt == "commons_rent":
            t = trial(top)
            return dict(t, regime="Charged", rc=money(M(r)), Toc=min(t["G"], To), spill=max(t["G"] - To, 0))
        t0 = trial(0)
        if t0["G"] == 0:
            return dict(t0, regime="Unused", rc=mp.mpf(0), Toc=mp.mpf(0), spill=mp.mpf(0))
        if t0["G"] <= To:
            return dict(t0, regime="Commons", rc=mp.mpf(0), Toc=t0["G"], spill=mp.mpf(0))
        t1 = trial(top)
        if t1["G"] >= To:
            regime = "Enclosed" if r != 0 else "Idle"
            return dict(t1, regime=regime, rc=money(M(r)), Toc=To, spill=t1["G"] - To)
        # Crowded: the least rc in [0, r] with G(rc) <= T_o. G falls continuously between the
        # rents where a type's plot demand drops (r_o = p_g (s0 - s_)/h, its q_enc), and jumps
        # down at each: find the piece, or the drop, where it crosses T_o.
        drops = sorted((price * (self.ex[i]["s0"] - self.ex[i]["sf"]) / self.ex[i]["h"], i) for i in self.takers
                       if not (force is not None and force[1] == i))
        lo, g_lo, hi, g_hi = mp.mpf(0), t0["G"] - To, top, t1["G"] - To
        for d, i in drops:
            if not 0 < d < top:
                continue
            left = trial(d, pin=(i, "P"))
            if left["G"] - To <= 0:
                hi, g_hi = d, left["G"] - To
                break
            right = trial(d, pin=(i, "F"))
            if right["G"] - To <= 0:
                # the crossing is the drop: type i splits between commons plots and its floor,
                # with the same supply either way (section 4.4)
                land = min(To - right["G"], self.ex[i]["h"] * (self.Nw[i] - right["nS"][i]))
                right["plots"][i] = land
                right["hh"][i] = land / self.ex[i]["h"]
                right["G"] += land
                return dict(right, regime="Crowded", rc=money(d), Toc=To, spill=mp.mpf(0), split=[i])
            lo, g_lo = d, right["G"] - To
        # On the piece (lo, hi) G is continuous: the Illinois variant of regula falsi, to G's
        # working precision or 2^-halv of r.
        side = 0
        width = max(top * mp.mpf(2) ** -halv, 8 * mp.eps * top)
        mid = lo
        for _ in range(3 * halv):
            if hi - lo <= width:
                mid = hi
                break
            mid = (lo * g_hi - hi * g_lo) / (g_hi - g_lo)
            if not lo < mid < hi:
                mid = (lo + hi) / 2
            g_mid = trial(mid)["G"] - To
            if abs(g_mid) <= 16 * mp.eps * To:
                break
            if g_mid > 0:
                lo, g_lo = mid, g_mid
                if side == 1:
                    g_hi /= 2
                side = 1
            else:
                hi, g_hi = mid, g_mid
                if side == -1:
                    g_lo /= 2
                side = -1
        th = trial(mid)
        return dict(th, regime="Crowded", rc=money(mid), Toc=To, spill=mp.mpf(0), split=[])

    def wall_price(self, t):
        """The exit good's price per unit of rent at the wall's end under t where it embodies no
        labour: its price at x = 1 with w = 0 and r = 1, its land total (section 12 item 16)."""
        w, p, pi, O, V = self.prices_at(M(1), t, mp.mpf(0), 1)
        HM = [self.tasks(j, M(1)) for j in range(self.C)]
        H = [h + lh for (h, _), lh in zip(HM, self.LH)]
        pb = leontief(self.Acc, [w * H[j] + pi * HM[j][1] + self.bc[j] for j in range(self.C)])
        return pb[self.g]

    # ---------------------------------------------------------------- one evaluation
    def ev(self, x, t, w=None, r=1, land=None, force=None, split=None, edge=None, halv=HALVINGS):
        """Section 5.1 at (x, t) on the line (w None), at a corner's wage w, or on the idle
        stretch (r = 0, w = 1, land given), with the task services split as given, an enclosure
        point's side forced, and a type at the edge of its reserved shortage."""
        x = M(x)
        ps = self.prices_at(x, t, w, r)
        if ps is None:
            return dict(viable=False)
        w, p, pi, O, V = ps
        C, I = self.C, self.I
        HM = [self.tasks(j, x) for j in range(C)]
        Mt = [m for _, m in HM]
        H = [h + lh for (h, _), lh in zip(HM, self.LH)]
        pbase = leontief(self.Acc, [w * H[j] + pi * Mt[j] + r * self.bc[j] for j in range(C)])
        Pbase = sum(z * q for z, q in zip(self.z, pbase))
        Hy = sum(y * h for y, h in zip(self.yhat, H))
        My = sum(y * m for y, m in zip(self.yhat, Mt))
        sp = split or [(t, 1)]
        qty = self.quantities(My, sp)
        out = dict(viable=True, x=x, t=t, w=w, r=M(r), p=p, pi=pi, O=O, V=V, H=H, Mt=Mt, pbase=pbase, Pbase=Pbase,
                   Hy=Hy, My=My, qty=qty, split=sp, short=None, gamma=self.gamma(x), edge=edge)
        if self.exit_free:
            sub = dict(regime="Unused", rc=mp.mpf(0), Toc=mp.mpf(0), spill=mp.mpf(0), G=mp.mpf(0),
                       branch=["D" if e is None else "F" for e in self.ex], m=[mp.mpf(0)] * I,
                       s=[mp.mpf(0)] * I, hh=[mp.mpf(0)] * I, plots=[mp.mpf(0)] * I)
        else:
            pg = Pbase if self.alt == "basket_q" else pbase[self.g]
            wall = self.wall_price(t) if r == 0 and pg == 0 else None
            sub = self.exit_state(w, Pbase, pg, r, force, halv, wall=wall)
            out.update(wall=wall)
        spill = sub["spill"] if self.alt != "no_spill" else mp.mpf(0)
        Tland = (self.T - spill) if land is None else M(land)
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
        c = [zeta[i] * self.sig[i] * self.lR[i] for i in range(I)]
        P, walled = self.walk(Pbase, e, c)
        out.update(zeta=zeta, e=e, c=c)
        if P is None:
            out.update(short=("ceiling", walled), f=INF)
            return out
        wages = [zeta[i] * self.sig[i] * P if i in walled else self.eps[i] * w for i in range(I)]
        m = sub["m"]
        nS = [self.Nw[i] * F(mp.log1p((wages[i] - m[i]) / (self.sig[i] * P + m[i])), self.chi[i]) for i in range(I)]
        S = sum(self.eps[i] * (nS[i] - D[i]) for i in range(I) if i not in walled)
        reserved_cost = [sum(wages[i] * self.R_chain[i][j] for i in range(I)) for j in range(C)]
        pc = [pbase[j] + reserved_cost[j] for j in range(C)]
        out.update(Ps=P, walled=walled, wages=wages, nS=nS, S=S, f=nD - S, reserved_cost=reserved_cost, pc=pc,
                   pg=pc[self.g])
        return out

    # ---------------------------------------------------------------- the path's pieces
    def at_span(self, span, s, halv=HALVINGS, force=None):
        kind = span[0]
        if kind == "AllHuman":
            return self.ev(0, span[1], w=self.corner_wage(span, s), force=force, halv=halv)
        if kind in ("Line", "Bottom"):
            return self.ev(s, span[1], force=force, halv=halv)
        if kind == "Wall":
            return self.ev(1, span[1], w=self.corner_wage(span, s), force=force, halv=halv)
        return self.ev(1, span[1], w=M(1), r=0, land=s, halv=halv)

    def corner_wage(self, span, omega):
        """Section 4.5 without reserved hours: v = omega B_s/(1 - omega L_s)."""
        L_s, B_s = span[2], span[3]
        return omega * B_s / (1 - omega * L_s)

    def corner_span(self, kind, x, t):
        q = self.ev(x, t, w=M(1))
        L_s, B_s, lt_c, _ = self.basket_totals(q)
        return (kind, t, L_s, B_s, lt_c[self.g])

    def exit_good_price(self, span, s):
        kind = span[0]
        if kind in ("AllHuman", "Wall"):
            x, w = (0 if kind == "AllHuman" else 1), self.corner_wage(span, s)
        else:
            x, w = s, None
        ps = self.prices_at(M(x), span[1], w, 1)
        w, p, pi, O, V = ps
        HM = [self.tasks(j, M(x)) for j in range(self.C)]
        H = [h + lh for (h, _), lh in zip(HM, self.LH)]
        pb = leontief(self.Acc, [w * H[j] + pi * HM[j][1] + self.bc[j] for j in range(self.C)])
        if self.alt == "basket_q":
            return sum(z * q for z, q in zip(self.z, pb))
        return pb[self.g]

    def on_floor(self, span, s, i):
        e = self.ex[i]
        return not (e["h"] < self.exit_good_price(span, s) * (e["s0"] - e["sf"]))

    def push_piece(self, path, span, lo, hi, end, to_the_end=False):
        """A piece from lo to hi with its end value; each enclosure point inside it as a pair
        of values (section 4.7), located where the exit good's price crosses r h/(s0 - s_)."""
        found = []
        if span[0] in ("AllHuman", "Line", "Wall"):
            for i in self.takers:
                f_lo = self.on_floor(span, lo, i)
                f_hi = (f_lo and span[4] <= 0) if to_the_end else self.on_floor(span, hi, i)
                if f_lo and not f_hi:
                    a, b = lo, hi
                    if b == INF:
                        b = max(lo, 1) * 2
                        while self.on_floor(span, b, i):
                            b *= 2
                    for _ in range(HALVINGS):
                        mid = (a + b) / 2
                        if self.on_floor(span, mid, i):
                            a = mid
                        else:
                            b = mid
                    # the last parameter at which the type is on its floor, as the oracle's
                    # largest such double: Below is then the floor's value (section 12 item 18)
                    found.append((a, i))
        found.sort(key=lambda z: (z[0], z[1]))
        frm = lo
        for s, i in found:
            below = self.at_span(span, s)
            above = self.at_span(span, s, force=("above", i))
            e = len(path["enclosures"])
            path["enclosures"].append(dict(span=span, s=s, worker=i))
            path["stations"].append((("EncBelow", e), below["f"]))
            path["pieces"].append((span, frm, s))
            path["stations"].append((("EncAbove", e), above["f"]))
            path["pieces"].append(None)
            frm = s
        path["stations"].append(end)
        path["pieces"].append((span, frm, hi))

    def path(self):
        """Section 5.3 step 2: the stations in path order, the pieces between them, and the
        enclosure points."""
        first, line_sw, wall_sw = self.envelope_ext()
        ltechs = [first] + [s[2] for s in line_sw]
        wtechs = [ltechs[-1]] + [s[2] for s in wall_sw]
        one = self.ev(1, ltechs[-1])
        if not one["viable"]:
            return None
        at0, atlo = self.ev(0, first), self.ev(LO, first)
        xs = [self.gamma_inv(g) for g, _, _ in line_sw]
        bounds = [LO] + xs + [M(1)]
        path = dict(stations=[("Start", INF)], pieces=[None], enclosures=[], one=one, at0=at0, atlo=atlo,
                    xs=xs, ltechs=ltechs, wtechs=wtechs, line_sw=line_sw, wall_sw=wall_sw)
        span = self.corner_span("AllHuman", 0, first)
        self.push_piece(path, span, mp.mpf(0), at0["w"] / at0["Ps"], ("Zero", at0["f"]))
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
        w_lo = one["w"]
        om_lo = one["w"] / one["Ps"]
        for s, t in enumerate(wtechs):
            span = self.corner_span("Wall", 1, t)
            if s < len(wall_sw):
                qb = self.ev(1, wall_sw[s][1], w=wws[s])
                self.push_piece(path, span, om_lo, wws[s] / qb["Ps"], (("WallBelow", s), qb["f"]))
                qa = self.ev(1, wall_sw[s][2], w=wws[s])
                path["stations"].append((("WallAbove", s), qa["f"]))
                path["pieces"].append(None)
                w_lo, om_lo = wws[s], wws[s] / qa["Ps"]
            else:
                om_end = 1 / span[2] if span[2] > 0 else INF
                te = t
                if self.exit_free:
                    end = self.wall_end(te)
                    f_end, t_inf = end["f"], self.T
                else:
                    t_inf = self.T - self.ev(1, te, w=M(1), r=0, land=self.T)["spill"]
                    f_end = self.ev(1, te, w=M(1), r=0, land=t_inf)["f"]
                path.update(om_end=om_end, f_end=f_end, t_inf=t_inf, te=te, last_start=path["stations"][-1][1])
                self.push_piece(path, span, om_lo, om_end, ("End", f_end), to_the_end=True)
        rest = self.ev(1, path["te"], w=M(1), r=0, land=0)
        path["stations"].append(("Rest", rest["f"]))
        path["pieces"].append((("Idle", path["te"]), mp.mpf(0), path["t_inf"]))
        path.update(wws=wws, S_inf=-rest["f"])
        return path

    @staticmethod
    def positive(kind, f):
        """Section 5.3 step 3."""
        if kind == "Start":
            return True
        if kind in ("One", "Rest"):
            return f >= 0
        return f > 0

    def scan_values(self, piece, scan):
        span, lo, hi = piece
        if span[0] not in ("AllHuman", "Line", "Wall") or not hi > lo:
            return []
        out = []
        for k in range(1, scan + 1):
            if hi == INF:
                s = lo + mp.mpf(2) ** (k * 1020 / (scan + 1) - 10)
            else:
                s = lo + (hi - lo) * k / (scan + 1)
            with mp.workdps(SCAN_DPS):
                f = +self.at_span(span, s, halv=SCAN_HALVINGS)["f"]
            out.append((s, f))
        return out

    def count(self, path, scan):
        """Section 5.3 step 3: the changes of side over the stations and the scan points."""
        pts = []
        scanning = scan > 0 and self.takers
        for k, (kind, f) in enumerate(path["stations"]):
            if scanning and path["pieces"][k] is not None:
                for s, fs in self.scan_values(path["pieces"][k], scan):
                    pts.append(("scan", k, s, fs, fs > 0))
            name = kind if isinstance(kind, str) else kind[0]
            pts.append(("station", k, None, f, self.positive(name, f)))
        changes = [(pts[j], pts[j + 1]) for j in range(len(pts) - 1) if pts[j][4] != pts[j + 1][4]]
        return changes

    # ---------------------------------------------------------------- the solve
    def solve(self, scan=SCAN):
        """Section 5.3. Returns (regime, report), the regime NotViable, NoMarket,
        MultipleEquilibria or the equilibrium's stretch (Contestable, Wall, AllHuman, Idle).
        Without exit values it is generate_1d.py's solve, with the idle stretch after the wall's
        end (section 2.12)."""
        if self.exit_free:
            return self.solve_exit_free()
        path = self.path()
        if path is None:
            return "NotViable", None
        changes = self.count(path, scan)
        if not changes:
            return "NoMarket", dict(f_end=path["f_end"], path=path)
        if len(changes) > 1:
            return "MultipleEquilibria", dict(changes=changes, path=path)
        st = path["stations"]
        k = next(k for k in range(1, len(st)) if self.positive(self.name(st[k - 1][0]), st[k - 1][1])
                 and not self.positive(self.name(st[k][0]), st[k][1]))
        return self.locate(path, k)

    def solve_exit_free(self):
        """Section 2.12: 1d's sequence with -S_inf appended. Where 1d has one equilibrium it is
        this one; 1d's LaborShort is an idle-land equilibrium (or the junction when f_inf = 0);
        a positive f_inf adds one change to 1d's MultipleEquilibria."""
        regime, r = g1d.Economy.solve(self)
        if regime == "NotViable":
            return regime, None
        te = r["wtechs"][-1]
        path = dict(f_end=r["f_end"], one=r["one"], at0=r["at0"], te=te, t_inf=self.T)
        rest = self.ev(1, te, w=M(1), r=0, land=0)
        assert rest["f"] < 0, "some type works at the ceiling of the real wage"
        if regime in ("Contestable", "Wall", "AllHuman"):
            w = None if regime == "Contestable" else r["w"]
            x = r["x"] if regime == "Contestable" else (0 if regime == "AllHuman" else 1)
            return regime, self.report_1e(path, x, r["t"], w=w, margin=regime, tie=r["tie"], edge=r["edge"])
        if regime == "LaborShort":
            return self.idle(path, junction=r["f_end"] == 0)
        extra = 1 if r["f_end"] > 0 else 0
        return regime, dict(changes=[None] * (r["changes"] + extra), path=path)

    @staticmethod
    def name(kind):
        return kind if isinstance(kind, str) else kind[0]

    def locate(self, path, k):
        """Section 5.3 step 4 between stations k - 1 and k."""
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
                return "Contestable", self.report_1e(path, path["xs"][i], b, tie=(a, share, path["line_sw"][i][0]),
                                                  margin="Contestable")
            s = before[1]
            g, b, a = path["wall_sw"][s]
            share = self.tie_share_1e(1, b, a, path["wws"][s])
            return "Wall", self.report_1e(path, 1, b, w=path["wws"][s], tie=(a, share, g), margin="Wall")
        span, lo, hi = piece
        if span[0] == "Idle":
            return self.idle(path, junction=False)
        if self.name(after) == "End" and f_after == 0 and f_before != 0:
            return self.idle(path, junction=True)
        f = lambda s: self.at_span(span, s)["f"]  # noqa: E731
        if self.name(before) == "Start":
            f_before = f(lo)
        if f_before == 0:
            s = lo
        elif f_after == 0:
            s = hi
        else:
            n = BISECTIONS if span[0] == "Line" else HALVINGS
            a, b = lo, hi
            for _ in range(n):
                mid = (a + b) / 2
                if f(mid) > 0:
                    a = mid
                else:
                    b = mid
            s = (a + b) / 2
        if span[0] in ("Line", "Bottom"):
            return "Contestable", self.report_1e(path, s, span[1], margin="Contestable")
        w = self.corner_wage(span, s)
        if span[0] == "Wall":
            return "Wall", self.report_1e(path, 1, span[1], w=w, margin="Wall")
        return "AllHuman", self.report_1e(path, 0, self.cheapest(w), w=w, margin="AllHuman")

    def tie_share_1e(self, x, below, above, w):
        """Section 4.6 on the market's land: sigma by bisection of f(sigma) on [0, 1] (the
        prices, and so the plots, do not depend on sigma)."""
        mix = lambda s: [(below, 1 - s), (above, s)]  # noqa: E731
        f = lambda s: self.ev(x, below, w=w, split=mix(s))["f"]  # noqa: E731
        fa, fb = f(mp.mpf(0)), f(mp.mpf(1))
        assert fa > 0 >= fb, (fa, fb)
        lo, hi = mp.mpf(0), mp.mpf(1)
        for _ in range(HALVINGS):
            mid = (lo + hi) / 2
            if f(mid) > 0:
                lo = mid
            else:
                hi = mid
        return (lo + hi) / 2

    def enclosure_tie(self, path, enc):
        """Section 4.7: T_m* = S/(n_D per unit of market land), psi in closed form."""
        span, s, i = enc["span"], enc["s"], enc["worker"]
        zero = self.at_span(span, s, force=("share", i, mp.mpf(0)))
        Tm = zero["Tland"] * zero["S"] / zero["nD"]
        E = self.Nw[i] - zero["nS"][i]
        psi = (self.T - Tm + self.To - zero["sub"]["G"]) / (self.ex[i]["h"] * E)
        assert 0 <= psi <= 1, psi
        # f is linear in psi: the root by the two ends agrees
        one = self.at_span(span, s, force=("share", i, mp.mpf(1)))
        assert rel(psi, zero["f"] / (zero["f"] - one["f"])) < TOL or zero["f"] == 0
        margin = dict(AllHuman="AllHuman", Line="Contestable", Wall="Wall")[span[0]]
        x = s if span[0] == "Line" else (0 if span[0] == "AllHuman" else 1)
        w = None if span[0] == "Line" else self.corner_wage(span, s)
        below = self.at_span(span, s)
        above = self.at_span(span, s, force=("above", i))
        return margin, self.report_1e(path, x, span[1], w=w, margin=margin, force=("share", i, psi),
                                   enclosure=dict(worker=i, psi=psi, f_below=below["f"], f_above=above["f"]))

    def idle(self, path, junction):
        """Section 4.6: T_m* in closed form without reserved hours, else by bisection on the
        idle stretch with the edge of a reserved shortage (1d section 12 item 16)."""
        te, t_inf = path["te"], path["t_inf"]
        ev = lambda t, edge=None: self.ev(1, te, w=M(1), r=0, land=t, edge=edge)  # noqa: E731
        if junction:
            return "Idle", self.report_1e(path, 1, te, w=M(1), r=0, land=t_inf, margin="Wall")
        if not any(l > 0 for l in self.lR):
            q = ev(t_inf)
            Tm = t_inf * q["S"] / q["nD"]
            assert abs(ev(Tm)["f"]) < TOL * q["nD"]
            # the closed form against bisection
            lo, hi = mp.mpf(0), t_inf
            for _ in range(HALVINGS):
                mid = (lo + hi) / 2
                if ev(mid)["f"] < 0:
                    lo = mid
                else:
                    hi = mid
            assert rel((lo + hi) / 2, Tm) < TOL
            return "Idle", self.report_1e(path, 1, te, w=M(1), r=0, land=Tm, margin="Wall")
        lo, hi = mp.mpf(0), t_inf
        for _ in range(HALVINGS):
            mid = (lo + hi) / 2
            if ev(mid)["f"] < 0:
                lo = mid
            else:
                hi = mid
        q_hi = ev(hi)
        if q_hi["short"] is None:
            return "Idle", self.report_1e(path, 1, te, w=M(1), r=0, land=(lo + hi) / 2, margin="Wall")
        i = q_hi["short"][1]
        kappa = self.edge_kappa(i, lambda k: ev(lo, edge=(i, k))["f"])
        return "Idle", self.report_1e(path, 1, te, w=M(1), r=0, land=lo, margin="Wall", edge=(i, kappa))

    # ---------------------------------------------------------------- the report
    def report_1e(self, path, x, t, w=None, r=1, land=None, margin=None, tie=None, force=None, edge=None,
               enclosure=None):
        """Section 4.8 at the equilibrium, every identity asserted to 1e-65."""
        split = [(t, 1)] if tie is None else [(t, 1 - tie[1]), (tie[0], tie[1])]
        q = self.ev(x, t, w=w, r=r, land=land, split=split, force=force, edge=edge)
        assert q["short"] is None
        K, C, I = self.K, self.C, self.I
        w, Y, Ps, r = q["w"], q["Y"], q["Ps"], q["r"]
        sub = q["sub"]
        X = [Y * xx for xx in q["qty"]["xhat"]]
        interest = sum(self.rho * om * Vk * Xk for om, Vk, Xk in zip(self.omega, q["V"], X))
        pooled = [i for i in range(I) if i not in q["walled"]]
        s_i = {i: self.eps[i] * (q["nS"][i] - q["D"][i]) for i in pooled}
        S = sum(s_i.values())
        pool = {i: (q["nD"] * s_i[i] / S if S != 0 else mp.mpf(0)) for i in pooled}
        hours = [q["D"][i] + (pool[i] / self.eps[i] if i in pooled and self.eps[i] > 0 else 0) for i in range(I)]
        wage_bill = w * q["nD"] + sum(q["wages"][i] * q["D"][i] for i in range(I))
        Tm = q["Tland"]
        income = wage_bill + r * Tm + interest
        provider = (r * Tm + interest) / Ps - self.nu
        Ntot = sum(self.Nw)
        spill = mp.mpf(0) if self.alt == "no_spill" else q["spill"]
        idle = self.T - Tm - spill if r == 0 else mp.mpf(0)
        E = [self.Nw[i] - q["nS"][i] for i in range(I)]
        pg = q["pg"]
        home_output = sum(pg * self.ex[i]["s0"] * sub["hh"][i] for i in range(I) if self.ex[i] is not None)
        home_floor = sum(pg * self.ex[i]["sf"] * (E[i] - sub["hh"][i]) for i in range(I) if self.ex[i] is not None)
        out = dict(margin=margin, x=q["x"], t=t, w=w, r=r, Ps=Ps, pg=pg, Y=Y, n_pool=q["nD"], n_a=sum(hours),
                   participation=sum(hours) / Ntot, income=income, interest=interest, wage_bill=wage_bill,
                   provider=provider, funded=provider > 0, real_wage=w / Ps, T_m=Tm, T_p=spill, idle=idle,
                   q=self.q_of(r, pg, t), kappa=r * self.T / (Ntot * Ps), regime=sub["regime"],
                   rc=sub["rc"], Toc=sub["Toc"], G=sub["G"], branch=sub["branch"], m=sub["m"], s=sub["s"],
                   hh=sub["hh"], plots=sub["plots"], E=E, hours=hours, wages=q["wages"], nS=q["nS"],
                   home_output=home_output, home_floor=home_floor, rent_in_kind=r * spill,
                   shadow=sub["rc"] * sub["Toc"], tie=tie, edge=edge, enclosure=enclosure, q_ev=q,
                   zeta=q["zeta"], pc=q["pc"], f_end=path["f_end"], f_line_1=path["one"]["f"],
                   f_line_0=path["at0"]["f"], X=X)
        out["parcels"] = self.parcel_report(out)
        self.assert_identities_1e(out, q, split)
        return out

    def q_of(self, r, pg, t):
        """q = r/p_g; at r = 0 its limit at the wall's end under t: 0, or one over the exit
        good's price per unit of rent there when it embodies no labour (section 12 item 16)."""
        if r != 0:
            return r / pg
        return 1 / self.wall_price(t) if pg == 0 else mp.mpf(0)

    def parcel_report(self, r):
        """Section 4.1: each parcel's share of its services in use, best first, and its rent
        and shadow rent per acre."""
        rows = {}
        for access, used in (("E", self.T if r["r"] == 1 else r["T_m"] + r["T_p"]), ("O", r["Toc"])):
            idx = sorted([k for k, pz in enumerate(self.parcels) if pz[3] == access],
                         key=lambda k: (-self.parcels[k][2], k))
            left = used
            for k in idx:
                _, A, Q, _ = self.parcels[k]
                cap = A * Q
                take = min(cap, max(left, 0))
                left -= take
                rows[k] = dict(used=(take / cap if cap > 0 else mp.mpf(0)),
                               rent=(r["r"] * Q if access == "E" else mp.mpf(0)),
                               shadow=(r["rc"] * Q if access == "O" else mp.mpf(0)))
        return [rows[k] for k in range(len(self.parcels))]

    def assert_identities_1e(self, r, q, split):
        """Every identity of section 4.8 at 1e-65, the regime's conditions and each type's
        branch and supply."""
        ch = {}
        I, C, K = self.I, self.C, self.K
        w, Y, Ps, rr = r["w"], r["Y"], r["Ps"], r["r"]
        ch["pool clears"] = rel(q["nD"], q["S"])
        ch["income Y P_s"] = rel(Y * Ps, r["income"])
        ch["income sum p z Y"] = rel(sum(p * z * Y for p, z in zip(q["pc"], self.z)), r["income"])
        ch["income supply side"] = rel(sum(q["wages"][i] * r["hours"][i] for i in range(I)) + rr * r["T_m"]
                                       + r["interest"], r["income"])
        ch["Y B = T_m"] = rel(Y * q["qty"]["land"], r["T_m"])
        # the cost system on the C + K rows at the rent r, and T_m = b^q'y
        n = C + K
        p = q["pc"] + q["p"]
        for j in range(C):
            cost = sum(self.Acc[j][l] * p[l] for l in range(C)) + sum(
                sh * q["Mt"][j] / self.theta[tt] * q["p"][tt] for tt, sh in split) + q["H"][j] * w + rr * self.bc[j] \
                + sum(q["wages"][i] * self.R[j][i] for i in range(I))
            ch[f"p row {j}"] = rel(p[j], cost) if p[j] != 0 else abs(cost)
        for k in range(K):
            cost = sum(self.Ahat[k][l] * q["p"][l] for l in range(K)) + self.lhat[k] * w + rr * self.bhat[k]
            ch[f"p machine {k}"] = rel(q["p"][k], cost) if q["p"][k] != 0 else abs(cost)
        land_used = self.B_y * Y + sum(b * xx for b, xx in zip(self.bq, r["X"]))
        ch["T_m = b'y"] = rel(land_used, r["T_m"])
        # the land's partition and the regime's conditions (section 4.4)
        ch["partition"] = abs(self.T - r["T_m"] - r["T_p"] - r["idle"]) / self.T
        sub = q["sub"]
        if not self.exit_free and rr == 1:
            if sub["regime"] == "Commons":
                assert sub["G"] <= self.To and sub["rc"] == 0
            elif sub["regime"] == "Crowded":
                ch["commons clears"] = rel(sub["G"], self.To)
                assert 0 < sub["rc"] < 1
            elif sub["regime"] == "Enclosed":
                assert sub["rc"] == 1
                assert r["enclosure"] is not None or self.alt or rel(sub["G"] - self.To, r["T_p"]) < TOL
        # each type's supply from its exit value (section 4.3)
        for i in range(I):
            if i in q["walled"]:
                continue
            m = r["m"][i]
            want = self.Nw[i] * F(mp.log1p((q["wages"][i] - m) / (self.sig[i] * Ps + m)), self.chi[i])
            ch[f"supply {i}"] = rel(q["nS"][i], want) if want != 0 else abs(q["nS"][i])
            if self.ex[i] is not None and rr == 1 and i in self.takers and r["enclosure"] is None:
                qq = r["rc"] / r["pg"]
                on_plot = r["branch"][i] == "P"
                assert on_plot == (qq < self.q_enc[i]) or sub["regime"] == "Crowded", ("branch", i)
        # coverage (SSRN eq 16) and the margin
        ch["kappa"] = rel(r["kappa"], rr * self.T / (sum(self.Nw) * Ps)) if rr else r["kappa"]
        if r["margin"] == "Contestable":
            ch["margin"] = rel(w, self.gamma(r["x"]) * q["pi"])
        elif r["margin"] == "Wall":
            assert w >= self.gamma(M(1)) * q["pi"] * (1 - TOL)
        else:
            assert w <= self.gamma(M(0)) * q["pi"] * (1 + TOL)
        if rr == 0:
            assert w == 1 and r["idle"] >= -TOL
            # at r = 0 plots past the commons stand free on idle land: never Enclosed
            assert sub["regime"] != "Enclosed" and sub["rc"] == 0
        bad = {k: v for k, v in ch.items() if v > TOL}
        assert not bad, f"identities fail at {DPS} digits: {bad}"

    # ---------------------------------------------------------------- Lemma 5 (section 5.4)
    def sigma(self, q, i):
        """Lemma 5's sigma_i at an evaluated point with market rent 1 and plot rent rc."""
        e = self.ex[i]
        L_s, B_s, lt_c, bt_c = self.basket_totals(q)
        v, Ps, rc = q["w"], q["Pbase"], q["sub"]["rc"]
        b_g = bt_c[self.g]
        m = q["sub"]["m"][i]
        base = self.sig[i] * B_s * (self.eps[i] * v - m)
        if e is None:
            return self.sig[i] * B_s * self.eps[i] * v
        if q["sub"]["branch"][i] == "P":
            return base + (e["s0"] * b_g - rc * e["h"]) * (self.sig[i] * Ps + self.eps[i] * v)
        return base + e["sf"] * b_g * (self.sig[i] * Ps + self.eps[i] * v)

    def assert_lemma5(self, path, grid=GRID):
        """Lemma 5's sign against a finite difference of each type's supply, at a fixed plot
        rent, at grid points of every piece; f nonincreasing on the grid of a certified economy.
        Returns (checked, negative)."""
        checked = negative = 0
        for piece in path["pieces"]:
            if piece is None or piece[0][0] not in ("AllHuman", "Line", "Wall") or not piece[2] > piece[1]:
                continue
            span, lo, hi = piece
            if hi == INF:
                continue
            fs = []
            for k in range(1, grid):
                s = lo + (hi - lo) * k / grid
                q = self.at_span(span, s)
                fs.append(q["f"])
                if q["sub"]["regime"] == "Crowded":
                    continue
                ds = (hi - lo) * mp.mpf(10) ** -30
                q2 = self.at_span(span, s + ds)
                if q2["sub"]["branch"] != q["sub"]["branch"] or q2["sub"]["regime"] != q["sub"]["regime"]:
                    continue
                # the supply at q2's prices with q's plot rent
                for i in range(self.I):
                    e = self.ex[i]
                    rc = q["sub"]["rc"]
                    pg2 = q2["pbase"][self.g]
                    if e is None:
                        m2 = mp.mpf(0)
                    elif q["sub"]["branch"][i] == "P":
                        m2 = pg2 * e["s0"] - rc * e["h"]
                    else:
                        m2 = pg2 * e["sf"]
                    z1 = mp.log1p((self.eps[i] * q["w"] - q["sub"]["m"][i]) / (self.sig[i] * q["Pbase"] + q["sub"]["m"][i]))
                    z2 = mp.log1p((self.eps[i] * q2["w"] - m2) / (self.sig[i] * q2["Pbase"] + m2))
                    dz = (z2 - z1) * (1 if q2["w"] > q["w"] else -1)
                    sg = self.sigma(q, i)
                    if abs(dz) > mp.mpf(10) ** -45 and abs(sg) > mp.mpf(10) ** -40:
                        assert (dz > 0) == (sg > 0), ("Lemma 5", i, dz, sg)
                        checked += 1
                        negative += sg < 0
            if self.certified:
                for f0, f1 in zip(fs, fs[1:]):
                    assert f1 <= f0 + TOL * (1 + abs(f0)), "a certified economy's f rises"
        return checked, negative


# ------------------------------------------------------------------------------ instances
def goodspace(N="4", T="10", h="1", exits=(None,), parcels=None, supports=None, chi="1", eta="1", lam="0.05",
              a="0.3", b="0.4", g0="0.2", g1="0.8", rho="0", alt=None, exit_good=0):
    """1a's G1 in parcel form (section 3.3): the good (the exit good, unless exit_good is 1,
    space) and space (h), one flow machine, one type (N, chi, 1, support)."""
    sig = supports[0] if supports else "1"
    parcels = parcels or (("LAND", T, "1", "E"),)
    return Economy(parcels=parcels, exits=exits, exit_good=exit_good, alt=alt,
                   workers=(worker("WORKER", N, chi, sig=sig),), eta=eta, g0=g0, g1=g1, k="1", rho=rho,
                   edges=("0", "1"), categories=(("GOOD", "1", "0", ("1",)), ("SPACE", h, "1", ("0",))),
                   intermediate=(("0", "0"), ("0", "0")),
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), ((a,), lam, b), "1", 1),))


def fields(commons=None, recut=False, by_law=False):
    """K's parcels: (FIELDS, 10, 1, enclosed) and WASTE of the given services."""
    p = (("FIELDS", "10", "1", "E"),)
    if recut:
        return p + (("WASTE", "5", "0.2", "O"),)
    if commons is None:
        return p
    return p + (("WASTE", commons, "1", "E" if by_law else "O"),)


def fork(parcels, exits):
    """F: 1c's M4 (the loom, engine and power, rho 0.04) with one type and food as exit good."""
    cats, acc = g1c.fork_categories()
    return Economy(parcels=parcels, exits=exits, exit_good=1, workers=(worker("WORKER", "4", "1"),),
                   eta="1", g0="0.2", g1="0.8", k="1", rho="0.04", edges=("0", "0.4", "0.75", "1"), categories=cats,
                   intermediate=acc, types=g1c.M4_TYPES)


def e6():
    """I3: 1d's E6 in parcel form, the trained short everywhere with the land fully used."""
    ws = (worker("ENTRANT", "8", "1"), worker("TRAINED", "0.5", "0.8", eps="1.5", sig="1.2"))
    return Economy(parcels=(("LAND", "10", "1", "E"),), exits=(None, None), exit_good=1, workers=ws, eta="1",
                   g0="0.2", g1="0.8", k="1", rho="0", edges=("0", "1"),
                   categories=(("SERVICES", "1", "0", ("0.75",)), ("GOODS", "1", "0", ("1",)),
                               ("SPACE", "1", "1", ("0",))),
                   intermediate=(("0",) * 3,) * 3,
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), (("0.3",), "0.05", "0.4"), "1", 1),),
                   required=("0.25", "0", "0"), reserved=(("0", "0.1"), ("0", "0.02"), ("0", "0")))


def parcel_form(e1d, exits=None):
    """E0: a generate_1d Economy in parcel form, one enclosed parcel of T acres of quality 1."""
    types = []
    for k in range(e1d.K):
        types.append(g1c.machine_type(e1d.tn[k], e1d.theta[k], (tuple(e1d.aop[k]), e1d.lop[k], e1d.bop[k]),
                                      (tuple(e1d.aI[k]), e1d.lI[k], e1d.bI[k]), e1d.delta[k], e1d.lag[k]))
    cats = tuple((e1d.names[j], e1d.z[j], e1d.bc[j], tuple(e1d.mu[j])) for j in range(e1d.C))
    ws = tuple(dict(name=e1d.wn[i], N=e1d.Nw[i], chi=e1d.chi[i], eps=e1d.eps[i], sig=e1d.sig[i]) for i in range(e1d.I))
    return Economy(parcels=(("LAND", e1d.T, "1", "E"),), exits=exits or (None,) * e1d.I, exit_good=0, workers=ws,
                   eta=e1d.eta, g0=e1d.g0, g1=e1d.g1, k=e1d.k, rho=e1d.rho, edges=tuple(e1d.e), categories=cats,
                   intermediate=tuple(tuple(row) for row in e1d.Acc), types=tuple(types),
                   required=tuple(e1d.LH), reserved=tuple(tuple(row) for row in e1d.R))


def solved(econ, stretch=None):
    regime, r = econ.solve()
    assert regime in ("Contestable", "Wall", "AllHuman", "Idle"), (regime, r and r.get("changes"))
    if stretch is not None:
        assert regime == stretch, (regime, stretch)
    r["stretch"] = regime
    return r


def assert_nests(e1d, label, exits=None):
    """Section 7: the parcel form of a 1d economy equals generate_1d.py's solve to 1e-65; a
    LaborShort row is an idle-land equilibrium with f_inf 1d's excess."""
    reg_d, qd = e1d.solve()
    reg_e, qe = parcel_form(e1d, exits).solve()
    if reg_d in ("Contestable", "Wall", "AllHuman"):
        assert reg_e == reg_d, (label, reg_d, reg_e)
        pairs = dict(x=(qe["x"], qd["x"]), v=(qe["w"], qd["w"]), Ps=(qe["Ps"], qd["Ps"]), Y=(qe["Y"], qd["Y"]),
                     n=(qe["n_a"], qd["n_a"]), income=(qe["income"], qd["income"]))
        worst = max(abs(a - b) / max(abs(b), mp.mpf(10) ** -300) for a, b in pairs.values())
        assert worst < TOL, (label, worst)
        return worst, reg_d, reg_e, qe
    if reg_d == "LaborShort":
        assert reg_e == "Idle", (label, reg_e)
        assert qe["f_end"] == qd["f_end"] or rel(qe["f_end"], qd["f_end"]) < TOL, label
        return mp.mpf(0), reg_d, reg_e, qe
    assert reg_e == reg_d, (label, reg_d, reg_e)
    return mp.mpf(0), reg_d, reg_e, qe


# ------------------------------------------------------------------------------ output
LINES = []
START = time.time()


def section(title):
    if os.environ.get("GENERATE_1E_PROGRESS"):
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


COMMON = (("X_STAR", "x", "x*"), ("V", "w", "v, the pool's wage"), ("P_S", "Ps", "P_s"),
          ("Y", "Y", "Y = T_m/B^q"), ("N_A", "n_a", "N_a, hours worked"),
          ("PARTICIPATION", "participation", "N_a/N"), ("INCOME", "income", "I = v N_a + r T_m + interest"))


def build():
    worst = mp.mpf(0)
    lemma = [0, 0]

    def lemma5(econ):
        path = econ.path()
        c, n = econ.assert_lemma5(path)
        lemma[0] += c
        lemma[1] += n

    # ------------------------------------------------------------------ P
    section("P: the exit value alone, check_pinning P3 and check_enclosure N-ii, N-iii, N-vi (docs/unit-1e.md section 4.2)")
    p3 = priced("10", "4", "6")
    assert (p3["s0"] - p3["sf"]) / p3["h"] == 1
    vals = [s_of_q(M(q), p3) for q in (mp.mpf(1) / 3, mp.mpf(2) / 3, 1, 2, 5)]
    assert vals == [8, 6, 4, 4, 4] or all(rel(a, b) < TOL for a, b in zip(vals, (8, 6, 4, 4, 4)))
    put("P3_Q_ENC", 1, "check_pinning P3 (10, 4, 6): q_enc = (s0 - s_)/h = 1")
    ne = priced("1.5", "0", "1")
    q_enc = (ne["s0"] - ne["sf"]) / ne["h"]
    n_crit = q_enc * 100 / (1 + q_enc * 1)
    q_star = {N: M(N) / (100 - M(N)) for N in (50, 60, 80)}
    assert q_enc == M("1.5") and n_crit == 60 and q_star[50] == 1 and q_star[80] == 4
    put("P_Q_ENC", q_enc, "check_enclosure (1.5, 0, 1): q_enc = 1.5")
    put("P_N_CRIT", n_crit, "N_crit = q_enc T/(g_s + q_enc h_s) = 60 at T 100, g_s = h_s = 1")
    for N in (50, 60, 80):
        put(f"P_Q_STAR_{N}", q_star[N], f"q* = N g_s/(T - N h_s) at N {N}")
    kappa = (mp.mpf(10) / 9) * 100 / (50 * (1 + mp.mpf(10) / 9))
    assert rel(kappa, mp.mpf(20) / 19) < TOL
    put("P_KAPPA_D3", kappa, "SSRN D.3: kappa(q = 10/9, N 50) = 20/19")
    nv = priced("1.5", "0.1", "1")
    take = lambda q: nv["s0"] - s_of_q(M(q), nv)  # noqa: E731
    assert take(1) == 1 and rel(take(10), M("1.4")) < TOL
    put("P_TAKE_CAP", (nv["s0"] - nv["sf"]), "N-vi (1.5, 0.1, 1): the take caps at h q_enc = 1.4")

    # ------------------------------------------------------------------ Q
    section("Q: the race, check_enclosure N-iii and SSRN D.3 inside equilibria (docs/unit-1e.md section 4.9)")
    EXQ = (priced("1.5", "0", "1"),)
    qres = {}
    for tag, N, eta in (("Q1", "50", "2.5"), ("Q2", "60", "2"), ("Q3", "80", "2"), ("Q4", "80", "1"),
                        ("Q5", "80", "0.7")):
        econ = goodspace(N=N, T="100", exits=EXQ, eta=eta)
        r = solved(econ, "Contestable")
        lemma5(econ)
        qres[tag] = r
        # kappa = q T/(N (1 + q)) exactly in Appendix B's basket
        assert rel(r["kappa"], r["q"] * 100 / (M(N) * (1 + r["q"]))) < TOL
        assert (r["kappa"] >= 1 - TOL) == (r["q"] >= q_star.get(int(N), M(N) / (100 - M(N))) * (1 - TOL))
        keys = COMMON + (("Q", "q", "q = r/p_g"), ("KAPPA", "kappa", "coverage kappa = q T/(N (1 + q))"),
                         ("T_P", "T_p", "enclosed land in rented plots"),
                         ("PROVIDER_BASKETS", "provider", "provider baskets (r T_m + interest)/P_s - nu"),
                         ("P_G", "pg", "the good's price, the exit good"))
        put_all(tag, r, keys)
    for tag in ("Q2", "Q3"):
        enc = qres[tag]["enclosure"]
        assert enc is not None and rel(qres[tag]["q"], M("1.5")) < TOL
        put(f"{tag}_SHARE", enc["psi"], "psi, the renting share of exiters at the enclosure tie")
        put(f"{tag}_F_BELOW", enc["f_below"], "f at the enclosure point, the type on its floor")
        put(f"{tag}_F_ABOVE", enc["f_above"], "f at the enclosure point, the type renting")
    assert qres["Q2"]["kappa"] == 1 or rel(qres["Q2"]["kappa"], 1) < TOL
    assert rel(qres["Q3"]["kappa"], M("0.75")) < TOL
    assert rel(qres["Q2"]["x"], qres["Q3"]["x"]) < TOL and rel(qres["Q2"]["Ps"], mp.mpf(5) / 3) < TOL
    assert all(b == "P" for b in qres["Q1"]["branch"]) and qres["Q4"]["branch"] == ["F"] == qres["Q5"]["branch"]
    assert not any(qres[t]["funded"] for t in ("Q1", "Q2", "Q3", "Q4")) and qres["Q5"]["funded"]
    # Q6: the race with chi_max 3 at eta 1.5 and N 14. The enclosure point lies on the wall, and
    # the economy sits at it: an enclosure tie on the wall, q = q_enc = 1.5 (section 12 item 18).
    econ = goodspace(N="14", T="100", exits=EXQ, eta="1.5", chi="3")
    r = solved(econ, "Wall")
    enc = r["enclosure"]
    assert enc is not None and rel(r["q"], M("1.5")) < TOL and r["branch"] == ["F"]
    assert rel(r["kappa"], r["q"] * 100 / (14 * (1 + r["q"]))) < TOL
    put_all("Q6", r, (("V", "w", "v, on the wall at the enclosure point"), ("P_S", "Ps", "P_s"),
                      ("Y", "Y", "Y = T_m/B^q"), ("N_A", "n_a", "N_a, hours worked"),
                      ("KAPPA", "kappa", "coverage kappa = q T/(N (1 + q))"),
                      ("T_P", "T_p", "enclosed land in rented plots")))
    put("Q6_SHARE", enc["psi"], "psi, the renting share of exiters at the enclosure tie on the wall")
    put("Q6_F_BELOW", enc["f_below"], "f at the wall's enclosure point, the type on its floor")
    put("Q6_F_ABOVE", enc["f_above"], "f at the wall's enclosure point, the type renting")

    # ------------------------------------------------------------------ K
    section("K: the commons, G1's economy with exit (0.5, 0, 0.1) (docs/unit-1e.md section 3.3)")
    EXK = (priced("0.5", "0", "0.1"),)
    kres = {}
    for tag, parcels, regime in (("K1", fields("1"), "Commons"), ("K2", fields("0.3"), "Crowded"),
                                 ("K3", fields(), "Enclosed"), ("K4", fields("1", by_law=True), "Enclosed"),
                                 ("K5", fields(recut=True), "Commons")):
        econ = goodspace(parcels=parcels, exits=EXK)
        r = solved(econ, "Contestable")
        assert r["regime"] == regime, (tag, r["regime"])
        kres[tag] = r
        if tag == "K5":
            assert all(rel(r[k], kres["K1"][k]) < TOL for k in ("x", "w", "Y", "n_a"))
            continue
        lemma5(econ)
        keys = COMMON + (("Q", "q", "q"), ("KAPPA", "kappa", "kappa"), ("P_G", "pg", "p_g"),
                         ("PLOT_RENT", "rc", "r_o, the plot rent"), ("T_OC", "Toc", "the commons occupied"),
                         ("T_P", "T_p", "enclosed land in rented plots"), ("T_M", "T_m", "the market's land"))
        put_all(tag, r, keys)
    assert kres["K2"]["n_a"] == 1 or rel(kres["K2"]["n_a"], 1) < TOL
    for a, b in (("K1", "K2"), ("K2", "K3"), ("K1", "K4")):
        assert kres[b]["participation"] > kres[a]["participation"] and kres[b]["w"] < kres[a]["w"]
    put("K1_WASTE_USED", kres["K1"]["parcels"][1]["used"], "the share of WASTE's services occupied")
    put("K2_EXIT_VALUE", kres["K2"]["m"][0], "e = p_g s0 - r_o h at the shadow rent")
    put("K3_EXIT_GOODS", kres["K3"]["s"][0], "s(q) = s0 - q h at the market rent")

    # ------------------------------------------------------------------ D
    section("D: a dead exit (1, 0, 1), q_enc = 1 below q: G1 (docs/unit-1e.md section 3.3)")
    econ = goodspace(exits=(priced("1", "0", "1"),))
    r = solved(econ, "Contestable")
    g1 = g1d.goodspace().solve()[1]
    assert all(rel(r[a], g1[b]) < TOL for a, b in (("x", "x"), ("w", "w"), ("Y", "Y"), ("n_a", "n_a")))
    assert r["regime"] == "Enclosed" and r["branch"] == ["F"] and r["T_p"] == 0
    put("D_Q", r["q"], "q at G1, above q_enc = 1: every exiter on the floor")

    # ------------------------------------------------------------------ I
    section("I: idle land at zero rent, prices per unit of the pool's wage (docs/unit-1e.md section 4.6)")
    for tag, econ in (("I1", goodspace(N="0.25")), ("I2", goodspace(lam="0.6", chi="3"))):
        r = solved(econ, "Idle")
        base = g1d.goodspace(N="0.25") if tag == "I1" else g1d.goodspace(lam="0.6", chi="3")
        rd, qd = base.solve()
        assert rd == "LaborShort" and rel(r["f_end"], qd["f_end"]) < TOL
        keys = (("Y", "Y", "Y"), ("T_M", "T_m", "the market's land in use"), ("IDLE", "idle", "T_idle"),
                ("N_A", "n_a", "N_a"), ("P_S", "Ps", "P_s in pool wages"),
                ("REAL_WAGE", "real_wage", "v/P_s, 1d's ceiling"), ("F_END", "f_end", "f_inf, 1d's excess"))
        put_all(tag, r, keys)
    econ = e6()
    r = solved(econ, "Idle")
    assert r["edge"] is not None and r["edge"][0] == 1
    rd, qd = g1d.entrant_trained("8", "0.5").solve()
    assert rd == "LaborShort" and qd["f_end"] == INF
    for key, val, note in (("Y", r["Y"], "Y = 25/6: the trained's reserved demand 0.12 Y = N_T"),
                           ("T_M", r["T_m"], "T_m = 20/3"),
                           ("TRAINED_CLEARING", r["edge"][1], "kappa_T, the trained's real wage per support basket"),
                           ("TRAINED_WAGE", r["wages"][1], "the trained's wage, in pool wages"),
                           ("P_S", r["Ps"], "P_s"), ("ENTRANT_HOURS", r["hours"][0], "the entrant's hours, 65/48"),
                           ("PARTICIPATION", r["participation"], "participation")):
        put(f"I3_{key}", val, note)
    econ = goodspace(lam="0.6", chi="3", exits=(priced("0.5", "0", "0.5"),),
                     parcels=(("FIELDS", "5", "1.5", "E"), ("HEATH", "5", "0.5", "E")))
    r = solved(econ, "Idle")
    assert r["parcels"][1]["used"] == 0
    for key, val, note in (("Y", r["Y"], "Y"), ("T_M", r["T_m"], "T_m"), ("T_P", r["T_p"], "free plots on idle land"),
                           ("N_A", r["n_a"], "N_a"), ("PARTICIPATION", r["participation"], "participation"),
                           ("FIELDS_USED", r["parcels"][0]["used"], "FIELDS' share of its 7.5 services used; HEATH idles"),
                           ("EXIT_VALUE", r["m"][0], "e = p_g s0 in pool wages, the plot free")):
        put(f"I4_{key}", val, note)
    reg, res = goodspace(lam="0.6", chi="3", exits=(priced("2", "0", "0.5"),),
                         parcels=(("FIELDS", "5", "1.5", "E"), ("HEATH", "5", "0.5", "E"))).solve()
    assert reg == "NoMarket"
    put("I5_F_END", res["f_end"], "NoMarket: I4 with exit (2, 0, 0.5), f at the end of the wall")

    # ------------------------------------------------------------------ L
    section("L: an exit good made of land alone, free at r = 0 (docs/unit-1e.md section 12 item 16)")
    # L1: W3 with N 5 and exit (3.2, 0, 1.8) in space. On the wall q = 1/b_space = 1 < q_enc = 16/9 and
    # the type rents; at r = 0 space is free, and the plots are decided at the wall's end, so the
    # idle stretch starts where the wall ends: f_inf is the wall's limit, and the one change of
    # side is on the wall.
    econ = goodspace(N="5", lam="0.6", chi="3", exits=(priced("3.2", "0", "1.8"),), exit_good=1)
    r = solved(econ, "Wall")
    assert r["regime"] == "Enclosed" and r["branch"] == ["P"] and r["q"] == 1
    path = econ.path()
    last = next(pc for pc in reversed(path["pieces"]) if pc is not None and pc[0][0] == "Wall")
    near_end = econ.at_span(last[0], last[2] * (1 - mp.mpf(10) ** -40))
    assert near_end["sub"]["regime"] == "Enclosed" and rel(near_end["f"], path["f_end"]) < mp.mpf(10) ** -30
    at_end = econ.ev(1, path["te"], w=M(1), r=0, land=path["t_inf"])
    assert at_end["sub"]["regime"] == "Idle" and at_end["sub"]["branch"] == ["P"] and at_end["wall"] == 1
    assert path["f_end"] < 0 < r["f_line_1"]
    for key, val, note in (("V", r["w"], "v, on the wall"), ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"),
                           ("N_A", r["n_a"], "N_a"), ("T_P", r["T_p"], "enclosed land in rented plots"),
                           ("F_END", path["f_end"], "f_inf with the plots on idle land: the wall's limit")):
        put(f"L1_{key}", val, note)
    # L2: W3 with exit (0.5, 0, 1) in space: q = 1 > q_enc = 0.5 on the wall and at its end, so
    # every exiter stands on the floor s_ = 0 there and on the idle stretch: I2 (1d's W3).
    econ = goodspace(lam="0.6", chi="3", exits=(priced("0.5", "0", "1"),), exit_good=1)
    r = solved(econ, "Idle")
    rd, qd = g1d.goodspace(lam="0.6", chi="3").solve()
    assert rd == "LaborShort" and rel(r["f_end"], qd["f_end"]) < TOL
    assert r["branch"] == ["F"] and r["T_p"] == 0 and r["q"] == 1 and r["regime"] == "Idle"

    # ------------------------------------------------------------------ T
    section("T: two priced types share a commons (docs/unit-1e.md section 3.3)")
    ws = (worker("ENTRANT", "4", "1"), worker("TRAINED", "1", "0.8", eps="1.5", sig="1.2"))
    econ = Economy(parcels=fields("0.34"), exits=(priced("0.5", "0", "0.1"), priced("1.2", "0", "0.05")),
                   exit_good=0, workers=ws, eta="1", g0="0.2", g1="0.8", k="1", rho="0", edges=("0", "1"),
                   categories=(("GOOD", "1", "0", ("1",)), ("SPACE", "1", "1", ("0",))),
                   intermediate=(("0", "0"), ("0", "0")),
                   types=(g1c.machine_type("M", "1", g1c.zero_recipe(1), (("0.3",), "0.05", "0.4"), "1", 1),))
    r = solved(econ, "Contestable")
    assert r["regime"] == "Crowded" and rel(sum(r["plots"]), M("0.34")) < TOL
    for key, val, note in (("X_STAR", r["x"], "x*"), ("V", r["w"], "v"), ("P_S", r["Ps"], "P_s"), ("Y", r["Y"], "Y"),
                           ("N_POOL", r["n_pool"], "the pool's efficiency hours"),
                           ("ENTRANT_HOURS", r["hours"][0], "the entrant's hours"),
                           ("TRAINED_HOURS", r["hours"][1], "the trained's hours"),
                           ("ENTRANT_EXITERS", r["E"][0], "the entrant's exiters"),
                           ("TRAINED_EXITERS", r["E"][1], "the trained's exiters"),
                           ("PLOT_RENT", r["rc"], "r_o, one shadow rent for both")):
        put(f"T_{key}", val, note)

    # ------------------------------------------------------------------ F
    section("F: 1c's M4 with a priced exit in food (0.1, 0, 0.05), rho 0.04 (docs/unit-1e.md section 3.3)")
    for tag, parcels, regime in (("F1", fields("1"), "Commons"), ("F2", fields(), "Enclosed")):
        econ = fork(parcels, (priced("0.1", "0", "0.05"),))
        assert all(econ.ex[0]["h"] <= econ.ex[0]["s0"] * econ.bbar_g for _ in (0,))
        r = solved(econ, "Contestable")
        assert r["regime"] == regime and r["t"] == 1
        lemma5(econ)
        keys = COMMON + (("P_FOOD", "pg", "p_food, through the chain"),
                         ("T_OC", "Toc", "the commons occupied"), ("T_P", "T_p", "rented plots"))
        put_all(tag, r, keys)

    # ------------------------------------------------------------------ M
    section("M: three equilibria, two inside one region of the line (docs/unit-1e.md section 3.3)")
    econ = goodspace(N="7", T="8.6", chi="0.67", a="0.075", lam="0.063", b="0.88", eta="1.16", g0="0.41",
                     g1="1.88", h="1.24", exits=(priced("1.1", "0.17", "0.36"),), supports=("0.1",))
    reg, res = econ.solve()
    assert reg == "MultipleEquilibria" and len(res["changes"]) == 3
    roots = []
    for (a, b) in res["changes"]:
        k = b[1]
        span = res["path"]["pieces"][k][0]
        lo = a[2] if a[0] == "scan" else res["path"]["pieces"][k][1]
        hi = b[2] if b[0] == "scan" else res["path"]["pieces"][k][2]
        positive = econ.at_span(span, lo)["f"] > 0
        for _ in range(HALVINGS):
            mid = (lo + hi) / 2
            if (econ.at_span(span, mid)["f"] > 0) == positive:
                lo = mid
            else:
                hi = mid
        s = (lo + hi) / 2
        q = econ.at_span(span, s)
        roots.append((span[0], s, q["w"]))
    assert [r_[0] for r_ in roots] == ["AllHuman", "Line", "Line"], roots
    put("M_V_CORNER", roots[0][2], "the all-human corner's equilibrium wage")
    put("M_X_LOW", roots[1][1], "the lower equilibrium on the line")
    put("M_V_LOW", roots[1][2], "its wage")
    put("M_X_HIGH", roots[2][1], "the upper equilibrium on the line")
    put("M_V_HIGH", roots[2][2], "its wage")
    put("M_F_LINE_1", res["path"]["one"]["f"], "f on the line at 1")
    assert rel(res["path"]["at0"]["f"], mp.mpf(-2) / 31) < TOL
    put("M_F_LINE_0", res["path"]["at0"]["f"], "f on the line at 0, -2/31: the line's ends have the same sign")
    # with the scan off the count is one
    assert len(econ.count(res["path"], 0)) == 1

    # ------------------------------------------------------------------ A
    section("A: the wrong units of docs/unit-1e.md section 3.3")
    r = solved(goodspace(parcels=fields("1"), exits=EXK, alt="commons_rent"))
    for key, val, note in (("X_STAR", r["x"], "x* with the market rent charged on commons plots (K1)"),
                           ("V", r["w"], "its v"), ("PARTICIPATION", r["participation"], "its participation")):
        put(f"A1_{key}", val, note)
    r2 = solved(goodspace(exits=EXK, alt="no_spill"))
    assert rel(r2["x"], r["x"]) < TOL
    put("A2_Y", r2["Y"], "Y with rented plots left in production (K3)")
    r = solved(goodspace(N="50", T="100", exits=EXQ, eta="2.5", alt="no_spill"))
    put("A3_X_STAR", r["x"], "x* of Q1 with rented plots left in production")
    put("A3_Y", r["Y"], "its Y")
    r = solved(goodspace(N="80", T="100", exits=EXQ, eta="2", alt="basket_q"), "Wall")
    assert rel(r["participation"], mp.mpf(3) / 224) < TOL
    put("A4_PARTICIPATION", r["participation"], "Q3 with q = r/P_s: the wall, 3/224")
    put("A4_Y", r["Y"], "its Y")

    # ------------------------------------------------------------------ nesting (section 7)
    for label, e1d in (("W1", g1d.goodspace(lam="0.6")), ("W4", g1d.goodspace(N="20", chi="0.05")),
                       ("W2", g1d.goodspace(N="0.25")), ("W3", g1d.goodspace(lam="0.6", chi="3")),
                       ("G1", g1d.goodspace()), ("B1", g1d.baumol(N="8")), ("B2", g1d.baumol(N="4")),
                       ("E1", g1d.entrant_trained("8", "3")), ("E3", g1d.entrant_trained("8", "2", "0.3")),
                       ("E5", g1d.entrant_trained("40", "2", "1", "0.05")), ("E6", g1d.entrant_trained("8", "0.5")),
                       ("E7", g1d.three_types("3", "0.6")), ("F1d", g1d.full("4", "3", "1")),
                       ("X1", g1d.wall_switch_economy("0.5")), ("J1", g1d.entrant_trained("40", "1", "1", "0.05"))):
        w_, _, _, _ = assert_nests(e1d, label)
        worst = max(worst, w_)
    # the exit option switched off is the dependence form, wherever there are no reserved hours
    for label, e1d in (("G1", g1d.goodspace()), ("W1", g1d.goodspace(lam="0.6")), ("W2", g1d.goodspace(N="0.25")),
                       ("B1", g1d.baumol(N="8"))):
        for h in ("0", "1"):
            w_, _, _, _ = assert_nests(e1d, f"{label} off h {h}", exits=(priced("0", "0", h),) * e1d.I)
            worst = max(worst, w_)

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens_1e.txt: the golden numbers for oracle unit 1e.",
        f"# Written by goldens/generate_1e.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits.",
        "# Equations: docs/unit-1e.md section 4. laborformal references are at 31b3482.",
        f"# The parcel form nests generate_1d.py's solves within {mp.nstr(worst, 3)} (1e-65 asserted).",
        f"# Lemma 5's sign checked at {lemma[0]} points against a finite difference, {lemma[1]} of them negative.",
        f"# fnv1a64 generate_1e.py = {fnv1a64(source_bytes(__file__)):016x}",
        f"# fnv1a64 generate_1d.py = {fnv1a64(source_bytes(g1d.__file__)):016x}",
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
                        help="compare with goldens_1e.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(HERE, "goldens_1e.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same = fh.read() == text
        print("goldens_1e.txt is current" if same else "goldens_1e.txt differs from generate_1e.py's output")
        return 0 if same else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
