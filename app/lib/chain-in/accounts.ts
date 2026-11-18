import {
  fetchEncodedAccount,
  getAddressEncoder,
  getBase64Decoder,
  getBase64Encoder,
  type Address,
  type Base64EncodedBytes,
  type GetAccountInfoApi,
  type GetProgramAccountsApi,
  type ReadonlyUint8Array,
  type Rpc,
} from "@solana/kit";
import {
  ChainInAccountError,
  decodeDailyCheckInAccount,
  decodeHabitAccount,
  decodeProfileAccount,
} from "./codecs";
import {
  DAILY_CHECK_IN_ACCOUNT_SIZE,
  DAILY_CHECK_IN_DISCRIMINATOR,
  HABIT_ACCOUNT_SIZE,
  HABIT_DISCRIMINATOR,
  CHAIN_IN_PROGRAM_ADDRESS,
} from "./program";
import {
  deriveDailyCheckInAddress,
  deriveHabitAddress,
  deriveProfileAddress,
} from "./pdas";
import type { DailyCheckIn, DecodedAccount, Habit, Profile } from "./types";

/** The RPC surface required by the Chain-In account fetchers. */
export type ChainInRpc = Rpc<GetAccountInfoApi & GetProgramAccountsApi>;

const addressEncoder = getAddressEncoder();

/** Encodes bytes as a base64 string for `memcmp` filters. */
function toBase64(bytes: ReadonlyUint8Array): Base64EncodedBytes {
  return getBase64Decoder().decode(bytes) as Base64EncodedBytes;
}

/** Decodes the base64 payload returned by `getProgramAccounts`. */
function fromBase64(value: Base64EncodedBytes): ReadonlyUint8Array {
  return getBase64Encoder().encode(value);
}

async function fetchDecodedAccount<T>(
  rpc: ChainInRpc,
  accountAddress: Address,
  decoder: (data: ReadonlyUint8Array) => T,
  accountName: string,
  signal?: AbortSignal
): Promise<DecodedAccount<T> | null> {
  const account = await fetchEncodedAccount(
    rpc,
    accountAddress,
    signal ? { abortSignal: signal } : undefined
  );

  if (!account.exists) {
    return null;
  }

  if (account.programAddress !== CHAIN_IN_PROGRAM_ADDRESS) {
    throw new ChainInAccountError(
      `${accountName} account ${accountAddress} is not owned by the Chain-In program.`
    );
  }

  return { address: account.address, data: decoder(account.data) };
}

/** Fetches the `Profile` PDA for a wallet, or `null` when uninitialized. */
export async function fetchProfile(
  rpc: ChainInRpc,
  authority: Address,
  signal?: AbortSignal
): Promise<DecodedAccount<Profile> | null> {
  const profileAddress = await deriveProfileAddress(authority);
  return fetchDecodedAccount(
    rpc,
    profileAddress,
    decodeProfileAccount,
    "Profile",
    signal
  );
}

/** Fetches a `Habit` account by owner and habit ID, or `null` if missing. */
export async function fetchHabit(
  rpc: ChainInRpc,
  owner: Address,
  habitId: bigint,
  signal?: AbortSignal
): Promise<DecodedAccount<Habit> | null> {
  const habitAddress = await deriveHabitAddress(owner, habitId);
  return fetchDecodedAccount(
    rpc,
    habitAddress,
    decodeHabitAccount,
    "Habit",
    signal
  );
}

/** Fetches a `DailyCheckIn` account by habit and UTC day, or `null`. */
export async function fetchDailyCheckIn(
  rpc: ChainInRpc,
  habit: Address,
  day: bigint,
  signal?: AbortSignal
): Promise<DecodedAccount<DailyCheckIn> | null> {
  const checkInAddress = await deriveDailyCheckInAddress(habit, day);
  return fetchDecodedAccount(
    rpc,
    checkInAddress,
    decodeDailyCheckInAccount,
    "DailyCheckIn",
    signal
  );
}

/** Lists every `Habit` owned by a wallet, sorted by habit ID ascending. */
export async function fetchHabitsByOwner(
  rpc: ChainInRpc,
  owner: Address
): Promise<DecodedAccount<Habit>[]> {
  const response = await rpc
    .getProgramAccounts(CHAIN_IN_PROGRAM_ADDRESS, {
      encoding: "base64",
      filters: [
        { dataSize: BigInt(HABIT_ACCOUNT_SIZE) },
        {
          memcmp: {
            offset: 0n,
            bytes: toBase64(HABIT_DISCRIMINATOR),
            encoding: "base64",
          },
        },
        {
          memcmp: {
            offset: 8n,
            bytes: toBase64(addressEncoder.encode(owner)),
            encoding: "base64",
          },
        },
      ],
    })
    .send();

  return response
    .map(({ pubkey, account }) => ({
      address: pubkey,
      data: decodeHabitAccount(fromBase64(account.data[0])),
    }))
    .sort((a, b) =>
      a.data.id < b.data.id ? -1 : a.data.id > b.data.id ? 1 : 0
    );
}

/**
 * Lists every `DailyCheckIn` owned by a wallet, newest day first.
 * It includes check-ins for habits that have been closed.
 */
export async function fetchCheckInsByOwner(
  rpc: ChainInRpc,
  owner: Address
): Promise<DecodedAccount<DailyCheckIn>[]> {
  return fetchCheckIns(rpc, [
    {
      memcmp: {
        offset: 8n,
        bytes: toBase64(addressEncoder.encode(owner)),
        encoding: "base64",
      },
    },
  ]);
}

/** Lists the check-ins recorded for a specific habit, newest day first. */
export async function fetchCheckInsByHabit(
  rpc: ChainInRpc,
  owner: Address,
  habit: Address
): Promise<DecodedAccount<DailyCheckIn>[]> {
  return fetchCheckIns(rpc, [
    {
      memcmp: {
        offset: 8n,
        bytes: toBase64(addressEncoder.encode(owner)),
        encoding: "base64",
      },
    },
    {
      memcmp: {
        offset: 40n,
        bytes: toBase64(addressEncoder.encode(habit)),
        encoding: "base64",
      },
    },
  ]);
}

async function fetchCheckIns(
  rpc: ChainInRpc,
  addressFilters: {
    memcmp: {
      offset: bigint;
      bytes: Base64EncodedBytes;
      encoding: "base64";
    };
  }[]
): Promise<DecodedAccount<DailyCheckIn>[]> {
  const response = await rpc
    .getProgramAccounts(CHAIN_IN_PROGRAM_ADDRESS, {
      encoding: "base64",
      filters: [
        { dataSize: BigInt(DAILY_CHECK_IN_ACCOUNT_SIZE) },
        {
          memcmp: {
            offset: 0n,
            bytes: toBase64(DAILY_CHECK_IN_DISCRIMINATOR),
            encoding: "base64",
          },
        },
        ...addressFilters,
      ],
    })
    .send();

  return response
    .map(({ pubkey, account }) => ({
      address: pubkey,
      data: decodeDailyCheckInAccount(fromBase64(account.data[0])),
    }))
    .sort((a, b) => {
      if (a.data.day !== b.data.day) {
        return a.data.day > b.data.day ? -1 : 1;
      }
      return a.data.checkedAt > b.data.checkedAt ? -1 : 1;
    });
}

/**
 * Checks whether a profile account exists at the expected PDA, without
 * decoding it. Useful as a lightweight existence probe.
 */
export async function profileExists(
  rpc: ChainInRpc,
  authority: Address
): Promise<boolean> {
  const profileAddress = await deriveProfileAddress(authority);
  const account = await fetchEncodedAccount(rpc, profileAddress);

  return account.exists && account.programAddress === CHAIN_IN_PROGRAM_ADDRESS;
}
