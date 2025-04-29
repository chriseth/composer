use boolean_circuit::Circuit;

pub fn build_permutation(permutation: Vec<String>) -> Result<Circuit, String> {
    let permutation = validate_permutation(permutation)?;
    Ok(Circuit::from_unnamed_outputs(
        permutation.map(|x| format!("i{x}").into()),
    ))
}

/// Validates the input permutation and returns a vector of values between zero
/// and n-1, where n is the length of the permutation.
fn validate_permutation(permutation: Vec<String>) -> Result<impl Iterator<Item = u64>, String> {
    let permutation = permutation
        .into_iter()
        .map(|s| {
            s.parse::<u64>().map_err(|e| {
                format!(
                "Error parsing permutation element '{s}', expected list of unsigned integers but got error: {e}"
            )
            })
        })
        .collect::<Result<Vec<u64>, _>>()?;
    let contains_zero = permutation.iter().any(|&x| x == 0);
    let occurrence_count =
        permutation
            .iter()
            .try_fold(vec![0; permutation.len()], |mut acc, &x| {
                let index = if contains_zero {
                    x as usize
                } else {
                    x as usize - 1
                };
                if index >= acc.len() {
                    return Err(format!(
                        "Number '{x}' is too large (only {} items provided)",
                        acc.len()
                    ));
                }
                if acc[index] > 0 {
                    return Err(format!("Number '{x}' occurs more than once."));
                }
                acc[index] += 1;
                Ok(acc)
            })?;
    assert!(occurrence_count.iter().all(|&x| x >= 1));

    Ok(permutation
        .into_iter()
        .map(move |x| if contains_zero { x } else { x - 1 }))
}

#[cfg(test)]
mod test {
    use boolean_circuit::evaluate;

    use super::*;

    fn string_to_permutation_input(permutation: &str) -> Vec<String> {
        permutation
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    fn test_validate_err(permutation: &str, expectation: &str) {
        match validate_permutation(string_to_permutation_input(permutation)) {
            Ok(_) => panic!("Expected error but got Ok"),
            Err(e) => assert_eq!(e, expectation),
        }
    }

    fn test_validate_ok(permutation: &str) {
        match validate_permutation(string_to_permutation_input(permutation)) {
            Ok(_) => {}
            Err(e) => panic!("Expected Ok but got error: {e}"),
        }
    }

    #[test]
    fn validate_empty() {
        test_validate_ok("");
    }

    #[test]
    fn validate_no_number() {
        test_validate_err("7 x", "Error parsing permutation element 'x', expected list of unsigned integers but got error: invalid digit found in string");
        test_validate_err("7x", "Error parsing permutation element '7x', expected list of unsigned integers but got error: invalid digit found in string");
        test_validate_err("2 x8", "Error parsing permutation element 'x8', expected list of unsigned integers but got error: invalid digit found in string");
    }

    #[test]
    fn validate_too_large() {
        test_validate_ok("0");
        test_validate_ok("1");
        test_validate_err("2", "Number '2' is too large (only 1 items provided)");
        test_validate_ok("0 1");
        test_validate_ok("2 1");
        test_validate_err("0 3 1", "Number '3' is too large (only 3 items provided)");
        test_validate_err("4 3 2", "Number '4' is too large (only 3 items provided)");
    }

    #[test]
    fn validate_multiple() {
        test_validate_ok("0 1");
        test_validate_err("1 1", "Number '1' occurs more than once.");
        test_validate_err("2 2", "Number '2' occurs more than once.");
        test_validate_err("1 2 3 2", "Number '2' occurs more than once.");
        test_validate_err("1 4 4 3 3", "Number '4' occurs more than once.");
    }

    #[test]
    fn evaluate_permutation() {
        // identity
        let circuit = build_permutation(vec!["0".to_string(), "1".to_string()]).unwrap();
        let output = evaluate(
            &circuit,
            &[("i0".to_string(), true), ("i1".to_string(), false)]
                .into_iter()
                .collect(),
        );
        assert_eq!(&output, &vec![true, false]);

        // swap
        let circuit = build_permutation(vec!["2".to_string(), "1".to_string()]).unwrap();
        let output = evaluate(
            &circuit,
            &[("i0".to_string(), true), ("i1".to_string(), false)]
                .into_iter()
                .collect(),
        );
        assert_eq!(&output, &vec![false, true]);
    }
}
