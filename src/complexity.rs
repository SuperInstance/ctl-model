use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks the number of fixpoint iterations performed per CTL operator.
///
/// This is useful for understanding the computational complexity of model
/// checking a particular formula on a particular structure. The theoretical
/// worst case is O(|S|) iterations per fixpoint, yielding O(|S|²) overall
/// for the fixpoint computation.
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
/// let result = check(&k, &CtlFormula::ef(CtlFormula::atom("q")));
/// println!("EF iterations: {:?}", result.complexity.iterations("EF"));
/// println!("Report:\n{}", result.complexity);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityReport {
    /// Map from operator name to list of iteration counts (one per occurrence).
    iterations: HashMap<String, Vec<u64>>,
}

impl ComplexityReport {
    /// Create an empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record iterations for an operator occurrence.
    pub fn record(&mut self, operator: &str, iterations: u64) {
        self.iterations
            .entry(operator.to_string())
            .or_default()
            .push(iterations);
    }

    /// Get the iteration counts for a specific operator.
    pub fn iterations(&self, operator: &str) -> &[u64] {
        static EMPTY: &[u64] = &[];
        self.iterations
            .get(operator)
            .map(|v| v.as_slice())
            .unwrap_or(EMPTY)
    }

    /// Total iterations across all operators.
    pub fn total_iterations(&self) -> u64 {
        self.iterations.values().flat_map(|v| v.iter()).sum()
    }

    /// Maximum iterations for any single operator occurrence.
    pub fn max_iterations(&self) -> u64 {
        self.iterations
            .values()
            .flat_map(|v| v.iter())
            .copied()
            .max()
            .unwrap_or(0)
    }

    /// Number of distinct operators that required fixpoint computation.
    pub fn operator_count(&self) -> usize {
        self.iterations.len()
    }

    /// Returns true if the complexity is within O(|S|²) bounds.
    ///
    /// Each fixpoint operator requires at most |S| iterations, and with at most
    /// |S| states, the total is bounded by |S|² × |f| where |f| is formula size.
    pub fn within_quadratic_bounds(&self, state_count: usize) -> bool {
        let bound = state_count as u64;
        self.iterations
            .values()
            .all(|counts| counts.iter().all(|&c| c <= bound))
    }
}

impl std::fmt::Display for ComplexityReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Complexity Report")?;
        writeln!(f, "=================")?;
        if self.iterations.is_empty() {
            writeln!(f, "No fixpoint operations performed.")?;
            return Ok(());
        }
        for (op, counts) in &self.iterations {
            let total: u64 = counts.iter().sum();
            let max = counts.iter().max().unwrap_or(&0);
            writeln!(
                f,
                "  {}: {} occurrence(s), total {} iterations, max {}",
                op,
                counts.len(),
                total,
                max
            )?;
        }
        writeln!(f, "  Total: {} iterations", self.total_iterations())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report() {
        let r = ComplexityReport::new();
        assert_eq!(r.total_iterations(), 0);
        assert_eq!(r.max_iterations(), 0);
        assert!(r.iterations("EF").is_empty());
    }

    #[test]
    fn record_and_query() {
        let mut r = ComplexityReport::new();
        r.record("EF", 3);
        r.record("EF", 5);
        r.record("AG", 2);
        assert_eq!(r.iterations("EF"), &[3, 5]);
        assert_eq!(r.total_iterations(), 10);
        assert_eq!(r.max_iterations(), 5);
        assert_eq!(r.operator_count(), 2);
    }

    #[test]
    fn quadratic_bounds() {
        let mut r = ComplexityReport::new();
        r.record("EF", 5);
        r.record("EG", 3);
        assert!(r.within_quadratic_bounds(10));
        assert!(!r.within_quadratic_bounds(3));
    }
}
