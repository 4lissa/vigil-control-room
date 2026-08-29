import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { RulesLink } from "./RulesLink";
import messages from "../../../../messages/en.json";

describe("RulesLink", () => {
  it("links to the team's automation rules page", () => {
    render(
      <NextIntlClientProvider locale="en" messages={messages}>
        <RulesLink teamId="team-123" />
      </NextIntlClientProvider>,
    );

    expect(
      screen.getByRole("link", { name: "Automation rules" }),
    ).toHaveAttribute("href", "/teams/team-123/rules");
  });
});
