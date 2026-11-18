mod common;

use anchor_lang::Space;
use chain_in::Profile;
use common::{
    PROGRAM_ID, expected_profile_len, initialize_profile_ix, profile_pda, read_profile, send,
    set_unix_timestamp, setup,
};
use solana_keypair::Keypair;
use solana_signer::Signer;

/// Deterministic timestamp used to assert `Profile.created_at`.
const TEST_TIMESTAMP: i64 = 1_700_000_000;

#[test]
fn initializes_profile_successfully() {
    let (mut svm, payer) = setup();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);

    let authority = payer.pubkey();
    let ix = initialize_profile_ix(&authority);

    let result = send(&mut svm, &payer, &[ix], &[&payer]);
    assert!(
        result.is_ok(),
        "initialize_profile should succeed: {:?}",
        result.err()
    );

    let (profile_address, _bump) = profile_pda(&authority);
    let account = svm
        .get_account(&profile_address)
        .expect("profile account should exist at the derived PDA");

    assert_eq!(account.owner, PROGRAM_ID);
    assert_eq!(account.data.len(), expected_profile_len());
    assert!(
        account.lamports >= svm.minimum_balance_for_rent_exemption(account.data.len()),
        "profile account should be rent exempt"
    );
}

#[test]
fn stores_correct_pda_and_fields() {
    let (mut svm, payer) = setup();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);

    let authority = payer.pubkey();

    // Derive the PDA independently using the documented seeds.
    let (expected_pda, _bump) =
        solana_address::Address::find_program_address(&[b"profile", authority.as_ref()], &PROGRAM_ID);
    assert_ne!(expected_pda, authority, "profile PDA must differ from the wallet");

    send(
        &mut svm,
        &payer,
        &[initialize_profile_ix(&authority)],
        &[&payer],
    )
    .expect("initialize_profile should succeed");

    assert!(
        svm.get_account(&expected_pda).is_some(),
        "profile must be stored at [b\"profile\", authority]"
    );

    let profile = read_profile(&svm, &authority);
    assert_eq!(profile.authority.to_bytes(), authority.to_bytes());
    assert_eq!(profile.next_habit_id, 0);
    assert_eq!(profile.created_at, TEST_TIMESTAMP);
    assert_eq!(Profile::INIT_SPACE, 48);
}

#[test]
fn rejects_duplicate_initialization() {
    let (mut svm, payer) = setup();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);

    let authority = payer.pubkey();
    send(
        &mut svm,
        &payer,
        &[initialize_profile_ix(&authority)],
        &[&payer],
    )
    .expect("first initialize_profile should succeed");

    let (profile_address, _bump) = profile_pda(&authority);
    let before = read_profile(&svm, &authority);
    let before_lamports = svm.get_account(&profile_address).unwrap().lamports;

    // Change the blockhash so the second transaction is not deduplicated.
    svm.expire_blockhash();

    let result = send(
        &mut svm,
        &payer,
        &[initialize_profile_ix(&authority)],
        &[&payer],
    );
    assert!(
        result.is_err(),
        "a second initialize_profile for the same wallet must fail"
    );

    let after = read_profile(&svm, &authority);
    assert_eq!(after.authority.to_bytes(), before.authority.to_bytes());
    assert_eq!(after.next_habit_id, before.next_habit_id);
    assert_eq!(after.created_at, before.created_at);

    let account = svm.get_account(&profile_address).unwrap();
    assert_eq!(account.owner, PROGRAM_ID);
    assert_eq!(account.lamports, before_lamports);
}

#[test]
fn creates_independent_profiles_per_wallet() {
    let (mut svm, payer) = setup();
    let other = Keypair::new();
    svm.airdrop(&other.pubkey(), 5_000_000_000)
        .expect("second wallet airdrop should succeed");

    let authority_a = payer.pubkey();
    let authority_b = other.pubkey();

    set_unix_timestamp(&mut svm, TEST_TIMESTAMP);
    send(
        &mut svm,
        &payer,
        &[initialize_profile_ix(&authority_a)],
        &[&payer],
    )
    .expect("profile A should initialize");

    // Distinct creation timestamps make sure each profile stores its own value.
    svm.expire_blockhash();
    set_unix_timestamp(&mut svm, TEST_TIMESTAMP + 1_000);
    send(
        &mut svm,
        &other,
        &[initialize_profile_ix(&authority_b)],
        &[&other],
    )
    .expect("profile B should initialize");

    let (pda_a, _bump_a) = profile_pda(&authority_a);
    let (pda_b, _bump_b) = profile_pda(&authority_b);
    assert_ne!(pda_a, pda_b, "each wallet must get a distinct profile PDA");

    let profile_a = read_profile(&svm, &authority_a);
    let profile_b = read_profile(&svm, &authority_b);

    assert_eq!(profile_a.authority.to_bytes(), authority_a.to_bytes());
    assert_eq!(profile_b.authority.to_bytes(), authority_b.to_bytes());

    assert_eq!(profile_a.next_habit_id, 0);
    assert_eq!(profile_b.next_habit_id, 0);

    assert_eq!(profile_a.created_at, TEST_TIMESTAMP);
    assert_eq!(profile_b.created_at, TEST_TIMESTAMP + 1_000);
}
