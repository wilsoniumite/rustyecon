# The demo's second pass: horses and fodder on the map

Dated 2026-09-30. Written at step D2.6 on branch `demo-v2`, after the build (D2.1–D2.4b), the scored
wave and one bounded verification with its fix round (D2.5). This file is the guide to the second
pass: what the world is now, how to run it, how to read its new lenses, and what the battery and
the long run found. The design and the as-built log are [WORLD-V2.md](WORLD-V2.md); the scored
results are [v2/results/README.md](v2/results/README.md); v1 is [WORLD.md](WORLD.md).

Nothing here is research. Every number in the world is `Assumed("illustrative demo …")`, and the
tape's name carries `[illustrative]`, so nothing it produces is scored against history or cited
(R4, R5). The battery and the long run check the agents against their own oracle, not against the
past.

## 1. What the world is now

The same 93 historic counties of the United Kingdom, 1750 to the first tick of 1901, with v1's
population, land, technology and relief history. What changed is the machine side of each county.
In v1 a county's machine-hours were a flow, bought and used up in the tick. Now they come from a
stock of working animals, and the stock has to be bred, fed and held:

- **land grows fodder**, from land alone (no labour, no horse-days, so no loop);
- **a maker breeds horses** from its own horse-days, labour, pasture and fodder, and sells them;
- **a capacity desk holds the herd**, buys new heads to keep it at its target, feeds it, and hires
  out horse-days, fodder and handler included ("wet");
- **the good desk** buys horse-days and labour and makes the good, as in v1;
- the owners sell land, the workers sell labour and buy baskets, and the provider funds one
  basket of support for every potential worker out of the rent roll, as in v1.

That is GOODS-CHAIN's stage v2a.1, rule A, on every county (decision 320). Each county has six
actors and six markets: labour, land, fodder, horses, horse-days and the good. The settings are the
same everywhere: the horse wears out at 8% a year, fodder takes 85% of a horse-day's land, a head
gives 52 horse-days a year, the dials are C2g, and the clock is 52 ticks a year. The maker withholds
its finished heads when its markup falls below ψ = 0.25 (the reservation, IDLE.md).

v1's history moves the same levers. Its ramps on land per machine-hour (b), labour per machine-hour
(λ) and own machine-hours (a) are mapped onto the chain's coefficients so that at every one of the
25,480 step dates the county's equilibrium is v1's exactly (1g against 1a within 8e-16). So the
map's oracle is v1's, and what is new is how the agents get there: through a herd that takes years
to build.

The tape is `tapes/demo-gb-v2.ron` (14.8 MB, 33,332 events, pin `0x45b7c1201f8ae633`). v1's tape,
`tapes/demo-gb.ron`, and its pin `0xfad880fe08d06645` are unchanged.

## 2. How to run it

**Your look at the map (Windows, PowerShell, from the worktree `D:/rustyecon-wt/d2`):**

```powershell
$env:CARGO_TARGET_DIR = 'D:/rustyecon-targets/d2-hand'
cargo run --release -p rustyecon-gui -- tapes/demo-gb-v2.ron
```

The first build takes some minutes. On WSL or Linux the same command runs with
`CARGO_TARGET_DIR=... cargo run --release -p rustyecon-gui -- tapes/demo-gb-v2.ron`. v1 opens the
same way with `tapes/demo-gb.ron`.

**In the window.** The tape opens paused at 1750 on "Wage in land".
- Space runs and pauses; `.` steps a tick; the toolbar steps a year or runs to a tick or date.
- `1`–`9` and `0` pick the first ten lenses, which on this tape are the eleven "Horses and fodder"
  lenses but the last; `[` and `]` step through all 36. The selector above the sidebar lists them
  by group.
- Drag pans, the wheel zooms about the pointer, a double click fits the map. The map fits itself
  again when the window changes size, until you pan or zoom.
- Hover a county for its value, rank, run and tick. Click it to select it: the sidebar shows its
  card, and the inspector, outliner and plots follow it.
- The timeline moves the cursor back through the record; the map then shows that tick and says so.

A run to 1901 takes about 28 seconds in the GUI on Windows (D2.3's measure, release build, a
machine shared with other builds: 281–284 ticks a second), against 16.5 s for v1. The record at 1901
holds 9,056 series; a test process holding it alone used 822 MB (D2.5's review, WSL, lean
catalogue).

**Without the window.**
- `rustyecon run tapes/demo-gb-v2.ron --until 7852 --hashes out.txt` runs it to the first tick of
  1901 and writes each tick's state hash; the last is the pin.
- `rustyecon worldgen worlds/demo-gb --stage v2a1` compiles the tape from `worlds/demo-gb/`'s
  tables; without `--stage` it writes v1's tape bit for bit.
- `scripts/gate.sh` runs the long run scored against the registration (`demo_v2_runs_to_1901`),
  and `scripts/gui.sh` holds the map's lenses to the engine and every lens to its domain over the
  whole run.

## 3. The new lenses

The v2 tape takes its own lens table, `worlds/demo-gb/lenses-v2a1.csv`: v1's 25 lenses and 11 new
ones, 36 in all. The new ones come first in the selector, as "Horses and fodder". Every domain is
fixed for the whole run, so a colour means the same number in 1750 and in 1900.

| lens | what it shows | how to read it |
|---|---|---|
| Horses per head | the county's working stock (the desk's herd and the maker's breeding stock) per unit of population | where the working stock is dense; 1.2 (Dunbartonshire 1900) to 7.1 (Northumberland 1889) in the engine's run |
| Horses per head since 1750 | its change in log | the herd growing, or thinning where people grow faster than horses |
| Horses against their equilibrium | ln(heads / heads\*), the herd against the oracle's at the params in force | capital's lag: purple is short. See §5 |
| Horses against the desk's own target | ln(held / target) | whether the desk is ordering to its own plan: near 0 means it is |
| Fodder's price in rent | p_f / r | 0.26 to 0.69; at rest it is 0.85 of land per horse-day |
| Horse price over its replacement cost | the maker's markup | see §4 |
| Horse-day markup over full cost | p_h / (running cost + wear) − 1 | the rent a horse-day earns over its cost; positive where horses are short |
| Land to fodder | the share of cleared land that grows fodder | 0.49 (West Riding 1900) to 0.72 (Norfolk 1802) |
| Land to the working stock | fodder and pasture together | the rest of the land is homes |
| Ticks the maker withheld | ticks of the last 52 with the markup below ψ | 0 in every county on this history |
| Ticks the horse market was idle | ticks of the last 52 in which no horse traded | 0 in every county on this history |

v1's lenses carry over, six with a changed reading (WORLD-V2 §7.1). The horse-day's price replaces
v1's machine-hour price; "Ticks without trade" counts the five markets that must trade and leaves
the horse market to its own lens; land per horse-day (b) and labour per horse-day (λ) are the folds
of the chain's coefficients; and the two oracle lenses now have values, from worldgen's
`oracle_gap` (1g at each county's params in force, never fed to an agent, R13).

Nine more of v1's lenses keep v1's reading with a wider domain, from D2.5: wage in goods, rent in
goods, the land and labour shares, the automation frontier, the good's price, the relief burden,
the shortfall and rationing. The second pass's run departs further from its oracle than v1's did,
so v1's domains ran out (rationing passed its old top in 26% of county-ticks). Each domain now
holds the engine's own run to 1901 with a margin, and a test holds it there every tick
(`lens_v2_domains_hold_the_engines_long_run`). Two notes were false on this tape and are rewritten:
- **Relief shortfall** is not 0 everywhere. Six thinly funded counties fall short on some ticks:
  Surrey (up to 0.21 of support unpaid, in 1834), Armagh, Caernarfonshire, Sussex, Down and Tyrone
  (O82).
- **Rationing** reaches 0.17 (Lanarkshire, end of 1850). Above 0.05 it is nearly always the owners'
  land offered and not sold.

## 4. How to read the horse's price

The lens "Horse price over its replacement cost" is the maker's net markup: what a head sells for,
net of the wear on the maker's own horse-days, over what it costs the maker to breed one at the
tick's posted prices. 1 is the replacement cost, which is where the oracle puts it.

- **Above 1**: horses sell for more than they cost to breed. That is the signal that draws breeding.
  In the engine's run it goes up to 1.66 (Lanarkshire, 1854), in the counties where the herd lags
  most.
- **Below 1**: horses sell for less than they cost. The lowest on this history is 0.981 (Cheshire,
  1880).
- **ψ = 0.25** is marked on the legend. Below it the maker offers none of its finished heads. So on
  any tick when a horse trades, the markup is at least ψ.
- **No value on a tick when no horse traded.** A market with no trade still posts a price, and that
  price can drift far from any value while it waits. The lens does not show it: the county is grey,
  and its card says "idle: no horse traded this tick; the posted price is X of the replacement cost"
  (decision 333; O47, O54). On this history no horse market idles, so every county has a value
  every tick; the lens matters under an edit, or in the battery's gluts.
- **The battery's lows are posted prices.** In the battery the posted price falls to 0.200 of its
  target (West Riding 1900, J_b × 0.5), and the maker's markup to 0.212. Both are posted on ticks
  when the maker withheld and no horse traded, which this lens would not show. In that West Riding
  run the lowest price on a tick with a trade is 0.219 of target, and the lowest markup 0.250
  (D2.5's review, Windows, every tick written).

The horse-day's markup uses the posted horse price on every tick, idle or not, because that is the
cost the capacity desk's own rule reads (its full cost is running cost plus δ·p_K/κ).

## 5. How to read capital's lag

On this history the herd is often short of its equilibrium, for decades, and most in the counties
that grow fastest. The map is built to show that as what it is: a stock catching up with a moving
target.

**Why it lags.** A capacity desk sets its target from its own coin (D-G14). Its herd grows only as
the scarcity rent it earns on horse-days builds up as coin. v1's history grows output by up to 3.3%
a year over a decade in the coal and cotton counties, faster than a herd built that way can follow.
Nothing fails: in the battery every county-date converges, but it takes a median of 3,010 ticks (58
years) and up to 5,661 (109 years) to come within tolerance, which is the scale of the whole
history. So the industrial counties never close the gap before the next ramp.

**What to look at.**
1. **Horses against their equilibrium** shows where the herd lags. In the engine's run the lowest
   are Glamorgan 0.717 of equilibrium (1863), Lanarkshire 0.725 (1850), Durham 0.728 (1863),
   Renfrewshire 0.743 (1779) and Lancashire 0.743 (1851), every tick read. 20 of the 26 coal and
   textile counties are still more than 5% short in 1901. No Highland county falls below 0.9.
2. **Horses against the desk's own target** stays near 0 (−0.14 to +0.06). The desk is ordering to
   its own plan; it is the plan that lags.
3. **Horse-day markup over full cost** is positive where the herd is short, up to +0.23 (Glamorgan,
   1873): the rent that is buying more horses.
4. **Gap to the county's own equilibrium** is the largest gap over 17 observables, in multiples of
   the probe's tolerance (1e-3 in log). Its legend writes 1, 10, 100 and 1,000: the tolerance, 1%,
   10% and 100% in log. The median county's median is 117 (about 12% in log), against v1's 11: the
   second pass runs about 11 times further from its equilibrium than v1 did. Glamorgan reaches 544.
   A county within the tolerance shows below the scale.
5. **The county's card** names the three observables furthest from their equilibrium, with signs,
   and the herd's three readings above. In Lancashire around 1850 the horse's and the horse-day's
   prices are well above their equilibrium and the heads well below.

Lancashire's herd shows the pattern: 0.76 of its equilibrium by the late 1770s, 0.86 by 1800, 0.74
in 1851 through the textile, steam and railway ramps, 0.89 by the mid-1880s, and 0.84 in 1901.

## 6. What the battery and the long run found

Registered before any engine run of the stage ([v2/registration.md](v2/registration.md), with
amendment A1 setting L per county-date), then run on the engine (D2.4b) and scored by a scorer
committed before the first job. Details and tables: [v2/results/README.md](v2/results/README.md).

- **GO at all 558 county-dates** (every county on 1 January 1750, 1800, 1825, 1850, 1875 and 1900).
  All 21 scored lines hold and no refutation criterion is hit.
- **The battery.** P2.2a's 93 runs at each county-date on funded targets, 51,260 runs: every one
  converges, in the mirror's class. The ticks to tolerance are the mirror's to the tick in 51,252
  runs and one tick off in eight. Every base kick set decays.
- **The long run**, 1750 to the first tick of 1901, is the mirror's to rounding in every county:
  - no dead, idle, no-order or withheld tick, and the ledger closes every tick;
  - 14,502 ticks of transfer shortfall in six thinly funded counties (Surrey 5,593, Armagh 3,619,
    Caernarfonshire 2,320, Sussex 1,479, Down 1,115, Tyrone 376), as registered (O82). In Surrey
    they are 71% of the history, with a mean of 10% of the transfer unpaid on those ticks;
  - the gap and the lag of §5.
- **Not as predicted, and reported, not scored:** the kick sets' fitted rate reads the kick's early
  decay, not the slowest mode (O91); the battery's highest horse price is not read for these
  instances (O92); and at genesis the six b × 2 trace diffs part after the maker withholds (O90).

"Healthy" in the results means no dead, idle or withheld tick and a closed ledger. It does not mean
near equilibrium: see §5.

## 7. What D2.5's verification changed

Two bounded reviews read the pass at `d015fca`. The world review found that the world and its
economics hold, with six minor findings, all about wording and disclosure. The map review found
three majors and six minors in the map. The fix round (D2.5) answered each:

- **The sidebar and the card** were cut at the app's own window sizes. The card and the lens's
  description now scroll above a ranked table that keeps its rows, and every line wraps to the
  sidebar. Tested in the whole app at 1,024 × 768 and 1,280 × 800.
- **v1's lens domains and notes** ran out on this tape (§3). They now hold the engine's run.
- **The whole app on the v2 tape** had no test, so an app showing v1's 25 lenses would have passed.
  It now has one, and the colour and credit checks run on the v2 store.
- **The legend** wraps its header, keeps ψ in its own row, always writes its ends (and the gap
  lens's 10 and 1,000), and keeps the scale bar clear of it; the title wraps; `|` and `×` replace
  two glyphs the bundled font lacks; the fitted view follows a resized window; a ranked value never
  wraps.
- **Disclosures** in the results and the registration's amendment are corrected (v2/results,
  "Corrected at D2.5").

The reviews and their evidence are in `D:/rustyecon-d2/review-world/` and
`D:/rustyecon-d2/review-map/`; the fix round's in `D:/rustyecon-d2/fix-report/`.

## 8. What the next pass adds

- **Stage v2a.1b: rule B's horse loop with plants, on funded and registered counties.** Fodder then
  takes horse-days, a loop of produced inputs, which P2.2b made GO on one county, chain8, with
  CAPACITY's plant on every loop desk and the reservation (docs/probe/LOOPS.md). A loop enters only
  through its own registration in its own funded county (decision 308). chain8 was funded only by
  moving N, so the next pass first searches the 93 counties for funded rule-B points, then registers
  the mirror's predictions for each such county before any engine run (O83).
- **Stage v2a.2: categories**, after O42. Grain to food, shelter and care, as unit 1b's categories
  on the one task line. It needs an ex-post category desk first: its rule, its nesting on one
  segment and a mirror (O42), since assignment after the fact is what the chain uses.
- **Later stages** (GOODS-CHAIN §5): land classes (v2a.3), minerals and engines (v2a.4, which
  replaces the horse that stands for every working stock, O87), textiles (v2a.5). Carriers and
  trade between counties wait for transport desks and an oracle with trade.
- **Open on this pass:** capital's lag itself (O81; its candidate fix is Phase 3's entry rule), the
  history's pace, bounded for a flow machine (O88), a chart of a lens over the run (O89), and the
  GUI's frame time at this size (O85, O86).
