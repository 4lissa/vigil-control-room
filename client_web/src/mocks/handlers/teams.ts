import { http, HttpResponse } from "msw";

const mockTeam = {
  id: "team-123",
  name: "My Team",
  created_by: "123",
  created_at: "2026-08-12T00:00:00Z",
};

const mockMember = {
  id: "member-123",
  team_id: "team-123",
  user_id: "123",
  role: "manager",
  joined_at: "2026-08-12T00:00:00Z",
};

const mockMemberWithUsername = {
  ...mockMember,
  username: "alissa",
};

const mockBan = {
  id: "ban-123",
  team_id: "team-123",
  user_id: "456",
  username: "bob",
  banned_by: "123",
  until: null,
  created_at: 1755000000,
};

export const teamsHandlers = [
  http.get("http://localhost:8080/teams", () => {
    return HttpResponse.json([mockTeam]);
  }),

  http.get("http://localhost:8080/teams/:teamId", () => {
    return HttpResponse.json(mockTeam);
  }),

  http.get("http://localhost:8080/teams/:teamId/members", () => {
    return HttpResponse.json([mockMemberWithUsername]);
  }),

  http.post("http://localhost:8080/teams", () => {
    return HttpResponse.json(
      { team: mockTeam, member: mockMember },
      { status: 201 },
    );
  }),

  http.post("http://localhost:8080/teams/join", () => {
    return HttpResponse.json({ team: mockTeam, member: mockMember });
  }),

  http.post("http://localhost:8080/teams/:teamId/invite", () => {
    return HttpResponse.json({ invitation_code: "ABC123" });
  }),

  http.post("http://localhost:8080/teams/:teamId/transfer", () => {
    return new HttpResponse(null, { status: 204 });
  }),

  http.get("http://localhost:8080/teams/:teamId/bans", () => {
    return HttpResponse.json([mockBan]);
  }),

  http.post("http://localhost:8080/teams/:teamId/members/:userId/kick", () => {
    return new HttpResponse(null, { status: 204 });
  }),

  http.post("http://localhost:8080/teams/:teamId/members/:userId/ban", () => {
    return new HttpResponse(null, { status: 204 });
  }),

  http.delete("http://localhost:8080/teams/:teamId/members/:userId/ban", () => {
    return new HttpResponse(null, { status: 204 });
  }),
];
