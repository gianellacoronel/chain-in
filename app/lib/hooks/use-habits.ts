"use client";

import { useMemo } from "react";
import type { Address } from "@solana/kit";
import { useRequest } from "@solana/react";
import { fetchHabitsByOwner } from "../chain-in";
import { useAppClient } from "../client-provider";

/**
 * Reads every habit owned by the wallet. `habits` is `undefined` until the
 * first request settles and an empty array when the wallet has no habits.
 */
export function useHabits(owner: Address | undefined) {
  const client = useAppClient();

  const fetcher = useMemo(() => {
    if (!owner) return null;

    return (signal: AbortSignal) =>
      fetchHabitsByOwner(client.rpc, owner, signal);
  }, [client, owner]);

  const { data, error, refresh, status } = useRequest(fetcher);

  return {
    habits: data,
    status,
    error,
    refresh,
  };
}
