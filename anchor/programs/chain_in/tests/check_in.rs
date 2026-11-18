mod common;

use chain_in::HabitStatus;
use common::{
    ERR_HABIT_NOT_ACTIVE, ERR_INVALID_CHECK_IN_DAY, PROGRAM_ID, archive_habit_ix, check_in_ix,
    create_habit_ix, custom_error, daily_checkin_pda, expected_checkin_len, habit_pda,
    initialize_profile_ix, pause_habit_ix, read_checkin, read_habit, send, set_unix_timestamp,
    setup,
};
use litesvm::LiteSVM;
use solana_address::Address;
use solana_keypair::Keypair;
use solana_signer::Signer;

const SECONDS_PER_DAY: i64 = 86_400;
const DAY: i64 = 20_000;

/// Noon timestamp for a UTC day index, safely inside the day.
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
        &[create_habit_ix(&owner.pubkey(), habit_id, name, "Test description")],
        &[owner],
    )
    .expect("create_habit should succeed");

    habit_pda(&owner.pubkey(), habit_id).0
}

fn check_in_on(svm: &mut LiteSVM, owner: &Keypair, habit_id: u64, day: i64) {
    set_unix_timestamp(svm, day_timestamp(day));
    send(
        svm,
        owner,
        &[check_in_ix(&owner.pubkey(), habit_id, day)],
        &[owner],
    )
    .expect("check_in should succeed");
}

#[test]
fn records_successful_check_in() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    let checked_at = day_timestamp(DAY);
    set_unix_timestamp(&mut svm, checked_at);

    let result = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    );
    assert!(result.is_ok(), "check_in should succeed: {:?}", result.err());

    let (checkin_address, _bump) = daily_checkin_pda(&habit_address, DAY);
    let account = svm
        .get_account(&checkin_address)
        .expect("daily check-in account should exist at the derived PDA");

    assert_eq!(account.owner, PROGRAM_ID);
    assert_eq!(account.data.len(), expected_checkin_len());
    assert!(
        account.lamports >= svm.minimum_balance_for_rent_exemption(account.data.len()),
        "daily check-in account should be rent exempt"
    );

    let checkin = read_checkin(&svm, &habit_address, DAY);
    assert_eq!(checkin.owner.to_bytes(), authority.to_bytes());
    assert_eq!(checkin.habit.to_bytes(), habit_address.to_bytes());
    assert_eq!(checkin.habit_id, 0);
    assert_eq!(checkin.habit_name_snapshot, "Meditate");
    assert_eq!(checkin.day, DAY);
    assert_eq!(checkin.checked_at, checked_at);

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 1);
    assert_eq!(habit.longest_streak, 1);
    assert_eq!(habit.last_check_in_day, Some(DAY));
}

#[test]
fn rejects_duplicate_check_in_for_same_day() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    let checked_at = day_timestamp(DAY);
    set_unix_timestamp(&mut svm, checked_at);
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    )
    .expect("first check_in should succeed");

    let habit_before = read_habit(&svm, &authority, 0);
    let checkin_before = read_checkin(&svm, &habit_address, DAY);

    // Same UTC day but a later timestamp; change the blockhash to avoid dedupe.
    svm.expire_blockhash();
    set_unix_timestamp(&mut svm, checked_at + 3_600);

    let result = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    );
    assert!(
        result.is_err(),
        "a second check-in on the same UTC day must fail"
    );

    let habit_after = read_habit(&svm, &authority, 0);
    assert_eq!(habit_after.current_streak, habit_before.current_streak);
    assert_eq!(habit_after.longest_streak, habit_before.longest_streak);
    assert_eq!(habit_after.last_check_in_day, habit_before.last_check_in_day);

    let checkin_after = read_checkin(&svm, &habit_address, DAY);
    assert_eq!(
        checkin_after.checked_at, checkin_before.checked_at,
        "the original check-in must remain unchanged"
    );
}

#[test]
fn rejects_check_in_for_inactive_habit() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let paused_habit = create_habit(&mut svm, &owner, 0, "Paused habit");
    let archived_habit = create_habit(&mut svm, &owner, 1, "Archived habit");

    send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("pause_habit should succeed");
    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 1)],
        &[&owner],
    )
    .expect("archive_habit should succeed");

    set_unix_timestamp(&mut svm, day_timestamp(DAY));

    let paused_result = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    );
    assert_eq!(
        custom_error(&paused_result),
        Some(ERR_HABIT_NOT_ACTIVE),
        "paused habits must reject check-ins"
    );

    let archived_result = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 1, DAY)],
        &[&owner],
    );
    assert_eq!(
        custom_error(&archived_result),
        Some(ERR_HABIT_NOT_ACTIVE),
        "archived habits must reject check-ins"
    );

    assert!(
        svm.get_account(&daily_checkin_pda(&paused_habit, DAY).0)
            .is_none(),
        "no check-in should be stored for a paused habit"
    );
    assert!(
        svm.get_account(&daily_checkin_pda(&archived_habit, DAY).0)
            .is_none(),
        "no check-in should be stored for an archived habit"
    );

    let paused = read_habit(&svm, &authority, 0);
    assert_eq!(paused.current_streak, 0);
    assert_eq!(paused.longest_streak, 0);
    assert!(paused.last_check_in_day.is_none());
    assert!(paused.status == HabitStatus::Paused);

    let archived = read_habit(&svm, &authority, 1);
    assert_eq!(archived.current_streak, 0);
    assert!(archived.status == HabitStatus::Archived);
}

#[test]
fn rejects_check_in_with_mismatched_utc_day() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    set_unix_timestamp(&mut svm, day_timestamp(DAY));

    let previous_day = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY - 1)],
        &[&owner],
    );
    assert_eq!(
        custom_error(&previous_day),
        Some(ERR_INVALID_CHECK_IN_DAY),
        "the day must match the current UTC day"
    );

    let next_day = send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY + 1)],
        &[&owner],
    );
    assert_eq!(
        custom_error(&next_day),
        Some(ERR_INVALID_CHECK_IN_DAY),
        "the day must match the current UTC day"
    );

    assert!(svm.get_account(&daily_checkin_pda(&habit_address, DAY).0).is_none());
    assert!(svm
        .get_account(&daily_checkin_pda(&habit_address, DAY - 1).0)
        .is_none());
    assert!(svm
        .get_account(&daily_checkin_pda(&habit_address, DAY + 1).0)
        .is_none());

    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 0);
    assert!(habit.last_check_in_day.is_none());
}

#[test]
fn accepts_check_ins_across_utc_day_boundaries() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let start_habit = create_habit(&mut svm, &owner, 0, "Start of day");
    let end_habit = create_habit(&mut svm, &owner, 1, "End of day");

    let day_start = DAY * SECONDS_PER_DAY;
    let day_end = day_start + SECONDS_PER_DAY - 1;

    set_unix_timestamp(&mut svm, day_start);
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    )
    .expect("check-in at 00:00:00 UTC should succeed");

    set_unix_timestamp(&mut svm, day_end);
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 1, DAY)],
        &[&owner],
    )
    .expect("check-in at 23:59:59 UTC should succeed");

    let start_checkin = read_checkin(&svm, &start_habit, DAY);
    assert_eq!(start_checkin.day, DAY);
    assert_eq!(start_checkin.checked_at, day_start);

    let end_checkin = read_checkin(&svm, &end_habit, DAY);
    assert_eq!(end_checkin.day, DAY);
    assert_eq!(end_checkin.checked_at, day_end);
}

#[test]
fn calculates_current_streak_and_resets_after_missed_day() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    create_habit(&mut svm, &owner, 0, "Meditate");

    check_in_on(&mut svm, &owner, 0, DAY);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 1);
    assert_eq!(habit.longest_streak, 1);

    check_in_on(&mut svm, &owner, 0, DAY + 1);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 2);
    assert_eq!(habit.longest_streak, 2);
    assert_eq!(habit.last_check_in_day, Some(DAY + 1));

    check_in_on(&mut svm, &owner, 0, DAY + 2);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 3);
    assert_eq!(habit.longest_streak, 3);
    assert_eq!(habit.last_check_in_day, Some(DAY + 2));

    // DAY + 3 is intentionally skipped.
    check_in_on(&mut svm, &owner, 0, DAY + 4);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(
        habit.current_streak, 1,
        "a missed day must reset the current streak"
    );
    assert_eq!(
        habit.longest_streak, 3,
        "the longest streak must be preserved"
    );
    assert_eq!(habit.last_check_in_day, Some(DAY + 4));

    check_in_on(&mut svm, &owner, 0, DAY + 5);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 2);
    assert_eq!(habit.longest_streak, 3);
}

#[test]
fn preserves_longest_streak_and_updates_it_when_exceeded() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    create_habit(&mut svm, &owner, 0, "Meditate");

    // Build a 3-day streak.
    for offset in 0..3 {
        check_in_on(&mut svm, &owner, 0, DAY + offset);
        let habit = read_habit(&svm, &authority, 0);
        assert_eq!(habit.longest_streak, (offset + 1) as u32);
    }

    // Skip DAY + 3 and check in: current resets, longest stays at 3.
    check_in_on(&mut svm, &owner, 0, DAY + 4);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 1);
    assert_eq!(habit.longest_streak, 3);

    // Build a new streak long enough to exceed the previous record.
    check_in_on(&mut svm, &owner, 0, DAY + 5);
    assert_eq!(read_habit(&svm, &authority, 0).longest_streak, 3);

    check_in_on(&mut svm, &owner, 0, DAY + 6);
    assert_eq!(read_habit(&svm, &authority, 0).longest_streak, 3);

    check_in_on(&mut svm, &owner, 0, DAY + 7);
    let habit = read_habit(&svm, &authority, 0);
    assert_eq!(habit.current_streak, 4);
    assert_eq!(
        habit.longest_streak, 4,
        "longest streak must increase once the record is exceeded"
    );
}
