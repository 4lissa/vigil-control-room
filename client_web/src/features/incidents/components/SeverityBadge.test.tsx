import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { SeverityBadge } from "./SeverityBadge";
import { Severity } from "../types";
import messages from "../../../../messages/en.json";

const renderBadge = (severity: Severity) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <SeverityBadge severity={severity} />
    </NextIntlClientProvider>,
  );

describe("SeverityBadge", () => {
  it.each([
    ["low", "Low"],
    ["medium", "Medium"],
    ["high", "High"],
    ["critical", "Critical"],
  ] as const)("renders the %s severity as %s", (severity, label) => {
    renderBadge(severity);
    expect(screen.getByText(label)).toBeInTheDocument();
  });

  it("renders an icon alongside the text for color-blind users", () => {
    renderBadge("critical");
    const badge = screen.getByText("Critical").closest("span");
    expect(badge?.querySelector("svg")).toBeInTheDocument();
  });
});
