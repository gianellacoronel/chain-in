mod common;

use chain_in::HabitStatus;
use common::{
    ERR_HABIT_NOT_ARCHIVABLE, ERR_HABIT_NOT_CLOSABLE, ERR_HABIT_NOT_PAUSEABLE,
    ERR_HABIT_NOT_RESUMABLE, archive_habit_ix, check_in_ix, close_habit_ix, create_habit_ix,
    custom_error, daily_checkin_pda, habit_pda, initialize_profile_ix, pause_habit_ix, read_checkin,
    read_habit, resume_habit_ix, send, set_unix_timestamp, setup,
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

/// True when the account no longer holds lamports (closed or purged).
fn is_closed(svm: &LiteSVM, address: &Address) -> bool {
    svm.get_account(address)
        .map(|account| account.lamports == 0)
        .unwrap_or(true)
}

#[test]
fn pauses_and_resumes_habit() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    create_habit(&mut svm, &owner, 0, "Meditate");

    set_unix_timestamp(&mut svm, day_timestamp(DAY));
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    )
    .expect("check_in should succeed");

    let before = read_habit(&svm, &authority, 0);
    assert_eq!(before.current_streak, 1);

    send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("pause_habit should succeed");

    let paused = read_habit(&svm, &authority, 0);
    assert!(paused.status == HabitStatus::Paused);
    assert_eq!(paused.current_streak, before.current_streak);
    assert_eq!(paused.longest_streak, before.longest_streak);
    assert_eq!(paused.last_check_in_day, before.last_check_in_day);
    assert_eq!(paused.name, before.name);

    send(
        &mut svm,
        &owner,
        &[resume_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("resume_habit should succeed");

    let resumed = read_habit(&svm, &authority, 0);
    assert!(resumed.status == HabitStatus::Active);
    assert_eq!(resumed.current_streak, before.current_streak);
    assert_eq!(resumed.longest_streak, before.longest_streak);
    assert_eq!(resumed.last_check_in_day, before.last_check_in_day);
}

#[test]
fn rejects_invalid_status_transitions() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    create_habit(&mut svm, &owner, 0, "Meditate");

    // Active cannot be resumed.
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[resume_habit_ix(&authority, 0)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_RESUMABLE));
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Active);

    // Active can be paused.
    svm.expire_blockhash();
    send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("pause_habit should succeed");
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Paused);

    // Paused cannot be paused twice.
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 0)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_PAUSEABLE));
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Paused);

    // Paused can be resumed.
    svm.expire_blockhash();
    send(
        &mut svm,
        &owner,
        &[resume_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("resume_habit should succeed");
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Active);

    // Active can be archived.
    svm.expire_blockhash();
    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("archive_habit should succeed");
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Archived);

    // Archived is terminal: no pause.
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 0)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_PAUSEABLE));
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Archived);

    // Archived is terminal: no resume.
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[resume_habit_ix(&authority, 0)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_RESUMABLE));
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Archived);

    // Archived cannot be archived again.
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 0)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_ARCHIVABLE));
    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Archived);
}

#[test]
fn archives_active_and_paused_habits() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    create_habit(&mut svm, &owner, 0, "Active habit");
    create_habit(&mut svm, &owner, 1, "Paused habit");

    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("archiving an Active habit should succeed");

    send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 1)],
        &[&owner],
    )
    .expect("pause_habit should succeed");
    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 1)],
        &[&owner],
    )
    .expect("archiving a Paused habit should succeed");

    assert!(read_habit(&svm, &authority, 0).status == HabitStatus::Archived);
    assert!(read_habit(&svm, &authority, 1).status == HabitStatus::Archived);
}

#[test]
fn closes_only_archived_habits_for_the_owner() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    // Active habits cannot be closed.
    let result = send(
        &mut svm,
        &owner,
        &[close_habit_ix(&authority, &habit_address)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_CLOSABLE));
    assert!(!is_closed(&svm, &habit_address));

    // Paused habits cannot be closed either.
    send(
        &mut svm,
        &owner,
        &[pause_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("pause_habit should succeed");
    svm.expire_blockhash();
    let result = send(
        &mut svm,
        &owner,
        &[close_habit_ix(&authority, &habit_address)],
        &[&owner],
    );
    assert_eq!(custom_error(&result), Some(ERR_HABIT_NOT_CLOSABLE));
    assert!(!is_closed(&svm, &habit_address));

    // Archive and attempt to close with a non-owner signer.
    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("archive_habit should succeed");

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
        "close_habit must reject a signer that is not the habit owner"
    );
    assert!(!is_closed(&svm, &habit_address));

    // The owner can close the archived habit and receives its rent.
    let habit_lamports = svm.get_account(&habit_address).unwrap().lamports;
    let owner_before = svm.get_balance(&authority).unwrap();

    svm.expire_blockhash();
    let metadata = send(
        &mut svm,
        &owner,
        &[close_habit_ix(&authority, &habit_address)],
        &[&owner],
    )
    .expect("close_habit should succeed for the owner of an Archived habit");

    let owner_after = svm.get_balance(&authority).unwrap();
    assert_eq!(
        owner_after,
        owner_before + habit_lamports - metadata.fee,
        "rent must be returned to the habit owner"
    );
    assert!(is_closed(&svm, &habit_address));
}

#[test]
fn closing_habit_preserves_check_in_history() {
    let (mut svm, owner) = setup();
    let authority = owner.pubkey();
    initialize_profile(&mut svm, &owner);
    let habit_address = create_habit(&mut svm, &owner, 0, "Meditate");

    set_unix_timestamp(&mut svm, day_timestamp(DAY));
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY)],
        &[&owner],
    )
    .expect("first check_in should succeed");

    set_unix_timestamp(&mut svm, day_timestamp(DAY + 1));
    send(
        &mut svm,
        &owner,
        &[check_in_ix(&authority, 0, DAY + 1)],
        &[&owner],
    )
    .expect("second check_in should succeed");

    let first_before = read_checkin(&svm, &habit_address, DAY);
    let second_before = read_checkin(&svm, &habit_address, DAY + 1);
    let first_lamports = svm
        .get_account(&daily_checkin_pda(&habit_address, DAY).0)
        .unwrap()
        .lamports;
    let second_lamports = svm
        .get_account(&daily_checkin_pda(&habit_address, DAY + 1).0)
        .unwrap()
        .lamports;

    send(
        &mut svm,
        &owner,
        &[archive_habit_ix(&authority, 0)],
        &[&owner],
    )
    .expect("archive_habit should succeed");
    send(
        &mut svm,
        &owner,
        &[close_habit_ix(&authority, &habit_address)],
        &[&owner],
    )
    .expect("close_habit should succeed");

    assert!(
        is_closed(&svm, &habit_address),
        "the habit account must be closed"
    );

    let first_after = read_checkin(&svm, &habit_address, DAY);
    let second_after = read_checkin(&svm, &habit_address, DAY + 1);

    assert_eq!(first_after.owner.to_bytes(), first_before.owner.to_bytes());
    assert_eq!(first_after.habit.to_bytes(), first_before.habit.to_bytes());
    assert_eq!(first_after.habit_id, first_before.habit_id);
    assert_eq!(
        first_after.habit_name_snapshot,
        first_before.habit_name_snapshot
    );
    assert_eq!(first_after.day, first_before.day);
    assert_eq!(first_after.checked_at, first_before.checked_at);

    assert_eq!(second_after.owner.to_bytes(), second_before.owner.to_bytes());
    assert_eq!(second_after.habit.to_bytes(), second_before.habit.to_bytes());
    assert_eq!(second_after.habit_id, second_before.habit_id);
    assert_eq!(
        second_after.habit_name_snapshot,
        second_before.habit_name_snapshot
    );
    assert_eq!(second_after.day, second_before.day);
    assert_eq!(second_after.checked_at, second_before.checked_at);

    assert_eq!(
        svm.get_account(&daily_checkin_pda(&habit_address, DAY).0)
            .unwrap()
            .lamports,
        first_lamports,
        "check-in rent must remain untouched"
    );
    assert_eq!(
        svm.get_account(&daily_checkin_pda(&habit_address, DAY + 1).0)
            .unwrap()
            .lamports,
        second_lamports,
        "check-in rent must remain untouched"
    );
}
