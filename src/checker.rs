use crate::complexity::ComplexityReport;
use crate::formula::CtlFormula;
use crate::kripke::KripkeStruct;
use std::collections::HashSet;

/// Result of model checking a formula on a Kripke structure.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CheckResult {
    /// States satisfying the formula.
    pub satisfying_states: HashSet<usize>,
    /// Whether the formula holds in all initial states.
    pub holds: bool,
    /// Complexity report for fixpoint operations.
    pub complexity: ComplexityReport,
}

/// Model check a CTL formula on a Kripke structure.
///
/// Returns the set of states satisfying the formula and whether it holds in
/// all initial states. Uses **iterative fixpoint computation** for all
/// temporal operators — no recursion into fixpoint loops.
///
/// # Complexity
///
/// The algorithm runs in **O(|S|² × |f|)** time where |S| is the number of
/// states and |f| is the size of the formula, matching the classic result
/// of Clarke and Emerson (1981).
///
/// # Example
///
/// ```
/// use ctl_model::kripke::KripkeStruct;
/// use ctl_model::formula::CtlFormula;
/// use ctl_model::checker::check;
///
/// let k = KripkeStruct::builder()
///     .state("s0").state("s1")
///     .transition(0, 1).transition(1, 0)
///     .label(0, "p").label(1, "q")
///     .initial_state(0)
///     .build().unwrap();
///
/// let result = check(&k, &CtlFormula::atom("p"));
/// assert!(result.holds);
/// ```
pub fn check(k: &KripkeStruct, formula: &CtlFormula) -> CheckResult {
    let mut ctx = CheckContext::new(k);
    let satisfying = ctx.eval(formula);
    let holds = k.initial_states.iter().all(|&s| satisfying.contains(&s));
    CheckResult {
        satisfying_states: satisfying,
        holds,
        complexity: ctx.complexity,
    }
}

struct CheckContext<'a> {
    k: &'a KripkeStruct,
    complexity: ComplexityReport,
}

impl<'a> CheckContext<'a> {
    fn new(k: &'a KripkeStruct) -> Self {
        Self {
            k,
            complexity: ComplexityReport::new(),
        }
    }

    /// Evaluate a CTL formula, returning the set of states satisfying it.
    ///
    /// This dispatches to iterative fixpoint algorithms for all temporal operators.
    fn eval(&mut self, formula: &CtlFormula) -> HashSet<usize> {
        match formula {
            CtlFormula::Atom(p) => self.eval_atom(p),
            CtlFormula::Not(phi) => self.eval_not(phi),
            CtlFormula::And(l, r) => self.eval_and(l, r),
            CtlFormula::Or(l, r) => self.eval_or(l, r),
            CtlFormula::Ex(phi) => self.eval_ex(phi),
            CtlFormula::Ax(phi) => self.eval_ax(phi),
            CtlFormula::Ef(phi) => self.eval_ef(phi),
            CtlFormula::Af(phi) => self.eval_af(phi),
            CtlFormula::Eg(phi) => self.eval_eg(phi),
            CtlFormula::Ag(phi) => self.eval_ag(phi),
            CtlFormula::Eu(l, r) => self.eval_eu(l, r),
            CtlFormula::Au(l, r) => self.eval_au(l, r),
        }
    }

    fn eval_atom(&self, p: &str) -> HashSet<usize> {
        let mut result = HashSet::new();
        for i in 0..self.k.state_count() {
            if self.k.labels(i).contains(p) {
                result.insert(i);
            }
        }
        result
    }

    fn eval_not(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let inner = self.eval(phi);
        let all: HashSet<usize> = (0..self.k.state_count()).collect();
        &all - &inner
    }

    fn eval_and(&mut self, l: &CtlFormula, r: &CtlFormula) -> HashSet<usize> {
        &self.eval(l) & &self.eval(r)
    }

    fn eval_or(&mut self, l: &CtlFormula, r: &CtlFormula) -> HashSet<usize> {
        &self.eval(l) | &self.eval(r)
    }

    /// EX φ: states that have at least one successor satisfying φ.
    fn eval_ex(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let sat = self.eval(phi);
        let mut result = HashSet::new();
        for i in 0..self.k.state_count() {
            for &succ in self.k.successors(i) {
                if sat.contains(&succ) {
                    result.insert(i);
                    break;
                }
            }
        }
        result
    }

    /// AX φ: all successors satisfy φ. Dual of EX: AX φ ≡ ¬EX ¬φ.
    fn eval_ax(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let sat = self.eval(phi);
        let mut result = HashSet::new();
        for i in 0..self.k.state_count() {
            if self.k.successors(i).iter().all(|&s| sat.contains(&s)) {
                result.insert(i);
            }
        }
        result
    }

    /// EF φ = φ ∨ EX EF φ — least fixpoint.
    /// Iterative: start with states satisfying φ, expand by one-step predecessor.
    fn eval_ef(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let mut current = self.eval(phi);
        let mut iterations = 0u64;
        loop {
            let preds = self.k.predecessors_of_set(&current);
            let next = &current | &preds;
            iterations += 1;
            if next == current {
                break;
            }
            current = next;
        }
        self.complexity.record("EF", iterations);
        current
    }

    /// AF φ — on all paths, φ eventually holds.
    /// AF φ = φ ∨ AX AF φ — least fixpoint.
    /// Iterative: start with states satisfying φ, add states whose ALL successors
    /// are already in the set.
    fn eval_af(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let mut current = self.eval(phi);
        let mut iterations = 0u64;
        loop {
            let mut new_states = HashSet::new();
            for i in 0..self.k.state_count() {
                if current.contains(&i) {
                    continue;
                }
                if self.k.successors(i).iter().all(|&s| current.contains(&s)) {
                    new_states.insert(i);
                }
            }
            iterations += 1;
            if new_states.is_empty() {
                break;
            }
            current.extend(new_states);
        }
        self.complexity.record("AF", iterations);
        current
    }

    /// EG φ = φ ∧ EX EG φ — greatest fixpoint.
    /// Iterative: start with ALL states satisfying φ, iteratively remove states
    /// that have no successor in the current set.
    fn eval_eg(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let mut current = self.eval(phi);
        let mut iterations = 0u64;
        loop {
            let mut to_remove = Vec::new();
            for &s in &current {
                // State must satisfy φ AND have at least one successor in current
                let has_successor_in = self
                    .k
                    .successors(s)
                    .iter()
                    .any(|&succ| current.contains(&succ));
                if !has_successor_in {
                    to_remove.push(s);
                }
            }
            iterations += 1;
            if to_remove.is_empty() {
                break;
            }
            for s in to_remove {
                current.remove(&s);
            }
        }
        self.complexity.record("EG", iterations);
        current
    }

    /// AG φ = φ ∧ AX AG φ — greatest fixpoint.
    /// Equivalent to ¬EF ¬φ.
    /// Iterative: start with all states satisfying φ, remove states that have
    /// any successor NOT in the current set.
    fn eval_ag(&mut self, phi: &CtlFormula) -> HashSet<usize> {
        let mut current = self.eval(phi);
        let mut iterations = 0u64;
        loop {
            let mut to_remove = Vec::new();
            for &s in &current {
                let all_successors_in = self
                    .k
                    .successors(s)
                    .iter()
                    .all(|&succ| current.contains(&succ));
                if !all_successors_in {
                    to_remove.push(s);
                }
            }
            iterations += 1;
            if to_remove.is_empty() {
                break;
            }
            for s in to_remove {
                current.remove(&s);
            }
        }
        self.complexity.record("AG", iterations);
        current
    }

    /// φ₁ EU φ₂ — exists a path where φ₁ holds until φ₂ holds.
    /// Least fixpoint: start with states satisfying φ₂, iteratively add
    /// predecessors satisfying φ₁.
    fn eval_eu(&mut self, l: &CtlFormula, r: &CtlFormula) -> HashSet<usize> {
        let sat_r = self.eval(r);
        let sat_l = self.eval(l);
        let mut current = sat_r.clone();
        let mut iterations = 0u64;
        loop {
            let preds = self.k.predecessors_of_set(&current);
            let addable: HashSet<usize> = preds.into_iter().filter(|s| sat_l.contains(s)).collect();
            let next: HashSet<usize> = current.union(&addable).copied().collect();
            iterations += 1;
            if next == current {
                break;
            }
            current = next;
        }
        self.complexity.record("EU", iterations);
        current
    }

    /// φ₁ AU φ₂ — on all paths, φ₁ holds until φ₂ holds.
    /// Least fixpoint: start with states satisfying φ₂, iteratively add states
    /// satisfying φ₁ whose ALL successors are in the current set.
    fn eval_au(&mut self, l: &CtlFormula, r: &CtlFormula) -> HashSet<usize> {
        let sat_r = self.eval(r);
        let sat_l = self.eval(l);
        let mut current = sat_r.clone();
        let mut iterations = 0u64;
        loop {
            let mut new_states = HashSet::new();
            for i in 0..self.k.state_count() {
                if current.contains(&i) {
                    continue;
                }
                if !sat_l.contains(&i) {
                    continue;
                }
                if self.k.successors(i).iter().all(|&s| current.contains(&s)) {
                    new_states.insert(i);
                }
            }
            iterations += 1;
            if new_states.is_empty() {
                break;
            }
            current.extend(new_states);
        }
        self.complexity.record("AU", iterations);
        current
    }
}
