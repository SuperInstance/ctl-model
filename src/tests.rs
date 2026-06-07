use crate::checker::check;
use crate::formula::CtlFormula;
use crate::kripke::KripkeStruct;

// ─── Kripke Structure Tests ───────────────────────────────────────────────────

#[test]
fn builder_basic() {
    let k = KripkeStruct::builder()
        .state("a")
        .state("b")
        .transition(0, 1)
        .transition(1, 0)
        .initial_state(0)
        .build()
        .unwrap();
    assert_eq!(k.state_count(), 2);
    assert_eq!(k.state_name(0), Some("a"));
    assert_eq!(k.state_name(1), Some("b"));
    assert_eq!(k.initial_states, vec![0]);
}

#[test]
fn builder_labels() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 1)
        .transition(1, 0)
        .label(0, "p")
        .label(0, "q")
        .label(1, "r")
        .initial_state(0)
        .build()
        .unwrap();
    assert!(k.labels(0).contains("p"));
    assert!(k.labels(0).contains("q"));
    assert!(k.labels(1).contains("r"));
    assert!(!k.labels(1).contains("p"));
}

#[test]
fn builder_validate_bad_transition() {
    let result = KripkeStruct::builder()
        .state("s0")
        .transition(0, 5) // invalid target
        .initial_state(0)
        .build();
    assert!(result.is_err());
}

#[test]
fn builder_validate_no_successors() {
    let result = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .label(0, "p")
        .initial_state(0)
        .build();
    assert!(result.is_err()); // no transitions = not total
}

#[test]
fn predecessors() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(2, 1)
        .transition(1, 0)
        .initial_state(0)
        .build()
        .unwrap();
    let preds = k.predecessors(1);
    assert!(preds.contains(&0));
    assert!(preds.contains(&2));
}

#[test]
fn default_empty() {
    let k = KripkeStruct::default();
    assert_eq!(k.state_count(), 0);
}

#[test]
fn self_loop() {
    let k = KripkeStruct::builder()
        .state("s0")
        .transition(0, 0)
        .label(0, "p")
        .initial_state(0)
        .build()
        .unwrap();
    assert_eq!(k.successors(0), &[0]);
}

// ─── Atom / Proposition Tests ─────────────────────────────────────────────────

#[test]
fn atom_true() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::atom("p"));
    assert!(r.satisfying_states.contains(&0));
}

#[test]
fn atom_false() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::atom("nonexistent"));
    assert!(r.satisfying_states.is_empty());
}

#[test]
fn atom_holds_in_initial() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::atom("p"));
    assert!(r.holds);
}

// ─── Boolean Operator Tests ───────────────────────────────────────────────────

#[test]
fn not_operator() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::not(CtlFormula::atom("p")));
    // s0 has p, s1 has q — so s1 should satisfy ¬p
    assert!(!r.satisfying_states.contains(&0));
    assert!(r.satisfying_states.contains(&1));
}

#[test]
fn and_operator() {
    let k = make_simple(); // s0: p, s1: q
    let r = check(
        &k,
        &CtlFormula::and(CtlFormula::atom("p"), CtlFormula::atom("q")),
    );
    // No state has both p and q
    assert!(r.satisfying_states.is_empty());
}

#[test]
fn or_operator() {
    let k = make_simple();
    let r = check(
        &k,
        &CtlFormula::or(CtlFormula::atom("p"), CtlFormula::atom("q")),
    );
    assert_eq!(r.satisfying_states.len(), 2);
}

#[test]
fn implies_convenience() {
    let k = make_simple();
    let f = CtlFormula::implies(CtlFormula::atom("p"), CtlFormula::atom("p"));
    let r = check(&k, &f);
    assert!(r.holds);
}

#[test]
fn iff_convenience() {
    let k = make_simple();
    let f = CtlFormula::iff(CtlFormula::atom("p"), CtlFormula::atom("p"));
    let r = check(&k, &f);
    assert!(r.holds);
}

// ─── EX / AX Tests ───────────────────────────────────────────────────────────

#[test]
fn ex_basic() {
    // s0 --p--> s1 (q)
    let k = make_simple();
    let r = check(&k, &CtlFormula::ex(CtlFormula::atom("q")));
    assert!(r.satisfying_states.contains(&0)); // s0 has successor s1 with q
}

#[test]
fn ex_no_match() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::ex(CtlFormula::atom("nonexistent")));
    assert!(r.satisfying_states.is_empty());
}

#[test]
fn ax_basic() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(0, 2)
        .transition(1, 0)
        .transition(2, 0)
        .label(1, "p")
        .label(2, "p")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::ax(CtlFormula::atom("p")));
    assert!(r.satisfying_states.contains(&0)); // both successors have p
}

#[test]
fn ax_fails() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(0, 2)
        .transition(1, 0)
        .transition(2, 0)
        .label(1, "p")
        // s2 has no p
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::ax(CtlFormula::atom("p")));
    assert!(!r.satisfying_states.contains(&0));
}

// ─── EF Tests ─────────────────────────────────────────────────────────────────

#[test]
fn ef_reachable() {
    // Linear chain: s0 -> s1 -> s2 (with q)
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 2)
        .label(0, "p")
        .label(2, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::ef(CtlFormula::atom("q")));
    assert!(r.holds);
    assert!(r.satisfying_states.contains(&0));
    assert!(r.satisfying_states.contains(&1));
    assert!(r.satisfying_states.contains(&2));
}

#[test]
fn ef_unreachable() {
    // s0 loops, s1 loops but s1 is unreachable from s0
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 0)
        .transition(1, 1)
        .label(0, "p")
        .label(1, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::ef(CtlFormula::atom("q")));
    assert!(!r.holds);
}

// ─── AF Tests ─────────────────────────────────────────────────────────────────

#[test]
fn af_on_linear_chain() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 2)
        .label(2, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::af(CtlFormula::atom("q")));
    assert!(r.holds);
}

#[test]
fn af_with_loop_avoiding() {
    // s0 -> s1 -> s0 (loop without q), s0 also -> s2 (has q)
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(0, 2)
        .transition(1, 0)
        .transition(2, 2)
        .label(2, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::af(CtlFormula::atom("q")));
    // s0 can reach q via s2, but also can loop s0->s1->s0 forever
    // So AF q does NOT hold in s0 or s1
    assert!(!r.satisfying_states.contains(&0));
}

#[test]
fn af_holds_when_no_escape() {
    // Linear chain to q
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 1)
        .transition(1, 1)
        .label(1, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::af(CtlFormula::atom("q")));
    assert!(r.holds);
}

// ─── EG Tests ─────────────────────────────────────────────────────────────────

#[test]
fn eg_on_self_loop() {
    let k = KripkeStruct::builder()
        .state("s0")
        .transition(0, 0)
        .label(0, "p")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::eg(CtlFormula::atom("p")));
    assert!(r.holds);
}

#[test]
fn eg_cycle() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 1)
        .transition(1, 0)
        .label(0, "p")
        .label(1, "p")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::eg(CtlFormula::atom("p")));
    assert!(r.holds);
    assert!(r.satisfying_states.contains(&0));
    assert!(r.satisfying_states.contains(&1));
}

#[test]
fn eg_fails_when_no_infinite_p_path() {
    // s0(p) -> s1(no p) -> s2(p) -> s2(p) loop
    // From s0, every path eventually leaves p (s1 has no p)
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 2)
        .label(0, "p")
        .label(2, "p")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::eg(CtlFormula::atom("p")));
    // s0 can only go to s1 (no p), so s0 fails EG p
    // s2 self-loops with p, so s2 satisfies EG p
    assert!(!r.satisfying_states.contains(&0));
    assert!(r.satisfying_states.contains(&2));
}

// ─── AG Tests ─────────────────────────────────────────────────────────────────

#[test]
fn ag_on_self_loop() {
    let k = KripkeStruct::builder()
        .state("s0")
        .transition(0, 0)
        .label(0, "p")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::ag(CtlFormula::atom("p")));
    assert!(r.holds);
}

#[test]
fn ag_fails_when_reachable_violation() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 1)
        .transition(1, 1)
        .label(0, "p")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(&k, &CtlFormula::ag(CtlFormula::atom("p")));
    assert!(!r.holds);
}

// ─── EU Tests ─────────────────────────────────────────────────────────────────

#[test]
fn eu_basic() {
    // s0(p) -> s1(p) -> s2(q)
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 2)
        .label(0, "p")
        .label(1, "p")
        .label(2, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(
        &k,
        &CtlFormula::eu(CtlFormula::atom("p"), CtlFormula::atom("q")),
    );
    assert!(r.holds);
    assert!(r.satisfying_states.contains(&0));
    assert!(r.satisfying_states.contains(&1));
    assert!(r.satisfying_states.contains(&2)); // q itself
}

#[test]
fn eu_fails_when_no_path() {
    // s0(p) loops, never reaches q
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 0)
        .transition(1, 1)
        .label(0, "p")
        .label(1, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(
        &k,
        &CtlFormula::eu(CtlFormula::atom("p"), CtlFormula::atom("q")),
    );
    assert!(!r.satisfying_states.contains(&0));
}

// ─── AU Tests ─────────────────────────────────────────────────────────────────

#[test]
fn au_basic() {
    // s0(p) -> s1(p) -> s2(q), no branching
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 2)
        .label(0, "p")
        .label(1, "p")
        .label(2, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(
        &k,
        &CtlFormula::au(CtlFormula::atom("p"), CtlFormula::atom("q")),
    );
    assert!(r.holds);
}

#[test]
fn au_fails_with_escape() {
    // s0 can go to s2(q) or back to s0(p) forever
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .transition(0, 1)
        .transition(0, 0)
        .transition(1, 2)
        .transition(2, 2)
        .label(0, "p")
        .label(2, "q")
        .initial_state(0)
        .build()
        .unwrap();
    let r = check(
        &k,
        &CtlFormula::au(CtlFormula::atom("p"), CtlFormula::atom("q")),
    );
    // s0 can loop on itself forever, never reaching q
    assert!(!r.satisfying_states.contains(&0));
}

// ─── Duality Tests ────────────────────────────────────────────────────────────

#[test]
fn duality_ef_and_ag() {
    let k = make_simple();
    let ef_q = check(&k, &CtlFormula::ef(CtlFormula::atom("q")));
    // EF q ≡ ¬AG ¬q
    let not_ag_not_q = check(
        &k,
        &CtlFormula::not(CtlFormula::ag(CtlFormula::not(CtlFormula::atom("q")))),
    );
    assert_eq!(ef_q.satisfying_states, not_ag_not_q.satisfying_states);
}

#[test]
fn duality_ex_and_ax() {
    let k = make_simple();
    let ex_q = check(&k, &CtlFormula::ex(CtlFormula::atom("q")));
    let _ax_not_q = check(&k, &CtlFormula::ax(CtlFormula::not(CtlFormula::atom("q"))));
    let not_ax_not_q = check(
        &k,
        &CtlFormula::not(CtlFormula::ax(CtlFormula::not(CtlFormula::atom("q")))),
    );
    assert_eq!(ex_q.satisfying_states, not_ax_not_q.satisfying_states);
}

#[test]
fn duality_eg_and_af() {
    let k = make_simple();
    let eg_p = check(&k, &CtlFormula::eg(CtlFormula::atom("p")));
    let not_af_not_p = check(
        &k,
        &CtlFormula::not(CtlFormula::af(CtlFormula::not(CtlFormula::atom("p")))),
    );
    assert_eq!(eg_p.satisfying_states, not_af_not_p.satisfying_states);
}

// ─── Complexity Tracking Tests ────────────────────────────────────────────────

#[test]
fn complexity_tracking() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::ef(CtlFormula::atom("q")));
    assert!(r.complexity.iterations("EF").iter().any(|&c| c > 0));
    assert!(r.complexity.within_quadratic_bounds(k.state_count()));
}

#[test]
fn complexity_nested() {
    let k = make_simple();
    let r = check(&k, &CtlFormula::ag(CtlFormula::ef(CtlFormula::atom("q"))));
    assert!(r.complexity.operator_count() >= 1);
}

// ─── Counterexample / Witness Integration Tests ───────────────────────────────

#[test]
fn counterexample_for_false() {
    let k = make_simple();
    let ce = crate::counterexample::find_counterexample(&k, &CtlFormula::atom("nonexistent"));
    assert!(ce.is_some());
}

#[test]
fn no_counterexample_for_true() {
    let k = make_simple();
    let ce = crate::counterexample::find_counterexample(&k, &CtlFormula::atom("p"));
    assert!(ce.is_none());
}

#[test]
fn witness_for_true() {
    let k = make_simple();
    let w = crate::witness::find_witness(&k, &CtlFormula::ef(CtlFormula::atom("q")));
    assert!(w.is_some());
    let w = w.unwrap();
    assert_eq!(w.path[0], 0); // starts at initial
}

#[test]
fn no_witness_for_false() {
    let k = make_simple();
    let w = crate::witness::find_witness(&k, &CtlFormula::atom("nonexistent"));
    assert!(w.is_none());
}

// ─── Larger Structure Tests ───────────────────────────────────────────────────

#[test]
fn five_state_chain() {
    let k = KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .state("s2")
        .state("s3")
        .state("s4")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 3)
        .transition(3, 4)
        .transition(4, 4)
        .label(4, "goal")
        .initial_state(0)
        .build()
        .unwrap();

    let r = check(&k, &CtlFormula::af(CtlFormula::atom("goal")));
    assert!(r.holds);
}

#[test]
fn diamond_with_branch() {
    let k = KripkeStruct::builder()
        .state("start") // 0
        .state("left") // 1
        .state("right") // 2
        .state("end") // 3
        .transition(0, 1)
        .transition(0, 2)
        .transition(1, 3)
        .transition(2, 3)
        .transition(3, 0)
        .label(1, "safe")
        .label(3, "done")
        .initial_state(0)
        .build()
        .unwrap();

    // AG EF done: from every state, can eventually reach done
    let r = check(
        &k,
        &CtlFormula::ag(CtlFormula::ef(CtlFormula::atom("done"))),
    );
    assert!(r.holds);

    // AX safe: does NOT hold (right branch is not safe)
    let r = check(&k, &CtlFormula::ax(CtlFormula::atom("safe")));
    assert!(!r.satisfying_states.contains(&0));
}

#[test]
fn three_state_cycle_all_p() {
    let k = KripkeStruct::builder()
        .state("a")
        .state("b")
        .state("c")
        .transition(0, 1)
        .transition(1, 2)
        .transition(2, 0)
        .label(0, "p")
        .label(1, "p")
        .label(2, "p")
        .initial_state(0)
        .build()
        .unwrap();

    let r = check(&k, &CtlFormula::ag(CtlFormula::atom("p")));
    assert!(r.holds);

    let r = check(&k, &CtlFormula::eg(CtlFormula::atom("p")));
    assert!(r.holds);

    let r = check(&k, &CtlFormula::ef(CtlFormula::atom("nonexistent")));
    assert!(!r.holds);
}

// ─── Formula Display & Size Tests ─────────────────────────────────────────────

#[test]
fn formula_display_complex() {
    let f = CtlFormula::ag(CtlFormula::implies(
        CtlFormula::atom("request"),
        CtlFormula::af(CtlFormula::atom("grant")),
    ));
    let s = format!("{f}");
    assert!(s.contains("AG"));
    assert!(s.contains("AF"));
    assert!(s.contains("request"));
    assert!(s.contains("grant"));
}

#[test]
fn formula_size_nested() {
    let f = CtlFormula::and(
        CtlFormula::or(
            CtlFormula::atom("a"),
            CtlFormula::not(CtlFormula::atom("b")),
        ),
        CtlFormula::ef(CtlFormula::atom("c")),
    );
    assert_eq!(f.size(), 7);
}

// ─── Stress Test: Linear Chain ────────────────────────────────────────────────

#[test]
fn large_linear_chain_iterative() {
    // 50-state linear chain — this would overflow a recursive implementation
    let n = 50;
    let mut builder = KripkeStruct::builder();
    for i in 0..n {
        builder = builder.state(format!("s{i}"));
    }
    for i in 0..n - 1 {
        builder = builder.transition(i, i + 1);
    }
    builder = builder.transition(n - 1, n - 1);
    builder = builder.label(n - 1, "goal");
    builder = builder.initial_state(0);
    let k = builder.build().unwrap();

    let r = check(&k, &CtlFormula::af(CtlFormula::atom("goal")));
    assert!(r.holds);

    let r = check(&k, &CtlFormula::ef(CtlFormula::atom("goal")));
    assert!(r.holds);
    assert!(r.satisfying_states.len() == n);

    let r = check(
        &k,
        &CtlFormula::ag(CtlFormula::not(CtlFormula::atom("goal"))),
    );
    assert!(!r.holds); // goal is reachable
}

// ─── Helper ───────────────────────────────────────────────────────────────────

fn make_simple() -> KripkeStruct {
    KripkeStruct::builder()
        .state("s0")
        .state("s1")
        .transition(0, 1)
        .transition(1, 0)
        .label(0, "p")
        .label(1, "q")
        .initial_state(0)
        .build()
        .unwrap()
}
