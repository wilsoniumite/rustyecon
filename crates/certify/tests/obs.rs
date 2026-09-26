//! What the batteries read (docs/CERTIFY.md §5): an `Obs` is the tick's report as it came, every
//! number by its bits, with each provider's transfer from its own record.

mod common;

use certify::Obs;
use common::*;
use rustyecon_engine::prelude::{Provenance, TickReport};
use rustyecon_engine::rustyecon_agents::ActorState;

fn minus_sum(r: &TickReport, g: usize, p: Provenance) -> f64 {
    -r.audit
        .lines
        .iter()
        .filter(|(good, prov, _)| good.idx() == g && *prov == p)
        .fold(0.0, |a, (_, _, q)| a + q)
}

#[test]
fn obs_reads_the_report() {
    // Every field an Obs holds is the report's, and a market trades exactly when its line does
    // (markets' predicate): on the gate world, and on bcycle, whose tick 6,000 has a good market
    // with no supply and whose provider falls short of its transfer.
    for (text, until) in [(GATE, 2080), (BCYCLE, 6100)] {
        let t = tape(text);
        let mut s = sim(&t);
        let w = s.world().clone();
        let provider = w
            .actors
            .iter()
            .find(|a| a.key.as_str() == "provider")
            .map(|a| a.id);
        let (mut idle, mut short) = (0, 0);
        while s.tick() < until {
            let r = s.step().unwrap();
            let o = Obs::of(&r, &s);
            assert_eq!(o.tick, r.tick);
            assert_eq!(o.markets.len(), r.markets.len());
            for (m, l) in o.markets.iter().zip(&r.markets) {
                let bits = |x: f64| x.to_bits();
                assert_eq!((m.node, m.good), (l.node, l.good));
                for (a, b) in [
                    (m.price, l.price),
                    (m.next_price, l.next_price),
                    (m.supply, l.supply),
                    (m.demand, l.demand),
                    (m.cleared, l.cleared),
                    (m.buyer_fill, l.buyer_fill),
                    (m.seller_fill, l.seller_fill),
                ] {
                    assert_eq!(bits(a), bits(b));
                }
                assert_eq!(m.trades(), l.trades());
                idle += usize::from(!l.trades());
            }
            assert_eq!(o.rationing.len(), r.rationing.len());
            for (x, l) in o.rationing.iter().zip(&r.rationing) {
                let line = r.markets[x.market as usize];
                assert_eq!((line.node, line.good), (l.node, l.good));
                assert_eq!((x.class, x.side), (l.class, l.side));
                assert_eq!(
                    (x.requested, x.feasible, x.filled),
                    (l.requested, l.feasible, l.filled)
                );
            }
            for g in 0..w.n_goods() {
                assert_eq!(o.consumed[g], minus_sum(&r, g, Provenance::Consumption));
                assert_eq!(o.spoiled[g], minus_sum(&r, g, Provenance::Spoilage));
            }
            assert_eq!(
                (o.margin, o.run_margin),
                (r.audit.max_margin, r.run.max_margin)
            );
            assert_eq!(o.price_shocks, 0);
            match provider {
                Some(a) => {
                    let Some(ActorState::Provider(p)) = s.actor_state(a) else {
                        panic!("a provider")
                    };
                    assert_eq!(o.transfers, [(a, p.due, p.paid)]);
                    short += usize::from(p.paid < p.due);
                }
                None => assert!(o.transfers.is_empty()),
            }
        }
        if text == BCYCLE {
            assert!(idle > 0 && short > 0, "{idle} {short}");
        }
    }
    // A dated price shock is counted on its tick.
    let gate = tape(GATE);
    let mut t = gate.clone();
    t.params.push(
        ron::from_str(
            "(key: \"shock.factor\", value: 1.5, unit: Dimensionless, basis: Assumed(\"test\"))",
        )
        .unwrap(),
    );
    t.events.push(
        ron::from_str(&format!(
            "(key: \"bread.shock\", at: \"{}\", basis: Assumed(\"test\"), act: ScalePrice(node: \
             \"town\", good: \"bread\", by: \"shock.factor\"))",
            date_of(&gate, 30)
        ))
        .unwrap(),
    );
    let mut s = sim(&t);
    let mut shocks = Vec::new();
    while s.tick() < 40 {
        let r = s.step().unwrap();
        shocks.push(Obs::of(&r, &s).price_shocks);
    }
    assert_eq!(shocks.iter().sum::<u32>(), 1);
    assert_eq!(shocks[30], 1);
}
