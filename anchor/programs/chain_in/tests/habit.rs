mod common;

use chain_in::HabitStatus;
use common::{
    ERR_HABIT_DESCRIPTION_TOO_LONG, ERR_HABIT_NAME_TOO_LONG, PROGRAM_ID, create_habit_ix,
    create_habit_ix_with_profile, custom_error, expected_habit_len, habit_pda,
    initialize_profile_ix, profile_pda, read_habit, read_profile, send, set_unix_timestamp, setup,
    update_habit_ix,
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

/// Deterministic timestamp used to assert `Habit.created_at`.
const TEST_TIMESTAMP: i64 = 1_700_000_000;

fn initialize_profile(svm: &mut LiteSVM, authority: &Keypair) {
    send(
        svm,
        authority,
        &[initialize_profile_ix(&authority.pubkey())],
        &[authority],
    )
    .expect("initialize_profile should succeed");
}

#[test]
fn creates_habit_successfully() {
    let (mut svm, owner) = setup();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);
    initialize_profile(&mut svm, &owner);

    let authority = owner.pubkey();
    let result = send(
        &mut svm,
        &owner,
        &[create_habit_ix(
            &authority,
            0,
            "Morning run",
            "Run 5km before breakfast",
        )],
        &[&owner],
    );
    assert!(
        result.is_ok(),
        "create_habit should succeed: {:?}",
        result.err()
    );

    let (habit_address, _bump) = habit_pda(&authority, 0);
    let account = svm
        .get_account(&habit_address)
        .expect("habit account should exist at the derived PDA");

    assert_eq!(account.owner, PROGRAM_ID);
    assert_eq!(account.data.len(), expected_habit_len());
    assert!(
        account.lamports >= svm.minimum_balance_for_rent_exemption(account.data.len()),
        "habit account should be rent exempt"
    );

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.owner.to_bytes(), authority.to_bytes());
    assert_eq!(habit.id, 0);
    assert_eq!(habit.name, "Morning run");
    assert_eq!(habit.description, "Run 5km before breakfast");
    assert!(habit.status == HabitStatus::Active);
    assert_eq!(habit.current_streak, 0);
    assert_eq!(habit.longest_streak, 0);
    assert!(
        habit.last_check_in_day.is_none(),
        "new habits must not have a check-in day"
    );
    assert_eq!(habit.created_at, TEST_TIMESTAMP);

    let profile = read_profile(&svm, &authority);
    assert_eq!(profile.next_habit_id, 1);
}

#[test]
fn assigns_sequential_habit_ids() {
    let (mut svm, owner) = setup();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);
    initialize_profile(&mut svm, &owner);

    let authority = owner.pubkey();
    let names = ["Habit A", "Habit B", "Habit C"];

    for (expected_id, name) in names.iter().enumerate() {
        let profile = read_profile(&svm, &authority);
        assert_eq!(profile.next_habit_id, expected_id as u64);

        send(
            &mut svm,
            &owner,
            &[create_habit_ix(&authority, expected_id as u64, name, "Description")],
            &[&owner],
        )
        .expect("create_habit should succeed");

        let habit = read_habit(&svm, &authority, expected_id as u64);
        assert_eq!(habit.id, expected_id as u64);
        assert_eq!(habit.name, *name);
    }

    let profile = read_profile(&svm, &authority);
    assert_eq!(profile.next_habit_id, 3);

    let mut addresses = Vec::new();
    for habit_id in 0..3u64 {
        let (address, _bump) = habit_pda(&authority, habit_id);
        addresses.push(address);
    }
    addresses.sort();
    addresses.dedup();
    assert_eq!(addresses.len(), 3, "each habit must have a unique PDA");
}

#[test]
fn rejects_create_habit_with_invalid_owner() {
    let (mut svm, wallet_a) = setup();
    let wallet_b = Keypair::new();
    svm.airdrop(&wallet_b.pubkey(), 5_000_000_000)
        .expect("second wallet airdrop should succeed");

    initialize_profile(&mut svm, &wallet_a);

    // Wallet B cannot create a habit against wallet A's Profile.
    let (profile_a, _bump) = profile_pda(&wallet_a.pubkey());
    let result = send(
        &mut svm,
        &wallet_b,
        &[create_habit_ix_with_profile(
            &wallet_b.pubkey(),
            &profile_a,
            0,
            "Intruder habit",
            "Should be rejected",
        )],
        &[&wallet_b],
    );
    assert!(
        result.is_err(),
        "create_habit must reject a Profile owned by another wallet"
    );

    // Wallet B cannot create a habit without its own initialized Profile either.
    let result = send(
        &mut svm,
        &wallet_b,
        &[create_habit_ix(
            &wallet_b.pubkey(),
            0,
            "No profile",
            "Should be rejected",
        )],
        &[&wallet_b],
    );
    assert!(
        result.is_err(),
        "create_habit must require an initialized Profile"
    );

    let (habit_b, _bump) = habit_pda(&wallet_b.pubkey(), 0);
    assert!(
        svm.get_account(&habit_b).is_none(),
        "no habit should be created for an invalid owner"
    );
    assert_eq!(read_profile(&svm, &wallet_a.pubkey()).next_habit_id, 0);
}

#[test]
fn rejects_create_habit_with_oversized_inputs() {
    let (mut svm, owner) = setup();
    initialize_profile(&mut svm, &owner);

    let authority = owner.pubkey();

    // 21 chars but 42 UTF-8 bytes: the limit is measured in bytes.
    let long_name = "ñ".repeat(21);
    assert_eq!(long_name.chars().count(), 21);
    assert_eq!(long_name.len(), 42);

    let result = send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, 0, &long_name, "Valid description")],
        &[&owner],
    );
    assert_eq!(
        custom_error(&result),
        Some(ERR_HABIT_NAME_TOO_LONG),
        "oversized name must be rejected"
    );
    assert!(
        svm.get_account(&habit_pda(&authority, 0).0).is_none(),
        "failed creation must not leave an account behind"
    );
    assert_eq!(read_profile(&svm, &authority).next_habit_id, 0);

    let long_description = "d".repeat(161);
    let result = send(
        &mut svm,
        &owner,
        &[create_habit_ix(
            &authority,
            0,
            "Valid name",
            &long_description,
        )],
        &[&owner],
    );
    assert_eq!(
        custom_error(&result),
        Some(ERR_HABIT_DESCRIPTION_TOO_LONG),
        "oversized description must be rejected"
    );
    assert!(
        svm.get_account(&habit_pda(&authority, 0).0).is_none(),
        "failed creation must not leave an account behind"
    );
    assert_eq!(read_profile(&svm, &authority).next_habit_id, 0);
}

#[test]
fn updates_habit_successfully_preserving_identity() {
    let (mut svm, owner) = setup();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);
    initialize_profile(&mut svm, &owner);

    let authority = owner.pubkey();
    send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, 0, "Old name", "Old description")],
        &[&owner],
    )
    .expect("create_habit should succeed");

    let (habit_address, _bump) = habit_pda(&authority, 0);
    send(
        &mut svm,
        &owner,
        &[update_habit_ix(
            &authority,
            &habit_address,
            "New name",
            "New description",
        )],
        &[&owner],
    )
    .expect("update_habit should succeed for the owner");

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.name, "New name");
    assert_eq!(habit.description, "New description");
    assert_eq!(habit.owner.to_bytes(), authority.to_bytes());
    assert_eq!(habit.id, 0);
    assert!(habit.status == HabitStatus::Active);
    assert_eq!(habit.created_at, TEST_TIMESTAMP);
}

#[test]
fn rejects_update_habit_from_non_owner() {
    let (mut svm, owner) = setup();
    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), 5_000_000_000)
        .expect("intruder airdrop should succeed");

    initialize_profile(&mut svm, &owner);
    let authority = owner.pubkey();

    send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, 0, "Original", "Original description")],
        &[&owner],
    )
    .expect("create_habit should succeed");

    let (habit_address, _bump) = habit_pda(&authority, 0);
    let result = send(
        &mut svm,
        &intruder,
        &[update_habit_ix(
            &intruder.pubkey(),
            &habit_address,
            "Hijacked",
            "Hijacked description",
        )],
        &[&intruder],
    );
    assert!(
        result.is_err(),
        "update_habit must reject a signer that is not the habit owner"
    );

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.name, "Original");
    assert_eq!(habit.description, "Original description");
    assert_eq!(habit.owner.to_bytes(), authority.to_bytes());
}

#[test]
fn rejects_update_habit_with_oversized_inputs() {
    let (mut svm, owner) = setup();
    initialize_profile(&mut svm, &owner);

    let authority = owner.pubkey();
    send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, 0, "Original", "Original description")],
        &[&owner],
    )
    .expect("create_habit should succeed");

    let (habit_address, _bump) = habit_pda(&authority, 0);

    let long_name = "n".repeat(41);
    let result = send(
        &mut svm,
        &owner,
        &[update_habit_ix(
            &authority,
            &habit_address,
            &long_name,
            "Valid description",
        )],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NAME_TOO_LONG));

    let long_description = "d".repeat(161);
    let result = send(
        &mut svm,
        &owner,
        &[update_habit_ix(
            &authority,
            &habit_address,
            "Valid name",
            &long_description,
        )],
        &[&owner],
    );
    assert_eq!(
        custom_error(&result),
        Some(ERR_HABIT_DESCRIPTION_TOO_LONG)
    );

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.name, "Original");
    assert_eq!(habit.description, "Original description");
}
