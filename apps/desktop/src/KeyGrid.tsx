import { KEYS } from "./deckLayout";

interface Props {
  navKeys: Set<number>;
  selected: number | null;
  pressed: number | null;
  previews: Record<number, string>;
  onSelect: (key: number) => void;
}

/** The deck as a grid of key tiles. Navigation keys are shown but cannot be edited. */
export function KeyGrid({ navKeys, selected, pressed, previews, onSelect }: Props) {
  return (
    <div className="grid">
      {KEYS.map((key) => {
        const nav = navKeys.has(key);
        const classes = ["key", selected === key && "selected", pressed === key && "pressed"]
          .filter(Boolean)
          .join(" ");
        return (
          <button
            key={key}
            className={classes}
            disabled={nav}
            title={nav ? "Navigation key" : `Key ${key}`}
            onClick={() => onSelect(key)}
          >
            {previews[key] && <img src={previews[key]} alt={`Key ${key}`} />}
            <span className="num">{key}</span>
          </button>
        );
      })}
    </div>
  );
}
