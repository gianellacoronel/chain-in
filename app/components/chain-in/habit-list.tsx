import type { Address } from "@solana/kit";
import type { DecodedAccount, Habit } from "../../lib/chain-in";
import { getErrorMessage } from "../../lib/errors";
import { HabitCard } from "./habit-card";

type HabitListProps = {
  habits: DecodedAccount<Habit>[] | undefined;
  status: "disabled" | "error" | "fetching" | "success";
  error: unknown;
  selectedAddress: Address | null;
  onSelect: (address: Address) => void;
  onRefresh: () => void;
};

export function HabitList({
  habits,
  status,
  error,
  selectedAddress,
  onSelect,
  onRefresh,
}: HabitListProps) {
  if (status === "error" && habits === undefined) {
    return (
      <section className="rounded-2xl border border-border-low bg-card p-6">
        <h2 className="text-sm font-semibold text-destructive">
          Could not load your habits
        </h2>
        <p className="mt-1 break-words text-xs text-muted [overflow-wrap:anywhere]">
          {getErrorMessage(error)}
        </p>
        <button
          onClick={onRefresh}
          className="mt-4 cursor-pointer rounded-lg border border-border-low bg-card px-3 py-2 text-xs font-medium transition hover:bg-cream"
        >
          Retry
        </button>
      </section>
    );
  }

  if (habits === undefined) {
    return (
      <section
        className="rounded-2xl border border-border-low bg-card p-6 text-sm text-muted"
        role="status"
      >
        Loading your habits...
      </section>
    );
  }

  if (habits.length === 0) {
    return (
      <section className="rounded-2xl border border-border-low bg-card p-6">
        <h2 className="text-sm font-semibold">No habits yet</h2>
        <p className="mt-1 text-xs leading-relaxed text-muted">
          Create your first habit above and check in daily to start a streak.
        </p>
      </section>
    );
  }

  return (
    <section>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-sm font-semibold">Your habits</h2>
        <div className="flex items-center gap-3">
          {status === "fetching" && (
            <span className="text-[11px] text-muted" role="status">
              Syncing...
            </span>
          )}
          <button
            onClick={onRefresh}
            className="cursor-pointer rounded-lg border border-border-low bg-card px-3 py-2 text-xs font-medium transition hover:bg-cream"
          >
            Refresh
          </button>
        </div>
      </div>

      <div className="mt-3 grid gap-3 sm:grid-cols-2">
        {habits.map((habit) => (
          <HabitCard
            key={habit.address}
            habit={habit}
            isSelected={habit.address === selectedAddress}
            onSelect={onSelect}
          />
        ))}
      </div>
    </section>
  );
}
