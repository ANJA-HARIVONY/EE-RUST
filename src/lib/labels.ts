import { messages, translateError } from "./i18n/index.svelte";

export function errText(error: unknown): string {
  if (typeof error === "string") return translateError(error);
  if (error instanceof Error) return translateError(error.message);
  return messages().common.unexpected;
}

export function fullName(first: string, last: string): string {
  return `${first} ${last}`.trim();
}
