import { useUiStore } from "../state/uiStore";
import { en } from "./en";
import { ko, type Messages } from "./ko";

export type { Messages } from "./ko";

const all = { ko, en };

export function useMessages(): Messages {
  return all[useUiStore((s) => s.locale)];
}
