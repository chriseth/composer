use boolean_circuit::{disjoint_union::disjoint_union, Circuit};

pub fn repeat_parallel(circuit: Circuit, repetitions: usize) -> Circuit {
    disjoint_union(std::iter::repeat_n(&circuit, repetitions))
}

#[cfg(test)]
mod test {
    use super::*;
    use boolean_circuit::{evaluate, Gate};

    #[test]
    fn repeat_parallel_test() {
        let circuit = Circuit::from_named_outputs([(Gate::from("a") & Gate::from("b"), "o")])
            .with_input_order(["b", "a"])
            .unwrap();
        let repeated_circuit = repeat_parallel(circuit, 3);
        assert_eq!(repeated_circuit.outputs().len(), 3);
        assert_eq!(
            repeated_circuit.input_names().collect::<Vec<_>>(),
            vec!["b", "a", "v_3", "v_2", "v_6", "v_5"]
        );
        assert_eq!(
            repeated_circuit.output_names(),
            vec!["o".to_string(), "v_1".to_string(), "v_4".to_string()]
        );

        let assignment = [true, false, true, true, false, false]
            .into_iter()
            .zip(repeated_circuit.input_names())
            .map(|(value, name)| (name.to_string(), value))
            .collect();
        let outputs = evaluate(&repeated_circuit, &assignment);
        assert_eq!(outputs, vec![false, true, false]);
    }

    #[test]
    fn repeat_parallel_zero() {
        let circuit = Circuit::from_named_outputs([(Gate::from("a") & Gate::from("b"), "o")])
            .with_input_order(["b", "a"])
            .unwrap();
        let repeated_circuit = repeat_parallel(circuit, 0);
        assert!(repeated_circuit.outputs().is_empty());
        assert!(repeated_circuit.input_names().next().is_none());
        assert!(repeated_circuit.output_names().is_empty());
    }
}
