import type { EditorRules, KeyConfig_Serialize as KeyConfig } from "./bindings";
import { BLANK_KEY } from "./KeyPanel";
import { HoldFields } from "./HoldFields";

interface Props {
  /** Undefined when the key has no config yet. */
  config: KeyConfig | undefined;
  builtins: string[];
  pageCount: number;
  holdMs: number;
  rules: EditorRules;
  onHoldMsChange: (ms: number) => void;
  onChange: (config: KeyConfig) => void;
}

/** The hold of the selected key: its action, and the hold time shared by all keys. */
export function HoldPanel({ config, builtins, pageCount, holdMs, rules, onHoldMsChange, onChange }: Props) {
  const value = config ?? BLANK_KEY;
  return (
    <HoldFields
      hold={value.hold ?? null}
      holdMs={holdMs}
      rules={rules}
      builtins={builtins}
      pageCount={pageCount}
      onChange={(hold) => onChange({ ...value, hold })}
      onHoldMsChange={onHoldMsChange}
    />
  );
}
