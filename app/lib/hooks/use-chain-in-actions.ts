"use client";

import { useCallback } from "react";
import type { Address } from "@solana/kit";
import {
  deriveHabitAddress,
  deriveProfileAddress,
  getChainInErrorMessage,
  getCreateHabitInstruction,
  getInitializeProfileInstruction,
  type DecodedAccount,
  type Profile,
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

  const createHabit = useCallback(
    async ({
      owner,
      profile,
      name,
      description,
    }: {
      owner: Address;
      profile: DecodedAccount<Profile>;
      name: string;
      description: string;
    }) => {
      // The program derives the habit PDA from the on-chain `next_habit_id`,
      // so the profile must be the freshest read before sending.
      const habit = await deriveHabitAddress(owner, profile.data.nextHabitId);

      return run(
        () =>
          client.sendTransaction([
            getCreateHabitInstruction({
              owner,
              profile: profile.address,
              habit,
              name,
              description,
            }),
          ]),
        "Habit created",
        getChainInErrorMessage
      );
    },
    [client, run]
  );

  return { initializeProfile, createHabit, isSending };
}
