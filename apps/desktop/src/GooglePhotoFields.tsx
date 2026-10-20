import { useState } from "react";
import { commands } from "./bindings";
import { Field } from "./Field";

/** Paste a Google Photos share link; the app downloads the cover photo as the key image. */
export function GooglePhotoFields({ onChange }: { onChange: (image: string | null) => void }) {
  const [link, setLink] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchPhoto = async () => {
    setBusy(true);
    setError(null);
    const result = await commands.importGooglePhoto(link);
    setBusy(false);
    if (result.status === "ok") onChange(result.data);
    else setError(result.error);
  };

  return (
    <div>
      <Field label="Or a Google Photos share link">
        <input
          value={link}
          placeholder="https://photos.app.goo.gl/…"
          onChange={(e) => setLink(e.target.value)}
        />
      </Field>
      <button disabled={busy || !link} onClick={() => void fetchPhoto()}>
        {busy ? "Downloading…" : "Use photo"}
      </button>
      {error && <div className="error">{error}</div>}
    </div>
  );
}
