"use client";

import { useSyncExternalStore, type ReactNode } from "react";
import { address } from "@solana/kit";
import {
  useConnectedWallet,
  useIsWalletReady,
} from "@solana/kit-plugin-wallet/react";
import type { DecodedAccount, Profile } from "../../lib/chain-in";
import { useChainInActions } from "../../lib/hooks/use-chain-in-actions";
import { useProfile } from "../../lib/hooks/use-profile";
import { useAppClient } from "../../lib/client-provider";
import { ellipsify } from "../../lib/explorer";
import { useCluster } from "../cluster-context";

const subscribeToHydration = () => () => {};

export type ProfileGateState = {
  profile: DecodedAccount<Profile>;
  refresh: () => void;
};

/**
 * Ensures the connected wallet has a Chain-In profile before rendering its
 * children. It owns the profile read, the initialization transaction, and all
 * loading/error/empty states.
 */
export function ProfileGate({
  children,
}: {
  children: (state: ProfileGateState) => ReactNode;
}) {
  const client = useAppClient();
  const connected = useConnectedWallet(client);
  const isWalletReady = useIsWalletReady(client);
  const isHydrated = useSyncExternalStore(
    subscribeToHydration,
    () => true,
    () => false
  );
  const { cluster } = useCluster();
  const isProgramAvailable = cluster === "devnet" || cluster === "localnet";

  const { profile, status, error, refresh } = useProfile(
    isProgramAvailable && connected
      ? address(connected.account.address)
      : undefined
  );
  const { initializeProfile, isSending } = useChainInActions();

  if (!isHydrated || !isWalletReady) {
    return <StatusCard>Restoring wallet connection...</StatusCard>;
  }

  if (!isProgramAvailable) {
    return (
      <StatusCard>
        Chain-In is deployed on devnet. Switch the network in the header to
        devnet or localnet to continue.
      </StatusCard>
    );
  }

  if (!connected) {
    return (
      <StatusCard>
        Connect your wallet to create or open your Chain-In profile.
      </StatusCard>
    );
  }

  if (!connected.signer) {
    return (
      <StatusCard>
        This wallet is connected in read-only mode. Connect a signing-capable
        account to use Chain-In.
      </StatusCard>
    );
  }

  if (!connected.supportedTransactionVersions.has(1)) {
    return (
      <StatusCard>
        This wallet does not support Version 1 transactions. Connect a
        V1-capable wallet to use Chain-In.
      </StatusCard>
    );
  }

  if (profile === undefined || (profile === null && status === "fetching")) {
    if (status === "error") {
      return (
        <section className="mt-8 rounded-2xl border border-border-low bg-card p-6">
          <h2 className="text-sm font-semibold text-destructive">
            Could not load your profile
          </h2>
          <p className="mt-1 break-words text-xs text-muted [overflow-wrap:anywhere]">
            {getErrorMessage(error)}
          </p>
          <button
            onClick={() => refresh()}
            className="mt-4 cursor-pointer rounded-lg border border-border-low bg-card px-3 py-2 text-xs font-medium transition hover:bg-cream"
          >
            Retry
          </button>
        </section>
      );
    }

    return (
      <StatusCard role="status">
        <span className="inline-flex items-center gap-2">
          <span className="h-2 w-2 animate-pulse rounded-full bg-muted" />
          Loading your profile...
        </span>
      </StatusCard>
    );
  }

  if (profile === null) {
    return (
      <section className="mt-8 rounded-2xl border border-border-low bg-card p-6">
        <h2 className="text-sm font-semibold">Create your profile</h2>
        <p className="mt-1 max-w-xl text-sm leading-relaxed text-muted">
          Chain-In keeps your habit profile in a per-wallet account on Solana.
          Creating it is a one-time transaction and only needs to happen once.
        </p>
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <button
            onClick={async () => {
              await initializeProfile(address(connected.account.address));
              // Refresh even after a failure: the profile may have been
              // created in another tab or wallet.
              refresh();
            }}
            disabled={isSending}
            className="cursor-pointer rounded-lg bg-primary px-4 py-2.5 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
          >
            {isSending ? "Creating profile..." : "Create profile"}
          </button>
          <p className="text-xs text-muted">
            Signed by{" "}
            <span className="font-mono">
              {ellipsify(connected.account.address)}
            </span>
          </p>
        </div>
      </section>
    );
  }

  return <>{children({ profile, refresh })}</>;
}

function StatusCard({
  children,
  role,
}: {
  children: ReactNode;
  role?: "alert" | "status";
}) {
  return (
    <div
      className="mt-8 rounded-2xl border border-border-low bg-card p-6 text-sm text-muted"
      role={role}
    >
      {children}
    </div>
  );
}

function getErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "Unknown RPC error.";
}
