use boolean_circuit::{disjoint_union::disjoint_union, Circuit};

pub fn repeat_parallel(circuit: &Circuit, repetitions: usize) -> Circuit {
    disjoint_union(std::iter::repeat_n(circuit, repetitions))
}

pub fn repeat_interleaved(circuit: &Circuit, repetitions: usize) -> Circuit {
    let repeated_circuit = repeat_parallel(circuit, repetitions);
    let outputs = repeated_circuit.named_outputs().collect::<Vec<_>>();
    let inputs = repeated_circuit.input_names().collect::<Vec<_>>();
    Circuit::from_named_outputs(
        interleave_permutation(circuit.outputs().len(), repetitions).map(|index| {
            let (g, n) = outputs[index];
            (g.clone(), n)
        }),
    )
    .with_input_order(
        interleave_permutation(circuit.input_names().count(), repetitions)
            .map(|index| inputs[index]),
    )
    .unwrap()
}

/// Returns an iterator that generates the interleaved version of a repeated sequence.
fn interleave_permutation(items: usize, repetitions: usize) -> impl Iterator<Item = usize> {
    (0..items)
        .flat_map(move |index| (0..repetitions).map(move |repetition| index + repetition * items))
}

pub fn concatenate<'a>(circuits: impl IntoIterator<Item = &'a Circuit>) -> Circuit {
    let circuits = circuits.into_iter().collect::<Vec<_>>();
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
        let repeated_circuit = repeat_parallel(&circuit, 3);
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
        let repeated_circuit = repeat_parallel(&circuit, 0);
        assert!(repeated_circuit.outputs().is_empty());
        assert!(repeated_circuit.input_names().next().is_none());
        assert!(repeated_circuit.output_names().is_empty());
    }

    #[test]
    fn repeat_empty_circuit() {
        let repeated_circuit = repeat_parallel(&Default::default(), 3);
        assert!(repeated_circuit.outputs().is_empty());
        assert!(repeated_circuit.input_names().next().is_none());
        assert!(repeated_circuit.output_names().is_empty());
    }

    #[test]
    fn interleave_permutation_test() {
        assert_eq!(
            interleave_permutation(3, 2).collect::<Vec<_>>(),
            vec![0, 3, 1, 4, 2, 5]
        );
        assert_eq!(
            interleave_permutation(3, 1).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(
            interleave_permutation(2, 4).collect::<Vec<_>>(),
            vec![0, 2, 4, 6, 1, 3, 5, 7]
        );
    }

    #[test]
    fn repeat_interleaved_test() {
        let circuit = Circuit::from_named_outputs([
            (Gate::from("a") | !Gate::from("b"), "o1"),
            (Gate::from("a") & Gate::from("b"), "o2"),
        ])
        .with_input_order(["a", "b"])
        .unwrap();
        let repeated_circuit = repeat_interleaved(&circuit, 3);
        assert_eq!(repeated_circuit.outputs().len(), 6);
        assert_eq!(
            repeated_circuit.input_names().collect::<Vec<_>>(),
            vec!["a", "v_3", "v_7", "b", "v_4", "v_8"]
        );
        assert_eq!(
            repeated_circuit.output_names(),
            vec!["o1", "v_1", "v_5", "o2", "v_2", "v_6"]
        );

        let assignment = [true, false, true, true, false, false]
            .into_iter()
            .zip(repeated_circuit.input_names())
            .map(|(value, name)| (name.to_string(), value))
            .collect();
        let outputs = evaluate(&repeated_circuit, &assignment);
        assert_eq!(outputs, vec![true, true, true, true, false, false]);
    }

    #[test]
    fn repeat_interleaved_empty() {
        let repeated_circuit = repeat_interleaved(&Default::default(), 3);
        assert!(repeated_circuit.outputs().is_empty());
        assert!(repeated_circuit.input_names().next().is_none());
        assert!(repeated_circuit.output_names().is_empty());
    }

    #[test]
    fn repeat_interleaved_zero() {
        let circuit = Circuit::from_named_outputs([(Gate::from("a") & Gate::from("b"), "o")])
            .with_input_order(["b", "a"])
            .unwrap();
        let repeated_circuit = repeat_interleaved(&circuit, 0);
        assert!(repeated_circuit.outputs().is_empty());
        assert!(repeated_circuit.input_names().next().is_none());
        assert!(repeated_circuit.output_names().is_empty());
    }
}
