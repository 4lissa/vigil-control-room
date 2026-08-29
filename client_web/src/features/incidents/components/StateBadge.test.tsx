import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { StateBadge } from "./StateBadge";
import { IncidentState } from "../types";
import messages from "../../../../messages/en.json";

const renderBadge = (state: IncidentState) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <StateBadge state={state} />
    </NextIntlClientProvider>,
  );

describe("StateBadge", () => {
  it.each([
    ["open", "Open"],
    ["acknowledged", "Acknowledged"],
    ["escalated", "Escalated"],
    ["resolved", "Resolved"],
  ] as const)("renders the %s state as %s", (state, label) => {
    renderBadge(state);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it("renders an icon alongside the text for color-blind users", () => {
    renderBadge("escalated");
    const badge = screen.getByText("Escalated").closest("span");
    expect(badge?.querySelector("svg")).toBeInTheDocument();
  });
});
