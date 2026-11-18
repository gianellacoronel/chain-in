"use client";

import { useState, type FormEvent } from "react";
import {
  MAX_HABIT_DESCRIPTION_LEN,
  MAX_HABIT_NAME_LEN,
  getUtf8ByteLength,
  validateHabitDescription,
  validateHabitName,
  type DecodedAccount,
  type Profile,
} from "../../lib/chain-in";
import { useChainInActions } from "../../lib/hooks/use-chain-in-actions";

type CreateHabitFormProps = {
  profile: DecodedAccount<Profile>;
  /** Called after every create attempt so the profile and list can refresh. */
  onSettled: () => void;
};

export function CreateHabitForm({ profile, onSettled }: CreateHabitFormProps) {
  const { createHabit, isSending } = useChainInActions();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [nameTouched, setNameTouched] = useState(false);

  const nameBytes = getUtf8ByteLength(name);
  const descriptionBytes = getUtf8ByteLength(description);
  const nameError = validateHabitName(name);
  const descriptionError = validateHabitDescription(description);
  const canSubmit =
    nameError === null && descriptionError === null && !isSending;

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setNameTouched(true);

    if (!canSubmit) return;

    const signature = await createHabit({
      owner: profile.data.authority,
      profile,
      name: name.trim(),
      description: description.trim(),
    });

    if (signature) {
      setName("");
      setDescription("");
      setNameTouched(false);
    }

    onSettled();
  };

  return (
    <section className="rounded-2xl border border-border-low bg-card p-6">
      <h2 className="text-sm font-semibold">Create a habit</h2>
      <p className="mt-1 text-xs leading-relaxed text-muted">
        Habits are stored on-chain and owned by your wallet. Names are limited
        to {MAX_HABIT_NAME_LEN} bytes and descriptions to{" "}
        {MAX_HABIT_DESCRIPTION_LEN} bytes.
      </p>

      <form className="mt-4 space-y-4" onSubmit={handleSubmit} noValidate>
        <div className="space-y-1.5">
          <div className="flex items-baseline justify-between gap-3">
            <label htmlFor="habit-name" className="text-xs font-medium">
              Name
            </label>
            <span
              className={`text-[11px] tabular-nums ${nameBytes > MAX_HABIT_NAME_LEN ? "text-destructive" : "text-muted"}`}
            >
              {nameBytes}/{MAX_HABIT_NAME_LEN}
            </span>
          </div>
          <input
            id="habit-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            onBlur={() => setNameTouched(true)}
            disabled={isSending}
            placeholder="e.g. Read 10 pages"
            autoComplete="off"
            aria-invalid={nameTouched && nameError != null}
            aria-describedby={
              nameTouched && nameError ? "habit-name-error" : undefined
            }
            className="w-full rounded-lg border border-border-low bg-background px-3 py-2 text-sm outline-none focus:border-ring disabled:opacity-50"
          />
          {nameTouched && nameError && (
            <p id="habit-name-error" className="text-xs text-destructive">
              {nameError}
            </p>
          )}
        </div>

        <div className="space-y-1.5">
          <div className="flex items-baseline justify-between gap-3">
            <label htmlFor="habit-description" className="text-xs font-medium">
              Description{" "}
              <span className="font-normal text-muted">(optional)</span>
            </label>
            <span
              className={`text-[11px] tabular-nums ${descriptionBytes > MAX_HABIT_DESCRIPTION_LEN ? "text-destructive" : "text-muted"}`}
            >
              {descriptionBytes}/{MAX_HABIT_DESCRIPTION_LEN}
            </span>
          </div>
          <textarea
            id="habit-description"
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            disabled={isSending}
            rows={3}
            placeholder="Why does this habit matter to you?"
            aria-invalid={descriptionError != null}
            aria-describedby={
              descriptionError ? "habit-description-error" : undefined
            }
            className="w-full resize-none rounded-lg border border-border-low bg-background px-3 py-2 text-sm outline-none focus:border-ring disabled:opacity-50"
          />
          {descriptionError && (
            <p
              id="habit-description-error"
              className="text-xs text-destructive"
            >
              {descriptionError}
            </p>
          )}
        </div>

        <button
          type="submit"
          disabled={!canSubmit}
          className="w-full cursor-pointer rounded-lg bg-primary px-4 py-2.5 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
        >
          {isSending ? "Creating habit..." : "Create habit"}
        </button>
      </form>
    </section>
  );
}
