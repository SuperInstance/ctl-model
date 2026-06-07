use serde::{Deserialize, Serialize};

/// A Kripke structure: a directed graph with labeled states used as the semantic
/// model for Computation Tree Logic.
///
/// # Formal Definition
///
/// A Kripke structure is a tuple **M = (S, R, L, S₀)** where:
/// - **S** is a finite set of states
/// - **R ⊆ S × S** is a total transition relation
/// - **L : S → 2^(AP)** is a labeling function mapping each state to the set of
///   atomic propositions true in that state
/// - **S₀ ⊆ S** is the set of initial states
///
/// # Construction
///
/// Use [`KripkeStruct::builder()`] to construct instances via the builder pattern.
///
/// ```
/// use ctl_model::kripke::KripkeStruct;
///
/// let k = KripkeStruct::builder()
///     .state("s0")
///     .state("s1")
///     .transition(0, 1)
///     .transition(1, 0)
///     .label(0, "p")
///     .label(1, "q")
///     .initial_state(0)
///     .build()
///     .unwrap();
///
/// assert_eq!(k.state_count(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KripkeStruct {
    /// Named states in order of insertion.
    pub states: Vec<String>,
    /// Adjacency list: transitions[s] = list of successor state indices.
    pub transitions: std::collections::HashMap<usize, Vec<usize>>,
    /// Labeling function: labeling[s] = set of atomic propositions true in state s.
    pub labeling: std::collections::HashMap<usize, std::collections::HashSet<String>>,
    /// Indices of initial states.
    pub initial_states: Vec<usize>,
}

impl KripkeStruct {
    /// Create a new empty Kripke structure.
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            transitions: std::collections::HashMap::new(),
            labeling: std::collections::HashMap::new(),
            initial_states: Vec::new(),
        }
    }

    /// Return a builder for constructing a Kripke structure.
    pub fn builder() -> KripkeBuilder {
        KripkeBuilder::new()
    }

    /// Number of states.
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Get the name of a state by index.
    pub fn state_name(&self, index: usize) -> Option<&str> {
        self.states.get(index).map(|s| s.as_str())
    }

    /// Get successors of a state.
    pub fn successors(&self, state: usize) -> &[usize] {
        self.transitions
            .get(&state)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the set of atomic propositions true in a state.
    pub fn labels(&self, state: usize) -> &std::collections::HashSet<String> {
        static EMPTY: std::sync::LazyLock<std::collections::HashSet<String>> =
            std::sync::LazyLock::new(std::collections::HashSet::new);
        self.labeling.get(&state).unwrap_or(&EMPTY)
    }

    /// Compute the set of predecessors of a state (states that have a transition to it).
    pub fn predecessors(&self, state: usize) -> Vec<usize> {
        let mut preds = Vec::new();
        for (&s, succs) in &self.transitions {
            if succs.contains(&state) {
                preds.push(s);
            }
        }
        preds
    }

    /// Compute the set of all predecessors of a set of states.
    pub fn predecessors_of_set(
        &self,
        states: &std::collections::HashSet<usize>,
    ) -> std::collections::HashSet<usize> {
        let mut preds = std::collections::HashSet::new();
        for (&s, succs) in &self.transitions {
            for &t in succs {
                if states.contains(&t) {
                    preds.insert(s);
                }
            }
        }
        preds
    }

    /// Validate the structure: all transitions refer to valid states,
    /// initial states are valid, and the transition relation is total.
    pub fn validate(&self) -> Result<(), String> {
        let n = self.states.len();
        for (&from, tos) in &self.transitions {
            if from >= n {
                return Err(format!("Invalid transition source: state index {from}"));
            }
            for &to in tos {
                if to >= n {
                    return Err(format!(
                        "Invalid transition target: state index {to} from {from}"
                    ));
                }
            }
        }
        for &s in &self.initial_states {
            if s >= n {
                return Err(format!("Invalid initial state index: {s}"));
            }
        }
        // Check totality: every state should have at least one successor
        for i in 0..n {
            if self.successors(i).is_empty() {
                return Err(format!(
                    "State {i} ({}) has no successors — transition relation must be total",
                    self.states[i]
                ));
            }
        }
        Ok(())
    }
}

impl Default for KripkeStruct {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for [`KripkeStruct`].
///
/// Provides a fluent API for incrementally constructing Kripke structures.
#[derive(Debug, Clone)]
pub struct KripkeBuilder {
    states: Vec<String>,
    transitions: std::collections::HashMap<usize, Vec<usize>>,
    labeling: std::collections::HashMap<usize, std::collections::HashSet<String>>,
    initial_states: Vec<usize>,
}

impl KripkeBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            transitions: std::collections::HashMap::new(),
            labeling: std::collections::HashMap::new(),
            initial_states: Vec::new(),
        }
    }

    /// Add a named state. The state's index equals the number of states added before it.
    pub fn state(mut self, name: impl Into<String>) -> Self {
        self.states.push(name.into());
        self
    }

    /// Add a transition from state `from` to state `to`.
    pub fn transition(mut self, from: usize, to: usize) -> Self {
        self.transitions.entry(from).or_default().push(to);
        self
    }

    /// Label a state with an atomic proposition.
    pub fn label(mut self, state: usize, prop: impl Into<String>) -> Self {
        self.labeling.entry(state).or_default().insert(prop.into());
        self
    }

    /// Mark a state as initial.
    pub fn initial_state(mut self, state: usize) -> Self {
        if !self.initial_states.contains(&state) {
            self.initial_states.push(state);
        }
        self
    }

    /// Build the Kripke structure, validating it in the process.
    pub fn build(self) -> Result<KripkeStruct, String> {
        let k = KripkeStruct {
            states: self.states,
            transitions: self.transitions,
            labeling: self.labeling,
            initial_states: self.initial_states,
        };
        k.validate()?;
        Ok(k)
    }
}

impl Default for KripkeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
