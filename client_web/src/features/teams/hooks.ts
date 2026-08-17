import { useEffect } from "react";
import { usePathname, useRouter } from "next/navigation";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getToken } from "@/features/auth/token";
import { useWebSocket } from "@/shared/providers/WebSocketProvider";
import {
  banMember,
  createTeam,
  generateInviteCode,
  getTeam,
  getTeamBans,
  getTeamMembers,
  getTeams,
  joinTeam,
  kickMember,
  transferManager,
  unbanMember,
} from "./api";
import { getDefaultTeamId } from "./lastTeamId";
import {
  BanMemberRequest,
  CreateTeamRequest,
  JoinTeamRequest,
  TransferManagerRequest,
} from "./types";

export const useTeams = () =>
  useQuery({
    queryKey: ["teams"],
    queryFn: () => getTeams(getToken()!),
  });

export const useDefaultTeamId = () => {
  const { data: teams } = useTeams();
  return getDefaultTeamId(teams);
};

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

export const useTeamBans = (teamId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "bans"],
    queryFn: () => getTeamBans(teamId, getToken()!),
    enabled: !!teamId,
  });

export const useKickMember = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (userId: string) => kickMember(teamId, userId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "members"] });
    },
  });
};

export const useBanMember = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      userId,
      body,
    }: {
      userId: string;
      body: BanMemberRequest;
    }) => banMember(teamId, userId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId] });
    },
  });
};

export const useUnbanMember = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (userId: string) => unbanMember(teamId, userId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "bans"] });
    },
  });
};

export const useTeamRealtime = (myUsername: string | undefined) => {
  const { onMessage } = useWebSocket();
  const queryClient = useQueryClient();
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    if (!myUsername) return;

    return onMessage((event) => {
      if (
        event.type !== "member_joined" &&
        event.type !== "member_kicked" &&
        event.type !== "member_banned"
      ) {
        return;
      }

      queryClient.invalidateQueries({ queryKey: ["teams", event.team_id] });

      if (event.type === "member_joined" || event.member !== myUsername) {
        return;
      }

      queryClient.invalidateQueries({ queryKey: ["teams"] });

      if (pathname.startsWith(`/teams/${event.team_id}`)) {
        router.replace("/incidents");
      }
    });
  }, [myUsername, pathname, router, onMessage, queryClient]);
};
