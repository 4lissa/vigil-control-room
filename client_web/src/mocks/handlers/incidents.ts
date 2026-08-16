import { http, HttpResponse } from "msw";

const mockIncident = {
  id: "incident-123",
  team_id: "team-123",
  title: "DB down",
  description: "Investigating a connection timeout",
  state: "open",
  severity: "high",
  created_by: "123",
  assigned_to: null,
  created_at: 1755000000,
  resolved_at: null,
};

const mockTimelineEntry = {
  id: "entry-123",
  incident_id: "incident-123",
  author_id: "123",
  content: "Investigating",
  created_at: 1755000000,
  edited_at: null,
};

export const incidentsHandlers = [
  http.get("http://localhost:8080/teams/:teamId/incidents", () => {
    return HttpResponse.json([mockIncident]);
  }),

  http.get("http://localhost:8080/teams/:teamId/incidents/:incidentId", () => {
    return HttpResponse.json(mockIncident);
  }),

  http.post("http://localhost:8080/teams/:teamId/incidents", () => {
    return HttpResponse.json(mockIncident, { status: 201 });
  }),

  http.post(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/acknowledge",
    () => {
      return HttpResponse.json({ ...mockIncident, state: "acknowledged" });
    },
  ),

  http.post(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/escalate",
    () => {
      return HttpResponse.json({
        ...mockIncident,
        state: "escalated",
        severity: "critical",
      });
    },
  ),

  http.post(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/resolve",
    () => {
      return HttpResponse.json({
        ...mockIncident,
        state: "resolved",
        resolved_at: 1755001000,
      });
    },
  ),

  http.post(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/assign",
    () => {
      return HttpResponse.json({ ...mockIncident, assigned_to: "123" });
    },
  ),

  http.get(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline",
    () => {
      return HttpResponse.json([mockTimelineEntry]);
    },
  ),

  http.post(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline",
    () => {
      return HttpResponse.json(mockTimelineEntry, { status: 201 });
    },
  ),

  http.patch(
    "http://localhost:8080/teams/:teamId/incidents/:incidentId/timeline/:entryId",
    () => {
      return HttpResponse.json({
        ...mockTimelineEntry,
        content: "Updated",
        edited_at: 1755002000,
      });
    },
  ),
];
