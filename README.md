# ctl-model

**Computation Tree Logic (CTL) model checking on Kripke structures.**

A Rust library implementing the classical CTL model checking algorithm with iterative fixpoint computation, counterexample/witness generation, and complexity tracking.

---

## Table of Contents

- [Overview](#overview)
- [Theory](#theory)
  - [Kripke Structures](#kripke-structures)
  - [CTL Syntax and Semantics](#ctl-syntax-and-semantics)
  - [Fixpoint Characterizations](#fixpoint-characterizations)
- [Module Overview](#module-overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Examples](#examples)
  - [Example 1: Mutual Exclusion](#example-1-mutual-exclusion)
  - [Example 2: Traffic Light Controller](#example-2-traffic-light-controller)
  - [Example 3: Communication Protocol](#example-3-communication-protocol)
- [API Reference](#api-reference)
- [Algorithm Details](#algorithm-details)
  - [Iterative Fixpoint Computation](#iterative-fixpoint-computation)
  - [Complexity Analysis](#complexity-analysis)
- [Design Decisions](#design-decisions)
- [ASCII Art: Computation Tree](#ascii-art-computation-tree)
- [References](#references)
- [License](#license)

---

## Overview

Computation Tree Logic (CTL) is a branching-time temporal logic used in formal verification to specify properties of reactive systems. Given a finite-state model (a Kripke structure) and a CTL formula, **model checking** determines whether the formula holds in the model.

This library provides:

- **Kripke structure construction** via a builder pattern
- **CTL formula parsing** with a typed AST and standard Display notation
- **Model checking** using iterative fixpoint algorithms (no recursion — safe for large structures)
- **Counterexample generation** — witness paths when formulas are false
- **Witness generation** — satisfying paths when formulas are true
- **Complexity tracking** — iteration counts per fixpoint operator

The implementation follows the seminal algorithm by Clarke and Emerson (1981), adapted for iterative computation to avoid stack overflow on non-trivial structures.

---

## Theory

### Kripke Structures

A **Kripke structure** is a tuple **M = (S, R, L, S₀)** where:

```
S  = finite set of states
R  ⊆ S × S = total transition relation (every state has ≥1 successor)
L  : S → 2^(AP) = labeling function (which propositions hold in each state)
S₀ ⊆ S = set of initial states
```

The transition relation must be **total**: every state has at least one successor. This ensures every state has at least one infinite path originating from it.

A Kripke structure **unfolds** into an infinite computation tree rooted at each initial state. CTL quantifies over paths in this tree.

### CTL Syntax and Semantics

**Syntax:**

```
φ ::= p | ¬φ | φ₁ ∧ φ₂ | φ₁ ∨ φ₂
    | EX φ | AX φ    -- next
    | EF φ | AF φ    -- eventually
    | EG φ | AG φ    -- globally
    | φ₁ EU φ₂ | φ₁ AU φ₂  -- until
```

**Path quantifiers:**
- **E** — "there exists a path"
- **A** — "for all paths"

**State quantifiers:**
- **X** — "in the next state"
- **F** — "eventually (sometime in the future)"
- **G** — "globally (always, from now on)"
- **U** — "until"

**Key semantic equations:**

```
M, s ⊨ p        iff p ∈ L(s)
M, s ⊨ ¬φ       iff M, s ⊭ φ
M, s ⊨ EX φ     iff ∃ successor s' of s: M, s' ⊨ φ
M, s ⊨ AX φ     iff ∀ successors s' of s: M, s' ⊨ φ
M, s ⊨ EG φ     iff ∃ path π from s: ∀i≥0: M, π[i] ⊨ φ
M, s ⊨ AG φ     iff ∀ paths π from s: ∀i≥0: M, π[i] ⊨ φ
M, s ⊨ EF φ     iff ∃ path π from s: ∃i≥0: M, π[i] ⊨ φ
M, s ⊨ AF φ     iff ∀ paths π from s: ∃i≥0: M, π[i] ⊨ φ
M, s ⊨ φ₁ EU φ₂ iff ∃ path π from s: ∃k≥0: M, π[k] ⊨ φ₂ ∧ ∀j<k: M, π[j] ⊨ φ₁
M, s ⊨ φ₁ AU φ₂ iff ∀ paths π from s: ∃k≥0: M, π[k] ⊨ φ₂ ∧ ∀j<k: M, π[j] ⊨ φ₁
```

**Duality laws:**

```
EF φ ≡ ¬AG ¬φ      AF φ ≡ ¬EG ¬φ
EX φ ≡ ¬AX ¬φ      AX φ ≡ ¬EX ¬φ
```

### Fixpoint Characterizations

The temporal operators can be characterized as fixpoints of appropriate functionals on the lattice of state sets (ordered by subset inclusion):

```
EF φ = μZ. (φ ∨ EX Z)         -- least fixpoint
AF φ = μZ. (φ ∨ AX Z)         -- least fixpoint
EG φ = νZ. (φ ∧ EX Z)         -- greatest fixpoint
AG φ = νZ. (φ ∧ AX Z)         -- greatest fixpoint
φ₁ EU φ₂ = μZ. (φ₂ ∨ (φ₁ ∧ EX Z))  -- least fixpoint
φ₁ AU φ₂ = μZ. (φ₂ ∨ (φ₁ ∧ AX Z))  -- greatest fixpoint
```

For **least fixpoints** (EF, AF, EU, AU), we start from the empty set and iteratively add states until convergence.

For **greatest fixpoints** (EG, AG), we start from the set of all states satisfying the immediate condition and iteratively remove states that violate the condition until convergence.

Each fixpoint converges in at most |S| iterations, where |S| is the number of states.

---

## Module Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        ctl-model                                 │
├──────────────┬──────────────┬────────────────────────────────────┤
│   kripke     │   formula    │           checker                   │
│              │              │                                      │
│ KripkeStruct │  CtlFormula  │  Iterative fixpoint model checking  │
│ KripkeBuilder│  Display     │  O(|S|² × |f|) time complexity     │
│ Validation   │  Convenience │  Tracks iterations per operator     │
├──────────────┴──────────────┼────────────────────────────────────┤
│    counterexample            │           witness                  │
│                              │                                     │
│  Find paths when FALSE       │  Find paths when TRUE              │
│  Universal → violating path  │  Existential → satisfying path     │
│  Existential → failure evid. │  Universal → all-paths evidence   │
├──────────────────────────────┴────────────────────────────────────┤
│                         complexity                                │
│                                                                   │
│  Iteration counts per operator                                   │
│  O(|S|²) bound verification                                      │
│  Detailed report formatting                                      │
└───────────────────────────────────────────────────────────────────┘
```

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `kripke` | Kripke structure definition & construction | `KripkeStruct`, `KripkeBuilder` |
| `formula` | CTL formula AST with Display | `CtlFormula` |
| `checker` | Iterative fixpoint model checking | `check()`, `CheckResult` |
| `counterexample` | Find violating/witness paths for FALSE formulas | `CounterExample`, `find_counterexample()` |
| `witness` | Find satisfying paths for TRUE formulas | `Witness`, `find_witness()` |
| `complexity` | Track and report fixpoint iterations | `ComplexityReport` |

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ctl-model = "0.1"
```

Or use `cargo add`:

```bash
cargo add ctl-model
```

### Requirements

- Rust edition 2024 (Rust 1.85+)
- `serde` for serialization (the only external dependency)

---

## Quick Start

```rust
use ctl_model::kripke::KripkeStruct;
use ctl_model::formula::CtlFormula;
use ctl_model::checker::check;

// Build a simple two-state system
let k = KripkeStruct::builder()
    .state("idle")
    .state("active")
    .transition(0, 1)
    .transition(1, 0)
    .label(0, "idle")
    .label(1, "active")
    .initial_state(0)
    .build()
    .unwrap();

// Check: is the system always either idle or active?
let formula = CtlFormula::ag(CtlFormula::or(
    CtlFormula::atom("idle"),
    CtlFormula::atom("active"),
));
let result = check(&k, &formula);
assert!(result.holds);

// Check: can we eventually reach the active state?
let liveness = CtlFormula::ef(CtlFormula::atom("active"));
let result = check(&k, &liveness);
assert!(result.holds);

// Examine complexity
println!("Iterations: {}", result.complexity);
```

---

## Examples

### Example 1: Mutual Exclusion

A simple model of a mutual exclusion protocol with three states: idle, requesting, and in the critical section.

```rust
use ctl_model::kripke::KripkeStruct;
use ctl_model::formula::CtlFormula;
use ctl_model::checker::check;
use ctl_model::counterexample::find_counterexample;
use ctl_model::witness::find_witness;

let mutex = KripkeStruct::builder()
    .state("idle")      // 0: not requesting
    .state("trying")    // 1: requesting access
    .state("critical")  // 2: in critical section
    .transition(0, 1)   // idle → trying
    .transition(1, 2)   // trying → critical
    .transition(2, 0)   // critical → idle
    .transition(0, 0)   // idle can self-loop (stay idle)
    .label(2, "in_cs")
    .label(1, "waiting")
    .initial_state(0)
    .build()
    .unwrap();

// Safety: we never stay in critical section forever
// AG (in_cs → EF ¬in_cs)  i.e., AG (in_cs → EF (idle ∨ trying))
let safety = CtlFormula::ag(
    CtlFormula::implies(
        CtlFormula::atom("in_cs"),
        CtlFormula::ef(CtlFormula::not(CtlFormula::atom("in_cs"))),
    )
);
let result = check(&mutex, &safety);
assert!(result.holds, "CS is always eventually exited");

// Liveness: from any state, we can eventually enter the CS
let liveness = CtlFormula::ag(CtlFormula::ef(CtlFormula::atom("in_cs")));
let result = check(&mutex, &liveness);
assert!(result.holds, "CS is always reachable");

// Witness: show a path from idle to critical section
let witness = find_witness(&mutex, &CtlFormula::ef(CtlFormula::atom("in_cs")));
assert!(witness.is_some());
let w = witness.unwrap();
println!("Path to CS: {:?}", w.path.iter()
    .map(|&i| mutex.state_name(i).unwrap())
    .collect::<Vec<_>>());
// Output: ["idle", "trying", "critical"]
```

### Example 2: Traffic Light Controller

Model a traffic light that cycles through red → green → yellow → red, and verify that it never shows green and red simultaneously.

```rust
use ctl_model::kripke::KripkeStruct;
use ctl_model::formula::CtlFormula;
use ctl_model::checker::check;

let traffic_light = KripkeStruct::builder()
    .state("red")      // 0
    .state("green")    // 1
    .state("yellow")   // 2
    .transition(0, 1)  // red → green
    .transition(1, 2)  // green → yellow
    .transition(2, 0)  // yellow → red
    .label(0, "red")
    .label(0, "stop")
    .label(1, "green")
    .label(1, "go")
    .label(2, "yellow")
    .label(2, "caution")
    .initial_state(0)
    .build()
    .unwrap();

// Safety: always either red, green, or yellow (exactly one)
let always_colored = CtlFormula::ag(CtlFormula::or(
    CtlFormula::or(
        CtlFormula::atom("red"),
        CtlFormula::atom("green"),
    ),
    CtlFormula::atom("yellow"),
));
assert!(check(&traffic_light, &always_colored).holds);

// Safety: never green and red at the same time
// This is trivially true since each state has exactly one color label,
// but we can express it as: AG ¬(green ∧ red)
let no_conflict = CtlFormula::ag(
    CtlFormula::not(CtlFormula::and(
        CtlFormula::atom("green"),
        CtlFormula::atom("red"),
    ))
);
assert!(check(&traffic_light, &no_conflict).holds);

// Liveness: red is always eventually followed by green
let progress = CtlFormula::ag(
    CtlFormula::implies(
        CtlFormula::atom("red"),
        CtlFormula::af(CtlFormula::atom("green")),
    )
);
assert!(check(&traffic_light, &progress).holds);

// Fairness: green occurs infinitely often on all paths
let fairness = CtlFormula::ag(CtlFormula::af(CtlFormula::atom("green")));
assert!(check(&traffic_light, &fairness).holds);
```

### Example 3: Communication Protocol

Model a simple sender-receiver protocol with message loss and verify reliable delivery.

```rust
use ctl_model::kripke::KripkeStruct;
use ctl_model::formula::CtlFormula;
use ctl_model::checker::check;
use ctl_model::counterexample::find_counterexample;

let protocol = KripkeStruct::builder()
    .state("idle")       // 0: nothing happening
    .state("sending")    // 1: sender transmitting
    .state("lost")       // 2: message was lost
    .state("received")   // 3: message received
    .state("acked")      // 4: acknowledgment sent
    // Transitions
    .transition(0, 1)    // idle → start sending
    .transition(0, 0)    // idle → idle (stay)
    .transition(1, 3)    // sending → received (success)
    .transition(1, 2)    // sending → lost (failure)
    .transition(2, 1)    // lost → retry sending
    .transition(3, 4)    // received → acked
    .transition(4, 0)    // acked → back to idle
    // Labels
    .label(1, "sending")
    .label(2, "error")
    .label(3, "delivered")
    .label(4, "complete")
    .initial_state(0)
    .build()
    .unwrap();

// Can we always eventually deliver a message once we start sending?
// AG (sending → AF (delivered ∨ error))
let eventual_outcome = CtlFormula::ag(
    CtlFormula::implies(
        CtlFormula::atom("sending"),
        CtlFormula::af(CtlFormula::or(
            CtlFormula::atom("delivered"),
            CtlFormula::atom("error"),
        )),
    )
);
let result = check(&protocol, &eventual_outcome);
println!("Eventual outcome: {}", result.holds);

// Is there a path where we keep losing messages forever?
// EF EG error (there exists a path where error holds globally from some point)
let infinite_loss = CtlFormula::ef(CtlFormula::eg(CtlFormula::atom("error")));
let result = check(&protocol, &infinite_loss);
// With retry, we always get out of the lost state, so EG error doesn't hold
assert!(!result.holds, "Cannot be stuck in error forever");

// Verify: from any state, we can always get back to idle
let always_recoverable = CtlFormula::ag(
    CtlFormula::ef(CtlFormula::atom("complete"))
);
let result = check(&protocol, &always_recoverable);
assert!(result.holds, "Protocol always reaches completion");

// Get complexity report
println!("Complexity:\n{}", result.complexity);
```

---

## API Reference

### `kripke` Module

```rust
// Create a Kripke structure
let k = KripkeStruct::builder()
    .state("name")          // Add a state (index = insertion order)
    .transition(from, to)   // Add a transition
    .label(state, "prop")   // Label a state with an atomic proposition
    .initial_state(idx)     // Mark a state as initial
    .build()?;              // Validate and construct

// Query the structure
k.state_count();            // Number of states
k.state_name(idx);          // Get name by index
k.successors(state);        // Get successor state indices
k.labels(state);            // Get atomic propositions at a state
k.predecessors(state);      // Get predecessor state indices
```

### `formula` Module

```rust
// Construct formulas using the builder methods
let f = CtlFormula::atom("p");                           // p
let f = CtlFormula::not(f);                               // ¬p
let f = CtlFormula::and(left, right);                     // left ∧ right
let f = CtlFormula::ex(phi);                              // EX φ
let f = CtlFormula::eu(CtlFormula::atom("p"),
                       CtlFormula::atom("q"));           // p EU q

// Convenience methods
let f = CtlFormula::implies(left, right);                 // ¬left ∨ right
let f = CtlFormula::iff(left, right);                     // (left → right) ∧ (right → left)

// Display
println!("{}", f);  // Standard CTL notation with ∧, ∨, ¬
```

### `checker` Module

```rust
let result = check(&kripke, &formula);
result.holds;                     // true if formula holds in all initial states
result.satisfying_states;         // HashSet<usize> of satisfying states
result.complexity;                // ComplexityReport with iteration counts
```

### `counterexample` Module

```rust
if let Some(ce) = find_counterexample(&k, &formula) {
    println!("Counterexample: {}", ce.description);
    println!("Path: {:?}", ce.path);
}
```

### `witness` Module

```rust
if let Some(w) = find_witness(&k, &formula) {
    println!("Witness: {}", w.description);
    println!("Path: {:?}", w.path);
}
```

### `complexity` Module

```rust
let report = result.complexity;
report.total_iterations();              // Sum across all operators
report.max_iterations();                // Max for any single occurrence
report.iterations("EF");                // [3, 5] — per occurrence
report.within_quadratic_bounds(n);      // true if within O(|S|²)
println!("{report}");                   // Human-readable report
```

---

## Algorithm Details

### Iterative Fixpoint Computation

This library uses **iterative** fixpoint computation for all temporal operators. This is a critical design choice — a naive recursive implementation would cause stack overflow on non-trivial structures.

**Least Fixpoint (EF, AF, EU, AU) — forward expansion:**

```text
EF φ:
  current = {s : s ⊨ φ}          // Start with states satisfying φ
  loop:
    preds = predecessors(current) // Find all predecessors
    next = current ∪ preds        // Expand
    if next == current: break     // Converged
  return current
```

**Greatest Fixpoint (EG, AG) — backward pruning:**

```text
EG φ:
  current = {s : s ⊨ φ}              // Start with all states satisfying φ
  loop:
    to_remove = {s ∈ current :       // Find states with no φ-successor
                   no successor of s is in current}
    if to_remove is empty: break     // Converged
    current = current \ to_remove    // Prune
  return current
```

**Why iterative instead of recursive?**

A recursive approach computes `EF φ` as `φ ∨ EX EF φ`, calling itself recursively. On a chain of n states, this creates n stack frames. For large structures (thousands of states), this overflows the stack. The iterative approach uses constant stack space regardless of structure size.

### Complexity Analysis

| Operator | Fixpoint Type | Max Iterations | Per-iteration Cost |
|----------|--------------|----------------|-------------------|
| EX/AX | None (direct) | 1 | O(\|E\|) |
| EF/AF | Least | O(\|S\|) | O(\|E\|) |
| EG/AG | Greatest | O(\|S\|) | O(\|S\|) |
| EU/AU | Least | O(\|S\|) | O(\|E\|) |

Where |S| = number of states, |E| = number of transitions.

**Total complexity:** O(|S| × (|S| + |E|) × |f|) = O(|S|² × |f|) for dense graphs, where |f| is the formula size.

This matches the classical result: CTL model checking is in **PTIME** with time complexity O(|M| × |f|) where |M| is the size of the Kripke structure.

The `ComplexityReport` tracks actual iteration counts, allowing you to verify that performance matches theoretical bounds.

---

## Design Decisions

### Why iterative fixpoints?

Recursive CTL model checking is elegant but dangerous. On a linear chain of n states, the recursive formulation of EF creates n stack frames. At n = 10,000, this overflows even generous stack sizes. Our iterative approach uses O(|S|) heap memory (via HashSet) and O(1) stack space.

### Why only `serde` as a dependency?

Kripke structures and CTL formulas are serializable data types. `serde` is the universal serialization framework in the Rust ecosystem. All other operations (model checking, fixpoint computation, path finding) are implemented from scratch for maximum clarity and educational value.

### Why `HashSet<usize>` for state sets?

State sets are the fundamental data structure in CTL model checking. `HashSet<usize>` provides O(1) insertion, membership testing, and efficient set operations (union, intersection, difference) via the standard library.

### Why builder pattern for Kripke structures?

Kripke structures have four interrelated components (states, transitions, labeling, initial states). The builder pattern allows incremental construction with validation at the end, catching issues like missing transitions or invalid state references.

### Why track complexity?

Educational and debugging value. Seeing that EG required exactly 3 iterations on a 5-state structure provides insight into how the algorithm behaves. The complexity report also serves as a regression test — if iterations suddenly exceed |S|, something is wrong.

---

## ASCII Art: Computation Tree

A Kripke structure and its computation tree unfolding:

```
Kripke Structure:                    Computation Tree (from s0):
                                    
   ┌───┐                              s0 (p)
   │s0 │ p,q                          
   └─┬─╲                              
     │   ╲                         ╱       ╲
     ↓    ↓                      s1(q)     s0(p,q)
   ┌───┐ ┌───┐                  ╱   ╲      ╱    ╲
   │s1 │ │s0 │ p,q             s0   s1    s1    s0
   │ q │ └───┘                 ↑↓   ↑↓   ↑↓    ↑↓
   └─┬─┘                     (infinite unfolding)
     │                          
     ↓                          
   ┌───┐                          
   │s0 │ p,q     ← self-loop     
   └───┘                          
     ↑──────────┘                 

  States: S = {s0, s1}
  Transitions: s0→s1, s0→s0, s1→s0
  Labels: L(s0)={p,q}, L(s1)={q}
  
  CTL Example: EX p  
    s0 ⊨ EX p  (s0 has successor s0 with p ✓)
    s1 ⊨ EX p  (s1 has successor s0 with p ✓)
```

**Fixpoint example for EG q:**

```
Step 0: current = {s0, s1}  (both states have q in their labels)
Step 1: Check successors:
  s0: successors are {s1, s0}, both in current → keep
  s1: successors are {s0}, in current → keep
  to_remove = {} → converged!

Result: EG q = {s0, s1}  (there exists a path where q always holds)
```

**Fixpoint example for AG p:**

```
Step 0: current = {s0}  (only s0 has p in its labels)
Step 1: Check successors:
  s0: successor s1 is NOT in current → remove s0
  to_remove = {s0}
Step 2: current = {} → converged

Result: AG p = {}  (p does not hold globally from any state)
```

---

## References

1. **Clarke, E.M. & Emerson, E.A.** (1981). "Design and Synthesis of Synchronization Skeletons Using Branching Time Temporal Logic." *Logics of Programs*, Lecture Notes in Computer Science, Vol. 131, Springer. — The original paper introducing CTL model checking.

2. **Baier, C. & Katoen, J.-P.** (2008). *Principles of Model Checking*. MIT Press. — The definitive textbook on model checking, covering CTL in Chapters 3 and 6.

3. **Clarke, E.M., Grumberg, O., & Peled, D.A.** (1999). *Model Checking*. MIT Press. — Comprehensive treatment of model checking algorithms and their complexity.

4. **Tarski, A.** (1955). "A Lattice-Theoretical Fixpoint Theorem and Its Applications." *Pacific Journal of Mathematics*, 5(2), 285–309. — The fixpoint theorem underlying CTL model checking.

5. **Emerson, E.A.** (1990). "Temporal and Modal Logic." In *Handbook of Theoretical Computer Science, Volume B*, Elsevier. — Survey of temporal logics including CTL, LTL, and CTL*.

6. **Huth, M. & Ryan, M.** (2004). *Logic in Computer Science: Modelling and Reasoning about Systems*. Cambridge University Press, 2nd edition. — Accessible introduction to CTL and model checking with worked examples.

7. **McMillan, K.L.** (1993). *Symbolic Model Checking*. Kluwer Academic Publishers. — Introduced BDD-based symbolic model checking for CTL.

---

## License

MIT
