//! A zero price markets can hold, on the engine (P2.4; docs/probe/FREE-RULES.md; the free scan,
//! docs/probe/free/SPEC.md §3, §6.4, §6.5): IL1 and CT2's points are the scan's 50-digit solve,
//! they rest at the oracle's point at every registered target with the free-able market at the
//! oracle's price, the tapes are their generator's output while every committed tape keeps its
//! text, ids and stream, the harness reads the free state through the rules' own code, and the
//! grammar, battery and families are the registered ones.

use certify::tape_hash;
use probe::markets::harness::{commons_regime, run, Row};
use probe::markets::instance::Instance;
use probe::markets::kick::kick_set;
use probe::markets::perturb::{battery, family, tier3s, Perturbation};
use probe::markets::probes::elasticity;
use probe::markets::setup::{genesis, tape_ron, Setup};
use probe::perturb::Start;
use rustyecon_engine::prelude::{Sim, Tape};

const IL1: &str = include_str!("../../../tapes/markets-il1.ron");
const CT2: &str = include_str!("../../../tapes/markets-ct2.ron");
const POINTS: &str = include_str!("../../../docs/probe/free/registered/points_fine.json");
const SCAN_F: &str = include_str!("../../../docs/probe/free/registered/scanF.jsonl");
const REG_H: &str = include_str!("../../../docs/probe/free/registered/regH.jsonl");

fn sim(text: &str) -> Sim {
    Sim::new(&Tape::from_ron(text).expect("the tape parses")).expect("the tape loads")
}

fn setup(id: &str, tpy: u32) -> Setup {
    Setup::registered(id, tpy).expect("the free instance")
}

/// Every registered target of an instance: the base, then each cost coefficient at its four
/// values, named as the scan names them (`<coef> x<factor>`, CT2's `exit.To` as `commons`).
fn targets(base: &Instance) -> Vec<(String, Instance)> {
    let mut out = vec![("base".to_string(), base.clone())];
    for c in &base.coefs {
        for (v, f) in c.values.iter().zip(["1.1", "0.9", "2", "0.5"]) {
            let mut i = base.clone();
            i.set(&c.param, v.parse().unwrap()).unwrap();
            let name = if c.name == "exit.To" {
                "commons"
            } else {
                c.name.as_str()
            };
            out.push((format!("{name} x{f}"), i));
        }
    }
    out
}

/// A string field of one point of the scan's points_fine.json, parsed.
fn point(key: &str, field: &str) -> f64 {
    point_text(key, field).parse().unwrap_or(f64::NAN)
}

fn point_text(key: &str, field: &str) -> String {
    let start = POINTS
        .find(&format!("\"{key}\": {{"))
        .unwrap_or_else(|| panic!("no point {key}"));
    let block = &POINTS[start..];
    let end = block.find("\n }").unwrap_or(block.len());
    let block = &block[..end];
    let at = block
        .find(&format!("\n  \"{field}\": "))
        .unwrap_or_else(|| panic!("{key}: no {field}"));
    let rest = &block[at + field.len() + 7..];
    rest.split([',', '\n'])
        .next()
        .unwrap()
        .trim()
        .trim_matches('"')
        .to_string()
}

/// The i-th string of a list field of a point.
fn point_list(key: &str, field: &str, i: usize) -> f64 {
    let start = POINTS.find(&format!("\"{key}\": {{")).unwrap();
    let block = &POINTS[start..];
    let at = block.find(&format!("\n  \"{field}\": [")).unwrap();
    let rest = &block[at..];
    let list = &rest[rest.find('[').unwrap() + 1..rest.find(']').unwrap()];
    list.split(',')
        .nth(i)
        .unwrap()
        .trim()
        .trim_matches(|c| c == '"' || c == '\n' || c == ' ')
        .parse()
        .unwrap()
}

#[test]
fn free_points_are_the_registered_ones() {
    // The free scan's §3.3: the harness's point, unit 1e's `ParcelEconomy` at the instance's
    // per-tick values (CT2 with two worker types), is the scan's independent 50-digit solve at all
    // 26 registered targets, which reads no oracle code: x*, P_s, every price over w, Y, S, each
    // pop's hours, T_m, T_p and r_o/w within 1e-13 (relative, or absolute at 0), and the same
    // regime and land market. IL1 is on the idle stretch in wage units (r 0, w 1) at every target
    // but inst.land × 0.5, the wall; CT2 is on the line with its commons in every regime.
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-13 * a.abs().max(b.abs()).max(1.0);
    let mut n = 0;
    for (id, key) in [("il1", "IL1"), ("ct2", "CT2")] {
        for (what, inst) in targets(&setup(id, 52).instance) {
            let k = format!("{key} {what}");
            let e = inst.point(52).unwrap_or_else(|x| panic!("{k}: {x}"));
            let c = e.commons.as_ref().expect("unit 1e's readouts");
            // The point's wage: w = v (1 on the idle stretch, v = w/r on the line or the wall).
            let w = e.v;
            let mut checks = vec![
                (e.x_star, "x".to_string(), point(&k, "x")),
                (e.p_s / w, "Ps_over_w".into(), point(&k, "Ps_over_w")),
                (
                    e.type_price[0] / w,
                    "pm_over_w".into(),
                    point(&k, "pm_over_w"),
                ),
                (e.y, "Y".into(), point(&k, "Y")),
                (e.pool, "S".into(), point(&k, "S")),
                (c.market_land, "Tm".into(), point(&k, "Tm")),
                (c.rented, "Tp".into(), point(&k, "Tp")),
                (c.plot_rent / w, "ro_over_w".into(), point(&k, "ro_over_w")),
                (e.rent / w, "r_over_w".into(), point(&k, "r_over_w")),
            ];
            for (j, cat) in inst.categories.iter().enumerate() {
                let start = POINTS.find(&format!("\"{k}\": {{")).unwrap();
                let block = &POINTS[start..];
                let at = block.find(&format!("\"{}\": \"", cat.key)).unwrap();
                let rest = &block[at + cat.key.len() + 5..];
                let want: f64 = rest.split('"').next().unwrap().parse().unwrap();
                checks.push((e.cat_price[j] / w, format!("p_over_w.{}", cat.key), want));
            }
            for (i, h) in e.hours.iter().enumerate() {
                checks.push((*h, format!("ns[{i}]"), point_list(&k, "ns", i)));
            }
            for (got, field, want) in checks {
                assert!(close(got, want), "{k}: {field} {got:e} against {want:e}");
            }
            // r_o is r at an Enclosed point (the scan writes its plot rent over w there).
            assert_eq!(c.regime, point_text(&k, "regime"), "{k}");
            // Each pop's money account at CT2 (the binding readout's target, O102): N_i +
            // (w·S_i + r_o·(T_o,i − its commons plots))/P_s where no plot is rented, its plots G_i
            // the 50-digit solve's `G_types`; the households' sum to Y.
            if key == "CT2" && c.rented == 0.0 {
                for (i, k2) in inst.commoners.iter().enumerate() {
                    let n_i = k2.workers / 52.0;
                    let t_o = k2.share / 52.0;
                    let want = n_i
                        + (e.v * e.hours[i] + c.plot_rent * (t_o - point_list(&k, "G_types", i)))
                            / e.p_s;
                    assert!(close(e.pop_baskets[i], want), "{k}: pop {i}'s baskets");
                }
            }
            let total: f64 = e.pop_baskets.iter().sum::<f64>() + e.provider_baskets;
            assert!(
                close(total, e.y),
                "{k}: the households' baskets {total} against Y"
            );
            let idle = point_text(&k, "where") == "idle";
            assert_eq!(c.land_market == "Idle", idle, "{k}");
            assert_eq!(e.rent == 0.0, idle, "{k}");
            n += 1;
        }
    }
    assert_eq!(n, 26);
}

#[test]
fn il1_ct2_rest_at_the_oracle() {
    // The free scan's §6.5 test 7 and §7: at every registered target (and the bases at 12 and
    // 365 ticks a year), three ticks from genesis at the oracle's point leave every observable
    // within 1e-12 of the oracle in log and every market trading; mode A's check passes (it does
    // not read the free-able market's fills while it posts 0); and the free-able market is at the
    // oracle's price: 0 where the oracle's is 0 (IL1's idle targets, CT2's Commons), and at its
    // positive price, r/w at IL1's wall and r_o/r at CT2's Crowded targets, within 1e-12.
    for (id, key) in [("il1", "land"), ("ct2", "commons")] {
        let base = setup(id, 52);
        let mut cases: Vec<(String, Instance, u32)> = targets(&base.instance)
            .into_iter()
            .map(|(what, i)| (what, i, 52))
            .collect();
        cases.push(("base".into(), base.instance.clone(), 12));
        cases.push(("base".into(), base.instance.clone(), 365));
        for (what, inst, tpy) in cases {
            let mut s = setup(id, tpy);
            s.instance = inst;
            let mut rows: Vec<Row> = Vec::new();
            let rec = run(&s, "hold", 3, &mut |r| rows.push(r.clone())).unwrap();
            assert_eq!(rec.hold_failure, None, "{id} {what} tpy {tpy}");
            let e = &rec.genesis.point;
            let c = e.commons.as_ref().unwrap();
            let m = s.instance.markets().iter().position(|x| x == key).unwrap();
            // The oracle's price of the free-able market, in the point's units.
            let star = if key == "land" { e.rent } else { c.plot_rent };
            assert_eq!(rows.len(), 3);
            for r in &rows {
                let worst = r.gap.iter().fold(0.0, |a: f64, &g| a.max(g));
                assert!(worst <= 1e-12, "{id} {what} tpy {tpy}: {worst:e}");
                assert!(
                    r.trades.iter().all(|&t| t),
                    "{id} {what}: every market trades"
                );
                let [posted, next, _] = r.free.expect("the free readout");
                if star == 0.0 {
                    assert_eq!(posted.to_bits(), 0.0_f64.to_bits(), "{id} {what}: free");
                    assert_eq!(next.to_bits(), 0.0_f64.to_bits(), "{id} {what}: stays free");
                } else {
                    let rel = |p: f64| (p / r.price[0]) / (star / e.v);
                    assert!(
                        (rel(posted) - 1.0).abs() <= 1e-12,
                        "{id} {what}: {posted:e}"
                    );
                    assert!((rel(next) - 1.0).abs() <= 1e-12, "{id} {what}: {next:e}");
                }
                assert_eq!(r.price[m], posted);
                // Every actor's coin is stationary: each pop's at CT2 holds its own commons rent,
                // which the pops' aggregate demand would hide.
                for (a, (&c1, &c0)) in r.coin.iter().zip(&rec.genesis.coin).enumerate() {
                    assert!(
                        (c1 - c0).abs() <= 1e-12 * c0.abs().max(1e-300),
                        "{id} {what} tpy {tpy}: actor {a}'s coin {c1} against {c0}"
                    );
                }
            }
        }
    }
}

#[test]
fn il1_ct2_tapes_are_their_generators_output() {
    // The free scan's §6.5 test 9: tapes/markets-il1.ron and markets-ct2.ron are `markets-tape
    // --inst il1` and `--inst ct2`'s output for the registered setup at 52 ticks a year, byte
    // for byte, each with its free step on its free-able good and, at CT2, the pops' market.
    for (id, text) in [("il1", IL1), ("ct2", CT2)] {
        let generated = tape_ron(&setup(id, 52)).expect("the generator runs");
        assert!(
            generated == text,
            "tapes/markets-{id}.ron differs from its generator: run `cargo run -p \
             rustyecon-probe --bin markets-tape -- --inst {id} tapes/markets-{id}.ron`"
        );
    }
    assert!(IL1.contains(
        "(key: \"land\", life: Instant, price_rate: Some(\"rate.land\"), free: Some((reference: \
         \"labour\", scale: \"free.land\"))),"
    ));
    assert!(IL1.contains("(node: \"home\", good: \"land\", price: 0.0),"));
    assert!(CT2.contains(
        "(key: \"commons\", life: Instant, price_rate: Some(\"rate.commons\"), free: \
         Some((reference: \"labour\", scale: \"free.commons\"))),"
    ));
    assert_eq!(CT2.matches("market: Some(\"commons\")").count(), 2);
    assert!(!CT2.contains("\"inst.workers\""));
}

/// Every committed tape before the free step: (name, text, tape_hash, world_id, the final hash
/// of its 2,000-tick stream where it is pinned here), as the pre-build binary recorded them
/// (build-free's base streams, 2026-09-30, at adf1ec6, equal on WSL and Windows).
fn committed() -> Vec<(&'static str, &'static str, u64, u64, Option<u64>)> {
    vec![
        (
            "appb",
            include_str!("../../../tapes/appb.ron"),
            0x8973_b237_f4c0_0029,
            0x26f1_2f8a_0bc2_7540,
            None,
        ),
        (
            "demo-gb",
            include_str!("../../../tapes/demo-gb.ron"),
            0x1bd5_6d66_433d_3b67,
            0x9701_ae49_997f_f8f5,
            None,
        ),
        (
            "gate",
            include_str!("../../../tapes/gate.ron"),
            0x5406_6d05_3474_846b,
            0x4362_8a8e_0fd5_f695,
            None,
        ),
        (
            "horses-h1",
            include_str!("../../../tapes/horses-h1.ron"),
            0x4976_7952_83cc_52ed,
            0xc92f_78b1_7dd4_9bb6,
            None,
        ),
        (
            "horses-h2",
            include_str!("../../../tapes/horses-h2.ron"),
            0x7104_4f8a_b931_81a8,
            0x386a_e50b_a941_4be3,
            None,
        ),
        (
            "horses-h3",
            include_str!("../../../tapes/horses-h3.ron"),
            0x7892_9ea1_9c81_fb81,
            0x1562_9782_d6e4_b359,
            None,
        ),
        (
            "horses-h4",
            include_str!("../../../tapes/horses-h4.ron"),
            0x72ee_0740_db8e_0fa2,
            0x5059_ba9b_2ff1_b015,
            None,
        ),
        (
            "horses-p7",
            include_str!("../../../tapes/horses-p7.ron"),
            0x70ca_bdbe_0e0f_a986,
            0xc9e4_2f5f_b078_f10f,
            None,
        ),
        (
            "horses-r1a",
            include_str!("../../../tapes/horses-r1a.ron"),
            0xf31c_56f9_d322_26d1,
            0xddc8_6dbc_11b7_24f3,
            None,
        ),
        (
            "loops-lb1",
            include_str!("../../../tapes/loops-lb1.ron"),
            0xd49b_cc5f_6168_1960,
            0x45d7_2a2c_3e89_c07f,
            None,
        ),
        (
            "loops-lb2",
            include_str!("../../../tapes/loops-lb2.ron"),
            0x7ff5_fdb5_7453_eb79,
            0xfbab_eaaf_a506_fd6a,
            None,
        ),
        (
            "loops-lb3",
            include_str!("../../../tapes/loops-lb3.ron"),
            0x2a04_74ec_e712_080f,
            0x3edb_d023_72b1_05aa,
            None,
        ),
        (
            "loops-lw1",
            include_str!("../../../tapes/loops-lw1.ron"),
            0x8192_c3e8_d230_3568,
            0x968a_acfe_6f60_b521,
            None,
        ),
        (
            "loops-lw2",
            include_str!("../../../tapes/loops-lw2.ron"),
            0x09f2_49ba_9f6e_83f4,
            0x87cb_3cfd_7f0e_7d11,
            None,
        ),
        (
            "loops-lw3",
            include_str!("../../../tapes/loops-lw3.ron"),
            0x4e2c_c849_4edd_6ae4,
            0x248f_4a60_ba48_8f7e,
            None,
        ),
        (
            "markets-c1",
            include_str!("../../../tapes/markets-c1.ron"),
            0x3122_821b_16c9_b76f,
            0xf63e_02fb_0360_98fb,
            Some(0x28df_3904_f206_ca92),
        ),
        (
            "markets-c1p",
            include_str!("../../../tapes/markets-c1p.ron"),
            0x9b48_c13d_e530_83fd,
            0xad31_7674_4831_3bd3,
            Some(0x7d51_33fb_7b90_3a8f),
        ),
        (
            "markets-c2",
            include_str!("../../../tapes/markets-c2.ron"),
            0x181a_824f_9455_2daf,
            0x57b6_8567_1f6b_adc7,
            Some(0xe155_f829_dbd4_fb37),
        ),
        (
            "markets-c2p",
            include_str!("../../../tapes/markets-c2p.ron"),
            0x0890_42f8_c45c_28f0,
            0x3991_5cb5_2d06_d287,
            Some(0xd5ad_10fe_6f6d_fc86),
        ),
        (
            "markets-g1",
            include_str!("../../../tapes/markets-g1.ron"),
            0xff3c_23ea_26a6_464e,
            0x4b0b_1301_cb7a_a351,
            Some(0xe71a_c0e9_6df3_9297),
        ),
        (
            "markets-i0",
            include_str!("../../../tapes/markets-i0.ron"),
            0x9338_5d64_d42f_858e,
            0x5068_f2ff_9dbb_fa00,
            Some(0x6c5b_f916_f3b6_9d35),
        ),
        (
            "markets-i1",
            include_str!("../../../tapes/markets-i1.ron"),
            0x4dbe_0ba9_88f9_4b02,
            0x965e_dbef_2664_b850,
            Some(0x8e3f_1bc7_7c0e_5a91),
        ),
        (
            "markets-i2",
            include_str!("../../../tapes/markets-i2.ron"),
            0x8917_3d95_2d0b_5e65,
            0x6c3e_2217_b9ee_fef9,
            Some(0x958d_5fb0_1a48_47b5),
        ),
        (
            "markets-i3",
            include_str!("../../../tapes/markets-i3.ron"),
            0xdea2_2786_21a5_2649,
            0xb631_f8a0_f7d9_ab95,
            Some(0x6749_dd31_93ba_1712),
        ),
        (
            "markets-is1",
            include_str!("../../../tapes/markets-is1.ron"),
            0xcfc9_a45a_69b2_3b96,
            0xdd77_db99_d4d4_5166,
            Some(0xa4b9_5240_fa8d_8f47),
        ),
        (
            "markets-iw1",
            include_str!("../../../tapes/markets-iw1.ron"),
            0xec2c_4d1c_8f62_ea28,
            0x7698_c09c_b7c5_4791,
            Some(0xe976_7adb_62e0_7cae),
        ),
        (
            "markets-l2",
            include_str!("../../../tapes/markets-l2.ron"),
            0x8aa5_0fca_e92a_6260,
            0xb47c_5301_5f7f_1637,
            Some(0xf45a_65d4_27f9_f629),
        ),
        (
            "markets-l3",
            include_str!("../../../tapes/markets-l3.ron"),
            0xde5e_9b54_8d5d_e83e,
            0xd525_2228_af8a_95a6,
            Some(0x010a_e4da_4eec_fef5),
        ),
    ]
}

#[test]
fn free_field_moves_no_world_id() {
    // The free scan's §6.5 test 5 (R1): the goods' `free` and the exit's `market` are left out of
    // the tape's canonical text and of `world_id`'s bincode when absent, so every committed tape
    // keeps its `tape_hash` and `world_id`, and the markets tapes their 2,000-tick stream, as the
    // pre-build binary recorded them. The pin can move: I1's tape with a free step on land has
    // another `world_id`.
    let tapes = committed();
    assert_eq!(tapes.len(), 28, "every committed tape but IL1's and CT2's");
    for (name, text, hash, id, fin) in tapes {
        let t = Tape::from_ron(text).unwrap();
        assert_eq!(tape_hash(&t), hash, "{name}: tape_hash");
        assert!(!t.to_ron().contains("free:"), "{name}");
        let mut s = sim(text);
        assert_eq!(s.world().world_id, id, "{name}: world_id");
        if let Some(fin) = fin {
            while s.tick() < 2000 {
                s.step().unwrap();
            }
            assert_eq!(s.hash(), fin, "{name}: the stream's final hash");
        }
    }
    let i1 = include_str!("../../../tapes/markets-i1.ron");
    let freed = i1
        .replacen(
            "(key: \"land\", life: Instant, price_rate: Some(\"rate.land\")),",
            "(key: \"land\", life: Instant, price_rate: Some(\"rate.land\"), free: Some((reference: \"labour\", scale: \"free.land\"))),",
            1,
        )
        .replacen(
            "        (key: \"price.ema_tc\",",
            "        (key: \"free.land\", value: 0.5, unit: Dimensionless, basis: Assumed(\"c\")),\n        (key: \"price.ema_tc\",",
            1,
        );
    assert_ne!(sim(&freed).world().world_id, sim(i1).world().world_id);
    assert!(Tape::from_ron(&freed).unwrap().to_ron().contains("free:"));
}

#[test]
fn harness_reads_free_state() {
    // The free scan's §6.5 test 8: the free readouts are read from what the market posted and the
    // regime through the rules' own code. IL1 displaced to land at 0.025·w: the land market is
    // free again at tick 4 (the mirror's trace), the posted price is 0 on exactly the ticks the
    // readout counts, each switch between 0 and a positive price is counted, the end is the next
    // price over the reference's, and the regime is `Idle` where the workers' rule says `Enclosed`
    // at r = 0; the provider's lowest coin is 0. CT2 at b.food × 2: the commons is free for 22
    // ticks and reopens to Crowded, read from its posted price, and each pop's commons bid and
    // offer are its rule's, summing to the market's demand and supply.
    let rows_of = |id: &str, name: &str, ticks: u64| {
        let mut rows: Vec<Row> = Vec::new();
        let rec = run(&setup(id, 52), name, ticks, &mut |r| rows.push(r.clone())).unwrap();
        (rec, rows)
    };
    let (rec, rows) = rows_of("il1", "p[land]=0.025", 300);
    let f = rec.stats.free.as_ref().expect("the free readouts");
    let zero: Vec<u64> = rows
        .iter()
        .filter(|r| r.free.unwrap()[0] == 0.0)
        .map(|r| r.tick)
        .collect();
    assert_eq!(f.ticks, zero.len() as u64);
    assert_eq!(f.first, Some(4));
    assert_eq!(zero[0], 4);
    let switches = rows
        .iter()
        .filter(|r| {
            let [p, n, _] = r.free.unwrap();
            (p == 0.0) != (n == 0.0)
        })
        .count() as u64;
    assert_eq!(f.switches, switches);
    let last = rows.last().unwrap().free.unwrap();
    assert_eq!(f.end, last[1] / last[2]);
    assert_eq!(f.star, 0.0);
    assert_eq!(f.regime_end, "Idle");
    assert_eq!(f.regime_star, "Idle");
    assert!(f.regimes.get("Idle").copied().unwrap_or(0) > 290);
    assert_eq!(f.provider_coin_low, 0.0);
    // r at the end over w (r starts at 0), and the commons' readouts at IL1: r_o/r reads 0 where
    // r is 0, and the provider's lowest coin, which starts at 0, absolute.
    let last_row = rows.last().unwrap();
    assert_eq!(rec.summary.r_end, last_row.price[1] / last_row.price[0]);
    let c = rec.stats.commons.as_ref().expect("the commons' readouts");
    assert_eq!((c.rent.0, c.rent.2), (0.0, 0.0));
    assert_eq!(c.provider_coin_low, 0.0);
    // r_end is r/w where r ends positive too: IL1 at its wall (inst.land × 0.5).
    let (rec, rows) = rows_of("il1", "inst.land=260@genesis", 300);
    let last_row = rows.last().unwrap();
    assert!(last_row.price[1] > 0.0);
    assert_eq!(rec.summary.r_end, last_row.price[1] / last_row.price[0]);
    // At CT2's b.food × 2.
    let (rec, rows) = rows_of("ct2", "b.food=1.2@genesis", 400);
    let f = rec.stats.free.as_ref().unwrap();
    assert_eq!(f.ticks, 22);
    assert_eq!(f.first, Some(0));
    assert_eq!(f.regime_end, "Crowded");
    assert_eq!(f.regime_star, "Crowded");
    let last = rows.last().unwrap().free.unwrap();
    assert!(last[1] > 0.0);
    assert_eq!(f.end, last[1] / last[2]);
    let n = rows[0].price.len();
    for r in &rows {
        let ro = r.price[n - 1];
        assert_eq!(
            commons_regime(ro, r.price[1]),
            if ro == 0.0 {
                "Commons"
            } else if ro < r.price[1] {
                "Crowded"
            } else {
                "Enclosed"
            }
        );
        assert_eq!(r.pops.len(), 2);
        let (bid, offer) = r
            .pops
            .iter()
            .fold((0.0, 0.0), |(b, o), p| (b + p.bid, o + p.offer));
        assert_eq!(
            r.supply[n - 1],
            offer,
            "tick {}: the commons offered",
            r.tick
        );
        assert!(
            (r.demand[n - 1] - bid).abs() <= 1e-15 * bid,
            "tick {}: the commons bid {} against {bid}",
            r.tick,
            r.demand[n - 1]
        );
    }
    // The commons is out of the dead-tick rule, as the mirror's: with no commons (exit.To 0) the
    // pops offer none, and once their plots spill onto enclosed land they bid none, so the
    // commons does not trade, and those ticks are not dead for it.
    let (_, rows) = rows_of("ct2", "exit.To=0@genesis", 300);
    let silent: Vec<&Row> = rows.iter().filter(|r| !r.trades[n - 1]).collect();
    assert!(silent.len() > 100, "{}", silent.len());
    assert!(silent.iter().filter(|r| r.dead).count() < silent.len() / 2);
    // A non-free instance keeps no free readouts.
    assert!(run(&setup("c2", 52), "hold", 2, &mut |_| {})
        .unwrap()
        .stats
        .free
        .is_none());
}

/// A registered JSON line's string field.
fn field<'a>(line: &'a str, key: &str) -> &'a str {
    let at = line
        .find(&format!("\"{key}\": \""))
        .unwrap_or_else(|| panic!("no {key} in {line}"));
    let rest = &line[at + key.len() + 5..];
    &rest[..rest.find('"').unwrap()]
}

/// A registered JSON line's number field, as text.
fn number(line: &str, key: &str) -> String {
    let at = line.find(&format!("\"{key}\": ")).unwrap();
    let rest = &line[at + key.len() + 4..];
    rest.split([',', '}']).next().unwrap().trim().to_string()
}

/// The registered runs of a set at an instance, sorted, named as the engine names them: a
/// desk's coin `coin.<d>` is `coin.desk.<d>`.
fn registered(inst: &str, set: &str) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = SCAN_F
        .lines()
        .chain(REG_H.lines())
        .filter(|l| field(l, "name") == format!("{inst}/zp05") && field(l, "set") == set)
        .map(|l| {
            let run = field(l, "run");
            let run = match run.strip_prefix("coin.") {
                Some(rest) if !rest.starts_with("workers") && !rest.starts_with("provider") => {
                    format!("coin.desk.{rest}")
                }
                _ => run.to_string(),
            };
            (run, number(l, "tier").trim_matches('"').to_string())
        })
        .collect();
    v.sort();
    v
}

#[test]
fn free_battery_and_families_are_the_registered_ones() {
    // The free scan's §4.2 and §9: the battery (IL1 in wage units, 100 runs; CT2 the commons
    // battery's order, 115), each run's tier, Tier 3S (22 and 24), stocks (39 and 41), joint2
    // and joint4 are the registered runs, name for name.
    for (id, key, n) in [("il1", "IL1", 100), ("ct2", "CT2", 115)] {
        let inst = setup(id, 52).instance;
        let mut b: Vec<(String, String)> = battery(&inst, 52)
            .unwrap()
            .into_iter()
            .map(|r| (r.name, r.tier.to_string()))
            .collect();
        assert_eq!(b.len(), n);
        b.sort();
        assert_eq!(b, registered(key, "battery"), "{id}: the battery");
        let names = |v: Vec<String>| {
            let mut v: Vec<String> = v;
            v.sort();
            v
        };
        let reg = |set: &str| names(registered(key, set).into_iter().map(|(r, _)| r).collect());
        assert_eq!(names(tier3s(&inst)), reg("tier3s"), "{id}: Tier 3S");
        for fam in ["stocks", "joint2", "joint4"] {
            assert_eq!(
                names(family(&inst, 52, fam).unwrap()),
                reg(fam),
                "{id}: {fam}"
            );
        }
        // Tier 3 is the dial family's and Tier 3 at 10·L's list.
        let t3 = names(
            battery(&inst, 52)
                .unwrap()
                .into_iter()
                .filter(|r| r.tier == 3)
                .map(|r| r.name)
                .collect(),
        );
        assert_eq!(t3, reg("tier3x10"), "{id}: Tier 3");
    }
}

#[test]
fn free_grammar_applies_as_named() {
    // The free scan's §6.4 grammar: `p[land]=V` sets land's genesis price to V·w, after the
    // factors, and its start distance is infinite (land has no log observable at r = 0); JA is a
    // nominal start at IL1, never VACUOUS; `exit.To=V` at CT2 sets each pop's share of the commons
    // to V/2, at genesis or by two dated changes at L/4; `coin.workers` scales both pops' coins;
    // `joint` draws the commons' price last, where it stays 0.
    let il1 = setup("il1", 52);
    let mut s = il1.clone();
    Perturbation::parse("p[labour]*2+p[land]=0.025")
        .unwrap()
        .apply(&mut s, 1000)
        .unwrap();
    let g = genesis(&s).unwrap();
    assert_eq!(g.prices[1], 0.025 * g.prices[0]);
    assert_eq!(g.prices[0], 2.0);
    let rec = run(&il1, "p[land]=0.025", 2, &mut |_| {}).unwrap();
    assert!(rec.d0.is_infinite());
    let rec = run(&il1, "JA(2)", 2, &mut |_| {}).unwrap();
    assert_eq!(rec.start, Start::Nominal);
    assert_eq!(rec.d0, 0.0);
    assert!(Perturbation::parse("JA(2)").unwrap().ja_only());
    let ct2 = setup("ct2", 52);
    let mut s = ct2.clone();
    Perturbation::parse("exit.To=17.55@genesis")
        .unwrap()
        .apply(&mut s, 1000)
        .unwrap();
    assert_eq!(
        s.at_genesis,
        vec![
            ("inst.wa.commons".to_string(), 8.775),
            ("inst.wb.commons".to_string(), 8.775)
        ]
    );
    let mut s = ct2.clone();
    Perturbation::parse("exit.To=39@dated")
        .unwrap()
        .apply(&mut s, 1000)
        .unwrap();
    assert_eq!(s.shocks.len(), 2);
    assert!(s.shocks.iter().all(|x| x.tick == 250 && x.value == 19.5));
    // The dated shocks load and fire (each schedule param carries its param's unit).
    let t = Tape::from_ron(&tape_ron(&s).unwrap()).unwrap();
    Sim::new(&t).expect("the dated commons shocks load");
    let mut s = ct2.clone();
    Perturbation::parse("coin.workers*2")
        .unwrap()
        .apply(&mut s, 1000)
        .unwrap();
    let g0 = genesis(&ct2).unwrap();
    let g = genesis(&s).unwrap();
    let actors = ct2.instance.actors();
    for (i, a) in actors.iter().enumerate() {
        let f = if a.starts_with("workers.") { 2.0 } else { 1.0 };
        assert_eq!(g.coin[i], g0.coin[i] * f, "{a}");
    }
    // joint(2,1): every market's price drawn in the mirror's order, the commons last and 0.
    let mut s = ct2.clone();
    Perturbation::parse("joint(2,1)")
        .unwrap()
        .apply(&mut s, 1000)
        .unwrap();
    let g = genesis(&s).unwrap();
    assert_eq!(*g.prices.last().unwrap(), 0.0);
    let order = [0usize, 1, 3, 4, 5, 6, 2, 7];
    let mut x = 1u64;
    let mut draw = || {
        x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let unit = ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64;
        rustyecon_core::num::pow(2.0, 2.0 * unit - 1.0)
    };
    for &m in &order {
        let f = draw();
        assert_eq!(g.prices[m], g0.prices[m] * f, "market {m}");
    }
    // Then each desk's share, times 2^u, at most 1: the commons' draw comes before them.
    for (j, sh) in g.shares.iter().enumerate() {
        let want = (g0.shares[j] * draw()).min(1.0);
        assert_eq!(*sh, want, "share {j}");
    }
}

#[test]
fn free_elasticity_and_kick_leave_out_a_free_market() {
    // The free scan's §9.1 E2: the engine's elasticity probe leaves out a market posting 0 at
    // genesis (τ 0) and gives the registered L, 56,000 (IL1) and 141,000 (CT2) at 52 a year; and
    // a kick set at IL1's rest kicks every market but land, which posts 0, and passes.
    for (id, key, l) in [("il1", "land", 56_000), ("ct2", "commons", 141_000)] {
        let e = elasticity(&setup(id, 52), 0.01).unwrap();
        let m = e.markets.iter().position(|x| x == key).unwrap();
        assert_eq!(e.tau[m], 0.0);
        assert_eq!(e.l, l, "{id}");
    }
    let k = kick_set(&setup("il1", 52), "hold", 300, 1500).unwrap();
    assert!(k.pass, "{:?}", k.notes);
    assert!(k.kicks.iter().all(|x| x.market != "land"));
    assert_eq!(k.kicks.len(), 2 * 6);
}
