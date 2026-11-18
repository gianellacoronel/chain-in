import {
  getAddressEncoder,
  getI64Encoder,
  getProgramDerivedAddress,
  getU64Encoder,
  type Address,
} from "@solana/kit";
import {
  CHAIN_IN_PROGRAM_ADDRESS,
  CHECKIN_SEED,
  HABIT_SEED,
  PROFILE_SEED,
} from "./program";

const addressEncoder = getAddressEncoder();
const u64Encoder = getU64Encoder();
const i64Encoder = getI64Encoder();

/** Derives the `Profile` PDA for a wallet: `[b"profile", authority]`. */
export async function deriveProfileAddress(
  authority: Address
): Promise<Address> {
  const [profileAddress] = await getProgramDerivedAddress({
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    seeds: [PROFILE_SEED, addressEncoder.encode(authority)],
  });

  return profileAddress;
}

/** Derives a `Habit` PDA: `[b"habit", owner, habit_id]`. */
export async function deriveHabitAddress(
  owner: Address,
  habitId: bigint
): Promise<Address> {
  const [habitAddress] = await getProgramDerivedAddress({
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    seeds: [
      HABIT_SEED,
      addressEncoder.encode(owner),
      u64Encoder.encode(habitId),
    ],
  });

  return habitAddress;
}

/** Derives a `DailyCheckIn` PDA: `[b"checkin", habit, day]`. */
export async function deriveDailyCheckInAddress(
  habit: Address,
  day: bigint
): Promise<Address> {
  const [checkInAddress] = await getProgramDerivedAddress({
    programAddress: CHAIN_IN_PROGRAM_ADDRESS,
    seeds: [CHECKIN_SEED, addressEncoder.encode(habit), i64Encoder.encode(day)],
  });

  return checkInAddress;
}
