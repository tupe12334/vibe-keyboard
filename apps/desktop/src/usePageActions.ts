import type { Dispatch, SetStateAction } from "react";
import { commands } from "./bindings";
import type { Profile } from "./types";

interface Args {
  draft: Profile | null;
  page: number;
  setDraft: (profile: Profile) => void;
  setPage: Dispatch<SetStateAction<number>>;
  setError: (message: string) => void;
  setSelected: (key: number | null) => void;
}

/** Add and delete pages. The core crate names and checks them; this only applies the result. */
export function usePageActions({ draft, page, setDraft, setPage, setError, setSelected }: Args) {
  const addPage = async () => {
    if (!draft) return;
    setDraft(await commands.addPage(draft));
    setPage(draft.pages.length);
  };
  const deletePage = async () => {
    if (!draft) return;
    const result = await commands.deletePage(draft, page);
    if (result.status === "error") {
      setError(result.error);
      return;
    }
    setDraft(result.data);
    setPage((p) => Math.max(0, p - 1));
    setSelected(null);
  };
  return { addPage, deletePage };
}
