import { useEffect } from "react";
import {
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { getToken } from "@/features/auth/token";
import { useTeams } from "@/features/teams";
import { getTeamMembers } from "@/features/teams/api";
import { useWebSocket } from "@/shared/providers/WebSocketProvider";
import { getConversation, listConversations, sendMessage } from "./api";
import { SendMessageRequest } from "./types";

export interface Contact {
  user_id: string;
  username: string;
}

export const useContacts = (currentUserId: string | undefined) => {
  const { data: teams, isLoading: teamsLoading } = useTeams();

  const memberQueries = useQueries({
    queries: (teams ?? []).map((team) => ({
      queryKey: ["teams", team.id, "members"],
      queryFn: () => getTeamMembers(team.id, getToken()!),
    })),
  });

  const isLoading =
    teamsLoading || memberQueries.some((query) => query.isLoading);

  const contacts = new Map<string, Contact>();
  for (const query of memberQueries) {
    for (const member of query.data ?? []) {
      if (member.user_id === currentUserId) continue;
      contacts.set(member.user_id, {
        user_id: member.user_id,
        username: member.username ?? member.user_id,
      });
    }
  }

  return { contacts: Array.from(contacts.values()), isLoading };
};

export const useConversations = () =>
  useQuery({
    queryKey: ["messages"],
    queryFn: () => listConversations(getToken()!),
  });

export const useConversation = (userId: string) =>
  useQuery({
    queryKey: ["messages", userId],
    queryFn: () => getConversation(userId, getToken()!),
    enabled: !!userId,
  });

export const useSendMessage = (userId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: SendMessageRequest) =>
      sendMessage(userId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    },
  });
};

export const useMessagesRealtime = (myUsername: string | undefined) => {
  const { onMessage } = useWebSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!myUsername) return;

    return onMessage((event) => {
      if (event.type !== "private_message_received") return;
      if (event.from !== myUsername && event.to !== myUsername) return;

      queryClient.invalidateQueries({ queryKey: ["messages"] });
    });
  }, [myUsername, onMessage, queryClient]);
};
