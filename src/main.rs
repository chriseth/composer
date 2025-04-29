use std::{
    io::{self, IsTerminal},
    process::ExitCode,
};

use bitmap::build_bitmap;
use boolean_circuit::{
    file_formats::aiger::{to_aiger, to_aiger_binary},
    Circuit,
};
use clap::{Parser, Subcommand};
use permutation::build_permutation;

mod bitmap;
mod permutation;

/// Tool to create and compose binary circuits in AIGER format.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a bit-mapping circuit from a lookup table given as a sequence of numbers.
    /// The length of the sequence has to be a power of two and determines the number of inputs.
    /// The magnitude of the numbers determines the number of outputs.
    ///
    /// The `i`th number in the sequence is the output for the binary representation of `i` at
    /// the inputs, where input `k` corresponds to the `k`th bit (counted from zero from the
    /// lower-order end) and similarly, output `k` computes the `k`th bit of the output
    /// (counted from zero from the lower-order end).
    ///
    /// Example: `composer bitmap 0 0 0 1` creates an AND circuit on two bits.
    Bitmap { inputs: Vec<u64> },

    /// Create a permutation circuit. The input should be a permutation on 0..n or 1..n.
    ///
    /// Example: `composer permutation 1 0 2` creates a circuit that swaps the first two bits.
    Permutation { permutation: Vec<String> },

    /// Create parallel copies of a circuit. If the circuit has `n` inputs, then
    /// the first `n` inputs of the new circuit go to the first copy of the circuit,
    /// the second `n` inputs go to the second copy of the circuit and so on.
    /// The same is true for the outputs.
    ///
    /// Example: Assuming `xor.aig` is a file containing the XOR function on two bits,
    /// then `composer repeat-parallel 16 xor.aig` creates a circuit with 32 inputs
    /// and 16 outputs such that the first and the second input are XOR-ed together,
    /// the third and fourth and so on.
    RepeatParallel {
        repetitions: usize,
        input: Option<String>,
    },

    /// Create parallel copies of a circuit where the inputs and outputs are interleaved.
    /// This means that the first input of the new circuit goes to the first input of the first
    /// circuit, the second input of the new circuit goes to the first input of the second
    /// circuit and so on. Similar for outputs.
    ///
    /// Example: Assuming `xor.aig` is a file containing the XOR function on two bits,
    /// then `composer repeat-interleaved 16 xor.aig` creates a circuit with 32 inputs
    /// and 16 outputs such that the first and the 17th input are XOR-ed together,
    /// the second and the 18th and so on.
    RepeatInterleaved {
        repetitions: usize,
        input: Option<String>,
    },

    RepeatSerial {
        repetitions: usize,
        input: Option<String>,
    },

    /// Concatenate two or more circuits serially, i.e. connects the outputs of each
    /// circuit to the inputs of the next circuit in order.
    /// Creates new inputs if the next circuit has more inputs than the previous circuit
    /// and if it has fewer inputs, creates new outputs.
    Concatenate { inputs: Vec<String> },

    /// Puts two or more circuits next to each other without establishing any connections.
    Parallel { inputs: Vec<String> },

    /// Puts two or more circuits next to each other, but interleaves the inputs and outputs.
    Interleave { inputs: Vec<String> },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Bitmap { inputs } => build_bitmap(&inputs).and_then(|c| write_aiger_to_stdout(&c)),
        Command::Permutation { permutation } => {
            build_permutation(permutation).and_then(|c| write_aiger_to_stdout(&c))
        }
        Command::RepeatParallel {
            input: _,
            repetitions: _,
        } => todo!(),
        Command::RepeatInterleaved {
            input: _,
            repetitions: _,
        } => todo!(),
        Command::RepeatSerial {
            repetitions: _,
            input: _,
        } => todo!(),
        Command::Concatenate { inputs: _ } => todo!(),
        Command::Parallel { inputs: _ } => todo!(),
        Command::Interleave { inputs: _ } => todo!(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn write_aiger_to_stdout(circuit: &Circuit) -> Result<(), String> {
    let stdout = io::stdout();
    let use_binary = !stdout.is_terminal();
    if use_binary {
        to_aiger_binary(stdout, circuit)?;
    } else {
        to_aiger(stdout, circuit)?;
    }
    Ok(())
}
