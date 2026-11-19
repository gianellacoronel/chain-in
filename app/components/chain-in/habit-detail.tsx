"use client";

import { useState } from "react";
import {
  formatUnixDay,
  getUnixDay,
  HabitStatus,
  type DecodedAccount,
  type Habit,
} from "../../lib/chain-in";
import { useChainInActions } from "../../lib/hooks/use-chain-in-actions";
import { useTodayCheckIn } from "../../lib/hooks/use-today-check-in";
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
  const [confirming, setConfirming] = useState<"archive" | "close" | null>(
    null
  );
  const {
    checkIn,
    pauseHabit,
    resumeHabit,
    archiveHabit,
    closeHabit,
    isSending,
  } = useChainInActions();
  const { isCheckedIn, refresh: refreshCheckIn } = useTodayCheckIn(
    habit.data.status === HabitStatus.Active ? habit.address : undefined
  );

  const handleCheckIn = async () => {
    const signature = await checkIn({ owner: habit.data.owner, habit });
    refreshCheckIn();
    if (signature) onSettled();
  };

  const handlePause = async () => {
    const signature = await pauseHabit({ owner: habit.data.owner, habit });
    if (signature) onSettled();
  };

  const handleResume = async () => {
    const signature = await resumeHabit({ owner: habit.data.owner, habit });
    if (signature) onSettled();
  };

  const handleArchive = async () => {
    setConfirming(null);
    const signature = await archiveHabit({ owner: habit.data.owner, habit });
    if (signature) {
      onSettled();
      onClose();
    }
  };

  const handleClose = async () => {
    setConfirming(null);
    const signature = await closeHabit({ owner: habit.data.owner, habit });
    if (signature) {
      onSettled();
      onClose();
    }
  };

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

          <div className="mt-4 flex flex-wrap items-center gap-2">
            {status === HabitStatus.Active && (
              <button
                onClick={handleCheckIn}
                disabled={isSending || isCheckedIn === true}
                className="cursor-pointer rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
              >
                {isCheckedIn === true
                  ? "Checked in today"
                  : isSending
                    ? "Checking in..."
                    : "Check in today"}
              </button>
            )}
            {status === HabitStatus.Active && (
              <button
                onClick={handlePause}
                disabled={isSending}
                className="cursor-pointer rounded-lg border border-border-low bg-card px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:pointer-events-none disabled:opacity-50"
              >
                Pause
              </button>
            )}
            {status === HabitStatus.Paused && (
              <button
                onClick={handleResume}
                disabled={isSending}
                className="cursor-pointer rounded-lg border border-border-low bg-card px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:pointer-events-none disabled:opacity-50"
              >
                Resume
              </button>
            )}
            {(status === HabitStatus.Active ||
              status === HabitStatus.Paused) && (
              <button
                onClick={() => setConfirming("archive")}
                disabled={isSending}
                className="cursor-pointer rounded-lg border border-border-low bg-card px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:pointer-events-none disabled:opacity-50"
              >
                Archive
              </button>
            )}
            {status === HabitStatus.Archived && (
              <button
                onClick={() => setConfirming("close")}
                disabled={isSending}
                className="cursor-pointer rounded-lg border border-destructive/40 bg-card px-4 py-2 text-sm font-medium text-destructive transition hover:bg-destructive/10 disabled:pointer-events-none disabled:opacity-50"
              >
                Close (return rent)
              </button>
            )}
            <button
              onClick={() => setIsEditing(true)}
              disabled={isSending}
              className="cursor-pointer rounded-lg border border-border-low bg-card px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:pointer-events-none disabled:opacity-50"
            >
              Edit habit
            </button>
          </div>

          {confirming === "archive" && (
            <div className="mt-3 rounded-lg border border-border-low bg-cream/50 p-3 text-xs">
              <p className="font-medium">
                Archive this habit? It will no longer accept check-ins.
              </p>
              <div className="mt-2 flex gap-2">
                <button
                  onClick={handleArchive}
                  disabled={isSending}
                  className="cursor-pointer rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:opacity-50"
                >
                  {isSending ? "Archiving..." : "Confirm archive"}
                </button>
                <button
                  onClick={() => setConfirming(null)}
                  disabled={isSending}
                  className="cursor-pointer rounded-lg border border-border-low bg-card px-3 py-1.5 text-xs font-medium transition hover:bg-cream disabled:opacity-50"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          {confirming === "close" && (
            <div className="mt-3 rounded-lg border border-destructive/40 bg-destructive/5 p-3 text-xs">
              <p className="font-medium text-destructive">
                Close this habit permanently? Its rent will be returned to your
                wallet and the account will disappear, but check-in history is
                preserved.
              </p>
              <div className="mt-2 flex gap-2">
                <button
                  onClick={handleClose}
                  disabled={isSending}
                  className="cursor-pointer rounded-lg bg-destructive px-3 py-1.5 text-xs font-medium text-destructive-foreground shadow-xs transition hover:bg-destructive/90 disabled:opacity-50"
                >
                  {isSending ? "Closing..." : "Confirm close"}
                </button>
                <button
                  onClick={() => setConfirming(null)}
                  disabled={isSending}
                  className="cursor-pointer rounded-lg border border-border-low bg-card px-3 py-1.5 text-xs font-medium transition hover:bg-cream disabled:opacity-50"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </>
      )}
    </section>
  );
}
