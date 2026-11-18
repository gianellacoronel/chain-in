mod common;

use chain_in::HabitStatus;
use common::{
    ERR_HABIT_ID_OVERFLOW, ERR_STREAK_OVERFLOW, PROGRAM_ID, SYSTEM_PROGRAM_ID, archive_habit_ix,
    archive_habit_ix_at, check_in_ix, check_in_ix_with_checkin, close_habit_ix, create_habit_ix,
    custom_error, daily_checkin_pda, expected_habit_len, habit_pda, initialize_profile_ix,
    overwrite_habit, overwrite_profile, pause_habit_ix_at, profile_pda, read_checkin, read_habit,
    read_profile, resume_habit_ix_at, send, serialize_account, set_unix_timestamp, setup,
    update_habit_ix, write_raw_account,
};
use litesvm::LiteSVM;
use solana_address::Address;
use solana_keypair::Keypair;
use solana_signer::Signer;

const SECONDS_PER_DAY: i64 = 86_400;
const DAY: i64 = 20_000;

fn day_timestamp(day: i64) -> i64 {
    day * SECONDS_PER_DAY + 12 * 3_600
}

fn initialize_profile(svm: &mut LiteSVM, owner: &Keypair) {
    send(
        svm,
        owner,
        &[initialize_profile_ix(&owner.pubkey())],
        &[owner],
    )
    .expect("initialize_profile should succeed");
}

fn create_habit(svm: &mut LiteSVM, owner: &Keypair, habit_id: u64, name: &str) -> Address {
    send(
        svm,
        owner,
        &[create_habit_ix(
            &owner.pubkey(),
            habit_id,
            name,
            "Test description",
        )],
        &[owner],
    )
    .expect("create_habit should succeed");

    habit_pda(&owner.pubkey(), habit_id).0
}

fn is_closed(svm: &LiteSVM, address: &Address) -> bool {
    svm.get_account(address)
        .map(|account| account.lamports == 0)
        .unwrap_or(true)
}

#[test]
fn rejects_non_owner_on_every_habit_instruction() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), 5_000_000_000)
        .expect("intruder airdrop should succeed");

    set_unix_timestamp(&mut svm, day_timestamp(DAY));
    let checkin_address = daily_checkin_pda(&habit_address, DAY).0;
    let before = read_habit(&svm, &authority, 0);

    let attempts = [
        update_habit_ix(
            &intruder.pubkey(),
            &habit_address,
            "Hijacked",
            "Hijacked description",
        ),
        pause_habit_ix_at(&intruder.pubkey(), &habit_address),
        resume_habit_ix_at(&intruder.pubkey(), &habit_address),
        archive_habit_ix_at(&intruder.pubkey(), &habit_address),
        close_habit_ix(&intruder.pubkey(), &habit_address),
        check_in_ix_with_checkin(&intruder.pubkey(), &habit_address, DAY, &checkin_address),
    ];

    for (index, instruction) in attempts.into_iter().enumerate() {
        let result = send(&mut svm, &intruder, &[instruction], &[&intruder]);
        assert!(
            result.is_err(),
            "non-owner attempt {index} must be rejected"
        );
    }

    let after = read_habit(&svm, &authority, 0);
    assert_eq!(after.name, before.name);
    assert_eq!(after.description, before.description);
    assert_eq!(after.current_streak, before.current_streak);
    assert_eq!(after.longest_streak, before.longest_streak);
    assert_eq!(after.last_check_in_day, before.last_check_in_day);
    assert!(after.status == HabitStatus::Active);

    assert!(
        svm.get_account(&checkin_address).is_none(),
        "no check-in must be created by an unauthorized signer"
    );
    assert_eq!(read_profile(&svm, &authority).next_habit_id, 1);
}

#[test]
fn rejects_invalid_habit_pda() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");
    let before = read_habit(&svm, &authority, 0);

    let fake_habit = Address::new_unique();

    let attempts = [
        update_habit_ix(&authority, &fake_habit, "Fake", "Fake description"),
        pause_habit_ix_at(&authority, &fake_habit),
        resume_habit_ix_at(&authority, &fake_habit),
        archive_habit_ix_at(&authority, &fake_habit),
        close_habit_ix(&authority, &fake_habit),
    ];

    for (index, instruction) in attempts.into_iter().enumerate() {
        let result = send(&mut svm, &owner, &[instruction], &[&owner]);
        assert!(
            result.is_err(),
            "instruction {index} with an invalid Habit PDA must be rejected"
        );
    }

    let after = read_habit(&svm, &authority, 0);
    assert_eq!(after.name, before.name);
    assert_eq!(after.description, before.description);
    assert!(after.status == HabitStatus::Active);
    assert!(!is_closed(&svm, &habit_address));
    assert!(
        svm.get_account(&fake_habit).is_none(),
        "the fabricated address must not become a habit account"
    );
}

#[test]
fn rejects_invalid_daily_check_in_pda() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let first_habit = create_habit(&mut svm, &owner, 0, "First habit");
    let second_habit = create_habit(&mut svm, &owner, 1, "Second habit");

    set_unix_timestamp(&mut svm, day_timestamp(DAY));

    // DailyCheckIn PDA derived from another habit.
    let cross_habit_checkin = daily_checkin_pda(&second_habit, DAY).0;
    let result = send(
        &mut svm,
        &owner,
        &[check_in_ix_with_checkin(
            &authority,
            &first_habit,
            DAY,
            &cross_habit_checkin,
        )],
        &[&owner],
    );
    assert!(
        result.is_err(),
        "a check-in account belonging to another habit must be rejected"
    );

    // DailyCheckIn PDA derived from a different day.
    let wrong_day_checkin = daily_checkin_pda(&first_habit, DAY - 1).0;
    let result = send(
        &mut svm,
        &owner,
        &[check_in_ix_with_checkin(
            &authority,
            &first_habit,
            DAY,
            &wrong_day_checkin,
        )],
        &[&owner],
    );
    assert!(
        result.is_err(),
        "a check-in account derived from another day must be rejected"
    );

    assert!(
        svm.get_account(&daily_checkin_pda(&first_habit, DAY).0)
            .is_none(),
        "no check-in must be stored for the first habit"
    );
    assert!(
        svm.get_account(&daily_checkin_pda(&second_habit, DAY).0)
            .is_none(),
        "no check-in must be stored for the second habit"
    );

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 0);
    assert_eq!(habit.longest_streak, 0);
    assert!(habit.last_check_in_day.is_none());
}

#[test]
fn rejects_fake_program_accounts() {
    // Profile PDA exists but is owned by the system program.
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let profile = read_profile(&svm, &authority);
    let (profile_address, _bump) = profile_pda(&authority);
    write_raw_account(
        &mut svm,
        profile_address,
        SYSTEM_PROGRAM_ID,
        serialize_account(&profile),
    );

    let result = send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, 0, "Fake profile", "Nope")],
        &[&owner],
    );
    assert!(
        result.is_err(),
        "create_habit must reject a Profile not owned by the program"
    );
    assert!(svm.get_account(&habit_pda(&authority, 0).0).is_none());

    // Habit PDA exists but is owned by the system program.
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");
    let habit = read_habit(&svm, &authority, 0);
    write_raw_account(
        &mut svm,
        habit_address,
        SYSTEM_PROGRAM_ID,
        serialize_account(&habit),
    );

    let result = send(
        &mut svm,
        &owner,
        &[update_habit_ix(&authority, &habit_address, "Nope", "Nope")],
        &[&owner],
    );
    assert!(
        result.is_err(),
        "update_habit must reject a Habit not owned by the program"
    );

    // Habit PDA is program-owned but has an invalid discriminator.
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");
    write_raw_account(
        &mut svm,
        habit_address,
        PROGRAM_ID,
        vec![0u8; expected_habit_len()],
    );

    let result = send(
        &mut svm,
        &owner,
        &[update_habit_ix(&authority, &habit_address, "Nope", "Nope")],
        &[&owner],
    );
    assert!(
        result.is_err(),
        "update_habit must reject an account with an invalid discriminator"
    );
}

#[test]
fn create_habit_rejects_habit_id_overflow() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);

    let mut profile = read_profile(&svm, &authority);
    profile.next_habit_id = u64::MAX;
    overwrite_profile(&mut svm, &profile);

    let result = send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, u64::MAX, "Overflow", "Overflow")],
        &[&owner],
    );
    assert_eq!(
        custom_error(&result),
        Some(ERR_HABIT_ID_OVERFLOW),
        "next_habit_id overflow must be rejected"
    );

    let profile = read_profile(&svm, &authority);
    assert_eq!(profile.next_habit_id, u64::MAX);
    assert!(
        svm.get_account(&habit_pda(&authority, u64::MAX).0)
            .is_none(),
        "no habit must be created when the ID overflows"
    );
}

#[test]
fn check_in_handles_streak_overflow_and_boundary() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    // u32::MAX + 1 must fail without corrupting the habit.
    let mut maxed = read_habit(&svm, &authority, 0);
    maxed.current_streak = u32::MAX;
    maxed.longest_streak = u32::MAX;
    maxed.last_check_in_day = Some(DAY);
    overwrite_habit(&mut svm, &maxed);

    set_unix_timestamp(&mut svm, day_timestamp(DAY + 1));
    let result = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY + 1)],
        &[&owner],
    );
    assert_eq!(
        custom_error(&result),
        Some(ERR_STREAK_OVERFLOW),
        "streak overflow must be rejected"
    );
    assert!(
        svm.get_account(&daily_checkin_pda(&habit_address, DAY + 1).0)
            .is_none()
    );

    let still_maxed = read_habit(&svm, &authority, 0);
    assert_eq!(still_maxed.current_streak, u32::MAX);
    assert_eq!(still_maxed.longest_streak, u32::MAX);
    assert_eq!(still_maxed.last_check_in_day, Some(DAY));

    // u32::MAX - 1 + 1 is a valid boundary and must succeed.
    let mut boundary = maxed;
    boundary.current_streak = u32::MAX - 1;
    boundary.longest_streak = u32::MAX - 1;
    overwrite_habit(&mut svm, &boundary);

    // New blockhash: the previous attempt had the same message/signature.
    svm.expire_blockhash();
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY + 1)],
        &[&owner],
    )
    .expect("check-in at the streak boundary should succeed");

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, u32::MAX);
    assert_eq!(habit.longest_streak, u32::MAX);
    assert_eq!(habit.last_check_in_day, Some(DAY + 1));

    let checkin = read_checkin(&svm, &habit_address, DAY + 1);
    assert_eq!(checkin.day, DAY + 1);
    assert_eq!(checkin.checked_at, day_timestamp(DAY + 1));
}

#[test]
fn accepts_maximum_length_inputs() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);

    // 20 x "ñ" = 40 UTF-8 bytes; 80 x "é" = 160 UTF-8 bytes.
    let name = "ñ".repeat(20);
    let description = "é".repeat(80);
    assert_eq!(name.len(), 40);
    assert_eq!(description.len(), 160);

    send(
        &mut svm,
        &owner,
        &[create_habit_ix(&authority, 0, &name, &description)],
        &[&owner],
    )
    .expect("inputs at the maximum byte length should be accepted");

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.name, name);
    assert_eq!(habit.description, description);
    assert_eq!(habit.name.len(), 40);
    assert_eq!(habit.description.len(), 160);

    let updated_name = "a".repeat(40);
    let updated_description = "b".repeat(160);
    let habit_address = habit_pda(&authority, 0).0;
    send(
        &mut svm,
        &owner,
        &[update_habit_ix(
            &authority,
            &habit_address,
            &updated_name,
            &updated_description,
        )],
        &[&owner],
    )
    .expect("update at the maximum byte length should be accepted");

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.name.len(), 40);
    assert_eq!(habit.description.len(), 160);
    assert_eq!(habit.name, updated_name);
    assert_eq!(habit.description, updated_description);
}

#[test]
fn close_rent_cannot_be_redirected() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("archive_habit should succeed");

    let habit_lamports = svm.get_account(&habit_address).unwrap().lamports;
    let owner_before = svm.get_balance(&authority).unwrap();

    let intruder = Keypair::new();
    svm.airdrop(&intruder.pubkey(), 5_000_000_000)
        .expect("intruder airdrop should succeed");

    let result = send(
        &mut svm,
        &intruder,
        &[close_habit_ix(&intruder.pubkey(), &habit_address)],
        &[&intruder],
    );
    assert!(
        result.is_err(),
        "a non-owner must not be able to close the habit"
    );
    assert_eq!(
        svm.get_balance(&authority).unwrap(),
        owner_before,
        "an unauthorized close must not move rent to the owner"
    );
    assert_eq!(
        svm.get_account(&habit_address).unwrap().lamports,
        habit_lamports,
        "an unauthorized close must not drain the habit account"
    );

    let metadata = send(
        &mut svm,
        &owner,
        &[close_habit_ix(&authority, &habit_address)],
        &[&owner],
    )
    .expect("the owner should be able to close the archived habit");

    assert_eq!(
        svm.get_balance(&authority).unwrap(),
        owner_before + habit_lamports - metadata.fee,
        "the habit rent must be returned only to its owner"
    );
    assert!(is_closed(&svm, &habit_address));

    // A closed habit cannot be closed again.
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[close_habit_ix(&authority, &habit_address)],
        &[&owner],
    );
    assert!(result.is_err(), "a closed habit must not be closable twice");
    assert!(is_closed(&svm, &habit_address));
}
