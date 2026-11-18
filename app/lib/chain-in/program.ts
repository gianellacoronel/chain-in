import { address, getUtf8Encoder, type Address } from "@solana/kit";

/**
 * Chain-In Anchor program, deployed on devnet.
 *
 * Keep in sync with `anchor/Anchor.toml` and
 * `app/lib/chain-in/idl/chain_in.json`.
 */
export const CHAIN_IN_PROGRAM_ADDRESS: Address = address(
  "Eai82xNfih16jrTcQktnKaAYWG9hkvUuQ6uRGPtDsWwM"
);

const seed = (value: string) => getUtf8Encoder().encode(value);

/** PDA seeds, matching `constants.rs` in the Anchor program. */
export const PROFILE_SEED = seed("profile");
export const HABIT_SEED = seed("habit");
export const CHECKIN_SEED = seed("checkin");

/** Size of the Anchor account discriminator prepended to every account. */
export const ACCOUNT_DISCRIMINATOR_SIZE = 8;

/** Account discriminators, copied from `idl/chain_in.json`. */
export const PROFILE_DISCRIMINATOR = new Uint8Array([
  184, 101, 165, 188, 95, 63, 127, 188,
]);

export const HABIT_DISCRIMINATOR = new Uint8Array([
  210, 33, 243, 232, 216, 99, 36, 217,
]);

export const DAILY_CHECK_IN_DISCRIMINATOR = new Uint8Array([
  156, 67, 142, 102, 165, 5, 90, 22,
]);

/**
 * Allocated account sizes, including the discriminator. They match the
 * `InitSpace` values from `state.rs`.
 */
export const PROFILE_ACCOUNT_SIZE = ACCOUNT_DISCRIMINATOR_SIZE + 32 + 8 + 8;

export const HABIT_ACCOUNT_SIZE =
  ACCOUNT_DISCRIMINATOR_SIZE +
  32 +
  8 +
  (4 + 40) +
  (4 + 160) +
  1 +
  4 +
  4 +
  (1 + 8) +
  8;

export const DAILY_CHECK_IN_ACCOUNT_SIZE =
  ACCOUNT_DISCRIMINATOR_SIZE + 32 + 32 + 8 + (4 + 40) + 8 + 8;

/** Maximum UTF-8 byte lengths enforced by the program. */
export const MAX_HABIT_NAME_LEN = 40;
export const MAX_HABIT_DESCRIPTION_LEN = 160;

/** Seconds in a UTC day, used to derive the on-chain check-in day index. */
export const SECONDS_PER_DAY = 86_400n;

/** Instruction discriminators, copied from `idl/chain_in.json`. */
export const INITIALIZE_PROFILE_DISCRIMINATOR = new Uint8Array([
  32, 145, 77, 213, 58, 39, 251, 234,
]);

export const CREATE_HABIT_DISCRIMINATOR = new Uint8Array([
  6, 233, 240, 164, 87, 155, 14, 8,
]);

export const UPDATE_HABIT_DISCRIMINATOR = new Uint8Array([
  127, 103, 133, 37, 116, 2, 76, 47,
]);

export const CHECK_IN_DISCRIMINATOR = new Uint8Array([
  209, 253, 4, 217, 250, 241, 207, 50,
]);

export const PAUSE_HABIT_DISCRIMINATOR = new Uint8Array([
  236, 30, 189, 208, 10, 128, 67, 107,
]);

export const RESUME_HABIT_DISCRIMINATOR = new Uint8Array([
  173, 166, 56, 139, 227, 97, 152, 172,
]);

export const ARCHIVE_HABIT_DISCRIMINATOR = new Uint8Array([
  79, 105, 54, 246, 225, 229, 147, 123,
]);

export const CLOSE_HABIT_DISCRIMINATOR = new Uint8Array([
  239, 79, 40, 126, 253, 236, 213, 185,
]);
