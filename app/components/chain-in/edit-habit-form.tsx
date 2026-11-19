"use client";

import { useId, useState, type FormEvent } from "react";
import {
  MAX_HABIT_DESCRIPTION_LEN,
  MAX_HABIT_NAME_LEN,
  getUtf8ByteLength,
  validateHabitDescription,
  validateHabitName,
  type DecodedAccount,
  type Habit,
} from "../../lib/chain-in";
import { useChainInActions } from "../../lib/hooks/use-chain-in-actions";

type EditHabitFormProps = {
  habit: DecodedAccount<Habit>;
  /** Called after every update attempt so the list can refresh. */
  onSettled: () => void;
  onCancel: () => void;
};

/**
 * Edits a habit's name and description. The program does not restrict updates
 * by status, so this form is also used for paused and archived habits.
 */
export function EditHabitForm({
  habit,
  onSettled,
  onCancel,
}: EditHabitFormProps) {
  const { updateHabit, isSending } = useChainInActions();
  const id = useId();
  const nameId = `${id}-name`;
  const descriptionId = `${id}-description`;
  const [name, setName] = useState(habit.data.name);
  const [description, setDescription] = useState(habit.data.description);
  const [nameTouched, setNameTouched] = useState(false);

  const nameBytes = getUtf8ByteLength(name);
  const descriptionBytes = getUtf8ByteLength(description);
  const nameError = validateHabitName(name);
  const descriptionError = validateHabitDescription(description);
  const trimmedName = name.trim();
  const trimmedDescription = description.trim();
  const isUnchanged =
    trimmedName === habit.data.name &&
    trimmedDescription === habit.data.description;
  const canSubmit =
    nameError === null &&
    descriptionError === null &&
    !isUnchanged &&
    !isSending;

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setNameTouched(true);

    if (!canSubmit) return;

    const signature = await updateHabit({
      owner: habit.data.owner,
      habit,
      name: trimmedName,
      description: trimmedDescription,
    });

    onSettled();

    if (signature) {
      onCancel();
    }
  };

  return (
    <form onSubmit={handleSubmit} noValidate>
      <div className="space-y-1.5">
        <div className="flex items-baseline justify-between gap-3">
          <label htmlFor={nameId} className="text-xs font-medium">
            Name
          </label>
          <span
            className={`text-[11px] tabular-nums ${nameBytes > MAX_HABIT_NAME_LEN ? "text-destructive" : "text-muted"}`}
          >
            {nameBytes}/{MAX_HABIT_NAME_LEN}
          </span>
        </div>
        <input
          id={nameId}
          value={name}
          onChange={(event) => setName(event.target.value)}
          onBlur={() => setNameTouched(true)}
          disabled={isSending}
          autoComplete="off"
          aria-invalid={nameTouched && nameError != null}
          aria-describedby={
            nameTouched && nameError ? `${id}-name-error` : undefined
          }
          className="w-full rounded-lg border border-border-low bg-background px-3 py-2 text-sm outline-none focus:border-ring disabled:opacity-50"
        />
        {nameTouched && nameError && (
          <p id={`${id}-name-error`} className="text-xs text-destructive">
            {nameError}
          </p>
        )}
      </div>

      <div className="mt-4 space-y-1.5">
        <div className="flex items-baseline justify-between gap-3">
          <label htmlFor={descriptionId} className="text-xs font-medium">
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
          id={descriptionId}
          value={description}
          onChange={(event) => setDescription(event.target.value)}
          disabled={isSending}
          rows={3}
          aria-invalid={descriptionError != null}
          aria-describedby={
            descriptionError ? `${id}-description-error` : undefined
          }
          className="w-full resize-none rounded-lg border border-border-low bg-background px-3 py-2 text-sm outline-none focus:border-ring disabled:opacity-50"
        />
        {descriptionError && (
          <p
            id={`${id}-description-error`}
            className="text-xs text-destructive"
          >
            {descriptionError}
          </p>
        )}
      </div>

      <div className="mt-4 flex items-center gap-2">
        <button
          type="submit"
          disabled={!canSubmit}
          className="cursor-pointer rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
        >
          {isSending ? "Saving..." : "Save changes"}
        </button>
        <button
          type="button"
          onClick={onCancel}
          disabled={isSending}
          className="cursor-pointer rounded-lg border border-border-low bg-card px-4 py-2 text-sm font-medium transition hover:bg-cream disabled:pointer-events-none disabled:opacity-50"
        >
          Cancel
        </button>
        {isUnchanged && !nameError && !descriptionError && (
          <span className="text-[11px] text-muted">No changes yet</span>
        )}
      </div>
    </form>
  );
}
