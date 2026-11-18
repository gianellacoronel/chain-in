import { SECONDS_PER_DAY } from "./program";

/**
 * Converts a unix timestamp (in seconds) to the UTC day index used by the
 * program: `unix_timestamp / SECONDS_PER_DAY`. Both values are `bigint` to
 * match the `i64` on-chain type.
 */
export function getUnixDay(timestampSeconds: bigint): bigint {
  return timestampSeconds / SECONDS_PER_DAY;
}

/** Returns the current UTC day index. */
export function getTodayUnixDay(now: number = Date.now()): bigint {
  return getUnixDay(BigInt(Math.floor(now / 1000)));
}

/** Returns `true` when the given on-chain day index is the current UTC day. */
export function isToday(day: bigint, now: number = Date.now()): boolean {
  return day === getTodayUnixDay(now);
}

/** Converts a UTC day index to a `Date` at 00:00 UTC. */
export function unixDayToDate(day: bigint): Date {
  return new Date(Number(day * SECONDS_PER_DAY) * 1000);
}

/** Formats a UTC day index as a human-readable date. */
export function formatUnixDay(day: bigint): string {
  return unixDayToDate(day).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    timeZone: "UTC",
  });
}
