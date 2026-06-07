//! # ctl-model: Computation Tree Logic Model Checking
//!
//! A library for CTL (Computation Tree Logic) model checking on Kripke structures,
//! implementing the classical algorithms from Clarke & Emerson (1981).
//!
//! # Module Overview
//!
//! - [`kripke`] — Kripke structure definition and builder
//! - [`formula`] — CTL formula AST with Display
//! - [`checker`] — Iterative fixpoint model checking algorithm
//! - [`counterexample`] — Counterexample generation for false formulas
//! - [`witness`] — Witness path generation for true formulas
//! - [`complexity`] — Fixpoint iteration tracking and complexity reporting
//!
//! # Quick Start
//!
//! ```
//! use ctl_model::kripke::KripkeStruct;
//! use ctl_model::formula::CtlFormula;
//! use ctl_model::checker::check;
//!
//! // Build a simple mutual exclusion model
//! let k = KripkeStruct::builder()
//!     .state("idle")       // 0
//!     .state("request")    // 1
//!     .state("critical")   // 2
//!     .transition(0, 1)    // idle -> request
//!     .transition(1, 2)    // request -> critical
//!     .transition(2, 0)    // critical -> idle
//!     .transition(0, 0)    // idle -> idle (self-loop)
//!     .label(2, "in_cs")
//!     .initial_state(0)
//!     .build()
//!     .unwrap();
//!
//! // Check: is it always possible to eventually enter the critical section?
//! let safety = CtlFormula::ag(CtlFormula::ef(CtlFormula::atom("in_cs")));
//! let result = check(&k, &safety);
//! assert!(result.holds, "AG EF in_cs should hold");
//!
//! // Check complexity
//! println!("Fixpoint iterations: {}", result.complexity.total_iterations());
//! ```

pub mod checker;
pub mod complexity;
pub mod counterexample;
pub mod formula;
pub mod kripke;
pub mod witness;

#[cfg(test)]
mod tests;
