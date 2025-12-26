use {
    crate::clock_utils::advance_clock,
    litesvm::{error::LiteSVMError, types::TransactionResult, LiteSVM},
    solana_account::Account,
    solana_clock::Clock,
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_pubkey::Pubkey,
    solana_transaction::versioned::VersionedTransaction,
    std::path::PathBuf,
};

pub mod deactivate_counter_v1;
pub mod initialize_counter_v1;
pub mod malicious_builders;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct TestContext {
    svm: LiteSVM,
    program_id: Pubkey,
}

impl TestContext {
    pub fn try_new() -> Result<Self, LiteSVMError> {
        let mut litesvm = LiteSVM::new();
        let program_id = Pubkey::new_unique();

        let program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/sbpf-solana-solana/release/pinocchio_counter_program.so")
            .canonicalize()?;

        litesvm.add_program_from_file(program_id, program_path)?;

        Ok(Self {
            svm: litesvm,
            program_id,
        })
    }

    pub fn advance_slot(&mut self, n_slots: u64) {
        let current_clock = self.svm.get_sysvar::<Clock>();
        let new_clock = advance_clock(current_clock, n_slots);
        self.svm.set_sysvar(&new_clock);
        self.svm.expire_blockhash();
    }

    pub fn airdrop_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> TransactionResult {
        self.svm.airdrop(&pubkey, lamports)
    }

    pub fn create_funded_keypair(&mut self) -> Keypair {
        let kp = Keypair::new();
        self.airdrop_lamports(kp.pubkey(), 1_000_000_000)
            .expect("Failed to airdrop lamports"); // :(
        kp
    }

    pub fn get_account(&self, pubkey: Pubkey) -> Option<Account> {
        self.svm.get_account(&pubkey)
    }

    pub fn latest_blockhash(&self) -> Hash {
        self.svm.latest_blockhash()
    }

    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    pub fn send_transaction(&mut self, tx: impl Into<VersionedTransaction>) -> TransactionResult {
        self.svm.send_transaction(tx)
    }
}
