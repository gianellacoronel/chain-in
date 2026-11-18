//! Shared LiteSVM harness for the Chain-In integration tests.
//!
//! Tests must be run after `anchor build` (or `cargo build-sbf`), because the
//! compiled program is embedded at compile time from `target/deploy/chain_in.so`.

#![allow(dead_code)]

use anchor_lang::{AccountDeserialize, Space};
use chain_in::{PROFILE_SEED, Profile};
use litesvm::LiteSVM;
use sha2::{Digest, Sha256};
use solana_address::{Address, address};
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub const PROGRAM_ID: Address = address!("Eai82xNfih16jrTcQktnKaAYWG9hkvUuQ6uRGPtDsWwM");

const SYSTEM_PROGRAM_ID: Address = Address::new_from_array([0u8; 32]);

const PROGRAM_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/chain_in.so"));

/// Creates a fresh LiteSVM instance with the program loaded and a funded payer.
pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    svm.add_program(PROGRAM_ID, PROGRAM_BYTES)
        .expect("program should load into LiteSVM");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000)
        .expect("payer airdrop should succeed");

    (svm, payer)
}

/// Derives the Profile PDA for a wallet using the documented seeds.
pub fn profile_pda(authority: &Address) -> (Address, u8) {
    Address::find_program_address(&[PROFILE_SEED, authority.as_ref()], &PROGRAM_ID)
}

/// Computes an Anchor instruction discriminator: `sha256("global:<name>")[..8]`.
pub fn anchor_discriminator(name: &str) -> [u8; 8] {
    let digest = Sha256::digest(format!("global:{name}").as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&digest[..8]);
    discriminator
}

/// Builds the `initialize_profile` instruction with its required accounts.
pub fn initialize_profile_ix(authority: &Address) -> Instruction {
    let (profile, _bump) = profile_pda(authority);

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(profile, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: anchor_discriminator("initialize_profile").to_vec(),
    }
}

/// Signs and sends a transaction through LiteSVM.
pub fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instructions: &[Instruction],
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    let blockhash = svm.latest_blockhash();
    let message = Message::new_with_blockhash(instructions, Some(&payer.pubkey()), &blockhash);
    let tx = Transaction::new(signers, message, blockhash);
    svm.send_transaction(tx)
}

/// Reads and deserializes the Profile PDA for a wallet.
pub fn read_profile(svm: &LiteSVM, authority: &Address) -> Profile {
    let (profile_pda, _bump) = profile_pda(authority);
    let account = svm
        .get_account(&profile_pda)
        .expect("profile account should exist");

    let mut data = account.data.as_slice();
    Profile::try_deserialize(&mut data).expect("profile account should deserialize")
}

/// Returns the expected serialized Profile length, including the discriminator.
pub fn expected_profile_len() -> usize {
    8 + Profile::INIT_SPACE
}

/// Overrides the LiteSVM Clock sysvar with a deterministic timestamp.
pub fn set_unix_timestamp(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar(&clock);
}
