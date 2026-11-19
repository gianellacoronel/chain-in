"use client";

import { useState } from "react";
import {
  formatUnixDay,
  getUnixDay,
  type DecodedAccount,
  type Habit,
} from "../../lib/chain-in";
import { EditHabitForm } from "./edit-habit-form";
import { StatusBadge } from "./status-badge";

type HabitDetailProps = {
  habit: DecodedAccount<Habit>;
  /** Called after every update attempt so the list can refresh. */
  onSettled: () => void;
  onClose: () => void;
};

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-border-low bg-cream/50 px-3 py-2">
      <dt className="text-[11px] font-medium tracking-wide text-muted uppercase">
        {label}
      </dt>
      <dd className="mt-0.5 text-sm font-semibold tabular-nums">{value}</dd>
    </div>
  );
}

export function HabitDetail({ habit, onSettled, onClose }: HabitDetailProps) {
  const [isEditing, setIsEditing] = useState(false);
  const { id, name, description, status, currentStreak, longestStreak } =
    habit.data;

  return (
    <section
      className="rounded-2xl border border-border-low bg-card p-6"
      aria-label={`Details for ${name}`}
    >
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-[11px] font-medium tracking-wide text-muted uppercase">
            Habit #{id.toString()}
          </p>
          <h2 className="mt-0.5 truncate text-lg font-semibold">{name}</h2>
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge status={status} />
          <button
            onClick={onClose}
            className="cursor-pointer rounded-lg border border-border-low bg-card px-3 py-1.5 text-xs font-medium transition hover:bg-cream"
          >
            Close
          </button>
        </div>
      </div>

      {isEditing ? (
        <div className="mt-4">
          <EditHabitForm
            key={habit.address}
            habit={habit}
            onSettled={onSettled}
            onCancel={() => setIsEditing(false)}
          />
        </div>
      ) : (
        <>
          <p className="mt-3 text-xs leading-relaxed text-muted">
            {description.trim() ? description : "No description yet."}
          </p>

          <dl className="mt-4 grid grid-cols-2 gap-2 sm:grid-cols-3">
            <Stat
              label="Current streak"
              value={`${currentStreak} ${currentStreak === 1 ? "day" : "days"}`}
            />
            <Stat
              label="Longest streak"
              value={`${longestStreak} ${longestStreak === 1 ? "day" : "days"}`}
            />
            <Stat
              label="Last check-in"
              value={
                habit.data.lastCheckInDay === null
                  ? "Never"
                  : formatUnixDay(habit.data.lastCheckInDay)
              }
            />
            <Stat
              label="Created"
              value={formatUnixDay(getUnixDay(habit.data.createdAt))}
            />
          </dl>

          <button
            onClick={() => setIsEditing(true)}
            className="mt-4 cursor-pointer rounded-lg border border-border-low bg-card px-4 py-2 text-sm font-medium transition hover:bg-cream"
          >
            Edit habit
          </button>
        </>
      )}
    </section>
  );
}
