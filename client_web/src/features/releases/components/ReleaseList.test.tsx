import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { ReleaseList } from "./ReleaseList";
import { ReleaseResponse } from "../types";
import messages from "../../../../messages/en.json";

const release: ReleaseResponse = {
  id: "release-123",
  team_id: "team-123",
  name: "v1.0.0",
  state: "in_progress",
  created_by: "123",
  created_at: 1755000000,
  completed_at: null,
};

const renderList = (releases: ReleaseResponse[]) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <ReleaseList teamId="team-123" releases={releases} />
    </NextIntlClientProvider>,
  );

describe("ReleaseList", () => {
  it("shows an empty state message when there are no releases", () => {
    renderList([]);
    expect(screen.getByText("No releases yet.")).toBeInTheDocument();
  });

  it("renders each release with its name and state", () => {
    renderList([release]);
    expect(screen.getByText("v1.0.0")).toBeInTheDocument();
    expect(screen.getByText("In progress")).toBeInTheDocument();
  });

  it("links each release to its detail page", () => {
    renderList([release]);
    expect(screen.getByRole("link")).toHaveAttribute(
      "href",
      "/teams/team-123/releases/release-123",
    );
  });
});
