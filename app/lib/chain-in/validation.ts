import { MAX_HABIT_DESCRIPTION_LEN, MAX_HABIT_NAME_LEN } from "./program";

/** UTF-8 byte length, matching the limits enforced by the Anchor program. */
export function getUtf8ByteLength(value: string): number {
  return new TextEncoder().encode(value).length;
}

/** Returns an error message when the habit name is invalid, or `null`. */
export function validateHabitName(name: string): string | null {
  if (name.trim().length === 0) {
    return "Enter a habit name.";
  }

  const bytes = getUtf8ByteLength(name);
  if (bytes > MAX_HABIT_NAME_LEN) {
    return `Habit name must be ${MAX_HABIT_NAME_LEN} bytes or fewer (currently ${bytes}).`;
  }

  return null;
}

/** Returns an error message when the description is invalid, or `null`. */
export function validateHabitDescription(description: string): string | null {
  const bytes = getUtf8ByteLength(description);
  if (bytes > MAX_HABIT_DESCRIPTION_LEN) {
    return `Description must be ${MAX_HABIT_DESCRIPTION_LEN} bytes or fewer (currently ${bytes}).`;
  }

  return null;
}
