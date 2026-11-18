"use client";

import { getUnixDay, formatUnixDay } from "./lib/chain-in";
import { ProfileGate } from "./components/chain-in/profile-gate";

export default function Home() {
  return (
    <main className="mx-auto max-w-4xl px-6 py-10">
      <h1 className="text-3xl font-black tracking-tight">Chain-In</h1>
      <p className="mt-2 max-w-xl text-sm leading-relaxed text-muted">
        Build better habits one day at a time. Create your on-chain profile, add
        habits, and check in daily to grow your streak.
      </p>

      <ProfileGate>
        {({ profile, refresh }) => (
          <section className="mt-8 rounded-2xl border border-border-low bg-card p-6">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div>
                <h2 className="text-sm font-semibold">Profile ready</h2>
                <p className="mt-1 text-xs text-muted">
                  This wallet has an initialized Chain-In profile on Solana.
                </p>
              </div>
              <button
                onClick={() => refresh()}
                className="cursor-pointer rounded-lg border border-border-low bg-card px-3 py-2 text-xs font-medium transition hover:bg-cream"
              >
                Refresh
              </button>
            </div>

            <dl className="mt-4 grid gap-3 sm:grid-cols-2">
              <div className="rounded-lg border border-border-low bg-cream/50 px-3 py-2">
                <dt className="text-[11px] font-medium tracking-wide text-muted uppercase">
                  Habits created
                </dt>
                <dd className="mt-0.5 text-sm font-semibold tabular-nums">
                  {profile.data.nextHabitId.toString()}
                </dd>
              </div>
              <div className="rounded-lg border border-border-low bg-cream/50 px-3 py-2">
                <dt className="text-[11px] font-medium tracking-wide text-muted uppercase">
                  Created
                </dt>
                <dd className="mt-0.5 text-sm font-semibold">
                  {formatUnixDay(getUnixDay(profile.data.createdAt))}
                </dd>
              </div>
            </dl>
          </section>
        )}
      </ProfileGate>
    </main>
  );
}
