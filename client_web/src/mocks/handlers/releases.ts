import { http, HttpResponse } from "msw";

const mockRelease = {
  id: "release-123",
  team_id: "team-123",
  name: "v1.0.0",
  state: "in_progress",
  created_by: "123",
  created_at: 1755000000,
  completed_at: null,
};

const mockStep = {
  id: "step-123",
  release_id: "release-123",
  name: "staging",
  position: 0,
  validated_at: null,
  validated_by: null,
};

export const releasesHandlers = [
  http.get("http://localhost:8080/teams/:teamId/releases", () => {
    return HttpResponse.json([mockRelease]);
  }),

  http.get("http://localhost:8080/teams/:teamId/releases/:releaseId", () => {
    return HttpResponse.json(mockRelease);
  }),

  http.post("http://localhost:8080/teams/:teamId/releases", () => {
    return HttpResponse.json(mockRelease, { status: 201 });
  }),

  http.get(
    "http://localhost:8080/teams/:teamId/releases/:releaseId/steps",
    () => {
      return HttpResponse.json([mockStep]);
    },
  ),

  http.post(
    "http://localhost:8080/teams/:teamId/releases/:releaseId/steps/:stepId/validate",
    () => {
      return HttpResponse.json({
        ...mockStep,
        validated_at: 1755001000,
        validated_by: "123",
      });
    },
  ),

  http.post(
    "http://localhost:8080/teams/:teamId/releases/:releaseId/cancel",
    () => {
      return HttpResponse.json({ ...mockRelease, state: "cancelled" });
    },
  ),
];
