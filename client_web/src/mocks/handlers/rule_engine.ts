import { http, HttpResponse } from "msw";

const mockRule = {
  id: "rule-123",
  team_id: "team-123",
  name: "CI failure > Incident",
  enabled: true,
  trigger: {
    service: "github",
    event: "workflow_run",
    filters: {
      "workflow_run.conclusion": "failure",
      "repository.full_name": "my-org/my-repo",
    },
  },
  reaction: {
    type: "vigil_create_incident",
    payload: {
      title: "CI broken on {{repository.full_name}}",
      severity: "high",
    },
  },
  created_by: "123",
  created_at: 1755000000,
};

export const ruleEngineHandlers = [
  http.get("http://localhost:8080/teams/:teamId/rules", () => {
    return HttpResponse.json([mockRule]);
  }),

  http.post("http://localhost:8080/teams/:teamId/rules", () => {
    return HttpResponse.json(mockRule, { status: 201 });
  }),

  http.patch("http://localhost:8080/teams/:teamId/rules/:ruleId", () => {
    return HttpResponse.json({ ...mockRule, enabled: false });
  }),

  http.delete("http://localhost:8080/teams/:teamId/rules/:ruleId", () => {
    return new HttpResponse(null, { status: 204 });
  }),
];
