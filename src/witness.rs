use crate::checker;
use crate::formula::CtlFormula;
use crate::kripke::KripkeStruct;
use std::collections::{HashMap, HashSet, VecDeque};

/// A witness for why a formula is true.
///
/// For **existential** operators (EX, EF, EG, EU): contains a path that
/// satisfies the formula.
///
/// For **universal** operators (AX, AF, AG, AU): contains a description of
/// why the property holds.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Witness {
    /// Description of why the formula holds.
    pub description: String,
    /// A path (sequence of state indices) demonstrating satisfaction.
    pub path: Vec<usize>,
    /// The formula that holds.
    pub formula: CtlFormula,
}

/// Find a witness path when a CTL formula holds in an initial state.
///
/// For **existential** formulas (EX, EF, EG, EU): finds an actual path
/// satisfying the formula.
///
/// For **universal** formulas (AX, AF, AG, AU): provides evidence that
/// all paths satisfy the formula.
///
/// Returns `None` if the formula does not hold in any initial state.
///
/// # Example
///
/// ```
/// use ctl_model::kripke::KripkeStruct;
/// use ctl_model::formula::CtlFormula;
/// use ctl_model::witness::find_witness;
///
/// let k = KripkeStruct::builder()
///     .state("s0").state("s1")
///     .transition(0, 1).transition(1, 0)
///     .label(0, "p").label(1, "q")
///     .initial_state(0)
///     .build().unwrap();
///
/// let w = find_witness(&k, &CtlFormula::ef(CtlFormula::atom("q")));
/// assert!(w.is_some());
/// let w = w.unwrap();
/// assert_eq!(w.path[0], 0);
/// assert_eq!(w.path[1], 1);
/// ```
pub fn find_witness(k: &KripkeStruct, formula: &CtlFormula) -> Option<Witness> {
    let result = checker::check(k, formula);
    if !result.holds {
        return None;
    }

    let start = k
        .initial_states
        .iter()
        .find(|&&s| result.satisfying_states.contains(&s))
        .copied()
        .unwrap_or(0);

    let path = build_witness_path(k, formula, start, &result.satisfying_states);

    Some(Witness {
        description: format!(
            "Formula {} is true in state {} ({})",
            formula,
            start,
            k.state_name(start).unwrap_or("?")
        ),
        path,
        formula: formula.clone(),
    })
}

fn build_witness_path(
    k: &KripkeStruct,
    formula: &CtlFormula,
    start: usize,
    _sat: &HashSet<usize>,
) -> Vec<usize> {
    match formula {
        CtlFormula::Atom(_) => vec![start],
        CtlFormula::Not(_phi) => vec![start],
        CtlFormula::And(_, _) | CtlFormula::Or(_, _) => vec![start],
        CtlFormula::Ex(phi) => {
            let inner_sat = checker::check(k, phi).satisfying_states;
            let mut path = vec![start];
            if let Some(succ) = k
                .successors(start)
                .iter()
                .find(|&&s| inner_sat.contains(&s))
            {
                path.push(*succ);
            }
            path
        }
        CtlFormula::Ax(_) => vec![start],
        CtlFormula::Ef(phi) => {
            let inner_sat = checker::check(k, phi).satisfying_states;
            find_path_to_target(k, start, &inner_sat)
        }
        CtlFormula::Af(_) => vec![start],
        CtlFormula::Eg(phi) => {
            let inner_sat = checker::check(k, phi).satisfying_states;
            build_cycle_path(k, start, &inner_sat)
        }
        CtlFormula::Ag(_) => vec![start],
        CtlFormula::Eu(l, r) => {
            let sat_r = checker::check(k, r).satisfying_states;
            let sat_l = checker::check(k, l).satisfying_states;
            find_eu_path(k, start, &sat_l, &sat_r)
        }
        CtlFormula::Au(_, _) => vec![start],
    }
}

/// BFS to find a path from start to any state in target.
fn find_path_to_target(k: &KripkeStruct, start: usize, target: &HashSet<usize>) -> Vec<usize> {
    if target.contains(&start) {
        return vec![start];
    }

    let mut queue = VecDeque::new();
    let mut parent: HashMap<usize, usize> = HashMap::new();
    let mut visited = HashSet::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        for &succ in k.successors(current) {
            if visited.contains(&succ) {
                continue;
            }
            parent.insert(succ, current);
            visited.insert(succ);

            if target.contains(&succ) {
                // Reconstruct path
                let mut path = vec![succ];
                while let Some(&p) = parent.get(path.last().unwrap()) {
                    path.push(p);
                }
                path.reverse();
                return path;
            }
            queue.push_back(succ);
        }
    }

    vec![start]
}

/// Find a cycle path for EG: a path through states in `good` that forms a loop.
fn build_cycle_path(k: &KripkeStruct, start: usize, good: &HashSet<usize>) -> Vec<usize> {
    if !good.contains(&start) {
        return vec![start];
    }

    let mut path = vec![start];
    let mut visited = HashSet::new();
    visited.insert(start);
    let mut current = start;

    for _ in 0..k.state_count() + 1 {
        // Look for a successor in good that closes a cycle or extends the path
        let succs = k.successors(current);
        // First, check if we can close the cycle back to start
        if succs.contains(&start) && path.len() > 1 {
            path.push(start);
            return path;
        }
        // Try to extend
        if let Some(&next) = succs
            .iter()
            .find(|&&s| good.contains(&s) && !visited.contains(&s))
        {
            path.push(next);
            visited.insert(next);
            current = next;
        } else if let Some(&next) = succs.iter().find(|&&s| good.contains(&s)) {
            // Close cycle to this state
            path.push(next);
            return path;
        } else {
            break;
        }
    }

    path
}

/// Find a path for EU: from start, go through l-states, end at r-state.
fn find_eu_path(
    k: &KripkeStruct,
    start: usize,
    sat_l: &HashSet<usize>,
    sat_r: &HashSet<usize>,
) -> Vec<usize> {
    if sat_r.contains(&start) {
        return vec![start];
    }

    let mut queue = VecDeque::new();
    let mut parent: HashMap<usize, usize> = HashMap::new();
    let mut visited = HashSet::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        for &succ in k.successors(current) {
            if visited.contains(&succ) {
                continue;
            }
            if !sat_l.contains(&current) && !sat_r.contains(&current) {
                continue;
            }
            parent.insert(succ, current);
            visited.insert(succ);

            if sat_r.contains(&succ) {
                let mut path = vec![succ];
                while let Some(&p) = parent.get(path.last().unwrap()) {
                    path.push(p);
                }
                path.reverse();
                return path;
            }
            queue.push_back(succ);
        }
    }

    vec![start]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_ef() {
        let k = KripkeStruct::builder()
            .state("s0")
            .state("s1")
            .transition(0, 1)
            .transition(1, 0)
            .label(0, "p")
            .label(1, "q")
            .initial_state(0)
            .build()
            .unwrap();

        let w = find_witness(&k, &CtlFormula::ef(CtlFormula::atom("q")));
        assert!(w.is_some());
        let w = w.unwrap();
        assert_eq!(w.path.len(), 2);
        assert_eq!(w.path[0], 0);
        assert_eq!(w.path[1], 1);
    }

    #[test]
    fn witness_ex() {
        let k = KripkeStruct::builder()
            .state("s0")
            .state("s1")
            .transition(0, 1)
            .transition(1, 0)
            .label(0, "p")
            .label(1, "q")
            .initial_state(0)
            .build()
            .unwrap();

        let w = find_witness(&k, &CtlFormula::ex(CtlFormula::atom("q")));
        assert!(w.is_some());
        let w = w.unwrap();
        assert_eq!(w.path, vec![0, 1]);
    }

    #[test]
    fn no_witness_when_false() {
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

        let w = find_witness(&k, &CtlFormula::atom("q"));
        assert!(w.is_none());
    }

    #[test]
    fn witness_eg_loop() {
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

        let w = find_witness(&k, &CtlFormula::eg(CtlFormula::atom("p")));
        assert!(w.is_some());
        let path = &w.unwrap().path;
        assert!(path.len() >= 2);
    }
}
