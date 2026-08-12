export type TeamRole = "observer" | "responder" | "manager";

export interface CreateTeamRequest {
  name: string;
}

export interface JoinTeamRequest {
  code: string;
}

export interface TransferManagerRequest {
  user_id: string;
}

export interface TeamResponse {
  id: string;
  name: string;
  created_by: string | null;
  created_at: string;
}

export interface InviteCodeResponse {
  invitation_code: string;
}

export interface TeamMemberResponse {
  id: string;
  team_id: string;
  user_id: string;
  role: TeamRole;
  joined_at: string;
}

export interface CreateTeamResponse {
  team: TeamResponse;
  member: TeamMemberResponse;
}
