"use client";

import { useMemo } from "react";
import type { Address } from "@solana/kit";
import { useRequest } from "@solana/react";
import { fetchProfile } from "../chain-in";
import { useAppClient } from "../client-provider";

/**
 * Reads the connected wallet's Chain-In profile.
 *
 * `profile` is `undefined` until the first request settles, `null` when the
 * wallet has no profile yet, and the decoded account once it exists.
 */
export function useProfile(authority: Address | undefined) {
  const client = useAppClient();

  const fetcher = useMemo(() => {
    if (!authority) return null;

    return (signal: AbortSignal) => fetchProfile(client.rpc, authority, signal);
  }, [client, authority]);

  const { data, error, refresh, status } = useRequest(fetcher);

  return {
    /** `undefined` until the first request settles, `null` when missing. */
    profile: data,
    status,
    error,
    refresh,
  };
}
