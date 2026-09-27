"""generate_1c.py: the golden numbers for oracle unit 1c.

Dated 2026-09-27. Every golden that crates/oracle's unit-1c tests use is computed here with
mpmath at 70 digits, from the equations of docs/unit-1c.md section 4: the full cost system of
SSRN 7226858 over categories and machine types (eq 2-7, 10-13, 19, A.1, A.4 and Appendix C),
main.tex:688-690 at laborformal 31b3482, and the two-recipe machine row of
dynamics/checks/check_dynamics.py (R1-R6, :93-124; U1-U6; L1-L4) at the same commit, with
its own Leontief solves (mp.lu_solve), the technique envelope and the switch points in closed
form, bisection to 2^-250 in each technique region, and the tie's share in closed form.
Nothing is imported from laborformal or from the oracle. The unit-1b generator
(generate_1b.py, and through it generate.py) is imported only to assert that the one-type
form nests it (docs/unit-1c.md section 7).

Run with any Python that has mpmath (1.3.0 was used), from this directory or any other:

    python goldens/generate_1c.py           # writes goldens/goldens_1c.txt beside this file
    python goldens/generate_1c.py --check   # exits 1 if goldens_1c.txt is not what this writes

Every value goes out with 30 significant digits. The Rust constants in
tests/gate/goldens_1c.rs are these values rounded to 20 significant digits, and the test
m8_goldens_file::constants_match_goldens_1c_txt enforces that.

The header of goldens_1c.txt records four FNV-1a 64-bit digests: of this file, of
generate_1b.py and generate.py (whose solves the nesting assertions use) and of the goldens
that follow the header, each with CRLF read as LF. The gate recomputes all four.
"""

import argparse
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import mpmath as mp  # noqa: E402

import generate_1b as g1b  # noqa: E402  (imports generate.py; both set mp.mp.dps to 70)

g1a = g1b.g1a

DPS = 70
"""Working precision in decimal digits, as in generate.py and generate_1b.py."""
SIG_OUT = 30
"""Significant digits written per value; the Rust constants keep 20 of them."""
BISECTIONS = 250
"""Halvings of a region of [1e-12, 1]: 2^-250 < 1e-75, below the working precision."""
IDENTITY_TOL = mp.mpf(10) ** -(DPS - 5)
"""An identity counts as exact at 70 digits when it holds to 1e-65."""
BRACKET_LO = "1e-12"
"""Left end of the root bracket, as in the oracle and macro.py:102."""
SCAN = 10 ** 4
"""Points of the task line on which the envelope is checked against argmin v_t."""
REGION_GRID = 40
"""Points per technique region on which f is asserted nonincreasing."""
JSON_TOL = mp.mpf("2e-15")
"""check_dynamics' targets are written as doubles; they must match the 70-digit values to
this relative error (docs/unit-1c.md section 7)."""

mp.mp.dps = DPS
assert g1b.DPS == DPS and g1b.BRACKET_LO == BRACKET_LO


def M(x):
    """An mpf from a decimal string, an int or a Python float (exactly)."""
    return mp.mpf(x)


def rel(a, b):
    """|a - b| relative to the larger magnitude, 0 when both are 0."""
    scale = max(abs(a), abs(b))
    return abs(a - b) / scale if scale != 0 else mp.mpf(0)


def eliminate(G, r):
    """Gaussian elimination without pivoting in index order (docs/unit-1c.md section 5.1).
    Returns (pivots, solution). Stops at the first pivot that is not positive, with solution
    None: past it the elimination divides by it and means nothing."""
    n = len(G)
    G = [row[:] for row in G]
    r = r[:]
    for k in range(n):
        if not G[k][k] > 0:
            return [G[i][i] for i in range(k + 1)], None
        for i in range(k + 1, n):
            m = G[i][k] / G[k][k]
            for j in range(k + 1, n):
                G[i][j] -= m * G[k][j]
            r[i] -= m * r[k]
    z = [mp.mpf(0)] * n
    for i in reversed(range(n)):
        s = r[i]
        for j in range(i + 1, n):
            s -= G[i][j] * z[j]
        z[i] = s / G[i][i]
    return [G[i][i] for i in range(n)], z


def least_pivot(pivots):
    """The first pivot that is not positive, or else the smallest (the oracle's d)."""
    for p in pivots:
        if not p > 0:
            return p
    return min(pivots)


def leontief(A, rhs):
    """(I - A)^-1 rhs by mp.lu_solve."""
    n = len(rhs)
    return list(mp.lu_solve(mp.eye(n) - mp.matrix(A), mp.matrix(rhs)))


def transpose(A):
    return [list(row) for row in zip(*A)]


def is_m_matrix(A):
    """I - A for A >= 0 is a nonsingular M-matrix, spectral radius of A below 1, exactly when
    every pivot of Gaussian elimination without pivoting is positive."""
    n = len(A)
    G = [[(1 if i == j else 0) - A[i][j] for j in range(n)] for i in range(n)]
    pivots, _ = eliminate(G, [mp.mpf(0)] * n)
    return len(pivots) == n and all(p > 0 for p in pivots)


def machine_type(name, theta, op, build, delta, lag):
    """A machine type: op and build are ((a_k1, ..., a_kK), lambda, b) as decimal strings."""
    return dict(name=name, theta=theta, op=op, build=build, delta=delta, J=lag)


def zero_recipe(K):
    return (("0",) * K, "0", "0")


class Economy:
    """Categories on one task line with intermediate inputs, and K machine types with an
    operating and a build recipe each (docs/unit-1c.md sections 2-4)."""

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
        # the machine types (section 3.1)
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
        # section 4.0: user cost and wealth per type
        self.u = [(self.rho + d) * (1 + self.rho) ** (j - 1) for d, j in zip(self.delta, self.lag)]
        self.omega = [(1 + self.rho) ** (j - 1) + d * sum((1 + self.rho) ** i for i in range(j - 1))
                      for d, j in zip(self.delta, self.lag)]
        for u, d, w, j in zip(self.u, self.delta, self.omega, self.lag):
            assert rel(u - d, self.rho * w) < IDENTITY_TOL or u == d  # L1-L2
            assert rel(u * (1 + self.rho) / (self.rho + d), (1 + self.rho) ** j) < IDENTITY_TOL  # L3
        # section 3.2: validation
        physical = [[self.aop[i][l] + self.aI[i][l] for l in range(K)] for i in range(K)]
        assert is_m_matrix(physical), "machine recipes are not productive"
        reach = leontief(physical, [self.bop[i] + self.bI[i] for i in range(K)])
        assert all(r > 0 for r in reach), "a machine type's chain reaches no land"
        assert any(t > 0 for t in self.theta)
        assert is_m_matrix(self.Acc), "intermediate inputs are not productive"
        # the price side (section 4.2) and the clearing side (section 4.5)
        self.Ahat = [[self.aop[i][l] + self.u[i] * self.aI[i][l] for l in range(K)] for i in range(K)]
        self.lhat = [self.lop[i] + self.u[i] * self.lI[i] for i in range(K)]
        self.bhat = [self.bop[i] + self.u[i] * self.bI[i] for i in range(K)]
        self.Aq = [[self.aop[i][l] + self.delta[i] * self.aI[i][l] for l in range(K)] for i in range(K)]
        self.lq = [self.lop[i] + self.delta[i] * self.lI[i] for i in range(K)]
        self.bq = [self.bop[i] + self.delta[i] * self.bI[i] for i in range(K)]
        if is_m_matrix(self.Ahat):
            self.lt = leontief(self.Ahat, self.lhat)
            self.bt = leontief(self.Ahat, self.bhat)
        else:
            self.lt = self.bt = None
        self.ltq = leontief(self.Aq, self.lq)
        self.btq = leontief(self.Aq, self.bq)
        # the category chain (section 4.4)
        self.yhat = leontief(transpose(self.Acc), self.z)
        self.Lbar_dir = [sum(m * (hi - lo) for m, lo, hi in zip(mu, self.e, self.e[1:])) for mu in self.mu]
        self.Lbar = leontief(self.Acc, self.Lbar_dir)
        self.bbar = leontief(self.Acc, self.bc)
        assert all(L > 0 or b > 0 for L, b in zip(self.Lbar, self.bbar))
        self.B_y = sum(y * b for y, b in zip(self.yhat, self.bc))
        self.chain_hours = sum(y * L for y, L in zip(self.yhat, self.Lbar_dir))
        assert self.B_y > 0 and self.chain_hours > 0
        # B_y = z' bbar (section 4.4)
        assert rel(self.B_y, sum(z * b for z, b in zip(self.z, self.bbar))) < IDENTITY_TOL

    # -------------------------------------------------------------- the task line
    def gamma(self, x):
        return self.eta * (self.g0 + self.g1 * x ** self.k)

    def J(self, x):
        return self.eta * (self.g0 * x + self.g1 * x ** (self.k + 1) / (self.k + 1))

    def gamma_inv(self, g):
        """x with gamma(x) = g, in closed form (only the generator inverts the schedule)."""
        return ((g / self.eta - self.g0) / self.g1) ** (1 / self.k)

    def tasks(self, j, x):
        """(H_j, M_j) at threshold x: section 4.1 (1b section 4.1)."""
        H = Mm = mp.mpf(0)
        for m, lo, hi in zip(self.mu[j], self.e, self.e[1:]):
            if x >= hi:
                Mm += m * (self.J(hi) - self.J(lo))
            elif x <= lo:
                H += m * (hi - lo)
            else:
                H += m * (hi - x)
                Mm += m * (self.J(x) - self.J(lo))
        return H, Mm

    # -------------------------------------------------------------- the machine block
    def closure_wage(self, t, g):
        """v_t(gamma) = gamma bt_t/(theta_t - gamma lt_t), None where t does no tasks or is
        not viable (section 4.3)."""
        if self.lt is None or self.theta[t] == 0:
            return None
        room = self.theta[t] - g * self.lt[t]
        return g * self.bt[t] / room if room > 0 else None

    def block(self, g, t):
        """Section 5.1 step 2: the (O, V) system at gamma with technique t, by elimination
        without pivoting, with its pivots. Asserts the closed forms of section 4.2."""
        K, u, th = self.K, self.u, self.theta[t]
        n = 2 * K
        G = [[mp.mpf(0)] * n for _ in range(n)]
        r = [mp.mpf(0)] * n
        for i in range(K):
            for l in range(K):
                c = 1 if l == t else 0
                G[i][l] = (1 if i == l else 0) - self.aop[i][l] - c * self.lop[i] * g / th
                G[i][K + l] = -self.aop[i][l] * u[l] - c * u[t] * self.lop[i] * g / th
                G[K + i][l] = -self.aI[i][l] - c * self.lI[i] * g / th
                G[K + i][K + l] = (1 if i == l else 0) - self.aI[i][l] * u[l] - c * u[t] * self.lI[i] * g / th
            r[i], r[K + i] = self.bop[i], self.bI[i]
        pivots, z = eliminate(G, r)
        d = least_pivot(pivots)
        if z is None:
            return dict(d=d, pivots=pivots, viable=False)
        O, V = z[:K], z[K:]
        p = [O[i] + u[i] * V[i] for i in range(K)]
        v = g * p[t] / th
        out = dict(d=d, pivots=pivots, viable=True, O=O, V=V, p=p, v=v, pi=p[t] / th)
        # the closed forms: p = v lt + bt, and v is t's closure wage (SSRN eq 4, A.1)
        assert self.lt is not None
        for i in range(K):
            assert rel(p[i], v * self.lt[i] + self.bt[i]) < IDENTITY_TOL, ("p = v lt + bt", i)
            assert rel(O[i], sum(self.aop[i][l] * p[l] for l in range(K)) + self.lop[i] * v + self.bop[i]) < IDENTITY_TOL
            assert rel(V[i], sum(self.aI[i][l] * p[l] for l in range(K)) + self.lI[i] * v + self.bI[i]) < IDENTITY_TOL
        assert rel(v, self.closure_wage(t, g)) < IDENTITY_TOL
        return out

    def envelope(self):
        """Section 5.2: the technique at gamma(1e-12) and each switch after it, in closed form,
        with the tie-breaks of the section (the one cheaper just above, then the lower index)."""
        tasks = [t for t in range(self.K) if self.theta[t] > 0]
        glo, ghi = self.gamma(M(BRACKET_LO)), self.gamma(M(1))
        if self.lt is None:
            return tasks[0], []
        flatter = lambda a, b: self.lt[a] * self.theta[b] < self.lt[b] * self.theta[a]  # noqa: E731
        first = None
        for t in tasks:
            v = self.closure_wage(t, glo)
            if v is None:
                continue
            if first is None or v < first[1] or (v == first[1] and flatter(t, first[0])):
                first = (t, v)
        if first is None:
            return tasks[0], []
        cur, g, visited, switches = first[0], glo, {first[0]}, []
        while True:
            best = None
            for l in tasks:
                if l in visited:
                    continue
                delta = self.bt[l] * self.lt[cur] - self.bt[cur] * self.lt[l]
                if not delta > 0:
                    continue
                gx = (self.bt[l] * self.theta[cur] - self.bt[cur] * self.theta[l]) / delta
                if not (g < gx <= ghi and self.theta[l] - gx * self.lt[l] > 0):
                    continue
                if best is None or gx < best[1] or (gx == best[1] and flatter(l, best[0])):
                    best = (l, gx)
            if best is None:
                break
            switches.append((best[1], cur, best[0]))
            visited.add(best[0])
            cur, g = best[0], best[1]
        return first[0], switches

    def technique_at(self, x, env):
        first, switches = env
        t, g = first, self.gamma(x)
        for gs, _, above in switches:
            if g >= gs:
                t = above
        return t

    def assert_envelope(self, env):
        """The envelope equals argmin v_t on SCAN points of the line (section 7); no type
        repeats (Lemma 1)."""
        first, switches = env
        seq = [first] + [s[2] for s in switches]
        assert len(seq) == len(set(seq))
        for i in range(SCAN + 1):
            x = M(BRACKET_LO) + (1 - M(BRACKET_LO)) * i / SCAN
            g = self.gamma(x)
            wages = [(self.closure_wage(t, g), t) for t in range(self.K)]
            wages = [(w, t) for w, t in wages if w is not None]
            if not wages:
                continue
            least = min(w for w, _ in wages)
            t = self.technique_at(x, env)
            w = self.closure_wage(t, g)
            assert w is not None and rel(w, least) < IDENTITY_TOL, (x, t, wages)

    # -------------------------------------------------------------- one evaluation
    def quantities(self, My, split):
        """Section 4.5 per basket: task services split [(type, share)], gross services xhat =
        (I - Aq')^-1 t, land and hours per basket."""
        task = [mp.mpf(0)] * self.K
        for t, share in split:
            task[t] += share * My / self.theta[t]
        xhat = leontief(transpose(self.Aq), task)
        land = self.B_y + sum(b * x for b, x in zip(self.bq, xhat))
        hours = sum(l * x for l, x in zip(self.lq, xhat))
        return dict(task=task, xhat=xhat, land=land, hours=hours)

    def at(self, x, t, split=None):
        """Sections 4.2-4.5 at threshold x with technique t (prices), and the task services
        split as given (by default all to t)."""
        x = M(x)
        g, Jx = self.gamma(x), self.J(x)
        b = self.block(g, t)
        if not b["viable"]:
            return dict(x=x, gamma=g, d=b["d"], viable=False)
        v, pi = b["v"], b["pi"]
        HM = [self.tasks(j, x) for j in range(self.C)]
        H = [h for h, _ in HM]
        Mt = [m for _, m in HM]
        pc = leontief(self.Acc, [v * h + pi * m + bj for h, m, bj in zip(H, Mt, self.bc)])
        Ps = sum(z * p for z, p in zip(self.z, pc))
        Hy = sum(y * h for y, h in zip(self.yhat, H))
        My = sum(y * m for y, m in zip(self.yhat, Mt))
        qty = self.quantities(My, split or [(t, 1)])
        Y = self.T / qty["land"]
        nD = Y * (Hy + qty["hours"])
        nS = self.N * min(max(mp.log1p(v / Ps) / self.chi_max, 0), 1)
        return dict(x=x, gamma=g, J=Jx, t=t, d=b["d"], viable=True, block=b, v=v, pi=pi, H=H,
                    Mt=Mt, pc=pc, Ps=Ps, Hy=Hy, My=My, qty=qty, Y=Y, nD=nD, nS=nS, f=nD - nS)

    # -------------------------------------------------------------- the solve
    def solve(self):
        """Section 5.3: the regime tests, the switch points, the sign sequence, and a root in
        a region or a tie at a switch. Returns (regime, values)."""
        env = self.envelope()
        first, switches = env
        techs = [first] + [s[2] for s in switches]
        one = self.at(1, techs[-1])
        if not one["viable"]:
            return "NotViable", dict(d_at_1=one["d"], env=env)
        if one["f"] >= 0:
            return "BoundaryNoMargin", dict(f_at_1=one["f"], env=env)
        lo = self.at(BRACKET_LO, first)
        if lo["f"] <= 0:
            return "NoInteriorAtZero", dict(f_at_0=lo["f"], env=env)
        self.assert_envelope(env)
        xs = [self.gamma_inv(g) for g, _, _ in switches]
        bounds = [M(BRACKET_LO)] + xs + [M(1)]
        seq = []
        for i, t in enumerate(techs):
            seq.append(lo["f"] if i == 0 else self.at(bounds[i], t)["f"])
            seq.append(one["f"] if i == len(techs) - 1 else self.at(bounds[i + 1], t)["f"])
        changes = [i for i in range(len(seq) - 1) if (seq[i] > 0) != (seq[i + 1] > 0)]
        # f is nonincreasing within each region (Lemma 2)
        for i, t in enumerate(techs):
            a, b = bounds[i], bounds[i + 1]
            fs = [self.at(a + (b - a) * s / REGION_GRID, t)["f"] for s in range(REGION_GRID + 1)]
            assert all(f1 <= f0 * (1 + IDENTITY_TOL) + IDENTITY_TOL for f0, f1 in zip(fs, fs[1:])), (i, t)
        base = dict(env=env, xs=xs, seq=seq, changes=len(changes), techs=techs, one=one, lo=lo)
        if len(changes) != 1:
            return "MultipleEquilibria", base
        c = changes[0]
        r = c // 2
        if c % 2 == 0:
            t = techs[r]
            a, b = bounds[r], bounds[r + 1]
            for _ in range(BISECTIONS):
                mid = (a + b) / 2
                if self.at(mid, t)["f"] > 0:
                    a = mid
                else:
                    b = mid
            q = self.report((a + b) / 2, t, None, base)
        else:
            below, above = techs[r], techs[r + 1]
            x = bounds[r + 1]
            share = self.tie_share(x, below, above)
            q = self.report(x, below, (above, share, switches[r][0]), base)
        return ("Tie" if q["tie"] else "Interior"), q

    def tie_share(self, x, a, b):
        """Section 4.7: sigma = B_a f_a/(B_a f_a - B_b f_b), with f_t = T L_t/B_t - n_S."""
        qa, qb = self.at(x, a), self.at(x, b)
        # at the switch the two techniques' prices are equal, so n_S is too
        assert rel(qa["v"], qb["v"]) < IDENTITY_TOL and rel(qa["nS"], qb["nS"]) < IDENTITY_TOL
        assert rel(qa["pi"], qb["pi"]) < IDENTITY_TOL  # the two delivered costs are equal
        Ba, Bb = qa["qty"]["land"], qb["qty"]["land"]
        fa, fb = qa["nD"] - qa["nS"], qb["nD"] - qa["nS"]
        assert fa > 0 >= fb
        return Ba * fa / (Ba * fa - Bb * fb)

    def report(self, x, t, tie, base):
        """Sections 4.4-4.8 at x* with technique t (at a tie, the type below, with (above,
        share, gamma)): every output, and every identity and bound asserted to 1e-65."""
        split = [(t, 1)] if tie is None else [(t, 1 - tie[1]), (tie[0], tie[1])]
        q = self.at(x, t, split)
        K, C = self.K, self.C
        b, v, g, Y = q["block"], q["v"], q["gamma"], q["Y"]
        p_m, O, V = b["p"], b["O"], b["V"]
        X = [Y * xx for xx in q["qty"]["xhat"]]
        task = [Y * tt for tt in q["qty"]["task"]]
        N_a = q["nD"]
        interest = sum(self.rho * w * Vk * Xk for w, Vk, Xk in zip(self.omega, V, X))
        income = v * N_a + self.T + interest
        provider = (self.T + interest) / q["Ps"] - self.N
        # the chain totals (section 4.4): per unit of task services, through the split of the
        # machine tasks (all to t, or the tie's; the two delivered costs are then equal, so
        # either decomposition of the price holds, and the split's keeps both sides alike)
        c_l = sum(s * self.lt[tt] / self.theta[tt] for tt, s in split)
        c_b = sum(s * self.bt[tt] / self.theta[tt] for tt, s in split)
        cq_l = sum(s * self.ltq[tt] / self.theta[tt] for tt, s in split)
        cq_b = sum(s * self.btq[tt] / self.theta[tt] for tt, s in split)
        Lstar = leontief(self.Acc, [h + m / g for h, m in zip(q["H"], q["Mt"])])
        lt_c = leontief(self.Acc, [h + m * c_l for h, m in zip(q["H"], q["Mt"])])
        bt_c = leontief(self.Acc, [bj + m * c_b for bj, m in zip(self.bc, q["Mt"])])
        lq_c = leontief(self.Acc, [h + m * cq_l for h, m in zip(q["H"], q["Mt"])])
        bq_c = leontief(self.Acc, [bj + m * cq_b for bj, m in zip(self.bc, q["Mt"])])
        pc = q["pc"]
        cats = []
        for j in range(C):
            c = dict(p=pc[j], real_wage=v / pc[j], L_star=Lstar[j], Lbar=self.Lbar[j], bbar=self.bbar[j],
                     lambda_tilde=lt_c[j], b_tilde=bt_c[j], lambda_q=lq_c[j], b_q=bq_c[j],
                     H=q["H"][j], M=q["Mt"][j], yhat=self.yhat[j], output=self.z[j] * Y,
                     gross=self.yhat[j] * Y, wage_floor=1 / (self.Lbar[j] + self.bbar[j] / v))
            c["wage_ceiling"] = v / bt_c[j] if bt_c[j] > 0 else None
            cats.append(c)
        types = []
        for k in range(K):
            types.append(dict(name=self.tn[k], p=p_m[k], O=O[k], V=V[k], u=self.u[k], omega=self.omega[k],
                              lt=self.lt[k], bt=self.bt[k], ltq=self.ltq[k], btq=self.btq[k],
                              X=X[k], task=task[k], W=self.omega[k] * V[k] * X[k],
                              interest=self.rho * self.omega[k] * V[k] * X[k],
                              closure_wage=self.closure_wage(k, g)))
        dot = lambda key: sum(z * c[key] for z, c in zip(self.z, cats))  # noqa: E731
        out = dict(base)
        out.update(x=q["x"], one_minus_x=1 - q["x"], gamma=g, J=q["J"], t=t, d=q["d"], v=v,
                   Ps=q["Ps"], Y=Y, N_a=N_a, final_hours=Y * q["Hy"], machine_hours=N_a - Y * q["Hy"],
                   income=income, interest=interest, labor_share=v * N_a / income,
                   capital_share=interest / income, real_wage=v / q["Ps"], provider=provider,
                   worker_baskets=self.N + v * N_a / q["Ps"], types=types, cats=cats, X=X,
                   tie=None if tie is None else dict(above=tie[0], share=tie[1], gamma=tie[2]),
                   L_s=dot("lambda_tilde"), B_s=dot("b_tilde"), L_q=dot("lambda_q"), B_q=dot("b_q"),
                   L_star_s=dot("L_star"), My=q["My"], Hy=q["Hy"], q=q)
        self.assert_identities(out, q, split)
        return out

    def assert_identities(self, r, q, split):
        """Every identity of sections 4.4-4.7 at 1e-65, and every bound (section 7)."""
        K, C, tol = self.K, self.C, IDENTITY_TOL
        v, Y, g = r["v"], r["Y"], r["gamma"]
        b = q["block"]
        p_m, V, X = b["p"], b["V"], r["X"]
        t = r["t"]
        checks = {}
        # both fork forms, the bounds and the pair (section 4.4)
        for j, c in enumerate(r["cats"]):
            checks[f"fork_direct_{j}"] = rel(c["p"], v * c["L_star"] + c["bbar"])
            checks[f"fork_totals_{j}"] = rel(c["p"], v * c["lambda_tilde"] + c["b_tilde"])
            s = 1 + tol
            assert c["bbar"] <= c["b_q"] * s and c["b_q"] <= c["b_tilde"] * s, ("chain", j)
            assert c["b_tilde"] <= c["p"] * s and c["p"] <= (v * c["Lbar"] + c["bbar"]) * s, ("chain", j)
            assert c["L_star"] <= c["Lbar"] * s and c["lambda_q"] <= c["lambda_tilde"] * s, ("L", j)
            assert c["wage_floor"] <= c["real_wage"] * s, ("floor", j)
            if c["wage_ceiling"] is not None:
                assert c["real_wage"] <= c["wage_ceiling"] * s, ("ceiling", j)
            if c["bbar"] == 0:
                assert c["real_wage"] * s >= 1 / c["Lbar"], ("1/Lbar", j)
        assert r["real_wage"] <= v / r["B_s"] * (1 + tol)
        checks["P_s totals"] = rel(r["Ps"], v * r["L_s"] + r["B_s"])
        checks["P_s direct"] = rel(r["Ps"], v * r["L_star_s"] + sum(z * bb for z, bb in zip(self.z, self.bbar)))
        # eq 11 on the clearing side
        checks["eq11 Y"] = rel(Y, self.T / r["B_q"])
        checks["eq11 n_D"] = rel(r["N_a"], self.T * r["L_q"] / r["B_q"])
        # the full system over categories and machine types (section 4.5)
        n = C + K
        Ap = [[mp.mpf(0)] * n for _ in range(n)]
        Aq = [[mp.mpf(0)] * n for _ in range(n)]
        lam, lamq, bb, bbq = [mp.mpf(0)] * n, [mp.mpf(0)] * n, [mp.mpf(0)] * n, [mp.mpf(0)] * n
        for j in range(C):
            for l in range(C):
                Ap[j][l] = Aq[j][l] = self.Acc[j][l]
            for tt, s in split:
                Ap[j][C + tt] += s * q["Mt"][j] / self.theta[tt]
                Aq[j][C + tt] += s * q["Mt"][j] / self.theta[tt]
            lam[j] = lamq[j] = q["H"][j]
            bb[j] = bbq[j] = self.bc[j]
        for i in range(K):
            for l in range(K):
                Ap[C + i][C + l] = self.Ahat[i][l]
                Aq[C + i][C + l] = self.Aq[i][l]
            lam[C + i], lamq[C + i] = self.lhat[i], self.lq[i]
            bb[C + i], bbq[C + i] = self.bhat[i], self.bq[i]
        p = r["q"]["pc"] + p_m
        for i in range(n):
            checks[f"p = Ap + lv + b, row {i}"] = rel(p[i], sum(Ap[i][l] * p[l] for l in range(n)) + lam[i] * v + bb[i])
        y = [Y * yy for yy in self.yhat] + X
        f = [y[i] - sum(Aq[l][i] * y[l] for l in range(n)) for i in range(n)]
        for j in range(C):
            assert abs(f[j] - Y * self.z[j]) <= tol * Y, ("f", j)
        for k in range(K):
            assert abs(f[C + k]) <= tol * max(y), ("f machines", k)
        checks["N_a = lq'y"] = rel(sum(l * yy for l, yy in zip(lamq, y)), r["N_a"])
        checks["T = bq'y"] = rel(sum(bq * yy for bq, yy in zip(bbq, y)), self.T)
        # income (section 4.6; SSRN App. C)
        checks["p'f"] = rel(sum(pi * fi for pi, fi in zip(p, f)), v * r["N_a"] + self.T + r["interest"])
        checks["income Y P_s"] = rel(Y * r["Ps"], r["income"])
        checks["income sum p z Y"] = rel(sum(c["p"] * c["output"] for c in r["cats"]), r["income"])
        if r["interest"] != 0:
            checks["interest (u - delta) V X"] = rel(r["interest"], sum((u - d) * Vk * Xk for u, d, Vk, Xk in zip(self.u, self.delta, V, X)))
        else:
            assert all((u - d) * Vk * Xk == 0 for u, d, Vk, Xk in zip(self.u, self.delta, V, X))
        checks["baskets"] = rel(r["worker_baskets"] + r["provider"], Y)
        # the ledger per type (L1-L4)
        for k in range(K):
            ty = r["types"][k]
            cash = self.u[k] * V[k] * X[k] - V[k] * self.delta[k] * X[k]
            assert abs(cash - self.rho * ty["W"]) <= tol * max(abs(cash), mp.mpf(10) ** -300), ("ledger", k)
            assert ty["W"] >= V[k] * X[k] * (1 - tol)
        # the closure per task type and the cheapest type (section 4.3)
        checks["closure"] = rel(v, self.closure_wage(t, g))
        for k in range(K):
            if self.theta[k] > 0:
                assert p_m[k] / self.theta[k] >= q["pi"] * (1 - tol), ("cheapest", k)
                w = self.closure_wage(k, g)
                assert w is None or w >= v * (1 - tol), ("unused methods", k)
        bad = {key: val for key, val in checks.items() if val > tol}
        assert not bad, f"identities fail at {DPS} digits: {bad}"


# ------------------------------------------------------------------------------ instances
def fork_categories(intermediate=True):
    """1b's fork economy (docs/unit-1b.md section 3.3) and M4's intermediate inputs
    (docs/unit-1c.md section 3.3)."""
    cats = (("MANUFACTURES", "0.3", "0", ("2", "0", "0")),
            ("FOOD", "1", "0.6", ("0.5", "1.5", "0")),
            ("CARE", "0.2", "0.1", ("0", "0", "1")),
            ("SHELTER", "0.8", "1", ("0", "0.4", "0")))
    acc = (("0", "0", "0", "0"), ("0.1", "0", "0", "0"), ("0", "0.2", "0", "0"), ("0.15", "0", "0", "0"))
    if not intermediate:
        acc = (("0",) * 4,) * 4
    return cats, acc


M4_TYPES = (
    machine_type("LOOM", "1", (("0", "0", "0.05"), "0.3", "0.05"), (("0.1", "0", "0"), "1", "0.3"), "0.1", 2),
    machine_type("ENGINE", "2", (("0", "0", "0.5"), "0.02", "0"), (("0", "0.1", "0"), "0.1", "0.4"), "0.05", 3),
    machine_type("POWER", "0", (("0", "0", "0"), "0.02", "0.5"), (("0", "0.1", "0"), "0.1", "0.1"), "0.05", 3),
)


def m4(**changes):
    """M4, the three-type fork economy (docs/unit-1c.md section 3.3)."""
    cats, acc = fork_categories()
    p = dict(workers="4", land="10", eta="1", g0="0.2", g1="0.8", k="1", chi_max="1", rho="0.04",
             edges=("0", "0.4", "0.75", "1"), categories=cats, intermediate=acc, types=M4_TYPES)
    p.update(changes)
    return Economy(**p)


M2_OP = (("0.5",), "0.1", "0.2")
M2_BUILD = (("0.1",), "0.2", "0.02")


def m3(op=M2_OP, build=M2_BUILD, rho="0.05", delta="0.1", lag=3):
    """M3, check_dynamics' machine in Appendix B's closure: G1's household, the good and
    space, gamma = 1 + 2x (docs/unit-1c.md section 3.3)."""
    return Economy(workers="4", land="10", eta="1", g0="1", g1="2", k="1", chi_max="1", rho=rho,
                   edges=("0", "1"), categories=(("GOOD", "1", "0", ("1",)), ("SPACE", "1", "1", ("0",))),
                   intermediate=(("0", "0"), ("0", "0")),
                   types=(machine_type("MACHINE", "1", op, build, delta, lag),))


FLOW = machine_type("FLOW", "1", (("0", "0"), "0.1", "0.5"), zero_recipe(2), "1", 1)
DURABLE = machine_type("DURABLE", "1", zero_recipe(2), (("0", "0"), "0.02", "1.85"), "0.02", 10)


def m5(rho, workers="4", space="1"):
    """M5: G1's household on gamma = 0.2 + 0.8x with a flow and a durable task type
    (docs/unit-1c.md section 3.3)."""
    return Economy(workers=workers, land="10", eta="1", g0="0.2", g1="0.8", k="1", chi_max="1", rho=rho,
                   edges=("0", "1"), categories=(("GOOD", "1", "0", ("1",)), ("SPACE", space, "1", ("0",))),
                   intermediate=(("0", "0"), ("0", "0")), types=(FLOW, DURABLE))


def one_type_form(edges, categories, **changes):
    """1b's economy (generate_1b.py's parameters) in one-type form: theta 1, no operating
    recipe, the build recipe (a, lambda, b), 1b's delta and J_b, no intermediate inputs."""
    p = dict(g1b.SCALARS)
    p.update(changes)
    C = len(categories)
    return Economy(workers=p["workers"], land=p["land"], eta=p["eta"], g0=p["g0"], g1=p["g1"], k=p["k"],
                   chi_max=p["chi_max"], rho=p["rho"], edges=edges, categories=categories,
                   intermediate=(("0",) * C,) * C,
                   types=(machine_type("M", "1", zero_recipe(1), ((p["a"],), p["lam"], p["b"]),
                                       p["delta"], int(p["build_lag"])),))


def interior(econ):
    regime, q = econ.solve()
    assert regime == "Interior", (regime, {k: v for k, v in q.items() if k in ("changes", "f_at_1", "d_at_1", "f_at_0")})
    return q


def assert_nests_1b(edges, cats, label, **changes):
    """The one-type form of a 1b instance equals generate_1b.py's solve to 1e-65."""
    r = g1b.interior(edges, cats, **changes)
    q = interior(one_type_form(edges, cats, **changes))
    ty = q["types"][0]
    pairs = dict(x=(q["x"], r["x"]), v=(q["v"], r["v"]), pm=(ty["p"], r["pm"]), Vm=(ty["V"], r["Vm"]),
                 Ps=(q["Ps"], r["Ps"]), Y=(q["Y"], r["Y"]), K=(ty["X"], r["K"]), n=(q["N_a"], r["n"]),
                 income=(q["income"], r["income"]), interest=(q["interest"], r["interest"]),
                 lt=(ty["lt"], r["lambda_tilde_machine"]), bt=(ty["bt"], r["b_tilde_machine"]),
                 L_s=(q["L_s"], r["L_s"]), B_s=(q["B_s"], r["B_s"]), L_q=(q["L_q"], r["L_q"]),
                 B_q=(q["B_q"], r["B_q"]))
    for j, (c, d) in enumerate(zip(q["cats"], r["cats"])):
        pairs[f"p{j}"] = (c["p"], d["p"])
        pairs[f"lt{j}"] = (c["lambda_tilde"], d["lambda_tilde"])
        pairs[f"Lstar{j}"] = (c["L_star"], d["L_star"])
    worst = max(abs(a - b) / max(abs(b), mp.mpf(10) ** -300) for a, b in pairs.values())
    assert worst < IDENTITY_TOL, (label, worst)
    return worst


def assert_nests_1a(econ, label, **changes):
    """A one-type economy in Appendix B's closure equals generate.py's solve to 1e-65: the same
    regime, and its diagnostic or every shared output."""
    regime_a, r = g1a.Economy(**changes).solve()
    regime, q = econ.solve()
    assert regime == regime_a, (label, regime, regime_a)
    if regime != "Interior":
        key = dict(NotViable="d_at_1", BoundaryNoMargin="f_at_1", NoInteriorAtZero="f_at_0")[regime]
        worst = rel(q[key], r[key])
        assert worst < IDENTITY_TOL, (label, worst)
        return worst
    ty = q["types"][0]
    pairs = dict(x=(q["x"], r["x"]), v=(q["v"], r["v"]), pm=(ty["p"], r["pm"]), Ps=(q["Ps"], r["Ps"]),
                 Y=(q["Y"], r["Y"]), K=(ty["X"], r["K"]), n=(q["N_a"], r["n"]), income=(q["income"], r["income"]))
    worst = max(abs(a - b) / max(abs(b), mp.mpf(10) ** -300) for a, b in pairs.values())
    assert worst < IDENTITY_TOL, (label, worst)
    return worst


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
    # ------------------------------------------------------------------ nesting (m1)
    ab_edges, ab_cats = g1b.appendix_b_form()
    for changes, label in ((dict(), "G1"), (dict(lam="0"), "lambda 0"), (dict(eta="0.5"), "eta 0.5"),
                           (dict(rho="0.05", delta="0.1", build_lag=1), "G4 B"),
                           (dict(rho="0.05", delta="0.1", build_lag=3), "G4 D")):
        worst = max(worst, assert_nests_1b(ab_edges, ab_cats, label, **changes))
    e, c = g1b.appendix_b_form("0.7")
    worst = max(worst, assert_nests_1b(e, c, "G5 durable", workers="5.2", land="12.5", a="0.22", lam="0.08",
                                       b="0.55", eta="2.3", g0="0.15", g1="0.9", chi_max="1.6", k="2.5",
                                       rho="0.04", delta="0.35", build_lag=2))
    for changes, label in ((dict(), "C3"), (dict(rho="0.04", delta="0.35", build_lag=2), "C3d"),
                           (dict(delta="0.1"), "C3z")):
        worst = max(worst, assert_nests_1b(g1b.FORK_EDGES, g1b.FORK, label, **changes))
    worst = max(worst, assert_nests_1b(g1b.GAP_EDGES, g1b.GAP, "gap", workers="5"))
    # M3's corners are 1a economies (check_dynamics R4, R5)
    flow = m3(build=zero_recipe(1))
    worst = max(worst, assert_nests_1a(flow, "M3 zero build", a="0.5", lam="0.1", b="0.2", g0="1", g1="2"))
    capital = m3(op=zero_recipe(1))
    assert rel(capital.u[0], M("0.165375")) < IDENTITY_TOL
    worst = max(worst, assert_nests_1a(capital, "M3 zero operating", a="0.1", lam="0.2", b="0.02", g0="1",
                                       g1="2", rho="0.05", delta="0.1", build_lag=3))

    # ------------------------------------------------------------------ M2
    section("M2: check_dynamics' machine (operating (0.5, 0.1, 0.2), build (0.1, 0.2, 0.02), rho 0.05, "
            "delta 0.1, J 3) as a price block at the targets' own margins (docs/unit-1c.md section 0.2)")
    block = m3()
    u, omega = block.u[0], block.omega[0]
    assert rel(u, M("0.165375")) < IDENTITY_TOL and rel(omega, M("1.3075")) < IDENTITY_TOL
    put("M2_U", u, "u = (rho + delta)(1 + rho)^(J - 1), check_dynamics U3")
    put("M2_OMEGA", omega, "omega = (1 + rho)^(J - 1) + delta sum (1 + rho)^i, L2")
    put("M2_LAMBDA_TILDE", block.lt[0], "lambda-tilde, price side: (lambda + u lambda_I)/(1 - a - u a_I)")
    put("M2_B_TILDE", block.bt[0], "b-tilde, price side: (b + u b_I)/(1 - a - u a_I)")
    put("M2_LAMBDA_TILDE_Q", block.ltq[0], "lambda-tilde^q = 0.12/0.49, clearing side")
    put("M2_B_TILDE_Q", block.btq[0], "b-tilde^q = 0.202/0.49, clearing side")
    assert rel(block.ltq[0], M("0.12") / M("0.49")) < IDENTITY_TOL and rel(block.btq[0], M("0.202") / M("0.49")) < IDENTITY_TOL
    a, lam, b, aI, lI, bI = M("0.5"), M("0.1"), M("0.2"), M("0.1"), M("0.2"), M("0.02")
    delta, rho = M("0.1"), M("0.05")
    json = dict(  # dynamics/checks/dynamics_ss_targets.json at 31b3482, as written (doubles)
        sloped=dict(xstar=0.5962490482385818, c=0.37397572659090933, w=1.2659064107675564,
                    r=0.0607097333057802, Y=1.381406447778144, X=3.685465266168454, pK=0.2917930494787178,
                    uK=0.165375),
        flat=dict(c=0.3333333333333333, w=1.0, r=0.1381118092872455, X=2.4150834730304904,
                  Y=1.1046536171646546, m=0.3570926015168396, uK=0.165375))
    assert rel(M(json["sloped"]["uK"]), u) < JSON_TOL and rel(M(json["flat"]["uK"]), u) < JSON_TOL
    sK = 1 - a - aI * delta

    def price_block(g):
        Den = 1 - a - lam * g - u * (aI + lI * g)  # R1
        pm = (b + u * bI) / Den
        v = g * pm
        V = aI * pm + lI * v + bI  # R3
        O = a * pm + lam * v + b
        q = block.block(g, 0)
        for key, val in (("p", pm), ("v", v), ("V", V), ("O", O)):
            got = q[key][0] if isinstance(q[key], list) else q[key]
            assert rel(got, val) < IDENTITY_TOL, key
        prod = mp.mpf(1)
        for pv in q["pivots"]:
            prod *= pv
        assert rel(prod, Den) < IDENTITY_TOL  # R6: the determinant is Den
        return Den, pm, v, V, O

    # sloped: gamma = 1 + 4x at the target's x* (the double), gamma_L = 1
    xs = M(json["sloped"]["xstar"])
    g = 1 + 4 * xs
    Ig = xs + 2 * xs * xs
    Den, pm, v, V, O = price_block(g)
    p_good = v * (1 - xs) + pm * Ig
    labour = (1 - xs) + block.ltq[0] * Ig
    Yg = 1 / labour
    x_per_y = Ig / sK
    for key, val, note in (
            ("M2_SLOPED_X_STAR", xs, "x*, the target's double (an input)"),
            ("M2_SLOPED_GAMMA", g, "gamma* = 1 + 4x*"),
            ("M2_SLOPED_J", Ig, "J(x*) = x* + 2x*^2"),
            ("M2_SLOPED_DEN", Den, "Den = 1 - a - lambda gamma* - u(a_I + lambda_I gamma*), R1"),
            ("M2_SLOPED_P_M", pm, "p_m = c/r = theta_c, R1"),
            ("M2_SLOPED_V", v, "v = w/r = theta_w = gamma* theta_c, R2"),
            ("M2_SLOPED_BUILD", V, "V = p_K/r = a_I theta_c + lambda_I theta_w + b_I, R3"),
            ("M2_SLOPED_OPERATING", O, "O = a p_m + lambda v + b"),
            ("M2_SLOPED_P_GOOD", p_good, "p_good = 1/r = v(1 - x*) + p_m J(x*)"),
            ("M2_SLOPED_LABOR_PER_GOOD", labour, "(1 - x*) + lambda-tilde^q J(x*), hours per unit of the good"),
            ("M2_SLOPED_Y", Yg, "Y = N/labour per good, N = 1"),
            ("M2_SLOPED_SERVICES_PER_GOOD", x_per_y, "X/Y = J(x*)/(1 - a - delta a_I)"),
            ("M2_SLOPED_SERVICES", Yg * x_per_y, "X = Y J(x*)/(1 - a - delta a_I)")):
        put(key, val, note)
    target = dict(c=pm / p_good, w=v / p_good, r=1 / p_good, Y=Yg, X=Yg * x_per_y, pK=V / p_good)
    for key, val in target.items():
        assert rel(M(json["sloped"][key]), val) < JSON_TOL, (key, rel(M(json["sloped"][key]), val))
        put(f"M2_SLOPED_TARGET_{key.upper()}", val, f"{key} in the target's units (the good as numeraire)")

    def land_share_residual(x):
        """check_dynamics' sloped closure (EJ, :326-341): land clearing with the Cobb-Douglas
        land share alpha 0.3, N 1, T 10, the good as numeraire."""
        g = 1 + 4 * x
        Ig = x + 2 * x * x
        c = 1 / (g * (1 - x) + Ig)
        w = g * c
        Den = 1 - a - lam * g - u * (aI + lI * g)
        r = c * Den / (b + u * bI)
        Y = 1 / ((1 - x) + (lam + lI * delta) * Ig / sK)
        X = Y * Ig / sK
        return (b + bI * delta) * X + M("0.3") * (w * 1 + r * 10) / r - 10

    root = mp.findroot(land_share_residual, M("0.596"))
    assert abs(land_share_residual(root)) < IDENTITY_TOL
    assert rel(root, xs) < JSON_TOL
    put("M2_SLOPED_LAND_SHARE_ROOT", root, "the land-share closure's root at 70 digits (not 1c's closure)")
    # flat: gamma-bar = 3, gamma_L = 1, at the target's m
    g = M(3)
    Den, pm, v, V, O = price_block(g)
    m = M(json["flat"]["m"])
    labour = (1 - m) + block.ltq[0] * g * m
    Yf = 1 / labour
    Xf = Yf * g * m / sK
    for key, val, note in (
            ("M2_FLAT_DEN", Den, "Den at gamma* = 3"),
            ("M2_FLAT_P_M", pm, "p_m = c/r at gamma* = 3"),
            ("M2_FLAT_V", v, "v = w/r at gamma* = 3"),
            ("M2_FLAT_BUILD", V, "V = p_K/r at gamma* = 3"),
            ("M2_FLAT_OPERATING", O, "O at gamma* = 3"),
            ("M2_FLAT_M", m, "m, the target's machine-task share (an input)"),
            ("M2_FLAT_Y", Yf, "Y = N/((1 - m) + lambda-tilde^q gamma-bar m), N = 1"),
            ("M2_FLAT_SERVICES", Xf, "X = Y gamma-bar m/(1 - a - delta a_I)")):
        put(key, val, note)
    target = dict(c=pm / v, w=M(1), r=1 / v, X=Xf, Y=Yf)
    for key, val in target.items():
        assert rel(M(json["flat"][key]), val) < JSON_TOL, (key, rel(M(json["flat"][key]), val))
    put("M2_FLAT_TARGET_C", pm / v, "c in the target's units (w = 1)")
    put("M2_FLAT_TARGET_R", 1 / v, "r in the target's units (w = 1)")
    # the flat closed form of check_dynamics :212-214 (E3) for m, X and Y, at T 3.8, alpha 0.3
    T, alpha = M("3.8"), M("0.3")
    thc = (b + u * bI) / Den
    rF = (1 / g) / thc
    XF = ((1 - alpha) * T - alpha * 1 * 1 / rF) / (b + bI * delta)
    MYF = XF * sK / g
    YF = (1 - (lam + lI * delta) * XF) + MYF
    assert rel(MYF / YF, m) < JSON_TOL and rel(XF, Xf) < JSON_TOL and rel(YF, Yf) < JSON_TOL
    put("M2_FLAT_LAND_SHARE_M", MYF / YF, "m from the land-share closure at 70 digits (not 1c's closure)")

    # ------------------------------------------------------------------ M3
    for tag, econ, label in (("M3", m3(), "rho 0.05"), ("M3Z", m3(rho="0"), "rho 0")):
        section(f"{tag}: M2's machine in Appendix B's closure, G1's household, gamma = 1 + 2x, {label} "
                f"(docs/unit-1c.md section 3.3)")
        q = interior(econ)
        ty = q["types"][0]
        good = q["cats"][0]
        for key, val, note in (
                ("X_STAR", q["x"], "x*"), ("ONE_MINUS_X_STAR", q["one_minus_x"], "1 - x*"),
                ("GAMMA_STAR", q["gamma"], "gamma(x*)"), ("V", q["v"], "v = w/r"),
                ("U", ty["u"], "u"), ("OMEGA", ty["omega"], "omega"),
                ("P_M", ty["p"], "p_m = O + u V"), ("OPERATING", ty["O"], "O, the operating cost"),
                ("BUILD", ty["V"], "V, the build cost"), ("P_GOOD", good["p"], "p of the good"),
                ("P_S", q["Ps"], "P_s"), ("Y", q["Y"], "Y"), ("SERVICES", ty["X"], "X = K"),
                ("N_A", q["N_a"], "N_a"), ("FINAL_HOURS", q["final_hours"], "final hours"),
                ("MACHINE_HOURS", q["machine_hours"], "machine hours (lambda + delta lambda_I) X"),
                ("INCOME", q["income"], "I = v N_a + T + interest"),
                ("LAMBDA_TILDE", ty["lt"], "lambda-tilde, price side"),
                ("LAMBDA_TILDE_Q", ty["ltq"], "lambda-tilde^q, clearing side"),
                ("B_TILDE", ty["bt"], "b-tilde, price side"),
                ("B_TILDE_Q", ty["btq"], "b-tilde^q, clearing side"),
                ("LABOR_SHARE", q["labor_share"], "v N_a/I"), ("REAL_WAGE", q["real_wage"], "v/P_s")):
            put(f"{tag}_{key}", val, note)
        if tag == "M3":
            assert ty["lt"] > ty["ltq"] and ty["bt"] > ty["btq"]
            put("M3_INTEREST", q["interest"], "interest = rho omega V X, L2")
            put("M3_WEALTH", ty["W"], "W = omega V X, machine wealth")
            put("M3_CAPITAL_SHARE", q["capital_share"], "interest/I")
        else:
            assert q["interest"] == 0 and ty["lt"] == ty["ltq"] and ty["bt"] == ty["btq"]
            assert rel(q["income"], q["v"] * q["N_a"] + econ.T) < IDENTITY_TOL

    # ------------------------------------------------------------------ M4
    section("M4: the three-type fork economy, eta 1, rho 0.04: loom, engine and power on 1b's fork "
            "economy with intermediate inputs (docs/unit-1c.md section 3.3)")
    econ = m4()
    q = interior(econ)
    (gsw, below, above), = q["env"][1]
    assert (below, above) == (0, 1) and q["t"] == 1
    put("M4_SWITCH_GAMMA", gsw, "gamma at the switch loom -> engine, closed form")
    put("M4_SWITCH_X", q["xs"][0], "the switch loom -> engine on the line")
    for key, val, note in (("X_STAR", q["x"], "x*, with the engine at the margin"), ("V", q["v"], "v"),
                           ("P_S", q["Ps"], "P_s"), ("Y", q["Y"], "Y"), ("N_A", q["N_a"], "N_a"),
                           ("INCOME", q["income"], "I"), ("INTEREST", q["interest"], "interest"),
                           ("REAL_WAGE", q["real_wage"], "v/P_s"), ("L_S", q["L_s"], "L_s, price side"),
                           ("B_S", q["B_s"], "B_s, price side"), ("L_S_Q", q["L_q"], "L_s^q, clearing side"),
                           ("B_S_Q", q["B_q"], "B_s^q, clearing side")):
        put(f"M4_{key}", val, note)
    for ty in q["types"]:
        n = ty["name"]
        for key, val, note in (("U", ty["u"], "u"), ("OMEGA", ty["omega"], "omega"),
                               ("LAMBDA_TILDE", ty["lt"], "lambda-tilde"), ("B_TILDE", ty["bt"], "b-tilde"),
                               ("LAMBDA_TILDE_Q", ty["ltq"], "lambda-tilde^q"), ("B_TILDE_Q", ty["btq"], "b-tilde^q"),
                               ("P", ty["p"], "p"), ("OPERATING", ty["O"], "O"), ("BUILD", ty["V"], "V"),
                               ("SERVICES", ty["X"], "X")):
            if key == "SERVICES" and val == 0:
                assert n == "LOOM"
                continue
            put(f"M4_{n}_{key}", val, f"{note} of the {n.lower()}")
        if ty["closure_wage"] is not None:
            put(f"M4_{n}_CLOSURE_WAGE", ty["closure_wage"], f"the {n.lower()}'s closure wage at x*")
    assert q["types"][0]["X"] == 0 and q["types"][0]["closure_wage"] > q["v"]
    assert q["types"][2]["closure_wage"] is None
    for name, c in zip(econ.names, q["cats"]):
        put(f"M4_{name}_P", c["p"], f"p of {name.lower()}")
        put(f"M4_{name}_REAL_WAGE", c["real_wage"], f"v/p of {name.lower()}")
    for name, y, L, bb in zip(econ.names, econ.yhat, econ.Lbar, econ.bbar):
        put(f"M4_{name}_YHAT", y, f"y-hat of {name.lower()}, the basket's gross output")
        put(f"M4_{name}_L_BAR", L, f"L-bar of {name.lower()} through the chain")
        put(f"M4_{name}_B_BAR", bb, f"b-bar of {name.lower()}, land through the chain")
    assert [mp.nstr(y, 10) for y in econ.yhat] == ["0.524", "1.04", "0.2", "0.8"]

    section("M4 path: the switch and the root as task automation lowers eta")
    last_x = None
    for tag, eta in (("ETA2", "2"), ("ETA1", "1"), ("ETA05", "0.5")):
        qq = interior(m4(eta=eta))
        assert qq["t"] == 1 and len(qq["xs"]) == 1 and qq["types"][0]["X"] == 0
        assert qq["types"][0]["closure_wage"] > qq["v"]
        if last_x is not None:
            assert qq["xs"][0] > last_x
        last_x = qq["xs"][0]
        put(f"M4_{tag}_SWITCH_X", qq["xs"][0], f"the switch at eta {eta}")
        put(f"M4_{tag}_X_STAR", qq["x"], f"x* at eta {eta}")
        put(f"M4_{tag}_V", qq["v"], f"v at eta {eta}")

    section("M4t: the tie at the switch, eta 0.5, N 8 (docs/unit-1c.md section 4.7)")
    econ = m4(eta="0.5", workers="8")
    regime, t = econ.solve()
    assert regime == "Tie" and t["t"] == 0 and t["tie"]["above"] == 1
    xsw = t["x"]
    fa = econ.at(xsw, 0)["f"]
    fb = econ.at(xsw, 1)["f"]
    assert fa > 0 > fb
    for key, val, note in (("X_STAR", xsw, "x* = the switch point"), ("GAMMA", t["tie"]["gamma"], "gamma at the switch"),
                           ("F_LOOM", fa, "f at the switch under the loom"),
                           ("F_ENGINE", fb, "f at the switch under the engine"),
                           ("SHARE", t["tie"]["share"], "sigma, the engine's share of the machine tasks"),
                           ("V", t["v"], "v"), ("P_S", t["Ps"], "P_s"), ("Y", t["Y"], "Y"), ("N_A", t["N_a"], "N_a"),
                           ("INCOME", t["income"], "I"), ("INTEREST", t["interest"], "interest"),
                           ("L_S_Q", t["L_q"], "L_s^q with the split"), ("B_S_Q", t["B_q"], "B_s^q with the split")):
        put(f"M4T_{key}", val, note)
    for ty in t["types"]:
        put(f"M4T_{ty['name']}_SERVICES", ty["X"], f"X of the {ty['name'].lower()} at the tie")
    assert rel(t["N_a"], t["q"]["nS"]) < IDENTITY_TOL

    section("M4 regimes")
    regime, d = m4(eta="0.25").solve()
    assert regime == "BoundaryNoMargin"
    put("M4_REG_ETA025_F_AT_1", d["f_at_1"], "f(1) at eta 0.25: BoundaryNoMargin")
    regime, d = m4(eta="50").solve()
    assert regime == "NotViable" and d["d_at_1"] < 0 and d["env"] == (1, [])
    put("M4_REG_ETA50_D_AT_1", d["d_at_1"], "d(1) at eta 50, the engine's least pivot: NotViable")
    regime, d = m4(workers="200", chi_max="0.01").solve()
    assert regime == "NoInteriorAtZero"
    put("M4_REG_N200_F_AT_LO", d["f_at_0"], "f(1e-12) at N 200, chi_max 0.01: NoInteriorAtZero")

    # rho = 0: the switch lowers labour demand (the Proposition of section 5.5)
    for econ, label in ((m4(rho="0"), "M4 rho 0"), (m4(rho="0", eta="0.5", workers="8"), "M4t rho 0")):
        for gs, a, b in econ.envelope()[1]:
            xs = econ.gamma_inv(gs)
            assert econ.at(xs, b)["nD"] < econ.at(xs, a)["nD"], label

    # ------------------------------------------------------------------ M5
    section("M5 path: interest selects the technique, G1's household on gamma = 0.2 + 0.8x with a flow "
            "and a durable task type (docs/unit-1c.md section 3.3)")
    flow_at = None
    for tag, rho, tech in (("RHO0", "0", 1), ("RHO0_05", "0.05", 1), ("RHO0_1", "0.1", 1),
                           ("RHO0_15", "0.15", 0), ("RHO0_3", "0.3", 0)):
        econ = m5(rho)
        q = interior(econ)
        assert q["t"] == tech, (rho, q["t"])
        for gs, a, b in q["env"][1]:
            if rho == "0":
                xs = econ.gamma_inv(gs)
                assert econ.at(xs, b)["nD"] < econ.at(xs, a)["nD"]
        put(f"M5_{tag}_X_STAR", q["x"], f"x* at rho {rho}")
        put(f"M5_{tag}_V", q["v"], f"v at rho {rho}")
        put(f"M5_{tag}_N_A", q["N_a"], f"N_a at rho {rho}")
        put(f"M5_{tag}_INCOME", q["income"], f"I at rho {rho}")
        if q["interest"] != 0:
            put(f"M5_{tag}_INTEREST", q["interest"], f"interest at rho {rho}")
        if rho == "0.1":
            assert len(q["xs"]) == 1 and q["xs"][0] < q["x"]
            put("M5_RHO0_1_SWITCH_X", q["xs"][0], "the switch flow -> durable at rho 0.1")
        if tech == 0:
            assert q["interest"] == 0
            if flow_at is None:
                flow_at = q
            else:
                assert rel(q["x"], flow_at["x"]) < IDENTITY_TOL and rel(q["v"], flow_at["v"]) < IDENTITY_TOL

    section("M5m: three equilibria at rho 0.1, N 60, h 0.2 (docs/unit-1c.md section 5.5)")
    econ = m5("0.1", workers="60", space="0.2")
    regime, d = econ.solve()
    assert regime == "MultipleEquilibria" and d["changes"] == 3
    seq, xs = d["seq"], d["xs"]
    put("M5M_SWITCH_X", xs[0], "the switch flow -> durable")
    put("M5M_F_AT_LO", seq[0], "f(1e-12), flow")
    put("M5M_F_FLOW_AT_SWITCH", seq[1], "f at the switch, flow")
    put("M5M_F_DURABLE_AT_SWITCH", seq[2], "f at the switch, durable")
    put("M5M_F_AT_1", seq[3], "f(1), durable")
    # the switch raises labour demand here
    assert econ.at(xs[0], 1)["nD"] > econ.at(xs[0], 0)["nD"]
    lo, hi = M(BRACKET_LO), xs[0]
    for _ in range(BISECTIONS):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if econ.at(mid, 0)["f"] > 0 else (lo, mid)
    put("M5M_FLOW_ROOT", (lo + hi) / 2, "the equilibrium below the switch (flow), for reference")
    fa = econ.at(xs[0], 0)
    fb = econ.at(xs[0], 1)
    Ba, Bb = fa["qty"]["land"], fb["qty"]["land"]
    f_a, f_b = fa["f"], fb["nD"] - fa["nS"]
    put("M5M_TIE_SHARE", Ba * f_a / (Ba * f_a - Bb * f_b), "sigma of the tie at the switch, for reference")
    lo, hi = xs[0], M(1)
    for _ in range(BISECTIONS):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if econ.at(mid, 1)["f"] > 0 else (lo, mid)
    put("M5M_DURABLE_ROOT", (lo + hi) / 2, "the equilibrium above the switch (durable), for reference")

    body = "\n".join(LINES) + "\n"
    header = [
        "# goldens_1c.txt: the golden numbers for oracle unit 1c.",
        f"# Written by goldens/generate_1c.py (mpmath, {DPS} digits). Do not edit by hand.",
        f"# Each line is KEY = value  # note. Values carry {SIG_OUT} significant digits.",
        "# Equations: docs/unit-1c.md section 4. laborformal references are at 31b3482.",
        f"# The one-type form nests generate_1b.py's and generate.py's solves within {mp.nstr(worst, 3)} "
        "(1e-65 asserted).",
        f"# fnv1a64 generate_1c.py = {fnv1a64(source_bytes(__file__)):016x}",
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
                        help="compare with goldens_1c.txt instead of writing it; exit 1 on a difference")
    args = parser.parse_args()
    text = build()
    path = os.path.join(HERE, "goldens_1c.txt")
    if args.check:
        with open(path, encoding="utf-8") as fh:
            same = fh.read() == text
        print("goldens_1c.txt is current" if same else "goldens_1c.txt differs from generate_1c.py's output")
        return 0 if same else 1
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print(f"wrote {path}: {sum(1 for line in text.splitlines() if ' = ' in line and not line.startswith('#'))} goldens")
    return 0


if __name__ == "__main__":
    sys.exit(main())
