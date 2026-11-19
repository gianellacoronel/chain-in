import type { Address } from "@solana/kit";
import type { DecodedAccount, Habit } from "../../lib/chain-in";
import { HabitStatus } from "../../lib/chain-in";
import { formatUnixDay } from "../../lib/chain-in";
import { useChainInActions } from "../../lib/hooks/use-chain-in-actions";
import { useTodayCheckIn } from "../../lib/hooks/use-today-check-in";
import { StatusBadge } from "./status-badge";

type HabitCardProps = {
  habit: DecodedAccount<Habit>;
  isSelected: boolean;
  onSelect: (address: Address) => void;
  onSettled: () => void;
};

export function HabitCard({
  habit,
  isSelected,
  onSelect,
  onSettled,
}: HabitCardProps) {
  const { checkIn, isSending } = useChainInActions();
  const { isCheckedIn, refresh: refreshCheckIn } = useTodayCheckIn(
    habit.data.status === HabitStatus.Active ? habit.address : undefined
  );

  const handleCheckIn = async () => {
    const signature = await checkIn({ owner: habit.data.owner, habit });
    refreshCheckIn();
    if (signature) onSettled();
  };

  const {
    name,
    description,
    status,
    currentStreak,
    longestStreak,
    lastCheckInDay,
  } = habit.data;

  return (
    <article
      className={`rounded-2xl border bg-card p-5 transition ${
        isSelected
          ? "border-primary/50 ring-2 ring-primary/10"
          : "border-border-low"
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3 className="truncate text-sm font-semibold">{name}</h3>
          <p className="mt-1 line-clamp-2 text-xs leading-relaxed text-muted">
            {description.trim() ? description : "No description"}
          </p>
        </div>
        <StatusBadge status={status} />
      </div>

      <dl className="mt-4 grid grid-cols-2 gap-2">
        <div className="rounded-lg border border-border-low bg-cream/50 px-3 py-2">
          <dt className="text-[11px] font-medium tracking-wide text-muted uppercase">
            Current streak
          </dt>
          <dd className="mt-0.5 text-sm font-semibold tabular-nums">
            {currentStreak} {currentStreak === 1 ? "day" : "days"}
          </dd>
        </div>
        <div className="rounded-lg border border-border-low bg-cream/50 px-3 py-2">
          <dt className="text-[11px] font-medium tracking-wide text-muted uppercase">
            Longest streak
          </dt>
          <dd className="mt-0.5 text-sm font-semibold tabular-nums">
            {longestStreak} {longestStreak === 1 ? "day" : "days"}
          </dd>
        </div>
      </dl>

      <div className="mt-3 flex items-center justify-between gap-3">
        <p className="text-[11px] text-muted">
          Last check-in:{" "}
          {lastCheckInDay === null ? "Never" : formatUnixDay(lastCheckInDay)}
        </p>
        <div className="flex items-center gap-2">
          {status === HabitStatus.Active && (
            <button
              onClick={handleCheckIn}
              disabled={isSending || isCheckedIn === true}
              className="cursor-pointer rounded-lg bg-primary px-2.5 py-1 text-[11px] font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
            >
              {isCheckedIn === true
                ? "Checked in today"
                : isSending
                  ? "Checking in..."
                  : "Check in"}
            </button>
          )}
          <button
            onClick={() => onSelect(habit.address)}
            aria-expanded={isSelected}
            className="cursor-pointer rounded-lg border border-border-low bg-card px-2.5 py-1 text-[11px] font-medium transition hover:bg-cream"
          >
            {isSelected ? "Hide details" : "Manage"}
          </button>
        </div>
      </div>
    </article>
  );
}
