use boolean_circuit::{
    builder::{reduce_conjunction, reduce_disjunction},
    Circuit, Gate,
};
use itertools::Itertools;

pub fn build_bitmap(inputs: &[u64]) -> Result<Circuit, String> {
    let (input_bits, output_bits) = validate_inputs(inputs)?;
    let input_gates = (0..input_bits)
        .map(|i| Gate::from(format!("i{i}")))
        .collect_vec();
    let output_gates = (0..output_bits).map(|bit| {
        reduce_disjunction(
            inputs
                .iter()
                .enumerate()
                // All indices in the inputs where this output bit is set to one.
                .filter(|(_, v)| (*v & (1 << bit)) != 0)
                .map(|(index, _)| {
                    reduce_conjunction((0..input_bits).map(|input_bit| {
                        // big endian
                        let input_gate = input_gates[input_bit].clone();
                        if (index & (1 << input_bit)) != 0 {
                            input_gate
                        } else {
                            !input_gate
                        }
                    }))
                }),
        )
    });
    Ok(Circuit::from_unnamed_outputs(output_gates))
}

/// Validates the inputs and returns the number of inputs and outputs needed.
fn validate_inputs(inputs: &[u64]) -> Result<(usize, usize), String> {
    if inputs.is_empty() {
        return Ok((0, 0));
    }
    let input_bits = inputs.len().ilog2() as usize;
    assert!(input_bits < 64);
    if 1u64 << input_bits != inputs.len() as u64 {
        return Err(format!(
            "Expected a power of two as number of inputs, but got {} inputs",
            inputs.len()
        ));
    }
    let largest_output = *inputs.iter().max().unwrap();
    if largest_output == 0 {
        return Ok((input_bits, 0));
    }
    let output_bits = largest_output.ilog2() as usize;
    Ok(if (1u64 << output_bits) > largest_output {
        (input_bits, output_bits)
    } else {
        (input_bits, output_bits + 1)
    })
}

#[cfg(test)]
mod test {
    use boolean_circuit::evaluate;

    use super::*;

    #[test]
    fn input_output_bits() {
        assert_eq!(validate_inputs(&[]), Ok((0, 0)));
        assert_eq!(validate_inputs(&[1]), Ok((0, 1)));
        assert_eq!(
            validate_inputs(&[1, 1, 1]),
            Err("Expected a power of two as number of inputs, but got 3 inputs".to_string())
        );
        assert_eq!(validate_inputs(&[0, 0]), Ok((1, 0)));
        assert_eq!(validate_inputs(&[0, 1, 1, 1]), Ok((2, 1)));
        assert_eq!(validate_inputs(&[0, 1, 2, 1]), Ok((2, 2)));
        assert_eq!(validate_inputs(&[0, 1, 2, 3]), Ok((2, 2)));
        assert_eq!(validate_inputs(&[0, 4, 2, 3]), Ok((2, 3)));
    }

    fn test_evaluate(inputs: &[u64]) {
        let circuit = build_bitmap(inputs).unwrap();
        let (input_bits, output_bits) = validate_inputs(inputs).unwrap();
        for (index, output) in inputs.iter().enumerate() {
            let assignments = (0..input_bits)
                .map(|bit| (format!("i{bit}"), index & (1 << bit) != 0))
                .collect();
            let evaluated = evaluate(&circuit, &assignments);
            let expected = (0..output_bits)
                .map(|bit| output & (1 << bit) != 0)
                .collect_vec();
            if evaluated != expected {
                assert_eq!(
                    evaluated, expected,
                    "Invalid output for input {index} - expected {output}"
                );
            }
        }
    }

    #[test]
    fn build() {
        test_evaluate(&[]);
        test_evaluate(&[0, 0]);
        test_evaluate(&[0, 1]);
        test_evaluate(&[1, 0]);
        test_evaluate(&[1, 1]);
        test_evaluate(&[1, 7]);
        test_evaluate(&[0, 0, 0, 1]);
        test_evaluate(&[0, 1, 0, 1]);
        test_evaluate(&[0, 0, 1, 1]);
        test_evaluate(&[0, 0, 0, 1]);
        test_evaluate(&[0, 3, 0, 1]);
    }
}
