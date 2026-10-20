import type { Page } from "./types";

interface Props {
  pages: Page[];
  page: number;
  onSelect: (index: number) => void;
  onAdd: () => void;
}

/** Left column: one button per page, then Add page. */
export function PageList({ pages, page, onSelect, onAdd }: Props) {
  return (
    <aside className="pages">
      <strong>Pages</strong>
      {pages.map((p, i) => (
        <button key={i} className={`page${i === page ? " active" : ""}`} onClick={() => onSelect(i)}>
          {p.name}
        </button>
      ))}
      <button onClick={onAdd}>Add page</button>
    </aside>
  );
}
