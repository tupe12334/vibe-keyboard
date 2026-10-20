/** A labelled form field. The label wraps the input so clicking the text focuses it. */
export function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return <label>{label}{children}</label>;
}
