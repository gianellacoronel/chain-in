import { AccountRole, type Address, type Instruction } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import {
  concatBytes,
  encodeCheckInArgs,
  encodeCreateHabitArgs,
  encodeUpdateHabitArgs,
} from "./codecs";
import {
  ARCHIVE_HABIT_DISCRIMINATOR,
  CHAIN_IN_PROGRAM_ADDRESS,
  CHECK_IN_DISCRIMINATOR,
  CLOSE_HABIT_DISCRIMINATOR,
  CREATE_HABIT_DISCRIMINATOR,
  INITIALIZE_PROFILE_DISCRIMINATOR,
  PAUSE_HABIT_DISCRIMINATOR,
  RESUME_HABIT_DISCRIMINATOR,
  UPDATE_HABIT_DISCRIMINATOR,
} from "./program";

const writableSigner = AccountRole.WRITABLE_SIGNER;
const readonlySigner = AccountRole.READONLY_SIGNER;
const writable = AccountRole.WRITABLE;
const readonly = AccountRole.READONLY;

/** Accounts for `initialize_profile`. */
export type InitializeProfileAccounts = {
  authority: Address;
  profile: Address;
};

/** Builds the `initialize_profile` instruction. */
export function getInitializeProfileInstruction({
  authority,
  profile,
}: InitializeProfileAccounts): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: authority, role: writableSigner },
      { address: profile, role: writable },
      { address: SYSTEM_PROGRAM_ADDRESS, role: readonly },
    ],
    data: INITIALIZE_PROFILE_DISCRIMINATOR,
  };
}

/** Accounts and arguments for `create_habit`. */
export type CreateHabitInput = {
  owner: Address;
  profile: Address;
  habit: Address;
  name: string;
  description: string;
};

/** Builds the `create_habit` instruction. */
export function getCreateHabitInstruction({
  owner,
  profile,
  habit,
  name,
  description,
}: CreateHabitInput): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: writableSigner },
      { address: profile, role: writable },
      { address: habit, role: writable },
      { address: SYSTEM_PROGRAM_ADDRESS, role: readonly },
    ],
    data: concatBytes(
      CREATE_HABIT_DISCRIMINATOR,
      encodeCreateHabitArgs({ name, description })
    ),
  };
}

/** Accounts and arguments for `update_habit`. */
export type UpdateHabitInput = {
  owner: Address;
  habit: Address;
  name: string;
  description: string;
};

/** Builds the `update_habit` instruction. */
export function getUpdateHabitInstruction({
  owner,
  habit,
  name,
  description,
}: UpdateHabitInput): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: readonlySigner },
      { address: habit, role: writable },
    ],
    data: concatBytes(
      UPDATE_HABIT_DISCRIMINATOR,
      encodeUpdateHabitArgs({ name, description })
    ),
  };
}

/** Accounts and arguments for `check_in`. */
export type CheckInInput = {
  owner: Address;
  habit: Address;
  dailyCheckIn: Address;
  day: bigint;
};

/** Builds the `check_in` instruction. */
export function getCheckInInstruction({
  owner,
  habit,
  dailyCheckIn,
  day,
}: CheckInInput): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: writableSigner },
      { address: habit, role: writable },
      { address: dailyCheckIn, role: writable },
      { address: SYSTEM_PROGRAM_ADDRESS, role: readonly },
    ],
    data: concatBytes(CHECK_IN_DISCRIMINATOR, encodeCheckInArgs(day)),
  };
}

/** Accounts for the habit lifecycle instructions. */
export type HabitLifecycleAccounts = {
  owner: Address;
  habit: Address;
};

/** Builds the `pause_habit` instruction. */
export function getPauseHabitInstruction({
  owner,
  habit,
}: HabitLifecycleAccounts): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: readonlySigner },
      { address: habit, role: writable },
    ],
    data: PAUSE_HABIT_DISCRIMINATOR,
  };
}

/** Builds the `resume_habit` instruction. */
export function getResumeHabitInstruction({
  owner,
  habit,
}: HabitLifecycleAccounts): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: readonlySigner },
      { address: habit, role: writable },
    ],
    data: RESUME_HABIT_DISCRIMINATOR,
  };
}

/** Builds the `archive_habit` instruction. */
export function getArchiveHabitInstruction({
  owner,
  habit,
}: HabitLifecycleAccounts): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: readonlySigner },
      { address: habit, role: writable },
    ],
    data: ARCHIVE_HABIT_DISCRIMINATOR,
  };
}

/**
 * Builds the `close_habit` instruction. The habit account is closed and its
 * rent lamports are returned to the owner.
 */
export function getCloseHabitInstruction({
  owner,
  habit,
}: HabitLifecycleAccounts): Instruction {
  return {
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    accounts: [
      { address: owner, role: writableSigner },
      { address: habit, role: writable },
    ],
    data: CLOSE_HABIT_DISCRIMINATOR,
  };
}
