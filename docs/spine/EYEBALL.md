# Breakpoint B pre-look: the eyeball sheet

Dated 2026-09-26. Branch `spine-eyeball`. Revised the same day after the R11 audit (S5.0.2).

This is a pre-look for your ruling. It is not the ruling. Breakpoint B is your call
(docs/PLAN.md, Phase 5).

The test, as PLAN words it: before any fit, the raw series must show three things. The
floor era's opposition between population and the wage. The escape. Land's exit.

Nothing here is fitted. There is no regression, no break test and no smoothing. No series
is spliced unless its source made the splice, and those are named. Where two sources cover
the same years, both are drawn. Every construction of ours is named where it appears.
"Index, base = 100" means a series divided by its own mean over the base years. That is a
rescale, not a splice. Dates marked "by eye" are readings of the figures, not estimates.

Sources, licences, coverage and validation are in [DATA_NOTES.md](DATA_NOTES.md). The
figures and every number quoted below come from `data/spine/eyeball.py`. Its read-off
(`readoff.txt`, `c1_episodes.csv`, `c4_range_exit.csv` and the decade tables) is written to
the spine's cache, `$SPINE_ROOT/eyeball/` (DATA_NOTES, "Where things are"; on the machine
that made it, `D:/rustyecon-spine/eyeball/`), outside every commit, because most inputs may
not be redistributed.

## The figures

The figures are not in the repository. They plot Bank of England and Clark numbers,
whose terms do not allow redistribution, so they are generated locally: run the fetch
scripts in `data/spine/`, then `data/spine/eyeball.py`, which writes them to
`docs/spine/figs/` (ignored by git). Decision 2 below records why.

1. `fig1_floor_time.png`: population and the real wage on one
   time axis, 1250–1869. Four panels: population (two sources); Clark's day wages;
   the Humphries–Weisdorf constructions; Allen's inputs as the Bank carries them.
2. `fig2_floor_phase_broadberry.png`: the phase plot.
   The wage against population, decades dated, one panel per wage construction. Population
   is Broadberry et al. (2015) via the Bank's file. It is built without wages.
3. `fig3_floor_phase_clark.png`: the same with Clark's
   population. Before 1540 that population is partly inferred from the wage (§1, "How
   robust"). Its points before the 1540s are not independent of the wage axis.
4. `fig4_escape.png`: the real wage 1700–2016 on a log scale, five
   constructions. The shaded band is a reading by eye.
5. `fig5_land_exit.png`: land rent's share of income, 1700–1920
   (every construction) and 1200–2008 (log scale); and a price proxy, rent and tithe per acre
   in days of the day wage, 1500–1912.

The wage constructions, by the short names used below:

| Short name | What it is | Geography | Years |
|---|---|---|---|
| Clark avg/COL | Clark 2015 male average day wage over Clark 2015 cost of living. Our ratio (equals BoE A48 col V except 1848, by 0.28%) | England | 1209–1869 |
| Clark farm, Clark labourer, Clark craftsman | Clark's own real day wages, `Wages 2014.xlsx` cols I, J, K | England | 1209–1869 |
| HW19 annual | Humphries–Weisdorf (2019) Table A2 col F: annual-contract income over basket cost (BNS digitisation). HW value the in-kind benefits at the basket cost, so col F is 1 + cash/basket by construction. Only the cash part moves | England | 1260s–1840s |
| HW19 cash share | HW19's cash pay over the basket cost, col C / (E/F). Our arithmetic. Shown beside HW19 annual; not a ninth construction | England | 1260s–1840s |
| HW19 day | the same table, col I: day wage x 250 over basket cost | England | 1260s–1840s |
| HW16 | Humphries–Weisdorf (2016) working paper as BoE A48 col E | England | 1265–1850 |
| Allen labourer | Allen's southern-England building labourer's day wage over the daily cost of Allen's respectable basket (BoE A47 AH / BH). Our ratio. It is not Allen's welfare ratio | S. England | 1301–1913 |
| BoE composite | BoE A48 col B, the Bank's chain-linked splice. A day wage (Clark's) to 1750; Feinstein's earnings 1770–1881; not a day wage after 1750 | England to 1750, GB, then UK | 1209–2016 |
| Feinstein/Allen prices | Feinstein (1998) GB earnings over Allen (2007) prices, BoE A48 col X | GB | 1770–1869 |

"The day-wage series" below means Clark avg/COL, Clark farm, Clark labourer, Clark
craftsman, HW19 day and Allen labourer. "The annual-income series" means HW19 annual and
HW16. The BoE composite is in neither group.

## 1. The floor era's opposition (C1)

### What the raw series show

Figures 1 to 3. Percent change between decade means (simple, not log). Population is
Broadberry et al. (2015) via the Bank ("BoE") and Clark's 2015 spreadsheet ("Clark").

| Episode | Pop. BoE | Pop. Clark | Clark avg/COL | Clark farm | Clark labourer | Clark craftsman | HW19 annual | HW19 cash share | HW19 day | HW16 | Allen labourer |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1260s → 1310s | +7% | +30% | −32% | −26% | −35% | −28% | −2% | −14% | −9% | −7% | (starts 1301) |
| 1340s → 1350s (Black Death) | −43% | −21% | +36% | +50% | +13% | +12% | 0% | +2% | +27% | −10% | −10% |
| 1340s → 1380s | −49% | −37% | +82% | +95% | +68% | +46% | +18% | +107% | +59% | +5% | +45% |
| 1340s → 1450s (population trough) | −58% | −49% | +116% | +133% | +106% | +64% | +69% | +401% | +88% | +45% | +83% |
| 1500s → 1580s | +68% | +39% | −32% | −31% | −31% | −31% | −28% | −58% | −24% | −23% | −22% |
| 1500s → 1640s | +135% | +112% | −46% | −47% | −43% | −42% | −21% | −43% | −47% | −19% | −42% |
| 1650s → 1740s | +7% | +8% | +14% | +6% | +16% | +27% | +28% | +65% | +16% | +26% | +39% |
| 1740s → 1810s | +81% | +70% | +7% | −4% | +4% | −7% | +31% | +56% | −6% | +29% | −10% |
| 1810s → 1840s | +50% | +52% | +40% | +27% | +35% | +37% | +30% | +46% | +17% | +14% | +31% |
| 1810s → 1860s | +91% | +91% | +66% | +40% | +58% | +65% | (ends) | (ends) | (ends) | (ends 1850) | +34% |

Stated plainly:

- Two large swings of population carry the floor era: the fall from the 1340s to the 1450s,
  and the rise from the 1500s to the 1640s. In both, all eight wage constructions move
  against Broadberry's population, the source built without wages. Population fell by 58%;
  wages rose by 45% to 133%. Population then rose by 135%; wages fell by 19% to 47%.
- The phase plots show this as a loop. Wages climb to the upper left through the 1450s,
  then run back down to the right until the 1640s.
- Before 1340 the opposition is not shown by independent evidence. From the 1260s to the
  1310s Clark's four day-wage series fall 26% to 35%. Broadberry's population rises only 7%
  over those decades, and between its benchmark years (1250, 1279, 1290, 1315) it is
  interpolation. Clark's population rises 30%, but before 1540 it is partly inferred from
  the wage (see "How robust"), so it cannot confirm the opposition. The Humphries–Weisdorf
  series fall 2% to 9%; HW19's cash share falls 14%.
- The first decade after the Black Death is mixed. Clark's farm wage rises 50%. The
  Allen labourer ratio and HW16 fall 10%. HW19 annual is flat, and so is its cash share
  (+2%). By the 1380s all eight are up.
- From the 1650s to the 1740s population is nearly flat and wages drift up.
- From the 1740s to the 1810s population rises 70–81%. The six day-wage series barely move
  (−10% to +7%; four of the six fall slightly). In the floor era a rise of that order
  brought a fall: from the 1500s to the 1580s population rose 68% (Broadberry) or 39%
  (Clark), and every wage construction fell 22–32%. The annual-income series rise about 30%
  over 1740s–1810s; HW19's cash share rises 56%.
- From the 1810s population and every wage series rise together.

### How robust

- The two big swings hold for all eight wage constructions under Broadberry's population,
  the source independent of wages. The magnitudes differ by a factor of two or more
  between constructions.
- Clark's population gives the same direction, but before 1540 it is not a second witness.
  Clark (2010, section 8) takes it from Clark (2007). The BNS digitisation of Clark (2007)
  Table 9 shows the method: a population implied by the marginal product of labour (the
  population a labour-demand curve needs to produce the observed real wage), averaged
  with a series scaled from sample communities. Clark 2015's decadal values are 0.88–1.05
  times that 2007 "best" estimate over the 1250s–1490s: a revision of the same estimate.
  So Clark's pairings before 1540, and the tighter loop in figure 3, are partly the wage
  plotted against itself. From 1540 Clark's population follows Wrigley et al. (1997).
- The size of the Black Death fall depends on the population source: −43% (Broadberry)
  against −21% (Clark) between the 1340s and 1350s. The two sources differ most inside the
  Black Death window. Their ratio, Broadberry over Clark, runs 0.73–1.12 in annual values
  over 1250–1540 (0.75–1.11 in decade means). The largest gap is 1359, where Broadberry is
  27% below Clark.
- Before 1541 the Broadberry series is 16 benchmark years, log-linear in between. The
  decade means before the 1540s average interpolated values (hollow points in figure 2).
  Clark's population is one value per decade before 1700. Its 1590s repeat the 1580s
  (3.554 m; a corrected 4.164 exists in Steinsson's file). Kept as printed and flagged.
- Several wage series are not independent. Clark avg/COL, Clark's three real wages and the
  BoE composite before 1661 all rest on Clark's cost of living. HW19 day and HW19 annual
  share one deflator. HW16 and the Allen ratio both use Allen's basket.
- HW19 annual is 1 + cash/basket by construction: HW value the in-kind benefits at the
  basket cost (implied benefits over implied basket cost are 0.993–1.001 in all 59
  decades). In the 1260s–1360s the cash share is 0.18–0.24, so about four-fifths of HW19
  annual's level (1.19–1.24) is fixed at 1. Its flatness before 1370 is partly that
  identity. Its percent changes are damped next to the day-wage series: from the 1500s to
  the 1640s it falls 21% while its cash share falls 43%.
- The HW constructions weaken the opposition before 1370 and turn it into a rise after 1740.
  That is the construction debate the long-record spec names (§4.1). It is visible here, not
  resolved. For HW19 annual the weakening before 1370 is partly mechanical (above); its
  cash share shows the thirteenth-century fall (−14%) but not the first post-plague rise.

### What would change the reading

- An annual population source through 1348–1351. None was found. The annual window is
  wages and prices only.
- A population source for 1250–1340 that is independent of wages and denser than
  Broadberry's benchmarks. None was found. Without it the pre-1340 opposition stays
  unconfirmed.
- A canonical wage band that leans on the annual-contract series. The first-post-plague
  opposition would then drop out, and the thirteenth-century fall would shrink to 2–9%
  (14% in the cash share). The two big swings would remain. Such a band should carry the
  cash share beside col F, because col F's level is mostly fixed at 1 before 1370.
- Clark's 2023 revision of the medieval record. Only its land rents and national income
  were compared here (§3); its wages and population were not examined.

## 2. The escape (C2)

### What the raw series show

Figure 4. Decade means, each series as an index with its 1770s = 100.

| Decade | BoE composite | Clark avg/COL | Clark farm | Allen labourer | Feinstein/Allen prices | HW19 annual |
|---|---|---|---|---|---|---|
| 1700s | 102 | 100 | 103 | 105 | – | 78 |
| 1740s | 116 | 108 | 109 | 119 | – | 86 |
| 1770s | 100 | 100 | 100 | 100 | 100 | 100 |
| 1800s | 112 | 105 | 99 | 95 | 110 | 109 |
| 1810s | 108 | 116 | 105 | 106 | 108 | 112 |
| 1820s | 118 | 134 | 114 | 136 | 112 | 138 |
| 1840s | 130 | 162 | 133 | 139 | 135 | 146 |
| 1860s | 147 | 192 | 146 | 143 | 157 | – |
| 1870s | 170 | – | – | 179 | – | – |
| 1900s | 241 | – | – | 240 | – | – |
| 1950s | 372 | – | – | – | – | – |
| 2000s | 1442 | – | – | – | – | – |

Stated plainly:

- From the 1700s to the 1810s the day-wage decade means in this table (Clark avg/COL,
  Clark farm, Allen labourer) stay between 93 and 119 on this index. That is inside the
  range the BoE composite covered from the 1250s to the 1740s, where it is Clark's average
  day wage (56 to 164 on this index).
- By eye, the day-wage series turn up for good in the 1810s–1820s. No decade mean of Clark
  avg/COL, Clark farm or the Allen ratio returns to the 1770s level after the 1800s.
- The rise before 1870 is slow in some series. The Allen labourer ratio jumps in the 1820s,
  then stays at 136–146 until the 1860s, and rises again from the 1870s. The BoE composite
  (which is Feinstein 1998 in 1770–1881) rises about 6% a decade from the 1810s to the 1860s.
- The day-wage level passes its fifteenth-century peak late. 1860s over the 1440s–1450s
  mean: Clark avg/COL 1.05, Clark labourer 1.08, Clark craftsman 1.13, Allen labourer 1.04,
  Clark farm 0.71; HW19 day (1840s) 0.69. Clark's farm wage and HW19 day do not pass it
  before their series end. The BoE composite, which is not a day wage here, is at 0.90 in
  the 1860s and first leaves its 1250s–1740s range in the 1870s.
- The annual-income series rise earlier. HW19 annual goes from 1.53 (1640s) to 2.34 (1750s)
  and 3.83 (1840s). By eye it rises with population from the 1740s–1770s: 2.24, 2.34 and
  2.27 in the 1740s–1760s, then 2.62 in the 1770s. Its 1840s level is 2.0 times its
  fifteenth-century peak (its cash share, 3.1 times). HW16 is 2.0 times by the 1850s.
- After 1913 only one construction in the figures runs: the BoE composite (Feinstein UK,
  then ONS). It rises about 14-fold from the 1770s to the 2000s.
- Clark's Table 34 carries a real wage for the 1860s–2000s (1860s = 100; 162 in the
  1910s, 887 in the 2000s). It is Clark (2005) England building wages over Feinstein's UK
  cost of living, then the RPI: the source's own splice of England wages over UK prices.
  Not plotted.

### How robust

- That a sustained rise happens is robust. Every construction that spans the 19th century
  shows it, and the BoE composite continues it for another century.
- When it begins is not robust. By eye: the 1740s–1770s (annual-income series, rising with
  population) or the 1810s–1820s (day-wage series). Under one stated rule, the first decade
  above the series' own 1250s–1740s range (§4): the 1750s for HW19 annual and HW16; the
  1830s to the 1860s for the day-wage series (Allen labourer 1830s, unbroken from the
  1850s; Clark craftsman 1850s; Clark avg/COL and Clark labourer 1860s); the 1870s for the
  BoE composite. Clark's farm wage and HW19 day do not leave their range before their
  series end (1869, the 1840s).
- Geography changes at the seams: England (Clark, to 1869), southern England (Allen, to
  1913), GB (Feinstein), UK (BoE composite after 1913). In these sources the only real
  wage past 1869 that starts from England wages is Clark's Table 34 splice (above), over
  UK prices. Clark's cost of living ends in 1869; his nominal wages run to 1914 but are not
  deflated here.
- The BoE composite is not independent of Feinstein in 1770–1881: the ratio of the two is
  constant there (to 2e-15). Before 1750 it is Clark's wage; 1661–1750 it uses
  Schumpeter–Gilboy prices and drifts 20% from Clark's own real wage by 1869.

### What would change the reading

- Allen's published welfare ratios (not reached; his site is a login wall).
- Feinstein (1998) read at source (paywalled; its numbers reach us only through the Bank).
- An England cost of living for 1870–1914. Clark's real wage could then run to 1914.
- The choice of canonical wage band at Breakpoint B. It moves the start of the escape by
  40 to 80 years by eye, and by 80 to 120 years under the range rule.

## 3. Land's exit (C3)

### What the raw series show

Figure 5. Land rent's share of income, by decade. The last column is Clark's 2023
workbook, a private file deposited by BNS: a cross-check only.

| Decade | Clark 2015 J/(O−N) | Clark 2010 T13 | Clark 2010 T34 | Clark 2002 / BoE GDP | Stamp / BoE GDP (GB) | A17 land+buildings (UK, upper bound) | Clark 2023 NDP (cross-check) |
|---|---|---|---|---|---|---|---|
| 1250s–1740s | 0.133–0.248 (median 0.189) | 0.135–0.253 | – | – | – | – | 0.071–0.249 |
| 1700s | 0.200 | 0.201 | – | 0.198 | – | – | 0.206 |
| 1770s | 0.194 | 0.202 | – | 0.174 | – | – | 0.204 |
| 1780s | 0.174 | 0.186 | – | 0.142 | – | – | 0.185 |
| 1800s | 0.157 | 0.177 | – | 0.131 | – | – | 0.172 |
| 1820s | 0.124 | 0.140 | – | 0.122 | – | – | 0.134 |
| 1840s | 0.095 | 0.108 | – | 0.088 | 0.088 | – | 0.101 |
| 1860s | 0.069 | 0.077 | 0.075 | 0.065 | 0.060 | 0.145 | 0.075 |
| 1880s | – | – | 0.052 | 0.042 | 0.043 | 0.148 | – |
| 1900s | – | – | 0.025 | 0.023 | 0.022 | 0.131 | – |
| 1910s | – | – | 0.022 | 0.022 (1910–12) | 0.019 (1910–13) | 0.099 | – |
| 1920s | – | – | 0.007 | – | – | 0.048 (1920) | – |
| 1940s–1980s | – | – | 0.004 | – | – | – | – |
| 1990s–2000s | – | – | 0.002 | – | – | – | – |

Stated plainly:

- From the 1250s to the 1740s land's share moves between 0.13 and 0.25 in Clark's 2015
  file, with no trend. It moves with population: 0.227 in the 1340s, 0.165 in the 1350s;
  0.167 in the 1500s, 0.248 in the 1600s. The top of that band (0.22–0.25 in the
  1600s–1640s) follows a doubling of Clark's land rents from the 1590s to the 1600s (4.37
  to 9.05 £m). The price proxy shows the same jump (below).
- By eye the final decline starts in the 1760s–1780s. Clark 2015 and Table 13 fall in
  every decade after the 1770s. The Clark 2002 construction falls from the 1760s, with one
  small rise in the 1810s.
- Whether and when the share leaves its floor-era range depends on the vintage. Clark 2015
  falls below its 1250s–1740s low (0.133, the 1560s) in the 1820s; Table 13 in the 1830s.
  Under Clark's 2023 vintage the share does not leave its range before the series ends in
  1869: its 1860s value, 0.075, is above its 1270s low of 0.071.
- The fall itself holds in every vintage: from about 0.20 in the 1770s to about 0.07 in the
  1860s (0.204 to 0.075 in the 2023 file). Table 34 continues it to 0.022 in the 1910s,
  0.004 from the 1940s and 0.002 by 2000. After 1914 Table 34's rents are Clark's
  projection, not a measured series (see "How robust").
- The fall is not a fall in land rents. Clark's land rents rose 2.2-fold from the 1770s to
  the 1860s; his national income rose 6.2-fold.
- The price proxy (panel C, our construction) is a separate fact. It does not show the
  exit before 1869. A year's rent and tithe on an acre cost 10.5 days of a building
  labourer's wage in 1770–74 and 11.0 days in 1865–69: flat. In a farm labourer's wage it
  cost 13.0 days and then 15.8 days: 22% more. So the share fell because labour and income
  grew against a fixed acreage, not because an acre got cheaper in labour. In farm-labour
  days an acre got dearer until 1869. From the 1600s to the 1860s the proxy stays at about
  9 to 13 building-labourer days (10 to 16 farm-labourer days). Before 1600 it was 3 to 5.
- The proxy falls only after 1869, and only in the building-labourer series (the farm wage
  ends in 1869). Its first step, −12% from 1865–69 to 1870–74, comes from the wage. The
  building labourer's nominal wage rises 14.1% from 1868 to 1869 (35.49 to 40.51 d), the
  last year before the years that only the GPIH copy carries. Between the two periods the
  wage's mean rises 20% and rent per acre 6%. From 1870–74 to 1895–99 rent per acre itself
  falls 30% and the wage rises 27%; the proxy reaches 5.3 days.
- A step at 1914–1920: Clark T34 falls from 0.022 (1910s) to 0.007 (1920s), and the A17
  upper bound from 0.117 (1913) to 0.048 (1920). The T34 step falls on Clark's method seam
  (Stamp to 1914, a projection after). No second land-only source covers it.

### How robust

- The direction of the fall holds across all five land-share constructions. They are not
  five independent sources. Before 1842 the three that exist (Clark 2015, Table 13, Clark
  2002) all rest on one author's rents: Clark's own rent series in its 2002, 2010 and 2015
  vintages. From 1842 four of the five (Clark
  2015, Tables 13 and 34, and our Stamp construction) share one numerator, Income Tax
  Schedule A "Lands": Clark 2015's rents are 0.9386 x Stamp's England and Wales figure in
  all 28 years 1842–1869. "Lands" includes tithe, farmhouses, woodland and building land.
  The constructions differ in their denominators and vintages, not in their sources. Their
  levels differ by up to 0.05 before 1840 (the 1790s–1800s) and by about 0.02 in the 1860s.
- The timing of the range exit is not robust: 1820s or 1830s in the 2015 and 2010
  vintages, none before 1869 in the 2023 vintage. The monotone fall from about 0.20 to
  0.07 by the 1860s is robust across vintages.
- Table 34 after 1914 is Clark's own construction: Stamp's rents projected with Feinstein's
  UK farm rents to 1944; then DEFRA land prices x an assumed 3% return x an assumed 28 m
  acres (1945–67); then DEFRA tenancy rents x 28 m acres. Its values from the 1920s rest on
  those assumptions.
- The A17 series (rent of land and buildings, Mitchell) falls much less: 0.150 (1855) to
  0.117 (1913). It is an upper bound that includes buildings. It is not a land share and it
  does not contradict the others.
- After 1869 every land share has a constructed denominator. Clark T34 scales UK income to
  England by population. The Clark 2002 and Stamp constructions divide by the Bank's England
  or GB GDP, and the Bank's England GDP is itself its England share of GB (84–87%,
  interpolated) times GB GDP.
- Clark 2002's rents column is labelled "rents and local taxes", but its arithmetic reads
  as rent plus tithe without local taxes. Unresolved. It moves the level by a few percent,
  not the trend.
- The price proxy uses Clark's rent and tithe per acre. Its England averages for 1500–39
  and 1560–79 have no North figure. It jumps 2.5-fold between 1580–99 and 1600–09 (0.141
  to 0.351 £). That is Clark's table as printed (double entry agrees). It predates the exit.

### What would change the reading

- Clark's 2023 workbook as the canonical vintage. It puts medieval land rents much lower:
  0.071–0.108 in the 1250s–1290s, against 0.155–0.197 in the 2015 file. From 1500 the two
  agree. Under it the fall from the 1770s is the same, but the share stays inside its
  floor-era range until the file ends in 1869.
- Allen's rent share for 1770–1913 exists only as a figure (Allen 2009, Fig. 2). Not
  digitised.
- A land-only series for 1914–1920 would test the step in the 1920s.
- A rent series for 1842–1914 that is not Schedule A would give the exit a second witness.

## 4. The turns side by side

This is not the D2 test. D2 needs two dates estimated independently by a fit. Below are
two raw readings. Each applies one criterion to both sides.

**By eye: the start of each series' final move.**

- Land's share: the 1760s–1780s (Clark 2015 and Table 13 fall in every decade after the
  1770s; the Clark 2002 construction from the 1760s).
- The day-wage series: the 1810s–1820s.
- The annual-income series: the 1740s–1770s.
- Land's turn against the annual-income turn: from 10 years before to 40 years after. The
  band includes coincidence. Against the day-wage turn: 30 to 60 years before.

**By rule: the first decade outside the series' own 1250s–1740s decade range** (land below
it, the wage above it; `c4_range_exit.csv`).

- Land's share: the 1820s (Clark 2015), the 1830s (Table 13). None before 1869 under
  Clark's 2023 vintage.
- The annual-income series: the 1750s (HW19 annual, HW16).
- The day-wage series: the 1830s (Allen labourer; unbroken from the 1850s), the 1850s
  (Clark craftsman), the 1860s (Clark avg/COL, Clark labourer). Clark farm and HW19 day:
  none before their series end. The BoE composite: the 1870s.
- Land's exit against the annual-income exit: 70 to 80 years after. Against the day-wage
  exits: 0 to 40 years before (20 to 40 years before if the wage's run outside its range
  must be unbroken). The band includes coincidence: Table 13 and the Allen ratio both
  first leave their ranges in the 1830s.
- The rule is crude. For the day wages, leaving the range means passing the
  fifteenth-century peak. For HW19 annual and HW16 it means passing their 1730s level,
  their floor-era high. For land it means falling below the 1560s low. The thresholds do
  not mean the same thing.

Rent per acre in days of labour stays flat (building labourer) or rises (farm labourer)
until 1869, and falls after. That fall starts at the 1869 step in the wage series.

The raw record does not show one joint date that all constructions agree on. It does not
rule one out: both dates are bands, and each reading allows coincidence for some pairing
(by eye, land with the annual-income series; by rule, land with the day-wage series).

## 5. Breakpoint A: the questions and what the data now answer

The five questions are in the long-record spec, §6 (laborformal `31b3482`). They are
yours. The data inform them. They answer none of them outright.

1. **Do D1–D3 justify the build, given the refutation narrowed the prize?** Not answered.
   Informed: D3's raw ingredients are present (§1: the floor-era opposition in the two big
   swings, and a Black Death window with wages and rents both measured, rents only by
   decade). The pre-1340 opposition is not confirmed by an independent population source.
   D2's two datings are visible raw and do not coincide under every construction (§4). So
   D2 can fail, which is what makes it a test. For D1: the price proxy stays flat in
   building-labourer days, and rises about a fifth in farm-labourer days, while population
   nearly triples (1770s–1860s). It falls after 1869, starting at a step in the wage
   series. That is a raw pattern, not D1's transition-path algebra, which stays open.
2. **Frequency: decadal baseline plus an annual Black Death window?** Partly answered.
   Decadal: every series supports it. Annual window: not for population or rents. No annual
   population exists before 1541 in these sources (Broadberry has benchmarks at 1348 and
   1351; Clark steps from 4.484 m in 1340–49 to 3.545 m in 1350–59, and before 1540 Clark's
   is partly inferred from the wage). Clark's land rents are one value per decade there.
   Wages and prices are annual (Clark, Allen).
3. **Scope: England-only spine, US fork as comparison?** Partly answered. England-only
   population runs to 2016. An England land share runs to 2008 (Clark T34, with a
   UK-derived denominator, and after 1914 a projection). A real wage deflated by English
   prices ends in 1869. Past 1869 the only real wage from England wages is Clark's own
   Table 34 splice over UK prices. An England spine needs a named GB/UK wage splice at
   1869–1882 (Feinstein GB to 1881; the BoE composite after), or Clark's.
4. **Fit strategy: calibration and a pre-registered moment table, SMM only if needed?**
   Not answered. Informed by the count: 62 decades 1250s–1860s; per decade a wage band of up
   to eight constructions, one population source independent of wages (two sources from
   1540), three land-share constructions that share Clark's rents, and rent per acre from
   1500. Before 1541 population is 16 benchmarks, not 29 decades.
5. **Does the thread stay live, or park?** Not answerable from data. This sheet informs it.
6. The standing question (spec §8), sequel or separate paper: not answerable from data.

## 6. Decisions, and what stays open

The user left the three calls this sheet raised to Claude (2026-09-26). They are
recorded here and in STATE.md, open to veto like any other decision.

1. **Breakpoint B passes.** The record shows the floor era's opposition in its two big
   swings, the escape, and land's exit as a fall, as "Our reading" states. The long-record
   purpose continues. The escape's start is carried as a band, the date of land's exit
   as vintage-dependent, and whether the two turns coincide is left to the fit (D2).
2. **The figures stay local.** The Bank's terms ask for permission before re-use, and
   Clark's files state no licence. The repository keeps the scripts, the manifests with
   checksums, the notes, BNS's CC0 files and our public-domain entries; anyone can
   rebuild the figures after fetching. The numbers quoted in this sheet's text are cited
   scholarly use. Publishing the figures waits on the Bank's permission.
3. **The welfare ratio is ours, labelled as ours.** Allen's published welfare ratios are
   still out of reach (his site is behind a login). The sheet and later work use Allen's
   day wage over the cost of his respectable basket, from a verified mirror of his own
   spreadsheet, always named as our construction and never as his series.

Still open:

- The canonical wage band and the technology-knot budget are Breakpoint B's other two
  decisions. They belong to Phase 6's registration and are not proposed here. If the
  band leans on HW19 annual, its level is mostly fixed at 1 before 1370 (§1).
- Allen's published welfare ratios, Feinstein (1998) at source, Wrigley–Schofield (1981)
  population and Turner's enclosure acreage were not reached. Nothing was substituted for
  them.
- The Bank's permission, if the figures are ever to be published.

## Our reading

A pre-look for your ruling at Breakpoint B, dated 2026-09-26, revised after the R11 audit.
It is not the ruling.

- **The floor era's opposition: shows, in the two big swings.** The fall from the 1340s to
  the 1450s and the rise from the 1500s to the 1640s have the real wage moving the other
  way in all eight wage constructions against Broadberry's population, the source built
  without wages. Population fell 58% and wages rose 45–133%; population rose 135% and
  wages fell 19–47%. Clark's population agrees, but before 1540 it is partly inferred from
  the wage, so it is not a second witness. The edges are weaker. Before 1340 the day wages
  fall 26–35% while Broadberry's population rises 7%, mostly by interpolation: not an
  opposition shown by independent evidence. The first decade after 1348 is mixed. HW19's
  annual-contract series is flat before 1370, partly by construction (it is 1 +
  cash/basket). Before 1541 population is 16 benchmarks, so only the big swings are data.
- **The escape: shows.** Every construction that spans the 19th century rises for good
  while population rises, and the BoE composite carries the rise to 2016. Its start is not
  one date. By eye: the 1810s–1820s in the day-wage series, the 1740s–1770s in the
  annual-income series. By the range rule: the 1750s (annual-income) to the 1830s–1870s
  (day wages and the BoE composite); Clark's farm wage and HW19 day do not leave their
  floor-era range before their series end. The date is construction-dependent and should
  be carried as a band.
- **Land's exit: shows as a fall; its date depends on the vintage.** Land's share falls
  from about 0.20 in the 1770s to about 0.07 in the 1860s in every Clark vintage, then to
  about 0.02 by the 1910s and 0.002 by 2000 (Table 34; after 1914 a projection). By eye
  the fall starts in the 1760s–1780s. Whether it leaves the floor-era range before 1869
  depends on the vintage: the 1820s (Clark 2015), the 1830s (Table 13), not at all (Clark
  2023). The top of the 2015 floor band rests on the doubling of Clark's rents between the
  1590s and the 1600s. The constructions share Clark's rents before 1842 and Schedule A
  from 1842: one source family, not five witnesses. The price proxy does not show the exit
  before 1869: an acre cost as many building-labourer days in the 1860s as in the 1770s,
  and more farm-labourer days.
- **Not shown: one joint date.** By eye, land turns in the 1760s–1780s and the wage in the
  1740s–1770s or the 1810s–1820s, by construction. By the range rule, land leaves its range
  in the 1820s–1830s and the wage in the 1750s or the 1830s–1870s. Each reading allows
  coincidence for some pairing and excludes it for others. The raw record neither shows
  one joint reconfiguration nor rules it out. That is D2's question and it needs the fit.
