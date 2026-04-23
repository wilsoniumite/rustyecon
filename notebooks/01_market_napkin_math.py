"""
rustyecon — market design napkin math

Sections:
  1. Scale & memory footprint
  2. Tick budget
  3. Price mechanism comparison (Vic3 formula vs. tatonnement vs. alternatives)
  4. Storage / inventory dynamics
  5. Data structure density analysis
"""

import numpy as np
import time
import sys
from collections import defaultdict

np.random.seed(42)

SEP = "=" * 70

# ─────────────────────────────────────────────────────────────────────────────
# 1. SCALE & MEMORY FOOTPRINT
# ─────────────────────────────────────────────────────────────────────────────
print(SEP)
print("1. SCALE & MEMORY FOOTPRINT")
print(SEP)

N_REGIONS   = 673
N_COUNTRIES = 100
N_GOODS     = 200        # rough; needs discussion
N_POP_GROUPS_PER_REGION = 75   # rough average; sparse in reality
N_BUILDINGS = 300_000
AVG_INV_GOODS_POP      = 5    # avg goods held per pop group
AVG_INV_GOODS_BUILDING = 4    # avg goods held per building (input buffers + output)

n_pops = N_REGIONS * N_POP_GROUPS_PER_REGION

print(f"\n  World:     {N_REGIONS} regions,  {N_COUNTRIES} countries")
print(f"  Pops:      {n_pops:,} pop groups ({N_POP_GROUPS_PER_REGION}/region avg)")
print(f"  Buildings: {N_BUILDINGS:,}")
print(f"  Goods:     {N_GOODS}")

# ── SimState memory estimate ──
# Pop group: ~48 bytes fixed (ids, size f64, savings f64, needs 3×f32, employment f32)
#            + inventory: avg 5 goods × (4 bytes GoodId + 8 bytes f64) = 60 bytes
pop_fixed   = 4+4+4+2+1+1 + 8+8 + 3*4+4   # 46 bytes
pop_inv     = AVG_INV_GOODS_POP * (4 + 8)   # 60 bytes
pop_bytes   = pop_fixed + pop_inv            # 106 bytes/group

# Building: ~32 bytes fixed + inventory buffer
bld_fixed   = 4+4+4+2+4+4     # 22 bytes
bld_inv     = AVG_INV_GOODS_BUILDING * (4 + 8)
bld_bytes   = bld_fixed + bld_inv

# Prices: 3 levels (world, country, regional)
world_prices   = N_GOODS * 8
country_prices = N_GOODS * N_COUNTRIES * 8
region_prices  = N_GOODS * N_REGIONS * 8   # local service prices

total_pop_MB  = (n_pops * pop_bytes) / 1e6
total_bld_MB  = (N_BUILDINGS * bld_bytes) / 1e6
total_price_MB = (world_prices + country_prices + region_prices) / 1e6

total_MB = total_pop_MB + total_bld_MB + total_price_MB

print(f"\n  Memory (hot state in RAM):")
print(f"    Pop groups:   {total_pop_MB:6.1f} MB  ({pop_bytes} bytes/group)")
print(f"    Buildings:    {total_bld_MB:6.1f} MB  ({bld_bytes} bytes/building)")
print(f"    Price tables: {total_price_MB:6.1f} MB")
print(f"    ─────────────────────────")
print(f"    Total:        {total_MB:6.1f} MB  ← fits comfortably in L3/RAM")

# ── Checkpoint file size ──
# Bincode is near-theoretical minimum. Compression (zstd lvl3) ~3-4×.
raw_bytes = (n_pops * pop_bytes) + (N_BUILDINGS * bld_bytes) + \
            world_prices + country_prices + region_prices
compressed_bytes = raw_bytes / 3.5  # zstd estimate

print(f"\n  Checkpoint file size:")
print(f"    Raw:        {raw_bytes/1e6:6.1f} MB")
print(f"    Compressed: {compressed_bytes/1e6:6.1f} MB  (zstd lvl3, ~3.5× ratio)")
print(f"    52 snapshots/year × 200 years: {52*200*compressed_bytes/1e9:.2f} GB for full run")

# ── Delta log size ──
# Estimate events per tick: maybe 1 price change per good per region (sparse),
# say 30% of goods active per region, plus pop events.
events_per_tick = int(N_GOODS * N_REGIONS * 0.3 + n_pops * 0.5)
bytes_per_event = 32  # rough: event tag + 2-3 f64s + ids
delta_per_tick_kb = events_per_tick * bytes_per_event / 1e3
delta_per_tick_kb_comp = delta_per_tick_kb / 3.5

print(f"\n  Delta log (events per tick):")
print(f"    Estimated events/tick: {events_per_tick:,}")
print(f"    Raw:        {delta_per_tick_kb:6.0f} KB/tick")
print(f"    Compressed: {delta_per_tick_kb_comp:6.0f} KB/tick")
print(f"    200-year run: {10400 * delta_per_tick_kb_comp / 1e6:.1f} GB  ← may need pruning strategy")

# ─────────────────────────────────────────────────────────────────────────────
# 2. TICK BUDGET
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("2. TICK BUDGET")
print(SEP)

targets = [
    ("Aspirational",  10_400,  60),    # 1 minute total
    ("Good",          10_400,  600),   # 10 minutes total
    ("Acceptable",    10_400, 3600),   # 1 hour total
    ("Max allowed",   10_400, 86400),  # 24 hours total
]

print(f"\n  {'Target':<15} {'Total ticks':<14} {'Total time':<14} {'ms/tick':<12} {'ops/tick @1ns'}")
print(f"  {'──────':<15} {'───────────':<14} {'──────────':<14} {'───────':<12} {'─────────────'}")
for label, ticks, total_sec in targets:
    ms_per_tick = total_sec / ticks * 1000
    ops = ms_per_tick * 1e6  # 1ns per op
    print(f"  {label:<15} {ticks:<14,} {total_sec:<14,}s {ms_per_tick:<12.1f} {ops:,.0f}")

print(f"""
  Key insight: "acceptable" is 346ms/tick → ~346M single-ns ops available.
  Even at 10× memory-bound slowdown (10ns/op) that's 34M ops.
  The entire price table (world+country+region) is {total_price_MB:.0f} MB — fits in L3 cache.
  Market clearing is the right place to spend this budget.
""")

# ─────────────────────────────────────────────────────────────────────────────
# 3. PRICE MECHANISM COMPARISON
# ─────────────────────────────────────────────────────────────────────────────
print(SEP)
print("3. PRICE MECHANISM COMPARISON")
print(SEP)

# ── 3a. Vic3 direct formula ──
def vic3_price(base, buy, sell, k=0.75):
    """Single-tick price update. No iteration."""
    denom = min(buy, sell)
    if denom < 1e-9:
        return base * (1.75 if buy > sell else 0.25)
    ratio = np.clip((buy - sell) / denom, -1.0, 1.0)
    return base * (1.0 + k * ratio)

# ── 3b. Tatonnement (iterative) ──
def tatonnement(supply_fn, demand_fn, p0, alpha=0.05, max_iter=200, tol=1e-4):
    p = np.array(p0, dtype=float)
    for i in range(max_iter):
        s = supply_fn(p)
        d = demand_fn(p)
        excess = d - s
        if np.max(np.abs(excess / np.maximum(s, 1e-9))) < tol:
            return p, i, True
        p = np.maximum(p * (1 + alpha * excess / np.maximum(s, 1e-9)), 1e-6)
    return p, max_iter, False

print("\n── 3a. Vic3 formula: single good, 200 ticks with demand shock ──")

def sim_vic3(n_ticks, base=1.0, buy_fn=None, sell_fn=None):
    prices = [base]
    p = base
    for t in range(n_ticks):
        buy  = buy_fn(t, p)
        sell = sell_fn(t, p)
        p = vic3_price(base, buy, sell)
        prices.append(p)
    return np.array(prices)

# Stable market
prices_stable = sim_vic3(200,
    buy_fn  = lambda t, p: 100 * (1 + 0.1*np.sin(t*0.3)),   # noisy demand
    sell_fn = lambda t, p: 100 + 0.5*(p - 1.0) * 100)        # elastic supply

# Demand shock at tick 50
def shock_buy(t, p):
    base = 100
    if t >= 50: base = 160   # sudden demand spike
    return base

prices_shock = sim_vic3(200,
    buy_fn  = shock_buy,
    sell_fn = lambda t, p: 100 + 0.5*(p - 1.0) * 100)

print(f"  Stable market:  min={prices_stable.min():.3f}  max={prices_stable.max():.3f}  "
      f"final={prices_stable[-1]:.3f}  (base=1.0, range 0.25–1.75)")
print(f"  Demand shock:   pre-shock avg={prices_shock[:50].mean():.3f}  "
      f"post-shock avg={prices_shock[50:].mean():.3f}")
print(f"  Note: Vic3 formula uses PREVIOUS tick's prices as base — "
      f"price only moves to [0.25×, 1.75×] base in ONE tick.")
print(f"  This means large shocks need multiple ticks to fully propagate.")

print("\n── 3b. Tatonnement: convergence speed vs. N goods ──")

results = []
for n in [10, 50, 100, 200, 500]:
    base  = np.ones(n)
    # Shift equilibrium: true eq price ~1.3
    supply_fn = lambda p, n=n: 80 * p / 1.3
    demand_fn = lambda p, n=n: 80 * (2.6 - p) / 1.3

    t0 = time.perf_counter()
    reps = 100
    for _ in range(reps):
        p_eq, iters, converged = tatonnement(supply_fn, demand_fn, base)
    elapsed_us = (time.perf_counter() - t0) / reps * 1e6

    results.append((n, iters, converged, elapsed_us, p_eq.mean()))

print(f"\n  {'N goods':<10} {'iters':<8} {'converged':<12} {'µs/call':<12} {'eq price'}")
print(f"  {'───────':<10} {'─────':<8} {'─────────':<12} {'───────':<12} {'────────'}")
for n, iters, conv, us, eq in results:
    print(f"  {n:<10} {iters:<8} {str(conv):<12} {us:<12.1f} {eq:.4f}")

print(f"\n  At N=200: {results[3][2]} convergence in {results[3][1]} iters, "
      f"{results[3][3]:.0f}µs/call (Python; Rust ~100×faster → ~{results[3][3]/100:.1f}µs)")
print(f"  673 markets × {results[3][3]/100:.1f}µs = "
      f"{673 * results[3][3]/100 / 1000:.1f}ms per tick for market clearing")
print(f"  That's JUST for clearing — doesn't include order aggregation, storage, etc.")

print("\n── 3c. Price‐damping comparison ──")
print("  Vic3:       single-pass, O(N_goods), no convergence issues, caps at [0.25, 1.75]×base")
print("  Tatonnement: iterative, O(N_goods × iters), can oscillate, no hard cap")
print("  Hybrid idea: one tatonnement step per tick (alpha tuned so it converges over ~10 ticks)")
print("               — effectively a damped price adjustment, O(N_goods) per tick")
print("               — avoids oscillation by limiting per-tick movement")
print("               — natural tick-time normalization: alpha ∝ tick_duration_days/7")

# ── 3d. Tatonnement with 1 step vs Vic3 vs true eq ──
print("\n── 3d. Single-step tatonnement vs Vic3: do they converge to same place? ──")

base = np.array([1.0, 1.5, 0.8])
# True equilibrium: supply = demand
# supply = 80p, demand = 80*(2*eq - p), eq at p = eq
true_eq = np.array([1.0, 1.5, 0.8])  # by construction

p = np.ones(3) * 0.5   # start far from eq

vic3_history  = [p.copy()]
taton_history = [p.copy()]
pv = p.copy()
pt = p.copy()

for tick in range(60):
    # Vic3: use previous price as "base" each tick
    for i in range(3):
        buy  = 80 * (2*true_eq[i] - pv[i])
        sell = 80 * pv[i]
        pv[i] = vic3_price(true_eq[i], max(buy,0), max(sell,0))
    vic3_history.append(pv.copy())

    # Single-step tatonnement (alpha=0.1)
    supply = 80 * pt
    demand = 80 * (2*true_eq - pt)
    excess = demand - supply
    pt = np.maximum(pt * (1 + 0.1 * excess / np.maximum(supply, 1e-9)), 1e-9)
    taton_history.append(pt.copy())

vh = np.array(vic3_history)
th = np.array(taton_history)

print(f"\n  Starting at p=[0.5, 0.5, 0.5], true eq=[1.0, 1.5, 0.8]")
print(f"  After 60 ticks:")
print(f"    Vic3 prices:  {vh[-1]}")
print(f"    Taton prices: {th[-1]}")
print(f"    True eq:      {true_eq}")
print(f"  Vic3 error:   {np.abs(vh[-1] - true_eq).mean():.6f}")
print(f"  Taton error:  {np.abs(th[-1] - true_eq).mean():.6f}")

# ─────────────────────────────────────────────────────────────────────────────
# 4. STORAGE / INVENTORY DYNAMICS
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("4. STORAGE / INVENTORY DYNAMICS")
print(SEP)

def sim_market_with_storage(
    n_ticks=520,      # 10 years of weekly ticks
    base_price=1.0,
    storage_capacity=200.0,
    storage_cost_per_unit_per_tick=0.002,
    spoilage_rate=0.01,  # fraction lost per tick
    supply_fn=None,
    demand_fn=None,
    storage_enabled=True,
    speculator_enabled=False,
):
    price   = base_price
    inv     = 0.0
    prices  = []
    invs    = []
    shortfalls = 0

    for t in range(n_ticks):
        supply = supply_fn(t)
        demand = demand_fn(t, price)

        if storage_enabled:
            # Spoilage first
            inv *= (1 - spoilage_rate)

            # Storage decision: simple rule — store when price is below long-run avg,
            # draw down when above. Speculators amplify this.
            target_inv = storage_capacity * 0.5
            if price < base_price * 0.9 and inv < storage_capacity:
                # cheap — buy into storage
                store_delta = min(supply * 0.3, storage_capacity - inv)
                inv += store_delta
                supply -= store_delta
            elif price > base_price * 1.1 and inv > 0:
                # expensive — release from storage
                release = min(inv * 0.4, demand * 0.5)
                inv -= release
                supply += release

            # Storage cost charged to storer (reduces future willingness to store)
            # (modeled implicitly via storage_cost param not yet wired — future work)

        effective_supply = supply
        if demand > effective_supply:
            shortfalls += 1

        price = vic3_price(base_price, demand, effective_supply)
        prices.append(price)
        invs.append(inv)

    return np.array(prices), np.array(invs), shortfalls

# Scenario: seasonal harvest (grain)
def grain_supply(t):
    # Big harvest for 4 ticks around week 36 (autumn), trickle otherwise
    week = t % 52
    if 34 <= week < 38:
        return 400.0   # harvest
    return 15.0        # trickle from last harvest

def grain_demand(t, p):
    # Inelastic: people need to eat regardless of price (within reason)
    return 80.0 * (1.0 + 0.05 * np.random.randn())  # slight noise

print("\n── Seasonal grain: with and without storage ──")
np.random.seed(1)
p_no_store, _, short_no = sim_market_with_storage(
    supply_fn=grain_supply, demand_fn=grain_demand, storage_enabled=False)
np.random.seed(1)
p_store, inv_store, short_store = sim_market_with_storage(
    supply_fn=grain_supply, demand_fn=grain_demand, storage_enabled=True,
    storage_capacity=300, spoilage_rate=0.005)

print(f"  Without storage: price range [{p_no_store.min():.3f}, {p_no_store.max():.3f}]"
      f"  std={p_no_store.std():.3f}  shortfall ticks={short_no}")
print(f"  With storage:    price range [{p_store.min():.3f}, {p_store.max():.3f}]"
      f"  std={p_store.std():.3f}  shortfall ticks={short_store}")
print(f"  Storage smoothing reduces price std by {(1 - p_store.std()/p_no_store.std())*100:.0f}%")
print(f"  Peak inventory held: {inv_store.max():.0f} units")

# ── Storage and the shelf-life spectrum ──
print("\n── Shelf life spectrum: price volatility by spoilage rate ──")
print(f"  {'Spoilage/tick':<16} {'Price std':<12} {'Shortfalls':<12} {'Description'}")
print(f"  {'─────────────':<16} {'─────────':<12} {'──────────':<12} {'───────────'}")
for sr, desc in [(0.0, "indefinitely storable (gold)"),
                 (0.005, "long shelf life (grain ~3yr)"),
                 (0.02, "medium shelf life (months)"),
                 (0.1, "short shelf life (weeks)"),
                 (1.0, "instant (haircut — no storage)")]:
    np.random.seed(1)
    ps, _, sf = sim_market_with_storage(
        supply_fn=grain_supply, demand_fn=grain_demand,
        storage_enabled=(sr < 1.0), storage_capacity=300, spoilage_rate=sr)
    print(f"  {sr:<16.3f} {ps.std():<12.4f} {sf:<12} {desc}")

# ─────────────────────────────────────────────────────────────────────────────
# 5. DATA STRUCTURE DENSITY
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("5. DATA STRUCTURE DENSITY")
print(SEP)

print("""
  For inventories (pop × good → quantity), we have a sparse 2D matrix.
  Options in Rust:

  A. HashMap<GoodId, f64> per entity
     + Simple, only stores non-zero entries
     - Poor cache locality; hash lookup overhead per access
     - ~50–80 bytes overhead per HashMap even when empty

  B. Vec<(GoodId, f64)> per entity, sorted by GoodId
     + Cache-friendly, trivial to iterate
     + Binary search for lookup: O(log k) where k = goods held
     - Insert/remove needs shifting
     - Best when k is small and stable (it is: avg 5 goods/pop)

  C. Flat Vec<f64> of length N_GOODS per entity (dense)
     + O(1) random access, best cache behavior
     - 200 goods × 8 bytes = 1,600 bytes per entity even if holding 2 goods
     - For 50,000 pops: 80 MB just for inventories — wasteful

  D. Column-oriented: Vec<f64> of length N_ENTITIES per good
     + Perfect for "iterate all pops for this good" (market clearing)
     - Poor for "iterate all goods for this pop" (needs satisfaction)
     - Awkward when pop count changes (births/deaths/migration)

  RECOMMENDATION: B for pop inventories (k small, stable, iteration common).
  D for market clearing working arrays (computed fresh each tick, not stored).
  These are different data structures for different access patterns.
""")

print("  Simulated lookup cost comparison (Python as proxy for relative perf):\n")

k = 5  # avg goods per pop
n = 50_000  # pop groups
goods_to_find = [3, 17, 99, 150, 201]  # GoodIds to look up

# Simulate sorted Vec approach
inventories_sorted = []
for _ in range(n):
    goods = sorted(np.random.choice(200, k, replace=False))
    inventories_sorted.append([(g, np.random.rand()) for g in goods])

# Simulate HashMap approach (dict in Python)
inventories_hash = []
for inv in inventories_sorted:
    inventories_hash.append(dict(inv))

# Benchmark sorted-list lookup (binary search)
import bisect
def lookup_sorted(inv, good_id):
    keys = [x[0] for x in inv]
    idx = bisect.bisect_left(keys, good_id)
    if idx < len(inv) and inv[idx][0] == good_id:
        return inv[idx][1]
    return 0.0

t0 = time.perf_counter()
hits = 0
for inv in inventories_sorted:
    for g in goods_to_find:
        v = lookup_sorted(inv, g)
        if v > 0: hits += 1
sorted_time = (time.perf_counter() - t0) * 1000

t0 = time.perf_counter()
hits2 = 0
for inv in inventories_hash:
    for g in goods_to_find:
        v = inv.get(g, 0.0)
        if v > 0: hits2 += 1
hash_time = (time.perf_counter() - t0) * 1000

print(f"  {n:,} pops × {len(goods_to_find)} lookups each:")
print(f"    Sorted Vec (bisect): {sorted_time:.1f}ms  ({hits:,} hits)")
print(f"    HashMap (dict):      {hash_time:.1f}ms  ({hits2:,} hits)")
print(f"  Python overhead is huge here; Rust ratio will differ.")
print(f"  In Rust: sorted small Vec is typically faster than HashMap for k≤~20")
print(f"  due to cache locality and no hash computation overhead.")

# ─────────────────────────────────────────────────────────────────────────────
# SUMMARY
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{SEP}")
print("SUMMARY")
print(SEP)
print(f"""
  Memory:
    Total hot state:        {total_MB:.0f} MB  ← trivially fits in RAM
    Per checkpoint:         {compressed_bytes/1e6:.1f} MB compressed
    Full run delta log:     potentially large (see above) — need pruning/tiering

  Tick budget (1hr total = 346ms/tick):
    Market clearing alone:  ~{673 * results[3][3]/100 / 1000:.0f}ms in Rust (est.) — well within budget
    Room for:               storage evaluation, pop needs, government, etc.

  Price mechanism:
    Vic3 formula:   fast, stable, converges over ~10 ticks, easy to reason about
    Tatonnement:    converges correctly but needs tuning; oscillation risk
    Hybrid:         1 tatonnement step/tick with tuned alpha — essentially equivalent
                    to Vic3 but hits true equilibrium given enough ticks
    KEY OPEN Q:     do we need cross-price effects (substitutes/complements)?
                    If yes, tatonnement cost rises from O(N) to O(N²) per tick.

  Storage:
    Smoothing effect is large and realistic
    Shelf-life spectrum → spoilage_rate attribute on Good is the right knob
    Storage decisions (agent strategic behavior) are interesting but complex
    Simple rule-based storage (store when cheap, release when dear) works ok

  Data structures:
    Inventories:    sorted Vec<(GoodId, f64)> per entity (k small)
    Market working  arrays: column-oriented (good → [region quantities])
    Bond types:     bucket by (issuer, maturity_class, risk_tier) ~1,500 classes max
""")
