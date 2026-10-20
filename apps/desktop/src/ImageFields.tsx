import { useState } from "react";
import { AppIconFields } from "./AppIconFields";
import { Field } from "./Field";

type Source = "none" | "file" | "app";

const APP_PREFIX = "app:";

function sourceOf(image: string | null): Source {
  if (!image) return "none";
  return image.startsWith(APP_PREFIX) ? "app" : "file";
}

/** Where a key's image comes from: none, a typed path or icon name, or an installed app's icon. */
export function ImageFields({
  image,
  preview,
  onChange,
}: {
  image: string | null;
  preview?: string;
  onChange: (image: string | null) => void;
}) {
  const [source, setSource] = useState<Source>(() => sourceOf(image));
  const choose = (next: Source) => {
    setSource(next);
    if (next === "none") onChange(null);
    if (next === "file" && image?.startsWith(APP_PREFIX)) onChange(null);
    if (next === "app" && !image?.startsWith(APP_PREFIX)) onChange(APP_PREFIX);
  };

  return (
    <div>
      <Field label="Image">
        <select value={source} onChange={(e) => choose(e.target.value as Source)}>
          <option value="none">None</option>
          <option value="file">Path or icon</option>
          <option value="app">App icon</option>
        </select>
      </Field>
      {source === "file" && (
        <Field label="Relative path, or icon:name">
          <input value={image ?? ""} onChange={(e) => onChange(e.target.value || null)} />
        </Field>
      )}
      {source === "app" && <AppIconFields image={image} preview={preview} onChange={onChange} />}
    </div>
  );
}
