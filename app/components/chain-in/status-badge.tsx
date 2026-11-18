import { HABIT_STATUS_LABELS, HabitStatus } from "../../lib/chain-in";

const STATUS_STYLES: Record<HabitStatus, string> = {
  [HabitStatus.Active]: "bg-green-500/10 text-green-700 dark:text-green-400",
  [HabitStatus.Paused]: "bg-amber-500/10 text-amber-700 dark:text-amber-400",
  [HabitStatus.Archived]: "bg-secondary text-muted-foreground",
};

export function StatusBadge({ status }: { status: HabitStatus }) {
  return (
    <span
      className={`inline-flex shrink-0 items-center rounded-full px-2 py-0.5 text-[11px] font-medium ${STATUS_STYLES[status]}`}
    >
      {HABIT_STATUS_LABELS[status]}
    </span>
  );
}
