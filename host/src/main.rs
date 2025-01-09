use json_core::Outputs;
use std::io::Read;
// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{RDF_CONTAINS_GUEST_ELF, RDF_CONTAINS_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde_json;

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // An executor environment describes the configurations for the zkVM
    // including program inputs.
    // A default ExecutorEnv can be created like so:
    // `let env = ExecutorEnv::builder().build().unwrap();`
    // However, this `env` does not have any inputs.
    //
    // To add guest input to the executor environment, use
    // ExecutorEnvBuilder::write().
    // To access this method, you'll need to use ExecutorEnv::builder(), which
    // creates an ExecutorEnvBuilder. When you're done adding input, call
    // ExecutorEnvBuilder::build().

    let mut file =
        std::fs::File::open("res/windsurf.nq").expect("Example file should be accessible");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Should not have I/O errors");

    // For example:
    // let input: u32 = 15 * u32::pow(2, 27) + 1;
    let env = ExecutorEnv::builder()
        .write(&data).unwrap()
        .write(&"CONSTRUCT WHERE { ?s ?p ?o . }").unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Proof information by proving the specified ELF binary.
    // This struct contains the receipt along with statistics about execution of the guest
    let prove_info = prover.prove(env, RDF_CONTAINS_GUEST_ELF).unwrap();

    // extract the receipt.
    let receipt = prove_info.receipt;

    // TODO: Implement code for retrieving receipt journal here.

    // For example:
    let outputs: Outputs = receipt.journal.decode().unwrap();

    println!("\nThe JSON file with hash\n  {:?}\nprovably contains a field 'critical_data' with value {}\n", hex::encode(outputs.data), hex::encode(outputs.query));
    // The receipt was verified at the end of proving, but the below code is an
    // example of how someone else could verify this receipt.
    receipt.verify(RDF_CONTAINS_GUEST_ID).unwrap();

    // Serialise the receipt
    let receipt_json = serde_json::to_string(&receipt).unwrap();
    println!("Receipt: {}", receipt_json);
    std::fs::write("receipt.json", receipt_json).expect("Unable to write file");
}
