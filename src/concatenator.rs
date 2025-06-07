use std::collections::{HashMap, HashSet};

use boolean_circuit::{Circuit, Gate, Operation};

pub fn concatenate<'a>(circuits: impl IntoIterator<Item = &'a Circuit>) -> Circuit {
    let circuits = circuits.into_iter().collect::<Vec<_>>();

    if circuits.is_empty() {
        return Circuit::default();
    }

    let concatenator = Concatenator::new(&circuits);
    let outputs = circuits
        .last()
        .unwrap()
        .named_outputs()
        .map(|(out, name)| (concatenator.map_gate(circuits.len() - 1, out), name))
        .collect::<Vec<_>>();

    Circuit::from_named_outputs(outputs.into_iter().chain(concatenator.new_outputs()))
        .with_input_order(concatenator.input_names())
        .unwrap()
}

struct Concatenator<'a> {
    circuits: &'a [&'a Circuit],
    /// New gate by circuit index and input name.
    input_name_substitutions: HashMap<(usize, String), Gate>,
    /// New gate by circuit index and gate ID.
    gate_substitutions: HashMap<(usize, usize), Gate>,
    used_input_names: HashSet<String>,
    used_output_names: HashSet<String>,
}

impl<'a> Concatenator<'a> {
    pub fn new(circuits: &'a [&'a Circuit]) -> Self {
        assert!(!circuits.is_empty(), "Cannot concatenate empty circuits");
        let replacements = Default::default();
        let used_input_names = circuits
            .first()
            .unwrap()
            .input_names()
            .map(|n| n.to_string())
            .collect();

        Self {
            circuits,
            input_name_substitutions: Default::default(),
            gate_substitutions: replacements,
            used_input_names,
            used_output_names: Default::default(),
        }
    }

    pub fn run(&mut self) -> Vec<(Gate, String)> {
        // Determine the "overhanging" outputs (including all outputs of the last circuit)
        let outputs =
            self.circuits
                .iter()
                .enumerate()
                .rev()
                .flat_map(|(circuit_index, circuit)| {
                    // Number of inputs in the next circuit.
                    let next_inputs = if circuit_index == self.circuits.len() - 1 {
                        0
                    } else {
                        // TODO store those in efficient index or at least vec
                        self.circuits[circuit_index + 1].input_names().count()
                    };
                    circuit
                        .named_outputs()
                        .skip(next_inputs)
                        .map(move |(gate, name)| (circuit_index, gate, name))
                });
        outputs
            .map(|(circuit_index, gate, name)| {
                let name = self.allocate_new_output_name(name);
                let gate = self.map_gate(circuit_index, gate);
                (gate, name)
            })
            .collect()
    }

    fn map_gate(&mut self, circuit_index: usize, gate: &Gate) -> Gate {
        for g in gate.post_visit_iter() {
            if !self
                .gate_substitutions
                .contains_key(&(circuit_index, g.id()))
            {
                let substitution = match g.operation() {
                    Operation::Variable(name) => self.map_input(circuit_index, name),
                    Operation::Constant(value) => Gate::from(*value),
                    Operation::Negation(inner) => !self.sub(circuit_index, inner),
                    Operation::Conjunction(left, right) => {
                        self.sub(circuit_index, left) & self.sub(circuit_index, right)
                    }
                    Operation::Disjunction(left, right) => {
                        self.sub(circuit_index, left) | self.sub(circuit_index, right)
                    }
                    Operation::Xor(left, right) => {
                        self.sub(circuit_index, left) ^ self.sub(circuit_index, right)
                    }
                };
                self.gate_substitutions
                    .insert((circuit_index, g.id()), substitution);
            }
        }
        self.sub(circuit_index, gate)
    }

    fn sub(&self, circuit_index: usize, gate: &Gate) -> Gate {
        // This is guaranteed to exist because we are using post visit order.
        self.gate_substitutions[&(circuit_index, gate.id())].clone()
    }

    fn map_input(&mut self, circuit_index: usize, name: &str) -> Gate {
        let name = name.to_string();
        if !self
            .input_name_substitutions
            .contains_key(&(circuit_index, name.clone()))
        {
            let substitution = if circuit_index == 0 {
                Gate::from(name.clone())
            } else {
                // TODO create lookup table
                let index = self.circuits[circuit_index]
                    .input_names()
                    .position(|n| n == name)
                    .unwrap();
                if index < self.circuits[circuit_index - 1].outputs().len() {
                    let output = &self.circuits[circuit_index - 1].outputs()[index];
                    self.map_gate(circuit_index - 1, output)
                } else {
                    self.allocate_new_input(&name)
                }
            };
            self.input_name_substitutions
                .insert((circuit_index, name.clone()), substitution);
        }
        self.input_name_substitutions[&(circuit_index, name)].clone()
    }

    fn allocate_new_input(&mut self, name_hint: &str) -> Gate {
        allocate_name(name_hint, &mut self.used_input_names).into()
    }

    fn allocate_new_output_name(&mut self, name_hint: &String) -> String {
        if name_hint.is_empty() {
            return String::new();
        }
        allocate_name(name_hint, &mut self.used_output_names)
    }
}

fn allocate_name(name_hint: &str, used_names: &mut HashSet<String>) -> String {
    let mut name = name_hint.to_string();
    let mut counter = 1;
    // TODO if we get multiple clashes, this could take quite long because we always start
    // counting from 1. Also we could remove a `_%d` suffix from the hint.
    while used_names.contains(&name) {
        name = format!("{name_hint}_{counter}");
        counter += 1;
    }
    used_names.insert(name.clone());
    name
}
