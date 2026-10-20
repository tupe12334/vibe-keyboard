import { QuitDialog } from "./QuitDialog";
import { useQuitPrompt } from "./useQuitPrompt";

interface Props {
  dirty: boolean;
  save: () => Promise<boolean>;
}

/** Answers quit requests. Shows the unsaved-edits prompt when the layout is dirty. */
export function QuitPrompt({ dirty, save }: Props) {
  const quit = useQuitPrompt(dirty, save);
  if (!quit.asking) return null;
  return <QuitDialog onSave={() => void quit.saveAndQuit()} onDiscard={quit.discard} onCancel={quit.cancel} />;
}
