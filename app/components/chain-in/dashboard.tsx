"use client";

import type { DecodedAccount, Profile } from "../../lib/chain-in";
import { useHabits } from "../../lib/hooks/use-habits";
import { CreateHabitForm } from "./create-habit-form";
import { HabitList } from "./habit-list";

type ChainInDashboardProps = {
  profile: DecodedAccount<Profile>;
  refreshProfile: () => void;
};

export function ChainInDashboard({
  profile,
  refreshProfile,
}: ChainInDashboardProps) {
  const { habits, status, error, refresh } = useHabits(profile.data.authority);

  const handleSettled = () => {
    // A create attempt changes both the habit list and the profile's
    // `next_habit_id`, so refresh both.
    refresh();
    refreshProfile();
  };

  return (
    <div className="mt-8 grid gap-4">
      <CreateHabitForm profile={profile} onSettled={handleSettled} />
      <HabitList
        habits={habits}
        status={status}
        error={error}
        onRefresh={() => refresh()}
      />
    </div>
  );
}
