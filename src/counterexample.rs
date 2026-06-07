use crate::checker;
use crate::formula::CtlFormula;
use crate::kripke::KripkeStruct;
use std::collections::HashSet;

/// A witness for why a formula is false (counterexample).
///
/// For existential formulas (EX, EF, EG), this contains a path that *could*
/// satisfy the formula but doesn't (or a path demonstrating failure).
///
/// For universal formulas (AX, AF, AG, AU), this contains a violating path.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CounterExample {
    /// Description of the counterexample.
    pub description: String,
    /// A path (sequence of state indices) demonstrating the violation.
    pub path: Vec<usize>,
    /// The formula that was violated.
    pub formula: CtlFormula,
}

/// Find a counterexample when a CTL formula does NOT hold in an initial state.
///
/// For **universal** operators (AX, AF, AG, AU): finds a path that violates
/// the formula.
///
/// For **existential** operators (EX, EF, EG, EU): finds evidence that no
/// satisfying path exists.
///
/// Returns `None` if the formula holds in all initial states.
///
/// # Example
///
/// ```
/// use ctl_model::kripke::KripkeStruct;
/// use ctl_model::formula::CtlFormula;
/// use ctl_model::counterexample::find_counterexample;
///
/// // s0 --p--> s1 (no q label)
/// let k = KripkeStruct::builder()
///     .state("s0").state("s1")
///     .transition(0, 1).transition(1, 0)
///     .label(0, "p").label(1, "p")
///     .initial_state(0)
///     .build().unwrap();
///
/// let ce = find_counterexample(&k, &CtlFormula::atom("q"));
/// assert!(ce.is_some());
/// assert!(ce.unwrap().path.contains(&0));
/// ```
pub fn find_counterexample(k: &KripkeStruct, formula: &CtlFormula) -> Option<CounterExample> {
    let result = checker::check(k, formula);
    if result.holds {
        return None;
    }

    // Find an initial state where the formula fails
    let failing_initial = k
        .initial_states
        .iter()
        .find(|&&s| !result.satisfying_states.contains(&s))
        .copied();

    let start = failing_initial.unwrap_or_else(|| {
        // Just pick any non-satisfying state
        (0..k.state_count())
            .find(|s| !result.satisfying_states.contains(s))
            .unwrap_or(0)
    });

    let path = build_counterexample_path(k, formula, start, &result.satisfying_states);

    Some(CounterExample {
        description: format!(
            "Formula {} is false in state {} ({})",
            formula,
            start,
            k.state_name(start).unwrap_or("?")
        ),
        path,
        formula: formula.clone(),
    })
}

fn build_counterexample_path(
    k: &KripkeStruct,
    formula: &CtlFormula,
    start: usize,
    _sat: &HashSet<usize>,
) -> Vec<usize> {
    match formula {
        CtlFormula::Atom(_) => {
            vec![start]
        }
        CtlFormula::Not(_phi) => {
            vec![start]
        }
        CtlFormula::And(_, _) | CtlFormula::Or(_, _) => {
            vec![start]
        }
        CtlFormula::Ex(phi) => {
            // EX φ fails when no successor satisfies φ
            let inner_sat = eval_simple(k, phi);
            let mut path = vec![start];
            if let Some(succ) = k.successors(start).first() {
                path.push(*succ);
            }
            if let Some(violating) = k
                .successors(start)
                .iter()
                .find(|&&s| !inner_sat.contains(&s))
            {
                path.push(*violating);
            }
            path
        }
        CtlFormula::Ax(phi) => {
            // AX φ fails when some successor doesn't satisfy φ
            let inner_sat = eval_simple(k, phi);
            let mut path = vec![start];
            if let Some(violating) = k
                .successors(start)
                .iter()
                .find(|&&s| !inner_sat.contains(&s))
            {
                path.push(*violating);
            }
            path
        }
        CtlFormula::Ef(phi) => {
            // EF φ fails — show path that never reaches φ
            build_path_avoiding(k, start, &eval_simple(k, phi))
        }
        CtlFormula::Af(phi) => {
            // AF φ fails — there's a path that never reaches φ
            build_path_avoiding(k, start, &eval_simple(k, phi))
        }
        CtlFormula::Eg(_phi) => {
            // EG φ fails — no infinite path where φ always holds
            vec![start]
        }
        CtlFormula::Ag(phi) => {
            // AG φ fails — there's a reachable state where φ is false
            let inner_sat = eval_simple(k, phi);
            find_path_to_violation(k, start, &inner_sat)
        }
        CtlFormula::Eu(_l, _r) => {
            // EU fails when no path from start goes through l-states to r-state
            vec![start]
        }
        CtlFormula::Au(_l, r) => {
            // AU fails — there's a path that avoids r without l always holding
            let sat_r = eval_simple(k, r);
            build_path_avoiding(k, start, &sat_r)
        }
    }
}

/// Build a path from `start` that avoids states in `avoid`.
fn build_path_avoiding(k: &KripkeStruct, start: usize, avoid: &HashSet<usize>) -> Vec<usize> {
    if avoid.contains(&start) {
        return vec![start];
    }

    let mut path = vec![start];
    let mut visited = HashSet::new();
    visited.insert(start);
    let mut current = start;

    // Follow transitions staying outside `avoid`
    for _ in 0..k.state_count() + 1 {
        let next = k.successors(current).iter().find(|&&s| !avoid.contains(&s));
        match next {
            Some(&s) if !visited.contains(&s) => {
                path.push(s);
                visited.insert(s);
                current = s;
            }
            _ => break,
        }
    }

    path
}

/// Find a path from start to a state NOT in `target`.
fn find_path_to_violation(k: &KripkeStruct, start: usize, target: &HashSet<usize>) -> Vec<usize> {
    use std::collections::VecDeque;

    let mut queue = VecDeque::new();
    let mut parent: HashMap<usize, usize> = HashMap::new();
    queue.push_back(start);
    let mut visited = HashSet::new();
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        if !target.contains(&current) {
            // Reconstruct path
            let mut path = vec![current];
            while let Some(&p) = parent.get(path.last().unwrap()) {
                if p == start {
                    path.push(p);
                    break;
                }
                path.push(p);
            }
            path.reverse();
            return path;
        }
        for &succ in k.successors(current) {
            if !visited.contains(&succ) {
                visited.insert(succ);
                parent.insert(succ, current);
                queue.push_back(succ);
            }
        }
    }

    vec![start]
}

use std::collections::HashMap;

/// Simple recursive eval for building counterexample paths.
/// This is NOT used for the main model checking algorithm.
fn eval_simple(k: &KripkeStruct, formula: &CtlFormula) -> HashSet<usize> {
    let result = checker::check(k, formula);
    result.satisfying_states
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_kripke() -> KripkeStruct {
        KripkeStruct::builder()
            .state("s0")
            .state("s1")
            .state("s2")
            .transition(0, 1)
            .transition(1, 2)
            .transition(2, 2)
            .label(0, "p")
            .label(1, "q")
            .label(2, "p")
            .initial_state(0)
            .build()
            .unwrap()
    }

    #[test]
    fn counterexample_atom_false() {
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

        let ce = find_counterexample(&k, &CtlFormula::atom("q"));
        assert!(ce.is_some());
        assert!(ce.unwrap().path.contains(&0));
    }

    #[test]
    fn no_counterexample_when_holds() {
        let k = simple_kripke();
        let ce = find_counterexample(&k, &CtlFormula::atom("p"));
        assert!(ce.is_none());
    }

    #[test]
    fn counterexample_ag() {
        let k = simple_kripke();
        let ce = find_counterexample(&k, &CtlFormula::ag(CtlFormula::atom("p")));
        assert!(ce.is_some());
        let ce = ce.unwrap();
        assert!(!ce.path.is_empty());
    }
}
