"use client";

import { useMemo } from "react";
import type { Address } from "@solana/kit";
import { useRequest } from "@solana/react";
import { fetchCheckInsByOwner } from "../chain-in";
import { useAppClient } from "../client-provider";

/**
 * Reads every DailyCheckIn owned by the wallet (newest day first), including
 * check-ins for habits that have since been closed.
 */
export function useCheckIns(owner: Address | undefined) {
  const client = useAppClient();

  const fetcher = useMemo(() => {
    if (!owner) return null;

    return () => fetchCheckInsByOwner(client.rpc, owner);
  }, [client, owner]);

  const { data, error, refresh, status } = useRequest(fetcher);

  return {
    checkIns: data,
    status,
    error,
    refresh,
  };
}
