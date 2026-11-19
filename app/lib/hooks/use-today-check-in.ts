"use client";

import { useMemo } from "react";
import type { Address } from "@solana/kit";
import { useRequest } from "@solana/react";
import { fetchDailyCheckIn, getTodayUnixDay } from "../chain-in";
import { useAppClient } from "../client-provider";

/**
 * Detects whether a check-in already exists for the current UTC day for the
 * given habit. `isCheckedIn` is `null` while loading, `true`/`false` once
 * settled.
 */
export function useTodayCheckIn(habit: Address | undefined) {
  const client = useAppClient();
  const today = getTodayUnixDay();

  const fetcher = useMemo(() => {
    if (!habit) return null;

    return (signal: AbortSignal) =>
      fetchDailyCheckIn(client.rpc, habit, today, signal);
  }, [client, habit, today]);

  const { data, error, refresh, status } = useRequest(fetcher);

  return {
    today,
    checkIn: data ?? null,
    isCheckedIn: data === undefined ? null : data !== null,
    status,
    error,
    refresh,
  };
}
