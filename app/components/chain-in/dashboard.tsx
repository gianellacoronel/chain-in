"use client";

import { useState } from "react";
import type { Address } from "@solana/kit";
import type { DecodedAccount, Profile } from "../../lib/chain-in";
import { useCheckIns } from "../../lib/hooks/use-check-ins";
import { useHabits } from "../../lib/hooks/use-habits";
import { CreateHabitForm } from "./create-habit-form";
import { HabitDetail } from "./habit-detail";
import { HabitList } from "./habit-list";
import { CheckInHistory } from "./check-in-history";

type ChainInDashboardProps = {
  profile: DecodedAccount<Profile>;
  refreshProfile: () => void;
};

export function ChainInDashboard({
  profile,
  refreshProfile,
}: ChainInDashboardProps) {
  const { habits, status, error, refresh } = useHabits(profile.data.authority);
  const {
    checkIns,
    status: checkInsStatus,
    error: checkInsError,
    refresh: refreshCheckIns,
  } = useCheckIns(profile.data.authority);
  const [selectedAddress, setSelectedAddress] = useState<Address | null>(null);

  // The detail view always reads from the refreshed list, so updates and
  // habits closed elsewhere are reflected without a second read.
  const selectedHabit = habits?.find(
    (habit) => habit.address === selectedAddress
  );

  const handleSettled = () => {
    refresh();
    refreshCheckIns();
    refreshProfile();
  };

  const handleSelect = (address: Address) => {
    setSelectedAddress((current) => (current === address ? null : address));
  };

  return (
    <div className="mt-8 grid gap-4">
      <CreateHabitForm profile={profile} onSettled={handleSettled} />
      {selectedHabit && (
        <HabitDetail
          habit={selectedHabit}
          onSettled={() => {
            refresh();
            refreshCheckIns();
          }}
          onClose={() => setSelectedAddress(null)}
        />
      )}
      <HabitList
        habits={habits}
        status={status}
        error={error}
        selectedAddress={selectedAddress}
        onSelect={handleSelect}
        onSettled={handleSettled}
        onRefresh={() => refresh()}
      />
      <CheckInHistory
        checkIns={checkIns}
        status={checkInsStatus}
        error={checkInsError}
        onRefresh={() => refreshCheckIns()}
      />
    </div>
  );
}
