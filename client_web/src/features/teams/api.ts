import { apiClient } from "@/shared/lib/api-client";
import {
  CreateTeamRequest,
  CreateTeamResponse,
  InviteCodeResponse,
  JoinTeamRequest,
  TeamMemberResponse,
  TeamResponse,
  TransferManagerRequest,
} from "./types";

export const createTeam = (
  body: CreateTeamRequest,
  token: string,
): Promise<CreateTeamResponse> =>
  apiClient.post<CreateTeamResponse>("/teams", body, token);

export const getTeams = (token: string): Promise<TeamResponse[]> =>
  apiClient.get<TeamResponse[]>("/teams", token);

export const getTeam = (teamId: string, token: string): Promise<TeamResponse> =>
  apiClient.get<TeamResponse>(`/teams/${teamId}`, token);

export const joinTeam = (
  body: JoinTeamRequest,
  token: string,
): Promise<TeamResponse> =>
  apiClient.post<TeamResponse>("/teams/join", body, token);

export const getTeamMembers = (
  teamId: string,
  token: string,
): Promise<TeamMemberResponse[]> =>
  apiClient.get<TeamMemberResponse[]>(`/teams/${teamId}/members`, token);

export const generateInviteCode = (
  teamId: string,
  token: string,
): Promise<InviteCodeResponse> =>
  apiClient.post<InviteCodeResponse>(`/teams/${teamId}/invite`, {}, token);

export const transferManager = (
  teamId: string,
  body: TransferManagerRequest,
  token: string,
): Promise<void> =>
  apiClient.post<void>(`/teams/${teamId}/transfer`, body, token);
