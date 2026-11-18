import { isCustomProgramError } from "../errors";
import { MAX_HABIT_DESCRIPTION_LEN, MAX_HABIT_NAME_LEN } from "./program";

export type ChainInErrorDefinition = {
  code: number;
  name: string;
  message: string;
};

/**
 * Anchor custom program errors. The program error code is `6000 + index`.
 * Messages are user-facing and avoid raw on-chain wording.
 */
export const CHAIN_IN_ERRORS: readonly ChainInErrorDefinition[] = [
  {
    code: 6000,
    name: "UnauthorizedProfile",
    message: "This wallet does not own that profile.",
  },
  {
    code: 6001,
    name: "UnauthorizedHabit",
    message: "This wallet does not own that habit.",
  },
  {
    code: 6002,
    name: "HabitNameTooLong",
    message: `Habit names can be at most ${MAX_HABIT_NAME_LEN} bytes long.`,
  },
  {
    code: 6003,
    name: "HabitDescriptionTooLong",
    message: `Habit descriptions can be at most ${MAX_HABIT_DESCRIPTION_LEN} bytes long.`,
  },
  {
    code: 6004,
    name: "HabitIdOverflow",
    message: "This profile has reached the maximum number of habits.",
  },
  {
    code: 6005,
    name: "HabitNotActive",
    message: "Only active habits can receive check-ins.",
  },
  {
    code: 6006,
    name: "InvalidCheckInDay",
    message:
      "That check-in day does not match the current UTC day. Try again after the day rolls over.",
  },
  {
    code: 6007,
    name: "StreakOverflow",
    message: "The streak counter overflowed.",
  },
  {
    code: 6008,
    name: "HabitNotPauseable",
    message: "Only active habits can be paused.",
  },
  {
    code: 6009,
    name: "HabitNotResumable",
    message: "Only paused habits can be resumed.",
  },
  {
    code: 6010,
    name: "HabitNotArchivable",
    message: "Only active or paused habits can be archived.",
  },
  {
    code: 6011,
    name: "HabitNotClosable",
    message: "Only archived habits can be closed.",
  },
];

/** Returns the Chain-In error definition matching a transaction error. */
export function getChainInError(
  error: unknown
): ChainInErrorDefinition | undefined {
  return CHAIN_IN_ERRORS.find((definition) =>
    isCustomProgramError(error, definition.code)
  );
}

/** Returns a user-facing message for a Chain-In program error, if any. */
export function getChainInErrorMessage(error: unknown): string | undefined {
  return getChainInError(error)?.message;
}
