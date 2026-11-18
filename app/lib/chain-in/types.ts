import type { Address } from "@solana/kit";

/**
 * On-chain habit status. Declared as a const object (not a TypeScript enum)
 * so it can also be used as a Kit enum codec lookup table.
 */
export const HabitStatus = {
  Active: 0,
  Paused: 1,
  Archived: 2,
} as const;

export type HabitStatus = (typeof HabitStatus)[keyof typeof HabitStatus];

/** Human-readable label for each status. */
export const HABIT_STATUS_LABELS: Record<HabitStatus, string> = {
  [HabitStatus.Active]: "Active",
  [HabitStatus.Paused]: "Paused",
  [HabitStatus.Archived]: "Archived",
};

/** Decoded `Profile` account data. */
export type Profile = {
  authority: Address;
  nextHabitId: bigint;
  createdAt: bigint;
};

/** Decoded `Habit` account data. */
export type Habit = {
  owner: Address;
  id: bigint;
  name: string;
  description: string;
  status: HabitStatus;
  currentStreak: number;
  longestStreak: number;
  lastCheckInDay: bigint | null;
  createdAt: bigint;
};

/** Decoded `DailyCheckIn` account data. */
export type DailyCheckIn = {
  owner: Address;
  habit: Address;
  habitId: bigint;
  habitNameSnapshot: string;
  day: bigint;
  checkedAt: bigint;
};

/** A decoded account paired with the address it was fetched from. */
export type DecodedAccount<T> = {
  address: Address;
  data: T;
};
