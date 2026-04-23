"""
rustyecon — channel mechanism & agent decision simulation

Sections:
  1. Two-region market, no shipper: baseline price dynamics
  2. Shipper variants: naive vs. hedged
  3. Multi-agent: varying agent count and noise level
  4. Fill-rate learning: naive vs. history-tracking agent
  5. Price convergence vs. channel parameters (alpha, capacity, op_cost)
  6. Recipe evaluation budget
"""

import numpy as np
import time

rng = np.random.default_rng(42)
SEP = "=" * 70


# ─────────────────────────────────────────────────────────────────────────────
# CORE PRIMITIVES
# ─────────────────────────────────────────────────────────────────────────────

class RegionMarket:
    def __init__(self, name, init_price, alpha=0.15, price_floor=0.1, price_ceil=10.0):
        self.name    = name
        self.price   = init_price
        self.alpha   = alpha
        self.floor   = price_floor
        self.ceil    = price_ceil
        self.history = [init_price]
        self.vols    = []            # (supply, demand) each tick

    def clear(self, total_supply, total_demand):
        """Transact at self.price (posted = last tick's). Update price for next tick."""
        if total_supply <= 0 and total_demand <= 0:
            self.history.append(self.price)
            self.vols.append((0.0, 0.0))
            return 1.0, 1.0

        if total_supply >= total_demand:
            buy_fill  = 1.0
            sell_fill = (total_demand / total_supply) if total_supply > 0 else 0.0
            imbalance = -((total_supply - total_demand) / total_supply)
        else:
            buy_fill  = (total_supply / total_demand) if total_demand > 0 else 0.0
            sell_fill = 1.0
            imbalance = (total_demand - total_supply) / total_demand

        new_price  = self.price * (1.0 + self.alpha * np.clip(imbalance, -1, 1))
        self.price = float(np.clip(new_price, self.floor, self.ceil))
        self.history.append(self.price)
        self.vols.append((total_supply, total_demand))
        return buy_fill, sell_fill

    def prices(self): return np.array(self.history)
    def imbalances(self):
        s = np.array([v[0] for v in self.vols])
        d = np.array([v[1] for v in self.vols])
        denom = np.maximum(np.maximum(s, d), 1e-9)
        return (d - s) / denom


# Supply and demand functions (calibrated so a shipper of cap=250 can roughly balance)
def supply_a(t, base=160.0):
    """Seasonal: big harvest around week 36."""
    week = t % 52
    pulse = 300.0 * max(0.0, np.sin((week - 28) * np.pi / 8)) if 28 <= week < 36 else 0.0
    return base + pulse

def demand_a(price, ref=0.95, base=120.0, elast=0.3):
    return base * (ref / max(price, 1e-9)) ** elast

def supply_b(_):
    return 30.0   # small local production

def demand_b(price, ref=1.05, base=170.0, elast=0.35):
    return base * (ref / max(price, 1e-9)) ** elast


# ─────────────────────────────────────────────────────────────────────────────
# SHIPPER CLASSES
# ─────────────────────────────────────────────────────────────────────────────

class NaiveShipper:
    """Pure proportional: ships everything the spread justifies."""
    def __init__(self, op_cost=0.06, capacity=250.0, working_capital=2000.0, **_):
        self.op_cost   = op_cost
        self.capacity  = capacity
        self.cash      = working_capital
        self.inventory = 0.0

    def step(self, mkt_a, mkt_b, _rng=None):
        sell_b = self.inventory
        spread = mkt_b.price - mkt_a.price - self.op_cost
        if spread > 0:
            buy_a = min(self.capacity, self.cash / max(mkt_a.price, 1e-9))
        else:
            buy_a = 0.0
        self.cash     -= buy_a  * mkt_a.price
        self.cash     += sell_b * (mkt_b.price - self.op_cost)
        self.cash      = max(self.cash, 0.0)
        self.inventory = buy_a
        return buy_a, sell_b


class HedgedShipper:
    """PID-style dampening + noise + floor + loss aversion."""
    def __init__(self, op_cost=0.06, capacity=250.0, working_capital=2000.0,
                 kp=0.6, kd=0.4, noise=0.15, loss_aversion=0.2,
                 floor_frac=0.02, rng=None):
        self.op_cost       = op_cost
        self.capacity      = capacity
        self.cash          = working_capital
        self.inventory     = 0.0
        self.kp            = kp
        self.kd            = kd
        self.noise         = noise
        self.loss_aversion = loss_aversion
        self.floor_frac    = floor_frac
        self.prev_spread   = 0.0
        self.fill_history  = [1.0] * 8
        self.rng           = rng or np.random.default_rng(99)

    def step(self, mkt_a, mkt_b, rng=None):
        rng   = rng or self.rng
        sell_b = self.inventory
        pa, pb = mkt_a.price, mkt_b.price
        spread = pb - pa - self.op_cost
        spread_delta = spread - self.prev_spread
        self.prev_spread = spread

        floor_vol = self.capacity * self.floor_frac

        if spread <= 0:
            buy_a = floor_vol
        else:
            p_term       = self.kp * np.clip(spread / max(pa, 1e-9), 0, 1)
            d_term       = self.kd * np.clip(spread_delta / max(pa, 1e-9), -0.5, 0.5)
            target_frac  = np.clip(p_term - d_term, self.floor_frac, 1.0)
            avg_fill     = float(np.mean(self.fill_history))
            hedged_frac  = target_frac * avg_fill * (1.0 - self.loss_aversion)
            hedged_frac  = np.clip(hedged_frac * rng.normal(1.0, self.noise),
                                   self.floor_frac, 1.0)
            buy_a        = max(hedged_frac * self.capacity, floor_vol)
            buy_a        = min(buy_a, self.cash / max(pa, 1e-9))

        self.fill_history.pop(0)
        self.fill_history.append(1.0)
        self.cash     -= buy_a  * pa
        self.cash     += sell_b * (pb - self.op_cost)
        self.cash      = max(self.cash, 0.0)
        self.inventory = buy_a
        return buy_a, sell_b


# ─────────────────────────────────────────────────────────────────────────────
# SHARED RUNNER
# ─────────────────────────────────────────────────────────────────────────────

def run_sim(shippers, n_ticks=260, seed=42):
    sim_rng = np.random.default_rng(seed)
    ma = RegionMarket("A", init_price=0.95, alpha=0.15)
    mb = RegionMarket("B", init_price=1.10, alpha=0.15)

    for t in range(n_ticks):
        sa  = supply_a(t)
        da  = demand_a(ma.price)
        sb  = supply_b(t)
        db  = demand_b(mb.price)

        total_buy_a = total_sell_b = 0.0
        for shipper in shippers:
            ba, sb_out = shipper.step(ma, mb, sim_rng)
            total_buy_a  += ba
            total_sell_b += sb_out

        ma.clear(sa,          da + total_buy_a)
        mb.clear(sb + total_sell_b, db)

    return ma, mb


def stats(ma, mb, label, op_cost, n_ticks):
    pa     = ma.prices()
    pb     = mb.prices()
    spread = pb - pa
    y1     = slice(0,  52)
    y2     = slice(52, 104)
    y5     = slice(max(0, n_ticks - 52), n_ticks)
    rev    = int(np.sum(np.diff(np.sign(np.diff(spread[52:]))) != 0))

    print(f"\n  [{label}]")
    print(f"  {'':30s} {'yr1':>8} {'yr2':>8} {'yr5':>8}")
    print(f"  {'Price A mean':30s} {pa[y1].mean():8.3f} {pa[y2].mean():8.3f} {pa[y5].mean():8.3f}")
    print(f"  {'Price B mean':30s} {pb[y1].mean():8.3f} {pb[y2].mean():8.3f} {pb[y5].mean():8.3f}")
    print(f"  {'Spread mean (B−A)':30s} {spread[y1].mean():8.3f} {spread[y2].mean():8.3f} {spread[y5].mean():8.3f}")
    print(f"  {'Spread std':30s} {spread[y1].std():8.4f} {spread[y2].std():8.4f} {spread[y5].std():8.4f}")
    print(f"  {'Op cost (target spread)':30s} {op_cost:>8.3f}")
    print(f"  {'Spread reversals (yr2+)':30s} {rev:>8}")
    print(f"  {'Shipper cash':30s} {shippers[0].cash if len(shippers)==1 else sum(s.cash for s in shippers):>8.0f}")


# ─────────────────────────────────────────────────────────────────────────────
# 1. BASELINE — NO SHIPPER
# ─────────────────────────────────────────────────────────────────────────────
print(SEP)
print("1. BASELINE: TWO REGIONS, NO SHIPPER")
print(SEP)

N = 260
ma0 = RegionMarket("A", init_price=0.95, alpha=0.15)
mb0 = RegionMarket("B", init_price=1.10, alpha=0.15)
for t in range(N):
    ma0.clear(supply_a(t), demand_a(ma0.price))
    mb0.clear(supply_b(t), demand_b(mb0.price))

pa0, pb0 = ma0.prices(), mb0.prices()
sp0 = pb0 - pa0
print(f"\n  {'':30s} {'yr1':>8} {'yr5':>8}")
print(f"  {'Price A mean':30s} {pa0[:52].mean():8.3f} {pa0[208:].mean():8.3f}")
print(f"  {'Price B mean':30s} {pb0[:52].mean():8.3f} {pb0[208:].mean():8.3f}")
print(f"  {'Spread (B−A) mean':30s} {sp0[:52].mean():8.3f} {sp0[208:].mean():8.3f}")
print(f"  {'Spread std':30s} {sp0[:52].std():8.4f} {sp0[208:].std():8.4f}")


# ─────────────────────────────────────────────────────────────────────────────
# 2. SHIPPER VARIANTS
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("2. SHIPPER VARIANTS")
print(SEP)

for label, cls, kwargs in [
    ("Naive (proportional)",             NaiveShipper,  {}),
    ("Hedged (PID + noise, kp=0.6)",     HedgedShipper, {"kp": 0.6, "kd": 0.4}),
    ("Hedged, more aggressive (kp=0.9)", HedgedShipper, {"kp": 0.9, "kd": 0.2}),
    ("Hedged, less noise (noise=0.05)",  HedgedShipper, {"kp": 0.6, "kd": 0.4, "noise": 0.05}),
]:
    shippers = [cls(working_capital=3000, **kwargs)]
    ma, mb = run_sim(shippers, n_ticks=N)
    stats(ma, mb, label, op_cost=0.06, n_ticks=N)


# ─────────────────────────────────────────────────────────────────────────────
# 3. MULTI-AGENT: AGENT COUNT vs. NOISE
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("3. MULTI-AGENT: AGENT COUNT vs. NOISE LEVEL")
print(SEP)
print(f"\n  {'agents':<8} {'noise':<8} {'yr5 spread mean':>16} {'yr5 spread std':>16} {'reversals':>12}")
print(f"  {'──────':<8} {'─────':<8} {'───────────────':>16} {'──────────────':>16} {'─────────':>12}")

for n_agents in [1, 3, 5, 10]:
    for noise in [0.00, 0.10, 0.20]:
        per_cap = 250.0 / n_agents
        shippers = [
            HedgedShipper(capacity=per_cap, working_capital=3000/n_agents,
                          noise=noise, rng=np.random.default_rng(i * 31 + 7))
            for i in range(n_agents)
        ]
        ma, mb = run_sim(shippers, n_ticks=N, seed=42)
        sp = mb.prices() - ma.prices()
        rev = int(np.sum(np.diff(np.sign(np.diff(sp[52:]))) != 0))
        print(f"  {n_agents:<8} {noise:<8.2f} {sp[208:].mean():>16.4f} {sp[208:].std():>16.4f} {rev:>12}")


# ─────────────────────────────────────────────────────────────────────────────
# 4. FILL-RATE LEARNING
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("4. FILL-RATE LEARNING (tight supply market)")
print(SEP)

def run_fill_rate_test(n_ticks=260, seed=55):
    sim_rng = np.random.default_rng(seed)
    ma = RegionMarket("A", init_price=0.95, alpha=0.20)
    mb = RegionMarket("B", init_price=1.10, alpha=0.20)

    # Two shippers competing; supply in A is capped/variable
    naive_cash, naive_inv     = 2000.0, 0.0
    learner_cash, learner_inv = 2000.0, 0.0
    learner_fills             = [1.0] * 8

    rows = []
    for t in range(n_ticks):
        tight_supply = 100.0 + 30.0 * np.sin(t * 0.25) + sim_rng.normal(0, 10)
        tight_supply = max(tight_supply, 20.0)
        available_for_export = tight_supply * 0.45

        pa, pb = ma.price, mb.price
        spread = pb - pa - 0.06

        # Naive: full capacity if spread > 0
        naive_want = min(120.0, naive_cash / max(pa, 1e-9)) if spread > 0 else 3.0

        # Learner: scales by rolling fill rate
        avg_fill     = float(np.mean(learner_fills))
        base_want    = min(120.0, learner_cash / max(pa, 1e-9)) if spread > 0 else 3.0
        learner_want = max(base_want * avg_fill * sim_rng.normal(1.0, 0.10), 3.0)

        total_want = naive_want + learner_want
        fill_rate  = min(available_for_export / total_want, 1.0) if total_want > 0 else 0.0

        naive_got   = naive_want  * fill_rate
        learner_got = learner_want * fill_rate

        learner_fills.pop(0)
        learner_fills.append(fill_rate)

        naive_cash   = max(naive_cash   - naive_got   * pa + naive_inv   * (pb - 0.06), 0)
        learner_cash = max(learner_cash - learner_got * pa + learner_inv * (pb - 0.06), 0)
        naive_inv, learner_inv = naive_got, learner_got

        ma.clear(tight_supply, demand_a(pa) + total_want)
        mb.clear(supply_b(t) + naive_inv + learner_inv, demand_b(pb))
        rows.append((naive_cash, learner_cash, fill_rate))

    return rows

rows = run_fill_rate_test()
nc = np.array([r[0] for r in rows])
lc = np.array([r[1] for r in rows])
fr = np.array([r[2] for r in rows])

print(f"\n  {'':30s} {'naive':>10} {'learner':>10}")
print(f"  {'Final cash':30s} {nc[-1]:>10.0f} {lc[-1]:>10.0f}")
print(f"  {'Cash std':30s} {nc.std():>10.1f} {lc.std():>10.1f}")
print(f"  {'Cash min':30s} {nc.min():>10.1f} {lc.min():>10.1f}")
print(f"  {'Avg fill rate (all ticks)':30s} {fr.mean():>10.3f}")
print(f"  {'Fill rate std':30s} {fr.std():>10.3f}")


# ─────────────────────────────────────────────────────────────────────────────
# 5. CONVERGENCE vs. CHANNEL PARAMETERS
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("5. CONVERGENCE vs. CHANNEL PARAMETERS")
print(SEP)

def convergence_ticks(alpha, capacity, op_cost, n_ticks=520, seed=42, tol=0.10):
    """Returns first tick where |spread - op_cost| < tol*op_cost, then stays within for 10 ticks."""
    sim_rng = np.random.default_rng(seed)
    ma = RegionMarket("A", init_price=0.95, alpha=alpha)
    mb = RegionMarket("B", init_price=1.10, alpha=alpha)
    shipper = HedgedShipper(op_cost=op_cost, capacity=capacity,
                             working_capital=4000, rng=sim_rng)
    spreads = []
    for t in range(n_ticks):
        ba, sb = shipper.step(ma, mb, sim_rng)
        ma.clear(supply_a(t),       demand_a(ma.price) + ba)
        mb.clear(supply_b(t) + sb,  demand_b(mb.price))
        spreads.append(mb.price - ma.price)

    sp = np.array(spreads)
    conv = None
    for i in range(10, len(sp) - 10):
        window = sp[i:i+10]
        if np.all(np.abs(window - op_cost) < tol * max(op_cost, 0.01)):
            conv = i
            break
    return conv or n_ticks, sp[-52:].mean(), sp[-52:].std()

print(f"\n  {'alpha':>7} {'cap':>7} {'op_cost':>9} {'conv_tick':>11} {'final_mean':>12} {'final_std':>11}  label")
print(f"  {'─────':>7} {'───':>7} {'───────':>9} {'─────────':>11} {'──────────':>12} {'─────────':>11}  ─────")

scenarios = [
    (0.15, 250, 0.06, "baseline"),
    (0.06, 250, 0.06, "sticky prices"),
    (0.30, 250, 0.06, "fast prices"),
    (0.15,  80, 0.06, "low capacity"),
    (0.15, 500, 0.06, "high capacity"),
    (0.15, 250, 0.15, "high op cost"),
    (0.15, 250, 0.02, "low op cost"),
    (0.40, 400, 0.01, "financial-like"),
    (0.04, 120, 0.06, "landlocked-like"),
]

for alpha, cap, cost, label in scenarios:
    ct, fm, fs = convergence_ticks(alpha, cap, cost)
    print(f"  {alpha:>7.2f} {cap:>7} {cost:>9.2f} {ct:>11} {fm:>12.4f} {fs:>11.5f}  {label}")


# ─────────────────────────────────────────────────────────────────────────────
# 6. RECIPE EVALUATION BUDGET
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("6. RECIPE EVALUATION BUDGET")
print(SEP)

N_POPS      = 50_475
N_BUILDINGS = 300_000
AVG_INV     = 5
AVG_RECIPES = 2.5

sample_inv = [
    {int(i): float(rng.random()) for i in rng.choice(200, AVG_INV, replace=False)}
    for _ in range(N_POPS)
]

t0 = time.perf_counter()
n_zero = 0
for inv in sample_inv:
    for good_id, qty in inv.items():
        new_qty = qty * 0.998
        if new_qty < 1e-4:
            n_zero += 1
py_ms = (time.perf_counter() - t0) * 1000

print(f"\n  Pop inventory spoilage pass ({N_POPS:,} groups × {AVG_INV} goods):")
print(f"    Python time:       {py_ms:.1f} ms")
print(f"    Entries zeroed:    {n_zero:,}")

print(f"\n  Total recipe evaluations per tick:")
print(f"    Spoilage (pops):   {N_POPS * AVG_INV:>10,}")
print(f"    Production (bldg): {int(N_BUILDINGS * AVG_RECIPES):>10,}")
print(f"    Total:             {int(N_POPS * AVG_INV + N_BUILDINGS * AVG_RECIPES):>10,}")

print(f"\n  Rust speed estimates (×80 Python, ×150 with SIMD):")
print(f"    ×80:   {py_ms/80:.2f} ms for pop pass")
print(f"    ×150:  {py_ms/150:.2f} ms for pop pass")
print(f"    Channel clearing est. from script 01: ~4 ms")
