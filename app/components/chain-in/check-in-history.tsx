import type { DecodedAccount, DailyCheckIn } from "../../lib/chain-in";
import { formatUnixDay } from "../../lib/chain-in";
import { getErrorMessage } from "../../lib/errors";

type CheckInHistoryProps = {
  checkIns: DecodedAccount<DailyCheckIn>[] | undefined;
  status: "disabled" | "error" | "fetching" | "success";
  error: unknown;
  onRefresh: () => void;
};

function formatTimestamp(seconds: bigint): string {
  return new Date(Number(seconds) * 1000).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function CheckInHistory({
  checkIns,
  status,
  error,
  onRefresh,
}: CheckInHistoryProps) {
  if (status === "error" && checkIns === undefined) {
    return (
      <section className="rounded-2xl border border-border-low bg-card p-6">
        <h2 className="text-sm font-semibold text-destructive">
          Could not load check-in history
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

  if (checkIns === undefined) {
    return (
      <section
        className="rounded-2xl border border-border-low bg-card p-6 text-sm text-muted"
        role="status"
      >
        Loading your check-in history...
      </section>
    );
  }

  if (checkIns.length === 0) {
    return (
      <section className="rounded-2xl border border-border-low bg-card p-6">
        <h2 className="text-sm font-semibold">Check-in history</h2>
        <p className="mt-1 text-xs leading-relaxed text-muted">
          No check-ins recorded yet. Check in on an active habit to start your
          history.
        </p>
      </section>
    );
  }

  return (
    <section>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 className="text-sm font-semibold">Check-in history</h2>
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

      <ul className="mt-3 divide-y divide-border-low overflow-hidden rounded-2xl border border-border-low bg-card">
        {checkIns.map((checkIn) => (
          <li
            key={checkIn.address}
            className="flex flex-wrap items-center justify-between gap-2 px-4 py-3"
          >
            <div className="min-w-0">
              <p className="truncate text-sm font-medium">
                {checkIn.data.habitNameSnapshot || "Unnamed habit"}
              </p>
              <p className="text-[11px] text-muted">
                Day {checkIn.data.day.toString()} ·{" "}
                {formatUnixDay(checkIn.data.day)}
              </p>
            </div>
            <p className="shrink-0 text-[11px] text-muted tabular-nums">
              {formatTimestamp(checkIn.data.checkedAt)}
            </p>
          </li>
        ))}
      </ul>
    </section>
  );
}
