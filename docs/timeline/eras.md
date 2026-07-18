# Economic Eras: 1836–2036

> **v2 triage (2026-07-18): KEEP throughout.** This file is worldgen research — it feeds the regions / tech / policy / war timeline tables (architecture/worldgen.md, PLAN Phase 8). Two standing notes: the span is 1800–2025 per the fundamentals (the 1836–2036 frame below predates that ruling — extend backward to 1800, trim forward to 2025, cut no content); the colonial, slavery, and grain-granularity questions remain open design challenges for the world compiler, now joined by price-controls-vs-posted-prices for the war eras.

A collaborative design document. Each era covers the goods, recipes, technologies,
trade conditions, policies, and design challenges relevant to the simulation.
The goal is to discover requirements from history rather than design in the abstract.

---

## Era 1: 1836–1880 — Steam and Cotton
> **v2: KEEP** — worldgen research; requirements-from-history is the right method and continues per era.

### Overview

The world in 1836 is sharply divided. Britain is the only fully industrialised
nation, exporting manufactured goods and importing raw materials from everywhere
else. The rest of Europe is proto-industrial at best. The Americas, Asia, and
Africa are overwhelmingly agrarian, with production organised around colonial
extraction, subsistence farming, or plantation agriculture built on enslaved
labour. The steam engine exists and is transforming British manufacturing, but
railways are just beginning and steam shipping still competes with sail.

By 1880 the picture is completely different: railways criss-cross Europe and
North America, the Bessemer converter has made steel cheap and abundant, the
telegraph connects continents, steamships dominate ocean trade, and several
nations are beginning to challenge British industrial supremacy.

---

### Goods

**Food and agricultural commodities**

Grain (wheat, maize, rice) is the foundation of every economy. In 1836 most
countries grow most of their own grain; international trade in grain is
significant but not yet dominant. This changes after the Corn Laws repeal in
1846 and accelerates with railways opening up the American Midwest in the
1860s–70s.

Cotton is the most important internationally traded industrial raw material
of this era. US Southern plantations supply Lancashire mills, which re-export
fabric worldwide. A disruption to cotton supply — as happened during the US
Civil War — cascades through the entire industrial economy.

Sugar and tea are the era's defining luxury consumption goods. Both flow
through colonial circuits: sugar from the Caribbean and Brazil, tea from China
and later India. They are not luxuries for the wealthy — they are everyday
items for British working families by 1836 — but they are overwhelmingly
imported.

Wool, silk, tobacco, timber, and fish round out the major primary goods.
Rubber appears late in the era (commercial Brazilian rubber from the 1850s)
and will become critical in later eras.

**Industrial inputs and outputs**

Coal is everything in this era. It powers the mills, the railways, the
steamships, and the foundries. Regions with coal are industrialising; regions
without it are not. Britain's coal advantage is one of the primary reasons for
its industrial lead.

Iron (as pig iron and wrought iron) is the structural material of the era.
Iron rails, iron ships, iron bridges, iron machinery. Steel exists but is
expensive and limited to specialist uses — knife blades, springs — because
no cheap process exists yet.

Fabric (cotton cloth, woollen cloth) is the first mass-produced industrial
good. The Lancashire mill system is the template for industrial capitalism.
From raw cotton to finished cloth is a chain of several steps, each becoming
mechanised at different points.

Chemicals at this point means soap, bleaches, basic dyes, and gunpowder.
Synthetic chemistry barely exists; aniline dyes are discovered in 1856 and
will transform this category in the next era.

**Weapons and military goods**

Small arms (muskets, early rifles) and artillery are significant trade goods.
A nation that can manufacture its own weapons is strategically independent; one
that cannot is dependent. The weapons technology gap between industrialised and
non-industrialised regions is already substantial in 1836 and widens throughout
the era.

---

### Recipes and Production

The production chain of this era is short and relatively legible.

Farms produce grain, cotton, wool, and livestock. The inputs are land and labour;
the productivity difference between regions is enormous but driven mainly by
soil quality, climate, and labour regime (free wage labour vs. enslaved vs.
serf vs. subsistence peasant).

Textile mills take raw cotton or wool and produce fabric. This is a multi-step
process — spinning, weaving, finishing — each step having been mechanised to
varying degrees. A mill in 1836 uses steam power for spinning but may still
use hand-looms for weaving; by 1880 the whole chain is mechanised.

Iron foundries take coal and iron ore and produce pig iron, which is then worked
into iron goods (rails, beams, machinery, tools). The Bessemer converter
arriving in 1856 transforms this: you now have a recipe that takes pig iron and
produces cheap steel. This is one of the most important recipe changes in the
entire simulation.

Shipyards in 1836 produce wooden sailing ships or early iron-hulled steamships.
The inputs are timber, iron, and engines. By 1880 the recipe has shifted
decisively: steel hull, steam engine, much less timber. This is a recipe
transition driven by technology unlocks.

---

### Technologies

The definitive technology of this era is the **railway**. It does not produce
a new good so much as it transforms the cost and speed of moving all existing
goods. A region gaining railway access effectively has its channel crossing
costs cut dramatically for all physical goods. A region without railways is
increasingly disadvantaged for export industries.

The **telegraph** is the era's information technology. In simulation terms,
it reduces the information lag for agents making trade decisions. It is
probably best modelled as reducing the price_adjustment_speed lag (alpha)
for goods in connected markets rather than as a separate good.

The **Bessemer converter** (1856) is the most important production recipe
change of the century. Cheap steel unlocks everything that comes after:
steel ships, steel-framed buildings, steel rails. Before it, steel is a
luxury material. After it, steel becomes the primary structural material.

**Steamship** displacement of sail happens gradually through this era. In
1836 steam supplements sail on shorter routes; by 1880 steam dominates ocean
routes. In simulation terms this means the capacity and crossing cost of
maritime channels improves progressively as a technology effect.

---

### Trade Outlook

Britain is the world's dominant trading nation and its only committed free
trader. The Corn Laws repeal in 1846 is a defining moment — Britain deliberately
opens its grain market, accepting dependence on imported food in exchange for
cheaper labour costs and larger export markets for manufactured goods. No
other major power makes this choice.

The dominant trade flows:
- Raw cotton: US South → Britain (one of the world's largest single commodity flows)
- Finished fabric: Britain → everywhere (Britain's primary export)
- Coal: Britain → Europe and beyond (Britain exports energy as well as goods)
- Grain: increasingly, New World → Britain and Europe as the era progresses
- Tea, sugar, spices: colonies → Britain and Europe

The US is the most significant non-European economy. Its northern states are
industrialising rapidly; its southern states are a raw material exporter built
on enslaved labour. This tension ends in the Civil War (1861–65), which disrupts
cotton supply globally and is the era's most significant economic shock.

---

### Policies

Most governments in 1836 are mercantilist by instinct: high tariffs, preference
for domestic industry, control of colonial trade. Britain is the exception.
The US, Germany (as Prussia and the German states), and France all use tariffs
aggressively to protect infant industries — this is actually the mainstream
economic argument of the time (Hamilton, List).

The **gold/silver standard** is the dominant monetary arrangement, though it
is more informal than it will become. Most major currencies are convertible
to gold or silver at fixed rates, but this is often suspended during wars and
crises. There is no international coordination; each country manages its own
metallic reserves.

Government spending is tiny by modern standards. No income tax in most places
(Britain reintroduces it temporarily in 1842). No welfare state. The primary
government expenses are military, debt service, and basic administration.
The Poor Laws in Britain represent one of the few transfer mechanisms — and
they are punitive by design (workhouses).

Labour law barely exists. Child labour in factories is being restricted in
Britain for the first time (Factory Acts 1833, 1844) but is entirely normal
everywhere else. Trade unions are either illegal or barely tolerated.

---

### Design Challenges and Open Questions

**The colonial question.** Colonial territories are not independent countries
but their resources are central to the global economy. Do we model them as
regions within the colonising country's market (with special extraction
mechanics), as nominally independent countries with very constrained policy,
or simply as regions with asymmetric trade terms? The East India Company is
a particularly strange entity — a corporation that functions as a state,
with its own army and territorial sovereignty. It disappears in 1858.

**Slavery as a labour regime.** Enslaved labour is economically significant
in this era — US cotton, Caribbean sugar, Brazilian coffee. In production terms
it represents extremely cheap labour with near-zero wage costs (subsistence
costs only) and zero bargaining power. The simulation needs to be able to
represent this regime and its abolition as a shock. One approach: a pop type
or labour law that sets wage costs to a floor value and removes employment
elasticity. Abolition is then a policy change that dramatically alters the
production economics of affected regions.

**Grain granularity.** The question of whether to model grain as one good or
as wheat/maize/rice separately. For this era, wheat is the politically
important one (Corn Laws, Irish Famine). Maize is the subsistence crop of
much of the world. Rice dominates Asia. Separating them probably matters for
regional modelling; aggregating them may be acceptable if we treat grain as
a region-specific production rather than a globally fungible commodity.

**Sail versus steam.** Whether to distinguish sail and steam as separate
shipping types with different cost curves and route access, or simply model
the transition as a continuous improvement in maritime channel capacity.
The distinction matters if we want to model things like the Suez Canal (1869),
which was economically insignificant for sailing ships but enormously
significant for steamships.

**Early financial services.** London in 1836 is the world's financial centre.
Merchant banks, insurance markets, and bond markets are highly developed.
This matters for the simulation because it means British-based entities can
finance productive investment globally in a way no other market can. Whether
we model this level of detail in Era 1 or abstract it is a real question.

---

*[Era 2: 1880–1914 — to be written]*
