import { apiClient } from "@/shared/lib/api-client";
import {
  BanMemberRequest,
  CreateTeamRequest,
  TeamMembershipResponse,
  InviteCodeResponse,
  JoinTeamRequest,
  TeamBanResponse,
  TeamMemberResponse,
  TeamResponse,
  TransferManagerRequest,
} from "./types";

export const createTeam = (
  body: CreateTeamRequest,
  token: string,
): Promise<TeamMembershipResponse> =>
  apiClient.post<TeamMembershipResponse>("/teams", body, token);

export const getTeams = (token: string): Promise<TeamResponse[]> =>
  apiClient.get<TeamResponse[]>("/teams", token);

export const getTeam = (teamId: string, token: string): Promise<TeamResponse> =>
  apiClient.get<TeamResponse>(`/teams/${teamId}`, token);

export const joinTeam = (
  body: JoinTeamRequest,
  token: string,
): Promise<TeamMembershipResponse> =>
  apiClient.post<TeamMembershipResponse>("/teams/join", body, token);

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

export const kickMember = (
  teamId: string,
  userId: string,
  token: string,
): Promise<void> =>
  apiClient.post<void>(`/teams/${teamId}/members/${userId}/kick`, {}, token);

export const banMember = (
  teamId: string,
  userId: string,
  body: BanMemberRequest,
  token: string,
): Promise<void> =>
  apiClient.post<void>(`/teams/${teamId}/members/${userId}/ban`, body, token);

export const unbanMember = (
  teamId: string,
  userId: string,
  token: string,
): Promise<void> =>
  apiClient.delete<void>(`/teams/${teamId}/members/${userId}/ban`, token);

export const getTeamBans = (
  teamId: string,
  token: string,
): Promise<TeamBanResponse[]> =>
  apiClient.get<TeamBanResponse[]>(`/teams/${teamId}/bans`, token);
