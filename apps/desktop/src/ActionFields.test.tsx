import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ActionFields } from "./ActionFields";

describe("ActionFields", () => {
  it("shows the URL of an open_url action and reports edits", () => {
    const onChange = vi.fn();
    render(
      <ActionFields
        action={{ type: "open_url", url: "https://example.com" }}
        builtins={[]}
        pageCount={1}
        onChange={onChange}
      />,
    );

    const input = screen.getByLabelText("URL");
    expect(input).toHaveProperty("value", "https://example.com");

    fireEvent.change(input, { target: { value: "https://example.org" } });
    expect(onChange).toHaveBeenCalledWith({ type: "open_url", url: "https://example.org" });
  });
});
