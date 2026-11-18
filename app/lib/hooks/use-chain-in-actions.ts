"use client";

import { useCallback } from "react";
import type { Address } from "@solana/kit";
import {
  deriveProfileAddress,
  getChainInErrorMessage,
  getInitializeProfileInstruction,
} from "../chain-in";
import { useAppClient } from "../client-provider";
import { useSend } from "./use-send";

/**
 * Chain-In transaction actions. Each action resolves to the transaction
 * signature, or `undefined` when the transaction failed or was rejected.
 */
export function useChainInActions() {
  const client = useAppClient();
  const { run, isSending } = useSend();

  const initializeProfile = useCallback(
    async (authority: Address) => {
      const profile = await deriveProfileAddress(authority);

      return run(
        () =>
          client.sendTransaction([
            getInitializeProfileInstruction({ authority, profile }),
          ]),
        "Profile created",
        getChainInErrorMessage
      );
    },
    [client, run]
  );

  return { initializeProfile, isSending };
}
