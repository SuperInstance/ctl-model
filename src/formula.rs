use serde::{Deserialize, Serialize};
use std::fmt;

/// A Computation Tree Logic (CTL) formula.
///
/// CTL extends propositional logic with temporal operators that quantify over
/// paths in the computation tree of a Kripke structure. Every temporal operator
/// is a pair of:
/// - A **path quantifier**: **E** (exists a path) or **A** (for all paths)
/// - A **state quantifier**: **X** (next), **F** (eventually), **G** (always), or **U** (until)
///
/// # Syntax
///
/// ```text
/// φ ::= p | ¬φ | φ₁ ∧ φ₂ | φ₁ ∨ φ₂
///     | EX φ | AX φ | EF φ | AF φ | EG φ | AG φ
///     | φ₁ EU φ₂ | φ₁ AU φ₂
/// ```
///
/// # Semantics (selected)
///
/// | Formula | Meaning |
/// |---------|---------|
/// | EX φ | There exists a successor where φ holds |
/// | AX φ | φ holds in all successors |
/// | EF φ | There exists a path where φ eventually holds |
/// | AF φ | On all paths, φ eventually holds |
/// | EG φ | There exists a path where φ always holds |
/// | AG φ | On all paths, φ always holds |
/// | φ₁ EU φ₂ | There exists a path where φ₁ holds until φ₂ holds |
/// | φ₁ AU φ₂ | On all paths, φ₁ holds until φ₂ holds |
///
/// # Example
///
/// ```
/// use ctl_model::formula::CtlFormula;
///
/// let f = CtlFormula::And(
///     Box::new(CtlFormula::Atom("p".into())),
///     Box::new(CtlFormula::Ex(Box::new(CtlFormula::Atom("q".into())))),
/// );
/// assert_eq!(format!("{f}"), "(p ∧ EX q)");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CtlFormula {
    /// Atomic proposition.
    Atom(String),
    /// Negation: ¬φ
    Not(Box<CtlFormula>),
    /// Conjunction: φ₁ ∧ φ₂
    And(Box<CtlFormula>, Box<CtlFormula>),
    /// Disjunction: φ₁ ∨ φ₂
    Or(Box<CtlFormula>, Box<CtlFormula>),
    /// Exists next: EX φ — there exists a successor satisfying φ.
    Ex(Box<CtlFormula>),
    /// All next: AX φ — all successors satisfy φ.
    Ax(Box<CtlFormula>),
    /// Exists eventually: EF φ — exists a path where φ eventually holds.
    Ef(Box<CtlFormula>),
    /// All eventually: AF φ — on all paths, φ eventually holds.
    Af(Box<CtlFormula>),
    /// Exists globally: EG φ — exists a path where φ always holds.
    Eg(Box<CtlFormula>),
    /// All globally: AG φ — on all paths, φ always holds.
    Ag(Box<CtlFormula>),
    /// Exists until: φ₁ EU φ₂ — exists a path where φ₁ until φ₂.
    Eu(Box<CtlFormula>, Box<CtlFormula>),
    /// All until: φ₁ AU φ₂ — on all paths, φ₁ until φ₂.
    Au(Box<CtlFormula>, Box<CtlFormula>),
}

impl fmt::Display for CtlFormula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CtlFormula::Atom(s) => write!(f, "{s}"),
            CtlFormula::Not(phi) => write!(f, "¬{}", paren_if_needed(phi)),
            CtlFormula::And(l, r) => write!(f, "({} ∧ {})", l, r),
            CtlFormula::Or(l, r) => write!(f, "({} ∨ {})", l, r),
            CtlFormula::Ex(phi) => write!(f, "EX {}", paren_if_needed(phi)),
            CtlFormula::Ax(phi) => write!(f, "AX {}", paren_if_needed(phi)),
            CtlFormula::Ef(phi) => write!(f, "EF {}", paren_if_needed(phi)),
            CtlFormula::Af(phi) => write!(f, "AF {}", paren_if_needed(phi)),
            CtlFormula::Eg(phi) => write!(f, "EG {}", paren_if_needed(phi)),
            CtlFormula::Ag(phi) => write!(f, "AG {}", paren_if_needed(phi)),
            CtlFormula::Eu(l, r) => write!(f, "({} EU {})", l, r),
            CtlFormula::Au(l, r) => write!(f, "({} AU {})", l, r),
        }
    }
}

fn paren_if_needed(phi: &CtlFormula) -> String {
    match phi {
        CtlFormula::Atom(_) => format!("{phi}"),
        _ => format!("({phi})"),
    }
}

impl CtlFormula {
    /// Construct an atom.
    pub fn atom(s: impl Into<String>) -> Self {
        CtlFormula::Atom(s.into())
    }

    /// Construct ¬φ.
    #[allow(clippy::should_implement_trait)]
    pub fn not(phi: CtlFormula) -> Self {
        CtlFormula::Not(Box::new(phi))
    }

    /// Construct φ₁ ∧ φ₂.
    pub fn and(l: CtlFormula, r: CtlFormula) -> Self {
        CtlFormula::And(Box::new(l), Box::new(r))
    }

    /// Construct φ₁ ∨ φ₂.
    pub fn or(l: CtlFormula, r: CtlFormula) -> Self {
        CtlFormula::Or(Box::new(l), Box::new(r))
    }

    /// Construct EX φ.
    pub fn ex(phi: CtlFormula) -> Self {
        CtlFormula::Ex(Box::new(phi))
    }

    /// Construct AX φ.
    pub fn ax(phi: CtlFormula) -> Self {
        CtlFormula::Ax(Box::new(phi))
    }

    /// Construct EF φ.
    pub fn ef(phi: CtlFormula) -> Self {
        CtlFormula::Ef(Box::new(phi))
    }

    /// Construct AF φ.
    pub fn af(phi: CtlFormula) -> Self {
        CtlFormula::Af(Box::new(phi))
    }

    /// Construct EG φ.
    pub fn eg(phi: CtlFormula) -> Self {
        CtlFormula::Eg(Box::new(phi))
    }

    /// Construct AG φ.
    pub fn ag(phi: CtlFormula) -> Self {
        CtlFormula::Ag(Box::new(phi))
    }

    /// Construct φ₁ EU φ₂.
    pub fn eu(l: CtlFormula, r: CtlFormula) -> Self {
        CtlFormula::Eu(Box::new(l), Box::new(r))
    }

    /// Construct φ₁ AU φ₂.
    pub fn au(l: CtlFormula, r: CtlFormula) -> Self {
        CtlFormula::Au(Box::new(l), Box::new(r))
    }

    /// Convenience: implies φ₁ → φ₂ = ¬φ₁ ∨ φ₂
    pub fn implies(l: CtlFormula, r: CtlFormula) -> Self {
        CtlFormula::Or(Box::new(CtlFormula::Not(Box::new(l))), Box::new(r))
    }

    /// Convenience: iff φ₁ ↔ φ₂ = (φ₁ → φ₂) ∧ (φ₂ → φ₁)
    pub fn iff(l: CtlFormula, r: CtlFormula) -> Self {
        CtlFormula::And(
            Box::new(CtlFormula::implies(l.clone(), r.clone())),
            Box::new(CtlFormula::implies(r, l)),
        )
    }

    /// Return the sub-formulas of this formula (immediate children).
    pub fn sub_formulas(&self) -> Vec<&CtlFormula> {
        match self {
            CtlFormula::Atom(_) => vec![],
            CtlFormula::Not(p)
            | CtlFormula::Ex(p)
            | CtlFormula::Ax(p)
            | CtlFormula::Ef(p)
            | CtlFormula::Af(p)
            | CtlFormula::Eg(p)
            | CtlFormula::Ag(p) => vec![p],
            CtlFormula::And(l, r)
            | CtlFormula::Or(l, r)
            | CtlFormula::Eu(l, r)
            | CtlFormula::Au(l, r) => vec![l, r],
        }
    }

    /// Count the total number of nodes in the formula tree.
    pub fn size(&self) -> usize {
        1 + self.sub_formulas().iter().map(|f| f.size()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_atom() {
        assert_eq!(format!("{}", CtlFormula::atom("p")), "p");
    }

    #[test]
    fn display_not() {
        assert_eq!(format!("{}", CtlFormula::not(CtlFormula::atom("p"))), "¬p");
    }

    #[test]
    fn display_and() {
        assert_eq!(
            format!(
                "{}",
                CtlFormula::and(CtlFormula::atom("p"), CtlFormula::atom("q"))
            ),
            "(p ∧ q)"
        );
    }

    #[test]
    fn display_ex() {
        assert_eq!(format!("{}", CtlFormula::ex(CtlFormula::atom("p"))), "EX p");
    }

    #[test]
    fn display_eu() {
        assert_eq!(
            format!(
                "{}",
                CtlFormula::eu(CtlFormula::atom("p"), CtlFormula::atom("q"))
            ),
            "(p EU q)"
        );
    }

    #[test]
    fn test_size() {
        let f = CtlFormula::and(
            CtlFormula::atom("p"),
            CtlFormula::not(CtlFormula::atom("q")),
        );
        assert_eq!(f.size(), 4);
    }
}
