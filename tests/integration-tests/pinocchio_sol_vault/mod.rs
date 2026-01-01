use {
    crate::clock_utils::{advance_clock, ClockAdvanceError},
    litesvm::{error::LiteSVMError, types::TransactionResult, LiteSVM},
    solana_account::Account,
    solana_clock::Clock,
    solana_hash::Hash,
    solana_keypair::{Keypair, Signer},
    solana_pubkey::Pubkey,
    solana_transaction::versioned::VersionedTransaction,
    std::path::PathBuf,
};

pub mod deactivate_vault_v1;
pub mod deposit_v1;
pub mod initialize_vault_v1;
pub mod malicious_builders;
pub mod reactivate_vault_v1;
pub mod withdraw_v1;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct TestContext {
    svm: LiteSVM,
    program_id: Pubkey,
}

impl TestContext {
    /// Creates a new test context with a loaded program.
    ///
    /// # Errors
    ///
    /// Returns an error if the program file cannot be found or loaded.
    pub fn try_new() -> Result<Self, LiteSVMError> {
        let mut litesvm = LiteSVM::new();
        let program_id = Pubkey::new_unique();

        let program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/sbpf-solana-solana/release/pinocchio_sol_vault_program.so")
            .canonicalize()?;

        litesvm.add_program_from_file(program_id, program_path)?;

        Ok(Self {
            svm: litesvm,
            program_id,
        })
    }

    pub fn advance_slot(&mut self, n_slots: u64) -> Result<(), ClockAdvanceError> {
        let current_clock = self.svm.get_sysvar::<Clock>();
        let new_clock = advance_clock(&current_clock, n_slots)?;
        self.svm.set_sysvar(&new_clock);
        self.svm.expire_blockhash();
        Ok(())
    }

    /// Airdrops lamports to the specified account.
    ///
    /// # Errors
    ///
    /// Returns an error if the airdrop transaction fails.
    #[allow(clippy::result_large_err)] // TransactionResult from litesvm has large error variant
    pub fn airdrop_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> TransactionResult {
        self.svm.airdrop(&pubkey, lamports)
    }

    /// Creates a new keypair and funds it with 1 SOL.
    ///
    /// # Panics
    ///
    /// Panics if the airdrop fails.
    pub fn create_funded_keypair(&mut self) -> Keypair {
        let kp = Keypair::new();
        self.airdrop_lamports(kp.pubkey(), 1_000_000_000)
            .expect("Failed to airdrop lamports"); // :(
        kp
    }

    #[must_use]
    pub fn get_account(&self, pubkey: Pubkey) -> Option<Account> {
        self.svm.get_account(&pubkey)
    }

    pub fn set_account(&mut self, pubkey: Pubkey, data: Account) -> Result<(), LiteSVMError> {
        self.svm.set_account(pubkey, data)
    }

    #[must_use]
    pub fn latest_blockhash(&self) -> Hash {
        self.svm.latest_blockhash()
    }

    #[must_use]
    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    /// Sends a transaction to the SVM.
    ///
    /// # Errors
    ///
    /// Returns an error if the transaction fails.
    #[allow(clippy::result_large_err)] // TransactionResult from litesvm has large error variant
    pub fn send_transaction(&mut self, tx: VersionedTransaction) -> TransactionResult {
        self.svm.send_transaction(tx)
    }
}
