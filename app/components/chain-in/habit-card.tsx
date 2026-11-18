import type { DecodedAccount, Habit } from "../../lib/chain-in";
import { formatUnixDay } from "../../lib/chain-in";
import { StatusBadge } from "./status-badge";

export function HabitCard({ habit }: { habit: DecodedAccount<Habit> }) {
  const {
    name,
    description,
    status,
    currentStreak,
    longestStreak,
    lastCheckInDay,
  } = habit.data;

  return (
    <article className="rounded-2xl border border-border-low bg-card p-5">
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

      <p className="mt-3 text-[11px] text-muted">
        Last check-in:{" "}
        {lastCheckInDay === null ? "Never" : formatUnixDay(lastCheckInDay)}
      </p>
    </article>
  );
}
