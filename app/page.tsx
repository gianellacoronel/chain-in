"use client";

import { ChainInDashboard } from "./components/chain-in/dashboard";
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
          <ChainInDashboard profile={profile} refreshProfile={refresh} />
        )}
      </ProfileGate>
    </main>
  );
}
