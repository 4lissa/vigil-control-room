import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NextIntlClientProvider } from "next-intl";
import { IncidentList } from "./IncidentList";
import { IncidentResponse } from "../types";
import messages from "../../../../messages/en.json";

const incident: IncidentResponse = {
  id: "incident-123",
  team_id: "team-123",
  title: "Database down",
  description: "",
  severity: "critical",
  state: "open",
  assigned_to: null,
  release_id: null,
  created_by: "123",
  created_at: 1755000000,
  resolved_at: null,
};

const renderList = (incidents: IncidentResponse[]) =>
  render(
    <NextIntlClientProvider locale="en" messages={messages}>
      <IncidentList teamId="team-123" incidents={incidents} />
    </NextIntlClientProvider>,
  );

describe("IncidentList", () => {
  it("shows an empty state message when there are no incidents", () => {
    renderList([]);
    expect(screen.getByText("No incidents yet.")).toBeInTheDocument();
  });

  it("renders each incident with its title, severity and state", () => {
    renderList([incident]);
    expect(screen.getByText("Database down")).toBeInTheDocument();
    expect(screen.getByText("Critical")).toBeInTheDocument();
    expect(screen.getByText("Open")).toBeInTheDocument();
  });

  it("links each incident to its detail page", () => {
    renderList([incident]);
    expect(screen.getByRole("link")).toHaveAttribute(
      "href",
      "/teams/team-123/incidents/incident-123",
    );
  });
});
