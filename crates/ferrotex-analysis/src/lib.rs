use serde::{Deserialize, Serialize};

/// An abstract value representing a set of possible concrete TeX values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbstractValue {
    /// Represents any possible token list (Top).
    Any,
    /// Represents a specific control sequence (e.g., `\foo`).
    ControlSequence(String),
    /// Represents a braced group `{ ... }`.
    Group,
    /// Represents a dimension value (abstracted).
    Dimension,
    /// Represents an integer value (abstracted).
    Integer,
    /// Represents the empty set (Bottom / Unreachable).
    Bottom,
    /// Represents a simpler token
    Token(String),
    /// Represents an error detected during analysis.
    AnalysisError(String),
}

/// The state of the abstract machine.
#[derive(Debug, Clone, Default)]
pub struct AbstractState {
    /// Abstract input stack.
    pub input_stack: Vec<AbstractValue>,
    /// Abstract register values.
    pub registers: std::collections::HashMap<String, AbstractValue>,
}

/// A simplified abstract machine for analyzing TeX macro behavior.
pub struct AbstractMachine {
    pub state: AbstractState,
    pub expansion_depth: usize,
    pub max_depth: usize,
    /// Stack of currently expanding control sequences to detect cycles.
    pub call_stack: Vec<String>,
}

impl Default for AbstractMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractMachine {
    pub fn new() -> Self {
        Self {
            state: AbstractState::default(),
            expansion_depth: 0,
            max_depth: 1000,
            call_stack: Vec::new(),
        }
    }

    /// Steps the abstract machine one abstract instruction.
    pub fn step(&mut self) -> Option<AbstractValue> {
        if self.expansion_depth > self.max_depth {
            return Some(AbstractValue::AnalysisError(
                "Maximum recursion depth exceeded".to_string(),
            ));
        }

        // Pop the next token from input
        if let Some(token) = self.state.input_stack.pop() {
            match &token {
                AbstractValue::ControlSequence(name) => {
                    if self.call_stack.contains(name) {
                        return Some(AbstractValue::AnalysisError(format!(
                            "Infinite recursion detected in control sequence: {}",
                            name
                        )));
                    }
                    self.call_stack.push(name.clone());
                    self.expansion_depth += 1;

                    self.execute_control_sequence(name);

                    self.expansion_depth -= 1;
                    self.call_stack.pop();
                    Some(token)
                }
                AbstractValue::Group => {
                    // Enter group scope (simplified)
                    Some(token)
                }
                _ => {
                    // "Print" or absorb other tokens
                    Some(token)
                }
            }
        } else {
            None
        }
    }

    fn execute_control_sequence(&mut self, name: &str) {
        match name {
            "\\def" | "\\newcommand" => {
                // Abstract def: consumes arguments.
                // For analysis, we might just pop N items from input if we assume they are arguments.
                // Since this is abstract, we don't know the body, but we simulate definition.
                // Ideally, we'd look ahead for the parameter text.
                self.state.input_stack.push(AbstractValue::Any); // Valid definition created
            }
            "\\if" | "\\ifx" => {
                // Control flow branch.
                // In abstract interpretation, we usually fork state or check both branches.
                // Here we might push a "MergePoint" or similar if we were building a CFG.
                // For this MVP, let's just abstractly assume "true" branch for now
                // or consume tokens until \else or \fi (difficult without layout).
            }
            _ => {
                // Unknown command.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abstract_def() {
        let mut machine = AbstractMachine::new();
        // Push \def to input stack
        machine
            .state
            .input_stack
            .push(AbstractValue::ControlSequence("\\def".to_string()));

        machine.step();

        // After \def, we expect 'Any' to be pushed (abstract result of definition)
        assert_eq!(machine.state.input_stack.pop(), Some(AbstractValue::Any));
    }

    #[test]
    fn test_abstract_unknown() {
        let mut machine = AbstractMachine::new();
        machine
            .state
            .input_stack
            .push(AbstractValue::ControlSequence("\\unknown".to_string()));
        machine.step();
        // Unknown command should just be consumed (popped) with no side effects
        assert_eq!(machine.state.input_stack.len(), 0);
    }

    #[test]
    fn test_infinite_recursion() {
        let mut machine = AbstractMachine::new();
        // Setup a situation where \foo calls \foo (simplified)
        // We simulate this by having \foo's execution push \foo back to the stack.
        // We'll need to mock the definition behavior properly.
        machine
            .state
            .input_stack
            .push(AbstractValue::ControlSequence("\\foo".to_string()));

        // We override execute_control_sequence or just simulate it here.
        // For the sake of the test, let's just push it once and see it fail on the second step if it were re-added.
        machine.call_stack.push("\\foo".to_string());
        let result = machine.step();

        if let Some(AbstractValue::AnalysisError(msg)) = result {
            assert!(msg.contains("Infinite recursion"));
        } else {
            panic!("Should have detected infinite recursion");
        }
    }

    #[test]
    fn test_max_depth() {
        let mut machine = AbstractMachine::new();
        machine.expansion_depth = 1001;
        machine
            .state
            .input_stack
            .push(AbstractValue::ControlSequence("\\any".to_string()));
        let result = machine.step();
        assert!(matches!(result, Some(AbstractValue::AnalysisError(_))));
    }

    #[test]
    fn test_abstract_machine_default() {
        let machine = AbstractMachine::default();
        assert_eq!(machine.expansion_depth, 0);
        assert_eq!(machine.max_depth, 1000);
    }

    #[test]
    fn test_abstract_group() {
        let mut machine = AbstractMachine::new();
        machine.state.input_stack.push(AbstractValue::Group);
        let result = machine.step();
        assert_eq!(result, Some(AbstractValue::Group));
    }

    #[test]
    fn test_abstract_tokens() {
        let mut machine = AbstractMachine::new();
        machine
            .state
            .input_stack
            .push(AbstractValue::Token("a".to_string()));
        let result = machine.step();
        assert_eq!(result, Some(AbstractValue::Token("a".to_string())));
    }

    #[test]
    fn test_abstract_empty_stack() {
        let mut machine = AbstractMachine::new();
        let result = machine.step();
        assert_eq!(result, None);
    }
}
