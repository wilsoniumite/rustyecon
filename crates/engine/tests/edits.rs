//! Edits (docs/ENGINE.md §7.6 and §11, engine; E1, N11): every intervention is a dated tape
//! event or a tape edit followed by a rerun or a resume. A checkpoint survives an edit dated at
//! or after its tick and is refused after any edit to the world or to the past. All comparisons
//! are of hashes or of exact values.

mod common;

use common::*;
use rustyecon_core::FlowPerYear;
use rustyecon_engine::prelude::*;

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
fn resume_boundary_is_the_firing_tick() {
    // N11, E1: a checkpoint "at tick t" holds the state before tick t runs, so whatever fires in
    // tick t is still its future. An edit that moves such a firing later keeps the checkpoint,
    // and the resumed run equals the edited tape's full rerun; one tick later the firing is in
    // the checkpoint's past, and the same edit is refused. Checked at a dated event, at a
    // recurring entry's first occurrence and at tick 0, so an off-by-one in the prefix a
    // checkpoint stores or in the one a resume computes is caught.
    let t = tape();
    let w = sim_of(&t).world().clone();
    let tail = |edited: &Tape, cp: &Checkpoint| -> Result<Vec<u64>, ResumeError> {
        let mut sim = Sim::resume(edited, cp)?;
        let mut out = Vec::new();
        sim.run_until(TICKS, &mut |r| out.push(r.hash)).unwrap();
        Ok(out)
    };
    let cases = [
        // The dated cut, a year later.
        (
            tick_of(&w, "1760-03-01"),
            edit(
                r#"(key: "mine.cut", at: "1760-03-01""#,
                r#"(key: "mine.cut", at: "1761-03-01""#,
            ),
        ),
        // The pension's first occurrence, and every later one, paid 6 instead of 5.
        (
            tick_of(&w, "1751-01-01"),
            edit(
                r#"act: Mint(holder: "pensioners", good: "coin", qty: 5.0)"#,
                r#"act: Mint(holder: "pensioners", good: "coin", qty: 6.0)"#,
            ),
        ),
        // A new event on the start date, which fires in tick 0.
        (
            0,
            edit(
                "    events: [\n",
                "    events: [\n        (key: \"a.day.one\", at: \"1750-01-01\", basis: \
                 Assumed(\"test\"), act: Mint(holder: \"workers\", good: \"coin\", qty: 11.0)),\n",
            ),
        ),
    ];
    for (at, text) in cases {
        let edited = tape_of(&text);
        let rerun = hashes(&edited, TICKS);
        assert_ne!(rerun, hashes(&t, TICKS), "the edit at {at} changes the run");
        let before = checkpoint_at(&t, at);
        assert_eq!(
            tail(&edited, &before).expect("the firing is in the checkpoint's future"),
            &rerun[at as usize..],
            "resumed at {at}"
        );
        match tail(&edited, &checkpoint_at(&t, at + 1)) {
            Err(ResumeError::WrongPrefix { tick, .. }) => assert_eq!(tick, at + 1),
            other => panic!("at {}: expected WrongPrefix, got {other:?}", at + 1),
        }
    }
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
                    assert_eq!(checkpoint, cp.world_id());
                }
                other => panic!(
                    "{to} at {}: expected WrongWorld, got {other:?}",
                    cp.state().tick()
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

#[test]
fn resume_after_a_new_dial_value_equals_full_rerun() {
    // E1, §7.6: a dial change is a dated SetParam, and a checkpoint taken before it fires stays
    // valid when the SetParam's value is new: the value a SetParam copies is read by the
    // schedule alone, so it is not part of the world. Two edits, each resumed from tick 520
    // (before the 1760 cut, which falls in tick 528): the cut's own value changed, and a new
    // dial with a new event in 1775 that sets the mine's capacity from it. Each resumed tail
    // equals the edited tape rerun from genesis.
    let t = tape();
    let cut = tick_of(sim_of(&t).world(), "1760-03-01");
    let early = checkpoint_at(&t, 520);
    let late = checkpoint_at(&t, 600);
    assert!(520 < cut && cut < 600);
    let deeper = edit(
        r#"(key: "mine.capacity.cut", value: 26.0,"#,
        r#"(key: "mine.capacity.cut", value: 20.0,"#,
    );
    let dial = edit(
        "    params: [\n",
        "    params: [\n        (key: \"mine.capacity.dial\", value: 40.0, unit: FlowPerYear, \
         basis: Assumed(\"a dial\")),\n",
    )
    .replacen(
        "    events: [\n",
        "    events: [\n        (key: \"mine.dial\", at: \"1775-01-01\", basis: Assumed(\"a dial\"), \
         act: SetParam(param: \"mine.capacity\", to: \"mine.capacity.dial\")),\n",
        1,
    );
    let reference = hashes(&t, TICKS);
    for (what, text) in [("the cut's value", deeper), ("a new dial", dial)] {
        let edited = tape_of(&text);
        let w = sim_of(&edited).world().clone();
        assert_eq!(
            w.world_id,
            early.world_id(),
            "{what}: the world is the same"
        );
        let rerun = hashes(&edited, TICKS);
        assert_ne!(rerun, reference, "{what}: the edit changes the run");
        assert_eq!(rerun[..cut as usize], reference[..cut as usize], "{what}");
        let mut resumed = Sim::resume(&edited, &early).expect("a future dial keeps the checkpoint");
        let mut tail = Vec::new();
        resumed
            .run_until(TICKS, &mut |r| tail.push(r.hash))
            .unwrap();
        assert_eq!(tail.as_slice(), &rerun[520..], "{what}");
    }
    // After the cut has fired, its value is part of the past: that checkpoint is refused. The
    // 1775 dial is still in the future of tick 600.
    match Sim::resume(
        &tape_of(&edit(
            r#"(key: "mine.capacity.cut", value: 26.0,"#,
            r#"(key: "mine.capacity.cut", value: 20.0,"#,
        )),
        &late,
    ) {
        Err(ResumeError::WrongPrefix { tick, .. }) => assert_eq!(tick, 600),
        other => panic!("expected WrongPrefix, got {other:?}"),
    }
    let dial = tape_of(
        &edit(
            "    params: [\n",
            "    params: [\n        (key: \"mine.capacity.dial\", value: 40.0, unit: FlowPerYear, \
             basis: Assumed(\"a dial\")),\n",
        )
        .replacen(
            "    events: [\n",
            "    events: [\n        (key: \"mine.dial\", at: \"1775-01-01\", basis: Assumed(\"a dial\"), \
             act: SetParam(param: \"mine.capacity\", to: \"mine.capacity.dial\")),\n",
            1,
        ),
    );
    Sim::resume(&dial, &late).expect("the dial fires after tick 600");
}

/// Replace, in a RON checkpoint, the text between the first `open` at or after `from` and the
/// next `close` with `with`.
fn replace_after(text: &str, from: &str, open: &str, close: &str, with: &str) -> String {
    let at = text.find(from).expect("the anchor");
    let start = at + text[at..].find(open).expect("the opening") + open.len();
    let end = start + text[start..].find(close).expect("the closing");
    format!("{}{with}{}", &text[..start], &text[end..])
}

/// The RON text with its digest set to what its state hashes to, as a forger would do.
fn with_matching_digest(text: &str) -> String {
    match Checkpoint::from_ron(text) {
        Err(CheckpointError::Digest { stored, computed }) => text.replacen(
            &format!("digest: {stored},"),
            &format!("digest: {computed},"),
            1,
        ),
        other => panic!("expected a digest mismatch, got {other:?}"),
    }
}

#[test]
fn resume_refuses_an_invalid_state() {
    // N11, §7.6: resume checks the state against the world before it runs anything. A state
    // edited after saving is first refused by its digest when it is decoded; one whose digest
    // was recomputed to match decodes, and resume refuses it as Invalid. Five edits of a RON
    // checkpoint at tick 100: a lot life its good cannot have, a written currency book slot, an
    // actor's holding dropped, an escrow held between ticks, and an actor's state dropped.
    let t = tape();
    let cp = checkpoint_at(&t, 100);
    let text = cp.to_ron();
    let edits: Vec<(&str, String)> = vec![
        (
            "a bread life of 1000 ticks",
            replace_after(&text, "holdings: {", "Some(", ")", "1000"),
        ),
        ("the currency slot's supply", {
            // The book is flat, node by node: slot 1 is (town, coin), whose supply is 0.
            let first = text.find("            supply: [\n").unwrap() + "            supply: [\n".len();
            let second = first + text[first..].find('\n').unwrap() + 1;
            let end = second + text[second..].find('\n').unwrap();
            assert_eq!(&text[second..end], "                0.0,");
            format!("{}                5.0,{}", &text[..second], &text[end..])
        }),
        ("the pensioners' holding dropped", {
            let start = text.find("            Actor(Pop((0))): [").unwrap();
            let end = start + text[start..].find("\n            ],\n").unwrap() + "\n            ],\n".len();
            format!("{}{}", &text[..start], &text[end..])
        }),
        (
            "an escrow between ticks",
            text.replacen(
                "        holdings: {\n",
                "        holdings: {\n            Escrow((0), (0)): [\n                ((0), [\n                    (1.0, Some(1)),\n                ]),\n            ],\n",
                1,
            ),
        ),
        (
            "the oven's state dropped",
            text.replacen(
                "            Desk((3)): Scripted((\n                active: false,\n            )),\n",
                "",
                1,
            ),
        ),
    ];
    for (what, edited) in edits {
        assert_ne!(edited, text, "{what}: the edit applies");
        assert!(
            matches!(
                Checkpoint::from_ron(&edited),
                Err(CheckpointError::Digest { .. })
            ),
            "{what}: the digest refuses the edit"
        );
        let forged = Checkpoint::from_ron(&with_matching_digest(&edited))
            .unwrap_or_else(|e| panic!("{what}: the forged text decodes: {e}"));
        match Sim::resume(&t, &forged) {
            Err(ResumeError::Invalid(e)) => {
                assert!(!e.to_string().is_empty(), "{what}");
            }
            other => panic!("{what}: expected Invalid, got {other:?}"),
        }
    }
    // The untouched text resumes.
    Sim::resume(&t, &Checkpoint::from_ron(&text).unwrap())
        .expect("the untouched checkpoint resumes");
}
