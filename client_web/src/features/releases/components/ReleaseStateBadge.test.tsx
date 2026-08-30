import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { ReleaseStateBadge } from "./ReleaseStateBadge";
import { ReleaseState } from "../types";
import messages from "../../../../messages/en.json";

const renderBadge = (state: ReleaseState) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <ReleaseStateBadge state={state} />
    </NextIntlClientProvider>,
  );

describe("ReleaseStateBadge", () => {
  it.each([
    ["created", "Created"],
    ["in_progress", "In progress"],
    ["completed", "Completed"],
    ["cancelled", "Cancelled"],
    ["blocked", "Blocked"],
  ] as const)("renders the %s state as %s", (state, label) => {
    renderBadge(state);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it("renders an icon alongside the text for color-blind users", () => {
    renderBadge("blocked");
    const badge = screen.getByText("Blocked").closest("span");
    expect(badge?.querySelector("svg")).toBeInTheDocument();
  });
});
