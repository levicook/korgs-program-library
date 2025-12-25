use std::{io, path::PathBuf};

use litesvm::{error::LiteSVMError, LiteSVM};
use solana_keypair::{Keypair, Signer};

fn program_path() -> io::Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/sbpf-solana-solana/release/pinocchio_counter.so")
        .canonicalize()
}

#[test]
fn test_pinocchio_counter() -> Result<(), LiteSVMError> {
    let mut litesvm = LiteSVM::new();

    let program_kp = Keypair::new();
    let program_id = program_kp.pubkey();

    litesvm.add_program_from_file(program_id, program_path()?)?;

    Ok(())
}
