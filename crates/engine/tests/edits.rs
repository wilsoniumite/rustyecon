//! Edits (docs/ENGINE.md §7.6 and §11, engine; E1, N11): every intervention is a dated tape
//! event or a tape edit followed by a rerun or a resume. A checkpoint survives an edit dated at
//! or after its tick and is refused after any edit to the world or to the past. All comparisons
//! are of hashes or of exact values.

mod common;

use common::*;
use rustyecon_engine::prelude::*;
use rustyecon_engine::rustyecon_core::FlowPerYear;

/// A checkpoint of the gate world's run at `tick`.
fn checkpoint_at(t: &Tape, tick: u64) -> Checkpoint {
    let mut sim = sim_of(t);
    sim.run_until(tick, &mut |_| {}).unwrap();
    sim.checkpoint().unwrap()
}

#[test]
fn resume_after_future_event_edit_equals_full_rerun() {
    // A checkpoint at tick 520 (early 1760) stays valid when the tape gains an event after it
    // and a later event moves, both still after 520; resuming the edited tape from it gives
    // exactly what rerunning the edited tape from genesis gives.
    let t = tape();
    let cp = checkpoint_at(&t, 520);
    let edited_text = edit(
        r#"(key: "mine.restored", at: "1770-06-15""#,
        r#"(key: "mine.restored", at: "1765-02-01""#,
    )
    .replace(
        "    recurring: [",
        "    recurring: [\n        (key: \"windfall\", first: \"1762-06-01\", every: \"pension.period\", \
         last: Some(\"1764-06-01\"), basis: Assumed(\"test\"), act: Mint(holder: \"workers\", good: \
         \"coin\", qty: 40.0)),",
    );
    let edited = tape_of(&edited_text);
    let rerun = hashes(&edited, TICKS);
    assert_ne!(rerun, hashes(&t, TICKS), "the edit changes the run");
    let mut resumed = Sim::resume(&edited, &cp).expect("a future edit keeps the checkpoint");
    let mut tail = Vec::new();
    resumed
        .run_until(TICKS, &mut |r| tail.push(r.hash))
        .unwrap();
    assert_eq!(tail.as_slice(), &rerun[520..]);
}

#[test]
fn resume_after_past_edit_is_refused() {
    // Anything that fired before the checkpoint's tick is part of its past: moving the cut
    // earlier, changing the pension, or moving an event from the future into the past are
    // refused with WrongPrefix. The same edits after the tick are accepted.
    let t = tape();
    let at = 600;
    let cp = checkpoint_at(&t, at);
    let edits = [
        (
            r#"(key: "mine.cut", at: "1760-03-01""#,
            r#"(key: "mine.cut", at: "1760-02-01""#,
        ),
        (
            r#"act: Mint(holder: "pensioners", good: "coin", qty: 5.0)"#,
            r#"act: Mint(holder: "pensioners", good: "coin", qty: 6.0)"#,
        ),
        (
            r#"(key: "oven.opens", at: "1768-04-01""#,
            r#"(key: "oven.opens", at: "1755-04-01""#,
        ),
    ];
    for (from, to) in edits {
        let edited = tape_of(&edit(from, to));
        match Sim::resume(&edited, &cp) {
            Err(ResumeError::WrongPrefix { tick, .. }) => assert_eq!(tick, at),
            other => panic!("{to}: expected WrongPrefix, got {other:?}"),
        }
    }
    // Moving the cut earlier is fine for a checkpoint taken before both dates.
    let early = checkpoint_at(&t, 100);
    let edited = tape_of(&edit(edits[0].0, edits[0].1));
    Sim::resume(&edited, &early).expect("an edit after the checkpoint's tick is allowed");
}

#[test]
fn world_edit_refuses_every_checkpoint() {
    // An edit to the world (a param's value, a genesis stock, a recipe coefficient, a key)
    // changes world_id, and every checkpoint of the old world is refused, whatever its tick.
    let t = tape();
    let cps: Vec<Checkpoint> = [0u64, 1, 520, 1040, 2079]
        .iter()
        .map(|&at| checkpoint_at(&t, at))
        .collect();
    let edits = [
        (
            r#"(key: "farm.capacity", value: 104.0"#,
            r#"(key: "farm.capacity", value: 105.0"#,
        ),
        (
            r#"(holder: "workers", goods: [("coin", 60.0)])"#,
            r#"(holder: "workers", goods: [("coin", 61.0)])"#,
        ),
        (
            r#"outputs: [("grain", 1.0)], capacity: "farm.capacity""#,
            r#"outputs: [("grain", 1.5)], capacity: "farm.capacity""#,
        ),
        (r#"classes: ["households","#, r#"classes: ["homes","#),
    ];
    for (from, to) in edits {
        let mut text = edit(from, to);
        if to.contains("homes") {
            text = text.replace(r#"class: "households""#, r#"class: "homes""#);
        }
        let edited = tape_of(&text);
        for cp in &cps {
            match Sim::resume(&edited, cp) {
                Err(ResumeError::WrongWorld { tape, checkpoint }) => {
                    assert_ne!(tape, checkpoint);
                    assert_eq!(checkpoint, cp.world_id);
                }
                other => panic!(
                    "{to} at {}: expected WrongWorld, got {other:?}",
                    cp.state.tick()
                ),
            }
        }
    }
    // A basis text is not part of the world: correcting a note keeps every checkpoint.
    let noted = tape_of(&edit(
        r#"basis: Assumed("gate world: half")"#,
        r#"basis: Assumed("gate world: half, corrected")"#,
    ));
    for cp in &cps {
        Sim::resume(&noted, cp).expect("a basis edit keeps the checkpoint");
    }
}

#[test]
fn dated_param_change_takes_effect_at_its_tick() {
    // E1: the 1760 cut is a dated SetParam. The mine's capacity changes in phase 0 of the tick
    // the date falls in, and so does its fuel output; not a tick before.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let (cap, fuel) = (param(&w, "mine.capacity"), good(&w, "fuel"));
    let cut = tick_of(&w, "1760-03-01");
    let base = w.clock.flow(FlowPerYear(52.0));
    let half = w.clock.flow(FlowPerYear(26.0));
    assert_ne!(base, half);
    while sim.tick() < cut + 3 {
        let k = sim.tick();
        // The state whose tick is k, before tick k runs.
        let expected = if k <= cut { 52.0 } else { 26.0 };
        assert_eq!(sim.param(cap), Some(expected), "state tick {k}");
        let r = sim.step().unwrap();
        let mint = if k < cut { base } else { half };
        assert_eq!(
            audit_line(&r, fuel, Provenance::Endowment),
            mint,
            "tick {k}"
        );
        assert_eq!(
            r.events.iter().any(|e| e.key.as_str() == "mine.cut"),
            k == cut
        );
    }
    assert_eq!(sim.param(cap), Some(26.0));
}

#[test]
fn entrant_activates_on_its_date() {
    // The oven is declared dormant and a dated SetActive wakes it: before its tick it posts
    // nothing, produces nothing and holds its genesis cash; from its tick it trades.
    let t = tape();
    let mut sim = sim_of(&t);
    let w = sim.world().clone();
    let oven = actor(&w, "oven");
    let opens = tick_of(&w, "1768-04-01");
    let dormant = ActorState::Scripted(ScriptState { active: false });
    let awake = ActorState::Scripted(ScriptState { active: true });
    let coin = good(&w, "coin");
    let genesis_cash = sim.holding(Holder::Actor(oven)).unwrap().get(coin);
    while sim.tick() < opens {
        assert_eq!(sim.actor_state(oven), Some(&dormant));
        let r = sim.step().unwrap();
        assert!(
            r.settlements.iter().all(|l| l.actor != oven),
            "tick {}",
            r.tick
        );
    }
    assert_eq!(
        sim.holding(Holder::Actor(oven)).unwrap().get(coin),
        genesis_cash
    );
    let r = sim.step().unwrap();
    assert_eq!(r.tick, opens);
    assert!(r.events.iter().any(|e| e.key.as_str() == "oven.opens"));
    assert_eq!(sim.actor_state(oven), Some(&awake));
    assert!(
        r.settlements
            .iter()
            .any(|l| l.actor == oven && l.side == SideTag::Buy && l.qty > 0.0),
        "the oven buys in the tick it wakes"
    );
    let later = reports(&mut sim, opens + 10);
    assert!(later.iter().any(|r| r
        .settlements
        .iter()
        .any(|l| l.actor == oven && l.side == SideTag::Sell)));
}
