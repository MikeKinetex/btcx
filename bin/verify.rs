//! To build the binary:
//!
//!     `cargo build --release --bin step`
//!
//! To build the circuit:
//!
//!     `./target/release/circuit_function_field build`
//!
//! To prove the circuit using evm io:
//!
//!    `./target/release/circuit_function_evm prove --input-json src/bin/circuit_function_evm_input.json`
//!
//! Note that this circuit will not work with field-based io.
//!
//!
//!
use btcx::consts::UPDATE_HEADERS_COUNT;
use btcx::verify::VerifyCircuit;
use plonky2x::backend::function::Plonky2xFunction;

fn main() {
    VerifyCircuit::<UPDATE_HEADERS_COUNT>::entrypoint();
}
