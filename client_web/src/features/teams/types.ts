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

export interface BanMemberRequest {
  until: number | null;
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
  username?: string;
  role: TeamRole;
  joined_at: string;
}

export interface TeamMembershipResponse {
  team: TeamResponse;
  member: TeamMemberResponse;
}

export interface TeamBanResponse {
  id: string;
  team_id: string;
  user_id: string;
  username: string;
  banned_by: string | null;
  until: number | null;
  created_at: number;
}
