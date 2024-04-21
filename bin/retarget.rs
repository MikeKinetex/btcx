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
use btcx::retarget::VerifyWithRetargetCircuit;
use plonky2x::backend::function::Plonky2xFunction;

fn main() {
    let headers_count = std::env::var("UPDATE_HEADERS_COUNT")
        .unwrap()
        .parse()
        .unwrap();

    match headers_count {
        10 => VerifyWithRetargetCircuit::<10>::entrypoint(),
        18 => VerifyWithRetargetCircuit::<18>::entrypoint(),
        36 => VerifyWithRetargetCircuit::<36>::entrypoint(),
        72 => VerifyWithRetargetCircuit::<72>::entrypoint(),
        144 => VerifyWithRetargetCircuit::<144>::entrypoint(),
        288 => VerifyWithRetargetCircuit::<288>::entrypoint(),
        576 => VerifyWithRetargetCircuit::<576>::entrypoint(),
        1008 => VerifyWithRetargetCircuit::<1008>::entrypoint(),
        2016 => VerifyWithRetargetCircuit::<2016>::entrypoint(),
        _ => panic!("Unsupported headers count"),
    }
}
