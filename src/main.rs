use std::{fs::File, io::Seek};

use clap::{Parser, Subcommand};

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
    /// The `i`th number in the sequence is the output for the binary representation of `i` at the inputs.
    ///
    /// Example: `composer bitmap 0 0 0 1` creates an AND circuit on two bits.
    Bitmap { inputs: Vec<u64> },

    /// Create a permutation circuit. The input should be a permutation on 0..n or 1..n.
    ///
    /// Example: `composer permutation 1 0 2` creates a circuit that swaps the first two bits.
    Permutation { permutation: Vec<String> },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Bitmap { inputs } => {
            unimplemented!();
        }
        Command::Permutation { permutation } => {
            unimplemented!();
        }
    }
}
