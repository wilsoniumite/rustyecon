# The demo world: the United Kingdom's historic counties, 1750–1900

Dated 2026-09-27. Written at the stage "world design" on branch `demo-world`, after D.1 (the atlas,
`data/atlas/`), and committed at D.2 with the compiler that turns it into `tapes/demo-gb.ron`
(§8).

This is an **illustrative** world, built so that the GUI's map has something to show: 93
counties, each running the probe's four roles, pushed along 150 years of dated history, with
lenses that colour the map as the run goes. The user asked for "a nice looking map of the UK,
and a fairly complex setup of regions, goods, and history, not necessarily perfectly accurate but
enough to get a feel for it", with Victoria-style map modes. This file says what the world is,
where each number comes from, how the history moves it, what the map's lenses show, and how
goods, machine types and carriers switch on later; §8 says how the compiler reads it and what
the compiled tape does when it runs.

Nothing from this world is research. Every number in it is
`Assumed("illustrative demo, 2026-09-27: <why>")`, and the tape's name carries the marker
`[illustrative]` (§8), so nothing it produces may be scored or cited (R4, R5, R11). As built at
D.4 (§8): `certify` seals any run of a tape whose name carries the marker UNSCORED, never PASS,
and refuses a tape whose bases say illustrative once its name has lost the marker; the compiler
writes illustrative worlds only. The scorecard's refusal waits for Phase 6, and citation is kept
out by its readers: those guards are procedural (docs/GUI.md U5).

The tables are in `worlds/demo-gb/`:

| File | What it is |
|---|---|
| `world.csv` | The tape's name and basis marker, the clock, the stepping rule and the dials every county shares (§8). |
| `counties.csv` | The rough facts: population benchmarks, area, and 0–1 tags for each county (§3.1). |
| `derive.py` | Writes the next two files from `counties.csv`. Standard library only; `--check` compares. |
| `regions.csv` | Each county's instance at 1750-01-01, with its tags and a basis note (§3). |
| `history.csv` | The dated ramps, 1750–1901, that move the instances (§4). |
| `lenses.csv` | The map's lenses: measure, unit, scale, reference, domain and inputs (§6). |

## 1. The world

- **93 counties**, the atlas's regions and keys (D7, `county.<chapman>`): England 41 (38
  counties and Yorkshire's three ridings, ruling 7), Wales 13 (with Monmouthshire), Scotland 33
  (Ross and Cromarty as one), Northern Ireland 6.
- **Northern Ireland is in.** A permissively licensed source exists: HCBP Definition B's UK file
  covers the six counties under the same terms as Great Britain ("free of charge for all
  personal, educational, non-commercial and commercial use", acknowledgement requested), and the
  atlas already carries them (data/atlas/README.md). They share no border with Great Britain and
  there is no trade yet, so they can be dropped without touching anything else. Before 1801 they
  lay in the Kingdom of Ireland; the rest of Ireland is not in the atlas and not in the world.
- **Each county is its own node**, quoting in one coin, with the four markets of the Appendix B
  economy (labour, land, machine services, the good) and the probe's four roles: `GoodDesk`,
  `MachDesk`, `Provider` and `Workers` (docs/probe/RULES.md §2). There are no channels, so no
  county trades with another. One category of goods, one machine type (§7 says how more come).
- **The dials are C2**, the probe's registered set, unchanged and shared by every county: price
  rates 5.2 (labour), 2.6 (the good), 1.3 (land, machines) a year; technique 2.6; spending 13;
  desk turnover 5.2; tilts 0; planned-x Leontief; `Saturate` (RULES §3).
- **The clock**: 52 ticks a year from 1750-01-01 to 1901-01-01, 7,851 ticks.
- **Genesis** puts every county at its own oracle point (unit 1a at (ρ, δ, J_b) = (0, 1, 1)),
  exactly as `tapes/appb.ron` is made (RULES §4): the oracle's prices, the human share 1 − x\*,
  one tick's output held by each desk, and each actor's stationary coin under the dials.
  Prices are scaled so that the good costs 1 coin in every county at genesis; the roles are
  homogeneous of degree zero in prices and coin, so the scale changes no real quantity.

## 2. The economy on each county, and why the base departs from Appendix B

The probe found the roles converge on the Appendix B instance (N = 4, T = 10 a tick, h = 1,
a = 0.3, λ = 0.05, b = 0.4, η = 1, g0 = 0.2, g1 = 0.8, k = 1, χ_max = 1). As a picture of 1750
that instance is odd: 86% of tasks done by machines and a labour share of 7%. The demo's base
county keeps its structure and moves four numbers:

| | Appendix B | demo base | why |
|---|---|---|---|
| η | 1.0 | 2.0 | machines are dearer at every task, so fewer tasks are automated in 1750 |
| λ | 0.05 | 0.03 | an organic economy's "machines" (horses, mills, tools) take little labour to make ... |
| b | 0.4 | 0.8 | ... and much land: fodder, wood and water sites |
| h | 1.0 | 0.15 | a basket's direct claim on land is small; land goes mostly to machine services |
| N/T | 0.4 | 0.45 | at the median county's density |

At N/T = 0.45 the oracle gives the base county x\* = 0.70, w/r = 1.86, w/P_s = 1.22, a labour
share of factor income of 0.40, participation 0.80 and output 2.42 goods a tick per unit of N.

**Why these and not others.** The Appendix B economy has one hard constraint beyond an interior
equilibrium: the provider must fund one basket for every potential worker out of rent,
T·r > N·P_s (the oracle's `funded`). Income per head is (Y/N)·P_s and support is P_s, so the
land share is at least N/Y. A 1750-like labour share of 0.3–0.45 therefore needs output per head
of 1.5–1.8 or more, which needs a small h. A random search of 200,000 instances (scratch, §9)
found labour shares above 0.40, with x\* in [0.35, 0.70], participation below 0.85 and a funding
margin of 0.3, only at h below 0.3. The base sits there.

**How the county's numbers move its equilibrium**, from the oracle at the base (a 10% move each):

| move | x\* | w/r | w/P_s | Y/N | labour share |
|---|---|---|---|---|---|
| η −10% | +0.016 | −9% | 0 | +5.9% | −2.2 pts |
| b −10% | +0.018 | −8% | +0.2% | +5.5% | −2.0 pts |
| λ −10% | −0.002 | −0.9% | −0.2% | +0.3% | −0.2 pts |
| T +10% | +0.018 | +2.0% | +1.2% | +6.3% | −1.5 pts |
| h −10% | +0.002 | +0.2% | +1.1% | +1.3% | +0.3 pts |
| χ_max −10% | −0.020 | −2.3% | −1.4% | +4.0% | +1.8 pts |
| N +10% | −0.019 | −2.1% | −1.2% | −5.8% | +1.6 pts |

Cheaper machines, by task (η) or by input (b), automate more tasks, lower the wage against land,
raise output and move income to land. More land per head raises everything but the labour share.
λ barely matters at 0.03: recursive automation has already pinned the machine price to land
through b (the paper's λ → 0 limit, w = γ(x\*)·b·r/(1 − a)).

## 3. The region table

### 3.1 The facts: `counties.csv`

One row per county: key, Chapman code, HCS code, name, nation, area, five population
benchmarks and fifteen tags. Keys, codes, names and nations are the atlas's, and the compiler
checks them against it (§8); at D.2 the ridings took the atlas's names ("West Riding of
Yorkshire" for "Yorkshire West Riding"), and `derive.py` carried them into `regions.csv`.

- **Areas** are the atlas's (`area_km2` in `data/atlas/gb.atlas.ron`, HCBP Definition B with the
  riding split).
- **Populations**, in thousands, are rough: rounded figures of the order found in the standard
  county estimates for c.1750 (Deane and Cole's for England and Wales, Webster's 1755 census for
  Scotland) and in the censuses of 1801, 1851 and 1901, written from general knowledge and
  **not transcribed** from any table. The Irish figures before 1821 are conjecture, and the six
  counties carry an 1841 benchmark so that the Famine shows. Totals: about 6.0 million for
  England in 1750, 0.48 million for Wales with Monmouthshire, 1.26 million for Scotland; 37
  million for Great Britain in 1901. Take them as orders of magnitude.
- **Tags** are judgements on a 0–1 scale, one per feature the history reads: `upland` (moor and
  mountain share), `coal` (coalfield weight), `textile` with `textile_kind` and `textile_from`
  (factory textiles: cotton, wool, linen, hosiery, and silk, pottery, ribbons, carpets, boots),
  `eng` (engineering and iron), `metro` (a capital's reach: Middlesex 1, Surrey 0.8, Kent,
  Sussex and Midlothian 0.3, Essex 0.2, Buckinghamshire and Hertfordshire 0.1), `port`
  (non-zero exactly where the atlas has a port site, graded by size), `canal`, `openfield`
  (Midland open fields), `improver` (agricultural improvement), `fen`, `mining` (copper, tin
  and lead), `slate`, `highland` (clearances), `speen` (Speenhamland allowance counties) and
  `kelp`. They follow common knowledge of where things
  were: the Lancashire and Clyde cotton districts, the West Riding's wool, Tyne and Wear coal,
  the Black Country, South Wales iron and coal, Dundee and Belfast linen, Cornish copper,
  Welsh slate, the Fens, the Highlands. None is measured.

### 3.2 The rules: `regions.csv`

`derive.py` turns each row into the county's 1750 instance. Flows are per year, as the tape
registers them (the roles read them per tick at 52 a year).

| column | unit | rule | why |
|---|---|---|---|
| `workers` (N) | FlowPerYear | population c.1750 in thousands ÷ 25, a tick, × 52 | N is potential hours; the economy has constant returns, so the scale only sets the size of the numbers |
| `land` (T) | FlowPerYear | N ÷ (N/T), with N/T = 0.45 · clamp((d/d_med)^0.25, 0.7, 1.3), then × (1 + 0.4·port) | d is people per effective km², where an upland km² counts 1 − 0.7·upland; real densities span 226-fold (Sutherland's 3.8 people a km² to Middlesex's 869), the model's N/T 1.7-fold (0.32 in the East Riding to 0.55 in Surrey), compressed so that every county stays inside the tested region; a harbour is a site, so a port adds land services |
| `space` (h) | Dimensionless | 0.15 · (1 + upland) · (1 + 0.25·metro) | a basket needs more poor land in the uplands, and more space in London |
| `eta`, `g0`, `g1`, `k` | Dimensionless | 2.0, 0.2, 0.8, 1.0 everywhere | the schedule's shape is Appendix B's; η is §2's; regional differences in η come from the history |
| `a` | Dimensionless | 0.3 | Appendix B's |
| `lam` (λ) | Dimensionless | 0.03 · (1 − 0.2·coal) · (1 − 0.1·eng) | coalfields and engineering towns make machines with less labour |
| `b` | Dimensionless | 0.8 · (1 − 0.3·coal) | coalfields: cheaper machine services, since coal stands in for land-grown fuel |
| `chi_max` | Dimensionless | 1.0 · (1 + 0.3·upland) | uplands: a larger work cost against dependence, so lower participation |
| `categories`, `machine_types`, `carriers` | keys | `good`, `mach`, empty | reserved (§7) |
| `why` | text | the county's own derivation | becomes the basis text of its params |

A county's params get the basis `Assumed("illustrative demo, 2026-09-27: <why>")`, for example
Lancashire's: "pop c.1750 ~297k on 4918 km2 (upland 0.3); N/T 0.377 a tick (density 1.90x the
median; compressed ^0.25 within (0.7, 1.3)); upland: h x1.30; chi x1.09; coal 0.8 / eng 0.8:
b x0.76; lam x0.77; port 1: T x1.40".

### 3.3 The check: every county, every step

The scratch checker (§9) reads `regions.csv` and `history.csv`, composes each county's history
exactly as §4.3 says the compiler must, and solves oracle unit 1a at genesis and after every
step: 93 counties and 25,480 step dates. On the tables as committed:

- **Interior and unique at every step.** Every solve is `Interior`, and n_D − n_S changes sign
  exactly once on a 401-point grid of x in (0, 1), as unit 1a's single crossing says it must.
  No other regime, no solve error.
- **Funded with margin**: the provider's own baskets per unit of N are at least 0.119
  (Surrey, 1834); the median county's is 0.56 in 1750 and 1.29 in 1900.
- **Participation interior**: N_a/N between 0.564 (Inverness-shire, 1901) and 0.870 (the East
  Riding, 1840); never saturated, as the probe's instance was not.
- **x\* between 0.640 (Surrey, 1840) and 0.919 (Lanarkshire, 1900)**, well inside (0, 1).
- **Inside the probe's GO region at C2.** The probe's battery (its 57 runs, with the b shocks
  scaled from the county's own b, and ten 1e-9 kicks of w, r, p_m, p and s), run on every
  county's instance in force on 1 January of 1750, 1800, 1825, 1850, 1875 and 1900, at the
  probe's L = 20,000: **GO on all 558 instances**. Every run converged (57 of 57 each, 31,806
  runs), and every kick decayed (the largest D̂ in the second half of a kicked run was 6.2e-6 of
  its start). The slowest run reached tolerance in 430 ticks (Dunbartonshire in 1900, r × 2),
  against the probe's 677; the most dead ticks in one run were 102 (Cheshire in 1850,
  JA(0.5)), against 111. Five instances' x\*/2 runs had a tick with no goods cleared, the
  probe's known keep-first case (REPORT §5); the history never displaces x\* like that. The
  probe's GO was established on the Appendix B instance alone; this is the evidence that each
  demo instance shares it.

A sample of the report (`oracle-path.csv`, §9, has every county at the first step of each
decade and at the end of its history), per tick in the model's units:

| county | x\* 1750 | w/r | Y | N_a | x\* 1901 | w/r | Y | N_a |
|---|---|---|---|---|---|---|---|---|
| Middlesex | 0.709 | 1.87 | 63.2 | 20.1 | 0.773 | 1.12 | 509 | 124 |
| Lancashire | 0.783 | 1.52 | 35.3 | 8.6 | 0.913 | 0.48 | 1,270 | 124 |
| West Riding of Yorkshire | 0.743 | 1.34 | 34.9 | 9.79 | 0.866 | 0.48 | 514 | 74.7 |
| Durham | 0.785 | 1.40 | 16.1 | 3.88 | 0.887 | 0.77 | 278 | 36.8 |
| Glamorgan | 0.807 | 1.43 | 6.76 | 1.49 | 0.881 | 0.73 | 175 | 24.1 |
| Lanarkshire | 0.801 | 1.48 | 10.4 | 2.36 | 0.919 | 0.52 | 361 | 33.4 |
| Norfolk | 0.728 | 1.92 | 22.9 | 6.93 | 0.774 | 1.08 | 63.8 | 15.7 |
| Wiltshire | 0.693 | 1.84 | 15.3 | 5.14 | 0.749 | 1.05 | 31.4 | 8.49 |
| Cornwall | 0.730 | 1.92 | 13.1 | 3.92 | 0.729 | 1.18 | 34.6 | 10.0 |
| Sutherland | 0.794 | 1.99 | 2.12 | 0.51 | 0.872 | 1.32 | 3.24 | 0.504 |
| Antrim | 0.753 | 1.90 | 13.3 | 3.68 | 0.835 | 0.98 | 85.2 | 15.4 |
| Armagh | 0.675 | 1.81 | 7.75 | 2.73 | 0.789 | 1.07 | 17.4 | 4.00 |

At the base four numbers depart from Appendix B by factors of 1.7 to 7, so the GO region could
have been lost. It was not: C2 converged on every instance tried during the design too. One
instance did fail there, Appendix B with b = 0.1, whose b × 0.5 shock reaches the boundary
x\* = 1 (`BoundaryNoMargin`, unit 1d's case) and so has no target; no demo county comes near it
(b ≥ 0.30, x\* ≤ 0.92).

## 4. The history: `history.csv`

### 4.1 The ramps, by lever

Each row is a ramp: a parameter moves smoothly between two dates. The paper's levers: **task
automation** moves γ through η; **recursive automation** moves λ and a (machines in machines);
**rents** move T, b and h (the non-produced input, the land in a machine service, the land in a
basket); **exit** moves χ_max; and **scarcity** moves N. A county's factor is `to^w`, with w its
weight (a tag, or 1).

| key | dates | regions | param | to (w = 1) | weight | lever | what it stands for |
|---|---|---|---|---|---|---|---|
| `population.*` | 1750–1801–(1841)–1851–1901 | each county | N | the benchmarks | | scarcity (N) | population, geometric between benchmarks; the Famine in the six counties (1841–1851), the Highland decline, Cornwall's emigration after 1861 |
| `sites.*` | the same | each county | T | (N′/N)^μ, μ = 0.5 + 0.5·max(metro, port/2) | | rents (T) | towns, harbours and works bring their own site services; London's grow with its people |
| `enclosure.open-fields.1` | 1760–1780 | openfield | T | ×1.06 | openfield | rents (T) | the first wave of parliamentary enclosure |
| `enclosure.wastes.2` | 1793–1815 | England, Wales | T | ×1.08 | upland outside the Highlands | rents (T) | the war-time wave: commons and wastes |
| `enclosure.open-fields.2` | 1793–1815 | openfield | T | ×1.04 | openfield | rents (T) | the rest of the open fields |
| `improvement.scotland` | 1760–1830 | Scotland | T | ×1.15 | improver | rents (T) | Lowland improvement: runrig ended, farms consolidated |
| `potato.ground` | 1750–1841 | the six counties | T | ×1.40 | | rents (T) | bog edge and hill land taken into tillage |
| `drainage.fens` | 1820–1850 | fen | T | ×1.15 | fen | rents (T) | steam pumping |
| `clearances.sheep` | 1780–1855 | highland | T | ×1.06 | highland | rents (T) | sheep walks (the people leave through N) |
| `kelp.boom`, `kelp.bust` | 1765–1810, 1815–1830 | kelp | T | ×1.10, ÷1.10 | kelp | rents (T) | kelp for alkali, and its collapse |
| `mines.metal.boom`, `.bust` | 1750–1860, 1866–1885 | mining | T | ×1.30, ×0.80 | mining | rents (T) | copper, tin and lead: Cornwall, Parys Mountain, the Peak |
| `mines.slate` | 1780–1900 | slate | T | ×1.60 | slate | rents (T) | Penrhyn, Dinorwic, Ffestiniog |
| `coal.mineral.1/2/3` | 1750–1830, 1830–1870, 1870–1901 | coal | T | ×1.30, ×1.60, ×1.30 | coal | rents (T) | coal's mineral rent: coal is a non-produced input, folded into T until unit 1e splits the inputs |
| `improvement.rotation` | 1750–1840 | England, Wales | h | ×0.85 | improver | rents (h) | the Norfolk four-course: a basket needs less land |
| `improvement.high-farming` | 1840–1875 | all | h | ×0.90 | lowland (1 − upland) | rents (h) | tile drainage, guano, superphosphate |
| `steam.coalfields` | 1770–1830 | coal | b | ×0.70 | coal | rents (b) | Watt's engine and coke iron, first on the coalfields |
| `canals` | 1760–1830 | canal | b | ×0.90 | canal | rents (b) | canals carry coal to the works |
| `railways.gb` | 1835–1875 | Great Britain | b | ×0.75 | 1 − coal/2 | rents (b) | railways carry coal everywhere, most where it was dearest; stands in for trade (§7) |
| `railways.ireland` | 1845–1885 | the six counties | b | ×0.80 | 1 − coal/2 | rents (b) | later and thinner |
| `railways.own-input` | 1835–1875 | all | a | ×0.90 | | recursive (a) | cheaper carriage of materials |
| `machine-tools` | 1800–1870 | eng | λ | ×0.70 | eng | recursive (λ) | Maudslay, Nasmyth, Whitworth |
| `engineering.spread` | 1850–1901 | all | λ | ×0.80 | | recursive (λ) | engineering everywhere |
| `textile.cotton` | textile_from–1835 | cotton | η | ×0.60 | textile | task (γ) | jenny, water frame, mule, power loom (Lancashire from 1768, the Clyde from 1780–1785) |
| `textile.wool` | textile_from–1850 | wool | η | ×0.70 | textile | task (γ) | the West Riding from 1790; the Borders from 1820 |
| `textile.linen` | textile_from–1870 | linen | η | ×0.70 | textile | task (γ) | wet-spun flax and jute: Dundee, Belfast |
| `textile.hosiery` | textile_from–1880 | hosiery | η | ×0.80 | textile | task (γ) | Nottingham and Leicester |
| `textile.other` | textile_from–1880 | the rest | η | ×0.85 | textile | task (γ) | silk, ribbons, carpets, pottery, boots |
| `threshing` | 1790–1830 | improver | η | ×0.95 | improver | task (γ) | threshing machines (broken in the Swing riots of 1830) |
| `mechanisation.general` | 1830–1901 | all | η | ×0.90 | | task (γ) | reapers, engineering, printing, food |
| `poor-law.speenhamland` | 1795–1800 | speen | χ_max | ×1.08 | speen | exit (χ) | allowances in the southern and eastern arable counties |
| `poor-law.1834` | 1834–1840 | England, Wales | χ_max | ×0.90 | | exit (χ) | the workhouse test, less eligibility |
| `poor-law.ireland` | 1838–1845 | the six counties | χ_max | ×0.95 | | exit (χ) | workhouse unions |
| `poor-law.scotland` | 1845–1850 | Scotland | χ_max | ×1.03 | | exit (χ) | parochial boards |

The Poor Law moves χ_max, the scale of the work cost against dependence, because that is the
exit form unit 1a has (the SSRN dependence form, F(ln(1 + w/P_s))). The support itself, one
basket per potential worker paid from rent, is the model's structure and cannot be cut without
leaving the oracle; s₀ and the idle margin of main.tex's s(q) wait for unit 1e.

What is left out, deliberately: the wars and their prices (no trade, no money), the corn laws
and their repeal (no trade), emigration as a flow (only as benchmarks), cholera, and every
event shorter than a few years (they would be shocks, §4.4).

### 4.2 The table's format

| column | meaning |
|---|---|
| `key` | the ramp's name; the compiler names a step's basis after the ramps that moved it |
| `start`, `end` | years; `start` may be the word `textile_from`, the county's own year |
| `regions` | `all`, `nation:a+b`, `tag:t` (tag above 0), `kind:k1+k2` (textile kind), or county keys separated by spaces |
| `param` | a column of `regions.csv`: `workers`, `land`, `space`, `eta`, `lam`, `b`, `a`, `chi_max` (and `g0`, `g1`, `k`) |
| `mode` | `scale`: the param is multiplied by a factor moving geometrically from 1 to `to^w`; `path`: the param moves geometrically from `from` to `to`, absolute and per year |
| `from`, `to` | for `scale`, 1 and the factor at `end`; for `path`, the values |
| `steps_per_year` | the grid on which the compiler looks for a step (12: the first of each month) |
| `weight` | a tag, or `waste` (upland outside the Highlands), `lowland` (1 − upland), `offcoal` (1 − coal/2), or empty for 1 |
| `lever`, `note` | the paper's lever and a note, for the basis text |

### 4.3 How the compiler composes and steps it

- **Composition.** A county's value of a param at time y is its path value (the latest `path`
  row begun, else its genesis value) times the product of every `scale` row's factor,
  `(to/from)^(w·s(y))`, with s(y) = clamp((y − start)/(end − start), 0, 1). Ramps on one param
  multiply, so overlapping waves compose.
- **Threshold stepping.** On the union of the param's rows' grids, the compiler emits a
  `SetParam` on the first of a month when the composed value has moved by 1% or more in log
  since the last one it emitted, and at each row's end when it has moved at all. Each step is
  a new schedule param, `county.<c>.<param>.<yyyy-mm>`, copied by an event of the same key
  dated `<yyyy-mm>-01`; its basis names the rows that moved it.
- **Values.** A step's value is the composed value to six significant figures, as the tables
  write their numbers; the threshold is measured from the composed value, so rounding never
  accumulates. The rule's size is `step_log` in `world.csv`, 0.01; 0.02 is §4.4's fallback.
- **The result**: 30,078 events on 25,480 county dates (1,706 calendar dates). The largest step
  at one date, over the oracle's relative prices and technique (1 − x\*, w/r, p_m/r, p/r), is
  2.6% in log (Durham, July 1843, when b, η and T crossed their thresholds together); over
  quantities (Y, K, N_a) 2.2% (Gloucestershire, May 1848). Over any trailing year (D.4) the
  largest moves are 3.9% in prices (Lanarkshire, to October 1848) and 5.1% in quantities
  (Monmouthshire, to March 1840).

### 4.4 Gradual, and why

The probe found that abrupt shocks give violent paths (O14; REPORT §5): a 12% fall in
equilibrium output cost 85% of output on the way. The scratch checker ran each county alone on
the engine, 1750-01-01 to 1901-01-01, with its history as the compiler would write it, and
scored every tick against the oracle at the params in force (the probe's D̂: the largest of ten
|ln(o/o\*)| over 1e-3; a dead tick when a market does not trade or clears below half its oracle
volume):

| history | events | dead ticks | lowest cleared volume over its oracle's | ticks with a transfer shortfall | largest D̂ | median county's median D̂ |
|---|---|---|---|---|---|---|
| gradual, 1% steps (the design) | 30,078 | 0 | 0.945 (Glamorgan) | 0 | 56 (Glamorgan) | 10.8 |
| gradual, 2% steps (the fallback) | 15,863 | 0 | 0.937 (Lanarkshire) | 0 | 67 (Lanarkshire) | |
| each ramp at once, at its end | 1,705 | 345 | 0.287 (Lanarkshire) | 3,027 | 1,249 (Lanarkshire) | |

So the gradual history never kills a market, never leaves the provider short, and never lets a
volume fall more than 5.5% below its moving target. The economy tracks its target with a lag:
D̂ is 11 at the median county (1.1% in log) and 29–34 in Durham, Lanarkshire and Glamorgan,
whose targets move fastest. That lag is what the gap lens shows (§6). A county whose history
has ended returns to tolerance: Perthshire and Sutherland end at D̂ 0.06 and 0.18.

The 2% fallback halves the events if the tape or the GUI's registry and timeline struggle with
30,000; its paths are nearly as smooth.

**What the compiler holds a history to (D.4).** D.2 bounded each date's move in the relative
prices and technique alone. D.2's verification (2026-09-27, `D:/rustyecon-demo/verify-world-r1/`)
compiled three abrupt histories that bound let through, and ran them: Middlesex's N and T
doubled within 1800 (no relative price moves, but quantities 6.4% a month; the county dead for
17 ticks, its provider short for 76, trough 0.475, D̂ 745); Middlesex's T ×1.3 within 1800 (no
date above 3%, but 0.19 in prices and 0.29 in quantities over the year; trough 0.78, D̂ 248); and
`max_step` raised to 5 with coal's rent tripled in 1750 (323 dead ticks in 10 counties, D̂ about
1,000). So the compiler now bounds, at every county date:

| | bound | the committed history's largest |
|---|---|---|
| the move at one date, relative prices and technique | `max_step`, 0.03 | 0.026 (Durham, 1843-07) |
| the move at one date, quantities (Y, K, N_a) | `max_step`, 0.03 | 0.022 (Gloucestershire, 1848-05) |
| the move over the trailing year, either | `MAX_YEAR`, 0.1 | 0.039 and 0.051 |
| `max_step` itself | at most 0.03 (`MAX_STEP_CEILING`) | 0.03 |
| `step_log` | at most 0.02 (`STEP_LOG_CEILING`, the fallback above) | 0.01 |
| the dials | the probe's registered C2, exactly (`tables::C2`) | C2 |

`MAX_YEAR` sits at twice the history's largest yearly move and below the 0.19 and 0.29 of the
second case; nothing between has been run. The dials are held to C2 because C2 is the one point
where the probe's battery, its kicks and this history's run are all evidence; the probe's GO
region (REPORT §4) is mapped one dial at a time and at rest, not along a history.

## 5. What the world shows

### 5.1 The picture, from the oracle at each county's params in force

| | 1750 | 1800 | 1850 | 1900 |
|---|---|---|---|---|
| x\*, median (range) | 0.73 (0.66 Surrey – 0.81 Glamorgan) | 0.72 | 0.74 (to 0.89) | 0.78 (0.66 Essex – 0.92 Lanarkshire) |
| output per head, median (max) | 2.42 (3.2 Lanarkshire) | 2.39 (3.8) | 2.47 (5.2 Lancashire) | 3.02 (7.1 Lancashire) |
| wage in goods w/p, median (max) | 1.37 (1.45) | 1.37 (1.49) | 1.38 (1.53) | 1.42 (1.56) |
| wage in land w/r, median (min) | 1.87 (1.34 West Riding) | 1.84 (1.01) | 1.49 (0.57) | 1.06 (0.48 West Riding) |
| wage in baskets w/P_s, median (range) | 1.20 (1.14–1.26) | 1.19 | 1.17 (0.98–1.24) | 1.14 (0.95 West Riding – 1.24) |
| rent in goods r/p, median (max) | 0.73 (1.04) | 0.74 (1.46) | 0.91 (2.59) | 1.35 (3.22 Lancashire) |
| land share, median (max) | 0.65 (0.74) | 0.64 (0.79) | 0.66 (0.87) | 0.73 (0.91 Lanarkshire) |
| relief burden N·P_s/(rT), median (max) | 0.64 (0.85 Surrey) | 0.66 (0.87 Surrey) | 0.63 (0.81 Surrey) | 0.44 (0.79 Essex) |

The map in motion: the automation frontier, output per head and rent rise first on the
Lancashire and Clyde cotton districts and the coalfields, and spread; the wage against land
falls fastest there; the relief burden is heaviest in the counties round London, in
pre-Famine Ulster and in Caernarfonshire, and lightest on the coalfields; participation is lowest
in the Highlands.

### 5.2 What it gets wrong, on purpose or by construction

- **The wage in baskets barely moves** (0.95 to 1.27 over every county and year) and falls
  where industry grows, while historical real wages roughly doubled between 1820 and 1900. In
  this economy the wage is pinned by the machine cost of the marginal task, w = p_m·γ(x\*), so
  the wage in goods can only rise with x\* and is capped by the schedule's shape at
  γ(1)/J(1) = 1/(g0 + g1/(k + 1)) = 1.67, while land, the one non-produced input, takes the
  rest. That is the paper's pinning and its real-wage fork (wage in goods up, wage in land
  down), shown county by county; history escaped it through trade, many goods and new work,
  none of which this world has yet.
- **Labour shares are low** by 1900 (0.09 in Lanarkshire, 0.27 at the median), because "land"
  here is every non-produced input: soil, sites, harbours and coal. There is no interest (ρ = 0),
  so no capital income; "land share" means everything that is not wages.
- **London is a large farm county.** Without trade it cannot be a port city; its site services
  grow with its people (μ = 1) and a harbour adds 40% to its land, which keeps it inside the
  funded region.
- **The Famine is a population ramp** over 1841–1851, not a crisis: a crisis would be a shock,
  which the probe says the roles do not survive well.
- **Railways appear as cheaper machine services**, not as trade.

## 6. The lenses: `lenses.csv`

A lens is GUI.md §4's `{ measure, unit, scale, domain }` with a reference for diverging
scales. One `LensVm` per lens makes the map, the ranked table and the chart, so the map's
values are the table's (G4's gate). Scales are neutral (colorous' sequential and diverging
maps, no good or bad colours), and fixed for the run: each domain covers the oracle's range
over every county and step (§3.3) with a margin for transients. As recorded at D.4, the test
`lens_domains_hold_the_oracle_range` computes that range through the lenses' own measures at
every county's oracle point, at genesis and after each of the 25,480 steps, and fails if it
leaves a domain; the table of ranges and margins is below the lens table. Every value is named
with its run key and its report tick, as every number in the GUI is (U3), and the lens's name
says its unit. Each level lens has a change lens, `since.<lens>` = ln(value ÷ the county's
genesis value), diverging around 0 with the county at genesis as its reference; two are listed.

| lens | measure | unit | scale | reference |
|---|---|---|---|---|
| `wage.baskets` | w/(p + h·r) | baskets per hour | sequential, [0.85, 1.35] | |
| `wage.goods` | w/p | goods per hour | sequential, [1.25, 1.7] | the schedule's ceiling, 1.667 |
| `wage.land` | w/r | land-service units per hour | sequential log, [0.4, 2.4] | |
| `rent.goods` | r/p | goods per land-service unit | sequential log, [0.6, 3.6] | |
| `share.land` | r·T_c/(w·L_c + r·T_c) | share of factor income | sequential, [0.5, 1] | |
| `share.labour` | w·L_c/(w·L_c + r·T_c) | share of factor income | sequential, [0, 0.5] | |
| `frontier.x` | 1 − the good desk's planned human share | share of tasks | sequential, [0.6, 1] | 1, every task |
| `participation` | L_c/N | share of potential hours | sequential, [0.5, 0.9] | 1, every hour |
| `output.per.head` | Y_c/N | goods per unit of N | sequential log, [1.5, 8] | |
| `price.good` | p/r | land-service units per good | sequential log, [0.28, 1.6] | |
| `price.mach` | p_m/r | land-service units per machine service | sequential log, [0.4, 1.4] | |
| `relief.burden` | the provider's due ÷ r·T_c | share of the rent roll | sequential, [0, 1] | 1, the rent roll no longer covers support |
| `shortfall` | (due − paid)/due | share | sequential, [0, 0.5] | |
| `rationing` | the county's largest 1 − filled/requested | share | sequential, [0, 0.05] | |
| `no.trade` | ticks of the last 52 in which a market did not trade | ticks a year | sequential, [0, 52] | |
| `gap.oracle` | D̂ against the county's own oracle point | multiples of 1e-3 in log | sequential log, [0.1, 100] | 1, the probe's tolerance |
| `gap.wage` | ln((w/r)/v\*) | log difference | diverging, [−0.06, 0.06] | 0, the oracle |
| `param.*` | N, T, η, b, λ, χ_max as the run holds them | as registered | sequential (log for N, T) | genesis |

Here w, r, p_m, p are the posted prices at the county's node, L_c, T_c and Y_c the cleared
volumes of labour, land and the good, h the county's `space`, and N its `workers` per tick.
The lenses are ratios of prices and volumes, so the coin's level never shows. `lenses.csv` adds
each lens's inputs and a note.

**The domains against the oracle** (`lens_domains_hold_the_oracle_range`, D.4): the range of
each lens over every county at genesis and after every step, and the margin left each side as a
share of the domain (in log for a log scale).

| lens | oracle's range | margin below, above |
|---|---|---|
| `wage.baskets` | 0.947 (West Riding, 1901) to 1.268 (East Riding, 1798) | 0.19, 0.16 |
| `wage.goods` | 1.299 (Surrey, 1840) to 1.566 (Lanarkshire, 1900) | 0.11, 0.30 |
| `wage.land` | 0.479 (West Riding, 1901) to 2.042 (Inverness-shire, 1750) | 0.10, 0.09 |
| `rent.goods` | 0.698 (Inverness-shire, 1750) to 3.242 (Lancashire, 1901) | 0.08, 0.06 |
| `share.land` | 0.563 (Armagh, 1825) to 0.906 (Lanarkshire, 1901) | 0.13, 0.19 |
| `share.labour` | 0.094 to 0.437, the same counties | 0.19, 0.13 |
| `frontier.x` | 0.640 (Surrey, 1840) to 0.919 (Lanarkshire, 1900) | 0.10, 0.20 |
| `participation` | 0.564 (Inverness-shire, 1901) to 0.870 (East Riding, 1840) | 0.16, 0.08 |
| `output.per.head` | 1.85 (Caernarfonshire, 1834) to 7.21 (Lancashire, 1900) | 0.13, 0.06 |
| `price.good` | 0.308 (Lancashire, 1901) to 1.432 (Inverness-shire, 1750) | 0.06, 0.06 |
| `price.mach` | 0.432 (West Riding, 1901) to 1.230 (Inverness-shire, 1750) | 0.06, 0.10 |
| `relief.burden` | 0.154 (Lancashire, 1900) to 0.894 (Surrey, 1834) | 0.15, 0.11 |
| `since.wage.baskets` | −0.191 (Lancashire, 1901) to 0.030 (East Lothian, 1862) | 0.18, 0.45 |
| `since.output.per.head` | −0.258 (Dunbartonshire, 1900) to 0.887 (Lancashire, 1900) | 0.39, 0.13 |
| `param.*` | the history's own range: each domain holds every county's value | 0.00 to 0.07 |

`shortfall`, `rationing` and `no.trade` are 0 at every rest point, so the oracle does not set
their domains. On this history they stay near 0 in the run too: shortfall and ticks without
trade are 0 in every county at every tick (§8), and rationing reaches 0.032 at most at the first
tick of each year of the run to 1901 (D.3's verification). Rationing's domain is therefore
[0, 0.05], narrowed at D.4 from [0, 0.5], on which it showed one colour all run. The other two
keep their domains and show one colour on this history; they move only under an abrupt edit.
The param lenses' domains end at the genesis values of η, b and λ, which only fall.

**Where each comes from.** No lens needs an engine accessor that does not exist:

- **From the report and params alone** (G0's catalogue records them): the wages, rent, prices,
  shares, participation, output per head, rationing, dead ticks and the param lenses. They read
  `MarketLine.price`, `.cleared` and `trades()`, `RationLine`, and `Sim::param` with the
  registry's per-tick conversion.
- **From actor states** (`Sim::actor_state`, which exists; G0's Extractor does not record it):
  the frontier (`GoodDeskState.share`), the relief burden and the shortfall
  (`ProviderState.due`, `.paid`). The Extractor gains the four roles' states per county.
- **From the oracle** (code that does not exist yet): `gap.oracle` and `gap.wage` need
  `crates/observe`'s `oracle_gap` (GUI.md §7.3, Phase 2): unit 1a solved outside the Sim at each
  county's params in force, read through `Sim::param`, never fed back (R13). The probe's
  observables and `Target::of` are the definition to move there; so is the probe's
  oracle-relative dead tick (O17). Until then `no.trade` counts the oracle-free half of it, a
  market that did not trade; it was called `dead` until D.4, which renamed it so that it cannot
  be read as the probe's dead-tick count.
- **U6**: the GUI computes nothing the engine defines, so the lens measures belong in a function
  the cli also calls (observe's `measure`), not in `vm/`. The map stage either brings
  `crates/observe` forward for the lenses or records the departure.

As built at D.3 (the map, §10): the departure is recorded. The measures are defined once, in
`crates/worldgen/src/lens.rs`, beside this table and the compiler that writes the keys they
read, and the GUI gathers each county's recorded numbers and calls them (docs/GUI.md, amended
at D.3, item 3). They move to observe's `measure` when it exists; the cli calls none of them
yet. The Extractor now records the four roles' states and each market's trade flag. The two
oracle lenses are listed and disabled until observe lands.

## 7. Goods, machine types and carriers, later

The tables are shaped so that these switch on without a redesign.

- **Reserved columns.** `regions.csv` carries `categories` (now `good`), `machine_types` (now
  `mach`) and `carriers` (now empty). The history's `param` column takes a qualifier when they
  arrive: `eta@textiles`, `b@steam`, `capacity@<channel>`.
- **Categories** (oracle unit 1b): a `categories.csv` with each category's key, basket weight z,
  direct land, and its segment of the shared task line, with the schedule per category. A
  sensible first set: food (land-heavy), textiles (the task line the textile ramps move), metal
  goods, shelter (space as a category, as unit 1b has it) and services. `categories` then lists
  a county's desks. The textile ramps move only the textiles segment, which is what they meant.
- **Machine types** (unit 1c): a `machine_types.csv` with each type's operating and build
  recipes over machine services, labour and land, δ, build lag and task efficiency θ: horse and
  water power, the steam engine, the railway. The coal and steam ramps then move the steam
  type's b, and the cheapest type takes the tasks, switching along the line as unit 1c solves.
- **Roles.** Both need the many-market roles being built on branch `phase2-markets` (P2.1, in
  progress on 2026-09-27: a basket provider and basket workers buying many items, a category
  desk on its segments of the task line, a type desk buying other types' services), with
  genesis from units 1b and 1c, which exist. Their instances need the probe's battery again,
  county by county, as §3.3 ran it.
- **Carriers.** A `channels.csv` from the atlas's neighbours (201 land borders with their
  lengths) and sea lanes between its 27 port sites, each with a carrier kind (road, canal,
  coast, rail), a capacity and a pass-through recipe. They need transport desks and home-node
  trading (ENGINE §13, Phases 4 and 9), which no branch has yet, and an equilibrium with trade,
  which no oracle unit has. So carriers switch on **gradually, from zero capacity**, like every
  other ramp, and the railway ramps on b come out as they go in. Until then each county's oracle
  is its own, and the gap lens compares each county with it.
- **In the compiler** (§8). Today it refuses any other value in the reserved columns, naming
  this section. When they switch on, `categories.csv`, `machine_types.csv` and `channels.csv`
  join its `Tables`; a history row's param becomes a (column, qualifier) pair; the writer emits
  a desk per category and type, and a carrier per channel; genesis comes from unit 1b or 1c at
  the county's instance; and the per-date check of §8 reads that unit's relative prices. The
  stepping (§4.3), the bases, the keys and the atlas checks stay as they are.

## 8. The compiler and the tape (D.2)

`rustyecon worldgen worlds/demo-gb --out tapes/demo-gb.ron` writes the tape. The cli reads the
five tables and hands their text to `rustyecon_worldgen::compile(&Tables, &Atlas)`, with the
atlas bundled (`Atlas::gb()`, D.1); the compiler reads no file. Without `--out` it checks and
reports only. `tapes/demo-gb.ron` is committed, and `demo_tape_is_its_compilers_output` checks
that it is exactly what the compiler writes, on WSL and on Windows.

**What it checks, before it writes a line.**

- **The tables.** Each header is exactly the expected columns; every row has the header's width;
  LF line endings. `world.csv`: every setting and dial present once, with its unit (a dial's
  unit is the one the roles read it in), ledger tolerances in (0, 1). From D.4: the basis marker
  starts with "illustrative" and the name carries `[illustrative]` (until Phase 4's research
  tables, this compiler writes illustrative worlds only, so the marker cannot be dropped from
  both cells), `max_step` at most 0.03 and `step_log` at most 0.02, and the dials exactly the
  probe's registered C2 (§4.4). `regions.csv` and `counties.csv`:
  every region of the atlas has a row and every row is a region; each row's Chapman code, HCS
  code, name and nation are the atlas's; `counties.csv`'s area is the atlas's to 0.05 km²; the
  eleven params in range (N, T, h, η, k, b, χ_max positive, g0, g1, λ at least 0, a in
  [0, 1)); tags in [0, 1]; a textile county has a kind and a start year inside the world, and no
  other county has either; the reserved columns hold `good`, `mach` and nothing (§7).
  `history.csv`: unique keys; spans inside the world; a `textile_from` start only with a `kind:`
  selector, and before the ramp's end in every county it moves; every selector, param and
  weight known; a scale ramp's `from` 1; a grid that divides 12; each county's paths one after
  another, the first starting at its genesis value and each at the last one's end. `lenses.csv`:
  unique keys, a known scale and source, a domain that fits its scale (positive for a log
  scale, around 0 for a diverging one), and `param.*` lenses naming a param.
- **Every county, at every step.** Unit 1a is solved at genesis and after each of the 25,480
  county dates, and each solve must be `Interior`, cross zero once on a 401-point grid, be
  funded, and have participation below 0.999. No date may move the oracle's relative prices
  and technique (1 − x\*, w/r, p_m/r, p/r), nor (from D.4) its quantities (Y, K, N_a), by more
  than `max_step`, 0.03 in log, and no trailing year may move either by more than `MAX_YEAR`,
  0.1: the probe's roles take gradual change well and abrupt change badly (O14, §4.4). So a
  history abrupt at one date or over a year does not compile, and neither does one whose bounds
  were loosened (`the_compiler_refuses_bad_tables` tries coal's mineral rent tripled in a year,
  η halved in a year, N and T doubled in a year, T ×1.3 in a year, `max_step` 5 and `step_log`
  0.05).
  On the committed tables the extremes are the scratch checker's (§3.3) to the digit: funding
  at least 0.119 (Surrey, 1834-04), participation 0.564 (Inverness-shire, 1901-01) to 0.870
  (the East Riding, 1840-09), x\* 0.640 (Surrey, 1840-02) to 0.919 (Lanarkshire, 1900-07), and
  the largest move at one date 0.0262 in log (Durham, 1843-07), 0.0222 in quantities
  (Gloucestershire, 1848-05), and over a trailing year 0.0393 (Lanarkshire, 1848-10) and 0.0506
  (Monmouthshire, 1840-03). The compile takes under 2 s.

**The tape.** `name: "demo-gb [illustrative]"`, schema 1, 52 ticks a year from 1750-01-01,
`Imbalance` with `Saturate`. Nodes `county.<c>` in the atlas's keys
(`atlas_keys_equal_node_keys`), all quoting in `coin`; goods `coin`, `good`, `labour`, `land`,
`mach` as in `tapes/appb.ron`; no channels. Per county four actors, `county.<c>.desk.good`,
`.desk.mach`, `.provider` and `.workers`, in the probe's four classes, shared by every node (a
rationing line names its node). Params: the fourteen dials of `world.csv` once,
`life.one_tick`, each county's eleven instance params `county.<c>.<column>` (1,023), and the
30,078 step values `county.<c>.<column>.<YYYY-MM>`, schedule params that only the event of the
same key reads, so they are outside `world_id`. Genesis is §1's:
`genesis_is_the_probes_rule_at_each_county` checks that each county's oracle point is
`probe::setup::genesis`'s bit for bit, and its prices and coins the probe's divided by p/r to
1e-14. Each county's actors carry a comment with its point.

- **Bases.** Every entry is `Assumed("illustrative demo, 2026-09-27: …")`
  (`every_basis_is_illustrative` reads every registry line, actor, genesis and event): a dial
  its `world.csv` note; a county param "<name>: <why>"; a step value "<name>'s <column> from
  <YYYY-MM>: <the ramps that moved it>", such as "Lancashire's eta from 1801-05:
  textile.cotton; threshing"; an event "history step", since the value it copies names its ramps
  (`FiredEvent.source`). The dials are the probe's registered C2
  (`dials_are_the_probes_registered_c2`).
- **Size.** 12,174,574 bytes and 64,169 lines, 0.8 MB compressed; nearly all of it is the
  30,078 step values and their events. `Tape::from_ron` and `Sim::new` take about a second
  together. The 2% fallback (`step_log` 0.02) would halve it.
- **Identity.** `tape_hash` 0x1bd56d66433d3b67, `world_id` 0x9701ae49997ff8f5.

**The run** (`demo_runs_to_1901`, ignored, and run by name in `scripts/gate.sh`). The tape runs
from genesis through the first tick of 1901: 7,852 ticks, to state tick 7,852. At D.2 it
stopped at state tick 7,851, 1901-01-01, one tick before the 372 steps dated 1901-01-01 fire (in
report tick 7,851), so the gate never ran them; from D.4 the test runs that tick too and checks
that all 30,078 events fired. Every tick the test reads each county's eleven params through
`Sim::param`, solves unit 1a again when they change (25,480 solves), and scores the county as
the probe does (docs/probe/RULES.md §6): D̂ over its ten observables, a dead tick when a market
does not trade or clears below half its oracle volume, and a shortfall when the provider pays
less than it owes.

| | result |
|---|---|
| final state hash | `0xfad880fe08d06645` at state tick 7,852 (D.4), equal on WSL and Windows, and to `rustyecon run` and the GUI's run (`gui_equals_cli_demo_gb`); `0x9c78e47631ea8224` at 7,851, D.2's |
| the hash stream (FNV-1a 64 over the 7,852 hashes, 8 bytes each, little-endian) | `0xdb63cc96f769fb3e`, equal on WSL and Windows (D.2's, over 7,851: `0xd420b740361846f1`) |
| conservation | every tick's ledger and the run's closed (the engine stops on a breach) |
| dead ticks, in any county | 0 |
| ticks with a transfer shortfall | 0 |
| lowest cleared volume over its oracle's | 0.945 (Glamorgan) |
| D̂ against each county's moving oracle point, median over all county-ticks | 11.4 (1.1% in log) |
| D̂, 90th percentile | 23.4 |
| the median county's median D̂ | 10.8 (Aberdeenshire) |
| the highest county median | 34.1 (Glamorgan; then Lanarkshire 30.7, Durham 29.4, Lancashire 28.8) |
| the largest D̂ | 56.3 (Glamorgan) |
| run time, release | 12.7 s through `rustyecon run` on WSL, about 620 ticks or 12 model years a second, the best run; the machine was shared with branch `phase2-markets`'s runs throughout (load average 170 to 950 on 48 threads), and the loaded runs took 21 to 62 s of stepping, 126 to 380 ticks a second, on WSL and on Windows |

These are §4.4's numbers, now from one 93-node run rather than 93 single-county runs. The
engine runs many nodes, each with its own four roles, as it is: no engine change was needed,
and no lens needs a new accessor (§6).

**Tests** (`crates/worldgen/tests/demo.rs`): `demo_tape_is_its_compilers_output`,
`atlas_keys_equal_node_keys`, `every_basis_is_illustrative`,
`the_tables_check_every_county_at_every_step`, `dials_are_the_probes_registered_c2` (and, from
D.4, the compiler's `tables::C2` equal to the probe's), `genesis_is_the_probes_rule_at_each_county`,
`the_compiler_refuses_bad_tables` (twenty-one refusals: a region missing from either table, a key
not in the atlas, a wrong name or area, a wrong unit, a name without the marker, a param out of
range, a reserved column used, a broken path, a textile start after its ramp, a lens domain that
does not fit, CRLF; and the abrupt histories and loosened bounds of §4.4, a dial off C2 and the
marker dropped from both cells), `lens_domains_hold_the_oracle_range` (D.4, §6) and the long
`demo_runs_to_1901`; and in the cli's tests `worldgen_writes_the_committed_demo_tape`
(the command writes exactly the committed tape and prints its `tape_hash`; tables that do not
compile are exit 1, a directory without them exit 3). `scripts/gate.sh` runs `derive.py --check`
when it finds a Python 3, and the long run by name.

**Left for later.**

- **Timelines.** ENGINE §13 puts population, technology and enclosure timelines in Phase 3, as
  world-level `Ext` deltas. The demo uses what exists now, dated `SetParam` steps; when those
  timelines land, `history.csv` compiles to them instead and its rows stay as they are.
- **Scoring.** *Done at D.4 for `certify`:* a run of a tape whose name carries `[illustrative]`
  is UNSCORED whatever its criteria (D.2's verification had certified the demo tape PASS
  against criteria written for it), and a tape whose bases say illustrative with the marker gone
  from its name does not certify (`certify_refuses_illustrative_tape`). Left: the marker joins
  the GUI-experiment marker in the scorecard's refusal (GUI.md U5, §7.3, Phases 6–7; STATE.md
  O21), and the identity chip should show it, as it shows "experiment".
- **The GUI's load.** G0's in-memory store would hold about 10,000 series over 7,851 ticks at 16
  bytes a point, roughly 1.3 GB (an estimate: 9 fields × 372 markets, 36 class-line values and
  24 settlement values a county, holdings, and 1,000 params), so the map stage wants the
  catalogue filter parked in GUI.md §10, or the lenses' inputs alone (about 5,000 series,
  0.6 GB). The 30,078 schedule params and events load in about a second, but the registry
  listing and the timeline have not been tried with them. *Done at D.3 (§10):* the lean
  catalogue and series kept in stretches, and the panels made to take 30,000 events.
- **The oracle lenses** (`gap.oracle`, `gap.wage`) need `crates/observe` (§6). The long run's
  scoring is the definition to move there, with the probe's.
- **The battery** of §3.3 at six dates stays in scratch; the 93-node run above stands in for
  §4.4's per-county runs.

## 9. Where the evidence is

The design's evidence is in scratch, `D:/rustyecon-demo/world-design/` (WSL `/mnt/d/...`),
built in release on WSL against this worktree at `a8d3f51` (D.1):

- `check/`: the checker, a standalone crate with path dependencies on the oracle, core, probe
  and engine. `demo-world-check world DIR STEP` (the static check and the oracle's picture),
  `world-battery DIR STEP YEARS` (the probe's battery per county), `world-run DIR STEP` (each
  county's history on the engine), `search N` (the random search of §2), and `battery k=v ...`
  and `k=v,v,...` (one instance, or a sweep).
- `world9.txt` (the static check), `battery-final.txt` (the battery at six dates),
  `run-final.txt`, `run-abrupt.txt` (§4.4), `oracle-path.csv` (every county's regime, v, Y,
  N_a, x\* and lens values at the first step of each decade and at the end of its history),
  `dhat-series.csv` (D̂ every 13th tick).

The compiler's evidence (D.2) is its tests, above, and their output in
`D:/rustyecon-demo/worldgen/`: the long run's per-county lines on WSL and on Windows.

## 10. The map (D.3)

`rustyecon-gui tapes/demo-gb.ron` opens on the map: the 93 counties coloured by a lens, the
legend, the ranked table and the selected county's card. docs/GUI.md's block "Amended at D.3"
says how it is built; this section says what it shows of this world.

- **Opening.** The tape opens paused at 1750 on "Wage in land", w/r, the paper's v. Before the
  first tick there is no report and the map says so; after it, every county has a value.
- **Lenses.** The 25 rows of `lenses.csv`, each drawn on its registered domain for the whole
  run, so a county's colour means the same number in 1750 and in 1900. 23 have a value in
  every county at each report tick `map_values_equal_table` samples (0, 51 and 259) of a run
  from genesis; `gap.oracle` and `gap.wage` wait for `crates/observe`. A change lens reads the
  county at the record's first tick, so it has no value on a branch resumed after genesis.
  `no.trade` counts over the ticks of the trailing year the record holds up to the cursor,
  fewer than 52 in the first year.
- **The record.** 93 nodes record the lean catalogue: 5,970 series, 8 bytes a point, so a run
  to 1901 holds about 0.37 GB of series. Measured: 46.9 million points, 375 MB of values,
  and the run ends at the cli's final hash, `0x9c78e47631ea8224` (docs/GUI.md, amended at
  D.3, item 11).
- **What it cannot show.** No trade (there are no channels), so no county's colour reflects
  another's; §5.2's caveats hold on the map as in the tables.
- **Its data's credit** (D.4). The map paints the atlas's credit in its lower right corner, the
  Historic County Borders Project and OpenStreetMap under the ODbL 1.0, with the whole
  attribution on its hover (data/atlas/ATTRIBUTION); `rustyecon licences` and
  `rustyecon-gui --licences` print the atlas's LICENSE and ATTRIBUTION, which both binaries
  bundle.
- **Held to the engine** (D.4). `lens_values_equal_the_engine` checks sixteen lenses for every
  county at report ticks 0, 51 and 259 against the engine's own numbers, and
  `rebuilt_mesh_colours_are_the_lens_colours` the colours of a mesh built afresh against the
  scale and the legend (docs/GUI.md, amended at D.4).
