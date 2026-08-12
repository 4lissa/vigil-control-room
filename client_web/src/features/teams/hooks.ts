import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useAuthStore } from "@/features/auth/store";
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

export const useTeams = () => {
  const { token } = useAuthStore();

  return useQuery({
    queryKey: ["teams"],
    queryFn: () => getTeams(token!),
    enabled: !!token,
  });
};

export const useTeam = (teamId: string) => {
  const { token } = useAuthStore();

  return useQuery({
    queryKey: ["teams", teamId],
    queryFn: () => getTeam(teamId, token!),
    enabled: !!token,
  });
};

export const useTeamMembers = (teamId: string) => {
  const { token } = useAuthStore();

  return useQuery({
    queryKey: ["teams", teamId, "members"],
    queryFn: () => getTeamMembers(teamId, token!),
    enabled: !!token,
  });
};

export const useCreateTeam = () => {
  const { token } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateTeamRequest) => createTeam(body, token!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
};

export const useJoinTeam = () => {
  const { token } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: JoinTeamRequest) => joinTeam(body, token!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
    },
  });
};

export const useGenerateInviteCode = (teamId: string) => {
  const { token } = useAuthStore();

  return useMutation({
    mutationFn: () => generateInviteCode(teamId, token!),
  });
};

export const useTransferManager = (teamId: string) => {
  const { token } = useAuthStore();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: TransferManagerRequest) =>
      transferManager(teamId, body, token!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "members"] });
    },
  });
};
