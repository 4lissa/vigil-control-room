import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getToken } from "@/features/auth/token";
import {
  createTeam,
  generateInviteCode,
  getTeam,
  getTeamMembers,
  getTeams,
  joinTeam,
  transferManager,
} from "./api";
import {
  CreateTeamRequest,
  JoinTeamRequest,
  TransferManagerRequest,
} from "./types";

export const useTeams = () =>
  useQuery({
    queryKey: ["teams"],
    queryFn: () => getTeams(getToken()!),
  });

export const useTeam = (teamId: string) =>
  useQuery({
    queryKey: ["teams", teamId],
    queryFn: () => getTeam(teamId, getToken()!),
  });

export const useTeamMembers = (teamId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "members"],
    queryFn: () => getTeamMembers(teamId, getToken()!),
    enabled: !!teamId,
  });

export const useCreateTeam = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateTeamRequest) => createTeam(body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
};

export const useJoinTeam = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: JoinTeamRequest) => joinTeam(body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
};

export const useGenerateInviteCode = (teamId: string) =>
  useMutation({
    mutationFn: () => generateInviteCode(teamId, getToken()!),
  });

export const useTransferManager = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: TransferManagerRequest) =>
      transferManager(teamId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "members"] });
    },
  });
};
