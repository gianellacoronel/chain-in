//! Shared LiteSVM harness for the Chain-In integration tests.
//!
//! Tests must be run after `anchor build` (or `cargo build-sbf`), because the
//! compiled program is embedded at compile time from `target/deploy/chain_in.so`.

#![allow(dead_code)]

use anchor_lang::{AccountDeserialize, Space};
use chain_in::{CHECKIN_SEED, DailyCheckIn, HABIT_SEED, PROFILE_SEED, Habit, Profile};
use litesvm::LiteSVM;
use sha2::{Digest, Sha256};
use solana_address::{Address, address};
use solana_clock::Clock;
use solana_instruction::{AccountMeta, Instruction, error::InstructionError};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;

pub const PROGRAM_ID: Address = address!("Eai82xNfih16jrTcQktnKaAYWG9hkvUuQ6uRGPtDsWwM");

/// Anchor custom error codes for `ErrorCode`, starting at the 6000 offset.
pub const ERR_UNAUTHORIZED_PROFILE: u32 = 6000;
pub const ERR_UNAUTHORIZED_HABIT: u32 = 6001;
pub const ERR_HABIT_NAME_TOO_LONG: u32 = 6002;
pub const ERR_HABIT_DESCRIPTION_TOO_LONG: u32 = 6003;
pub const ERR_HABIT_NOT_ACTIVE: u32 = 6005;
pub const ERR_INVALID_CHECK_IN_DAY: u32 = 6006;

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

/// Derives the Habit PDA for an owner and sequential habit ID.
pub fn habit_pda(owner: &Address, habit_id: u64) -> (Address, u8) {
    Address::find_program_address(
        &[HABIT_SEED, owner.as_ref(), &habit_id.to_le_bytes()],
        &PROGRAM_ID,
    )
}

/// Derives the DailyCheckIn PDA for a habit and UTC day index.
pub fn daily_checkin_pda(habit: &Address, day: i64) -> (Address, u8) {
    Address::find_program_address(
        &[CHECKIN_SEED, habit.as_ref(), &day.to_le_bytes()],
        &PROGRAM_ID,
    )
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

/// Borsh-encodes a `String`: u32 little-endian byte length followed by UTF-8 bytes.
pub fn borsh_string(value: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(4 + value.len());
    encoded.extend_from_slice(&(value.len() as u32).to_le_bytes());
    encoded.extend_from_slice(value.as_bytes());
    encoded
}

/// Builds `create_habit` using the signer's own Profile PDA.
pub fn create_habit_ix(
    owner: &Address,
    habit_id: u64,
    name: &str,
    description: &str,
) -> Instruction {
    let (profile, _bump) = profile_pda(owner);
    create_habit_ix_with_profile(owner, &profile, habit_id, name, description)
}

/// Builds `create_habit` with an explicit Profile account, used for negative tests.
pub fn create_habit_ix_with_profile(
    owner: &Address,
    profile: &Address,
    habit_id: u64,
    name: &str,
    description: &str,
) -> Instruction {
    let (habit, _bump) = habit_pda(owner, habit_id);

    let mut data = anchor_discriminator("create_habit").to_vec();
    data.extend(borsh_string(name));
    data.extend(borsh_string(description));

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*profile, false),
            AccountMeta::new(habit, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

/// Builds `update_habit` for a given signer and habit account.
pub fn update_habit_ix(
    signer: &Address,
    habit: &Address,
    name: &str,
    description: &str,
) -> Instruction {
    let mut data = anchor_discriminator("update_habit").to_vec();
    data.extend(borsh_string(name));
    data.extend(borsh_string(description));

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*signer, true),
            AccountMeta::new(*habit, false),
        ],
        data,
    }
}

/// Builds `check_in` for a habit on a specific UTC day index.
pub fn check_in_ix(owner: &Address, habit_id: u64, day: i64) -> Instruction {
    let (habit, _bump) = habit_pda(owner, habit_id);
    let (daily_check_in, _bump) = daily_checkin_pda(&habit, day);

    let mut data = anchor_discriminator("check_in").to_vec();
    data.extend_from_slice(&day.to_le_bytes());

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(habit, false),
            AccountMeta::new(daily_check_in, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

/// Builds `pause_habit` for the habit's owner.
pub fn pause_habit_ix(owner: &Address, habit_id: u64) -> Instruction {
    let (habit, _bump) = habit_pda(owner, habit_id);

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(habit, false),
        ],
        data: anchor_discriminator("pause_habit").to_vec(),
    }
}

/// Builds `archive_habit` for the habit's owner.
pub fn archive_habit_ix(owner: &Address, habit_id: u64) -> Instruction {
    let (habit, _bump) = habit_pda(owner, habit_id);

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(habit, false),
        ],
        data: anchor_discriminator("archive_habit").to_vec(),
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

/// Reads and deserializes a Habit PDA.
pub fn read_habit(svm: &LiteSVM, owner: &Address, habit_id: u64) -> Habit {
    let (habit_pda, _bump) = habit_pda(owner, habit_id);
    let account = svm
        .get_account(&habit_pda)
        .expect("habit account should exist");

    let mut data = account.data.as_slice();
    Habit::try_deserialize(&mut data).expect("habit account should deserialize")
}

/// Returns the expected serialized Habit length, including the discriminator.
pub fn expected_habit_len() -> usize {
    8 + Habit::INIT_SPACE
}

/// Reads and deserializes a DailyCheckIn PDA.
pub fn read_checkin(svm: &LiteSVM, habit: &Address, day: i64) -> DailyCheckIn {
    let (checkin_pda, _bump) = daily_checkin_pda(habit, day);
    let account = svm
        .get_account(&checkin_pda)
        .expect("daily check-in account should exist");

    let mut data = account.data.as_slice();
    DailyCheckIn::try_deserialize(&mut data).expect("daily check-in account should deserialize")
}

/// Returns the expected serialized DailyCheckIn length, including the discriminator.
pub fn expected_checkin_len() -> usize {
    8 + DailyCheckIn::INIT_SPACE
}

/// Extracts an Anchor custom error code from a failed transaction, if present.
pub fn custom_error(result: &litesvm::types::TransactionResult) -> Option<u32> {
    match result {
        Ok(_) => None,
        Err(failed) => match &failed.err {
            TransactionError::InstructionError(_, InstructionError::Custom(code)) => Some(*code),
            _ => None,
        },
    }
}

/// Overrides the LiteSVM Clock sysvar with a deterministic timestamp.
pub fn set_unix_timestamp(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar(&clock);
}
