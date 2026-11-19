"use client";

import { useCallback } from "react";
import type { Address } from "@solana/kit";
import {
  deriveDailyCheckInAddress,
  deriveHabitAddress,
  deriveProfileAddress,
  getChainInErrorMessage,
  getCheckInInstruction,
  getCreateHabitInstruction,
  getInitializeProfileInstruction,
  getTodayUnixDay,
  getUpdateHabitInstruction,
  type DecodedAccount,
  type Habit,
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

  const updateHabit = useCallback(
    async ({
      owner,
      habit,
      name,
      description,
    }: {
      owner: Address;
      habit: DecodedAccount<Habit>;
      name: string;
      description: string;
    }) => {
      // The program checks ownership and string lengths only; the status does
      // not restrict updates, so paused and archived habits can be edited too.
      return run(
        () =>
          client.sendTransaction([
            getUpdateHabitInstruction({
              owner,
              habit: habit.address,
              name,
              description,
            }),
          ]),
        "Habit updated",
        getChainInErrorMessage
      );
    },
    [client, run]
  );

  const checkIn = useCallback(
    async ({
      owner,
      habit,
    }: {
      owner: Address;
      habit: DecodedAccount<Habit>;
    }) => {
      const day = getTodayUnixDay();
      const dailyCheckIn = await deriveDailyCheckInAddress(habit.address, day);

      const resolveError = (err: unknown): string | undefined => {
        const text = JSON.stringify(err, Object.getOwnPropertyNames(err ?? {}));
        if (/already in use|already exists/i.test(text)) {
          return "You already checked in today.";
        }
        return (
          getChainInErrorMessage(err) ??
          "Could not check in. You may have already checked in today."
        );
      };

      return run(
        () =>
          client.sendTransaction([
            getCheckInInstruction({
              owner,
              habit: habit.address,
              dailyCheckIn,
              day,
            }),
          ]),
        "Checked in",
        resolveError
      );
    },
    [client, run]
  );

  return { initializeProfile, createHabit, updateHabit, checkIn, isSending };
}
