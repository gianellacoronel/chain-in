import {
  addDecoderSizePrefix,
  addEncoderSizePrefix,
  getAddressDecoder,
  getI64Decoder,
  getI64Encoder,
  getNullableDecoder,
  getStructDecoder,
  getStructEncoder,
  getU32Decoder,
  getU32Encoder,
  getU64Decoder,
  getU8Decoder,
  getUtf8Decoder,
  getUtf8Encoder,
  type ReadonlyUint8Array,
} from "@solana/kit";
import {
  DAILY_CHECK_IN_ACCOUNT_SIZE,
  DAILY_CHECK_IN_DISCRIMINATOR,
  HABIT_ACCOUNT_SIZE,
  HABIT_DISCRIMINATOR,
  PROFILE_ACCOUNT_SIZE,
  PROFILE_DISCRIMINATOR,
} from "./program";
import {
  HabitStatus,
  type DailyCheckIn,
  type Habit,
  type Profile,
} from "./types";

/** Raised when RPC account data does not match the Chain-In schema. */
export class ChainInAccountError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ChainInAccountError";
  }
}

/** Concatenates byte chunks into a single `Uint8Array`. */
export function concatBytes(...chunks: ReadonlyUint8Array[]): Uint8Array {
  const length = chunks.reduce((total, chunk) => total + chunk.length, 0);
  const result = new Uint8Array(length);

  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }

  return result;
}

const profileDataDecoder = getStructDecoder([
  ["authority", getAddressDecoder()],
  ["nextHabitId", getU64Decoder()],
  ["createdAt", getI64Decoder()],
]);

const habitDataDecoder = getStructDecoder([
  ["owner", getAddressDecoder()],
  ["id", getU64Decoder()],
  ["name", addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
  ["description", addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder())],
  ["status", getU8Decoder()],
  ["currentStreak", getU32Decoder()],
  ["longestStreak", getU32Decoder()],
  ["lastCheckInDay", getNullableDecoder(getI64Decoder())],
  ["createdAt", getI64Decoder()],
]);

const dailyCheckInDataDecoder = getStructDecoder([
  ["owner", getAddressDecoder()],
  ["habit", getAddressDecoder()],
  ["habitId", getU64Decoder()],
  [
    "habitNameSnapshot",
    addDecoderSizePrefix(getUtf8Decoder(), getU32Decoder()),
  ],
  ["day", getI64Decoder()],
  ["checkedAt", getI64Decoder()],
]);

const createOrUpdateHabitArgsEncoder = getStructEncoder([
  ["name", addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
  ["description", addEncoderSizePrefix(getUtf8Encoder(), getU32Encoder())],
]);

const checkInArgsEncoder = getStructEncoder([["day", getI64Encoder()]]);

/** Validates the discriminator and exact size of an account buffer. */
function assertAccountHeader(
  data: ReadonlyUint8Array,
  discriminator: ReadonlyUint8Array,
  accountSize: number,
  accountName: string
): void {
  if (data.length !== accountSize) {
    throw new ChainInAccountError(
      `${accountName} account has an unexpected size: expected ${accountSize} bytes, received ${data.length}.`
    );
  }

  for (let index = 0; index < discriminator.length; index += 1) {
    if (data[index] !== discriminator[index]) {
      throw new ChainInAccountError(
        `${accountName} account discriminator does not match the Chain-In program.`
      );
    }
  }
}

function parseHabitStatus(value: number): HabitStatus {
  if (
    value === HabitStatus.Active ||
    value === HabitStatus.Paused ||
    value === HabitStatus.Archived
  ) {
    return value;
  }

  throw new ChainInAccountError(`Unknown habit status: ${value}.`);
}

/** Decodes a `Profile` account, including discriminator validation. */
export function decodeProfileAccount(data: ReadonlyUint8Array): Profile {
  assertAccountHeader(
    data,
    PROFILE_DISCRIMINATOR,
    PROFILE_ACCOUNT_SIZE,
    "Profile"
  );
  return profileDataDecoder.decode(data.subarray(8));
}

/** Decodes a `Habit` account, including discriminator validation. */
export function decodeHabitAccount(data: ReadonlyUint8Array): Habit {
  assertAccountHeader(data, HABIT_DISCRIMINATOR, HABIT_ACCOUNT_SIZE, "Habit");

  const raw = habitDataDecoder.decode(data.subarray(8));

  return {
    ...raw,
    status: parseHabitStatus(raw.status),
  };
}

/** Decodes a `DailyCheckIn` account, including discriminator validation. */
export function decodeDailyCheckInAccount(
  data: ReadonlyUint8Array
): DailyCheckIn {
  assertAccountHeader(
    data,
    DAILY_CHECK_IN_DISCRIMINATOR,
    DAILY_CHECK_IN_ACCOUNT_SIZE,
    "DailyCheckIn"
  );
  return dailyCheckInDataDecoder.decode(data.subarray(8));
}

export type CreateOrUpdateHabitArgs = {
  name: string;
  description: string;
};

/** Encodes the `create_habit` instruction arguments (Borsh). */
export function encodeCreateHabitArgs(
  args: CreateOrUpdateHabitArgs
): ReadonlyUint8Array {
  return createOrUpdateHabitArgsEncoder.encode(args);
}

/** Encodes the `update_habit` instruction arguments (Borsh). */
export function encodeUpdateHabitArgs(
  args: CreateOrUpdateHabitArgs
): ReadonlyUint8Array {
  return createOrUpdateHabitArgsEncoder.encode(args);
}

/** Encodes the `check_in` instruction arguments (Borsh). */
export function encodeCheckInArgs(day: bigint): ReadonlyUint8Array {
  return checkInArgsEncoder.encode({ day });
}
