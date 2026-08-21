import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getToken } from "@/features/auth/token";
import { useWebSocket } from "@/shared/providers/WebSocketProvider";
import {
  acknowledgeIncident,
  addReaction,
  addTimelineEntry,
  assignResponder,
  createIncident,
  editTimelineEntry,
  escalateIncident,
  getAvailableEmojis,
  getIncident,
  getIncidents,
  getReactions,
  getTimelineEntries,
  removeReaction,
  resolveIncident,
} from "./api";
import {
  AddTimelineEntryRequest,
  AssignResponderRequest,
  CreateIncidentRequest,
  EditTimelineEntryRequest,
  Emoji,
  EscalateIncidentRequest,
} from "./types";

export const useIncidents = (teamId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "incidents"],
    queryFn: () => getIncidents(teamId, getToken()!),
    enabled: !!teamId,
  });

export const useIncident = (teamId: string, incidentId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "incidents", incidentId],
    queryFn: () => getIncident(teamId, incidentId, getToken()!),
    enabled: !!teamId && !!incidentId,
  });

export const useCreateIncident = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateIncidentRequest) =>
      createIncident(teamId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents"],
      });
    },
  });
};

export const useAcknowledgeIncident = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => acknowledgeIncident(teamId, incidentId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents"],
      });
    },
  });
};

export const useEscalateIncident = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: EscalateIncidentRequest) =>
      escalateIncident(teamId, incidentId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents"],
      });
    },
  });
};

export const useResolveIncident = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => resolveIncident(teamId, incidentId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents"],
      });
    },
  });
};

export const useAssignResponder = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: AssignResponderRequest) =>
      assignResponder(teamId, incidentId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents"],
      });
    },
  });
};

export const useTimelineEntries = (teamId: string, incidentId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "incidents", incidentId, "timeline"],
    queryFn: () => getTimelineEntries(teamId, incidentId, getToken()!),
    enabled: !!teamId && !!incidentId,
  });

export const useAddTimelineEntry = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: AddTimelineEntryRequest) =>
      addTimelineEntry(teamId, incidentId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents", incidentId, "timeline"],
      });
    },
  });
};

export const useEditTimelineEntry = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      entryId,
      body,
    }: {
      entryId: string;
      body: EditTimelineEntryRequest;
    }) => editTimelineEntry(teamId, incidentId, entryId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents", incidentId, "timeline"],
      });
    },
  });
};

export const useAvailableEmojis = () =>
  useQuery({
    queryKey: ["reactions", "available"],
    queryFn: () => getAvailableEmojis(getToken()!),
  });

export const useReactions = (teamId: string, incidentId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "incidents", incidentId, "reactions"],
    queryFn: () => getReactions(teamId, incidentId, getToken()!),
    enabled: !!teamId && !!incidentId,
  });

export const useAddReaction = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ entryId, emoji }: { entryId: string; emoji: Emoji }) =>
      addReaction(teamId, incidentId, entryId, { emoji }, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents", incidentId, "reactions"],
      });
    },
  });
};

export const useRemoveReaction = (teamId: string, incidentId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ entryId, emoji }: { entryId: string; emoji: Emoji }) =>
      removeReaction(teamId, incidentId, entryId, emoji, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents", incidentId, "reactions"],
      });
    },
  });
};

export const useIncidentsRealtime = (teamId: string) => {
  const { onMessage } = useWebSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    return onMessage((event) => {
      if (
        event.type === "incident_state_changed" ||
        event.type === "incident_escalated" ||
        event.type === "incident_assigned" ||
        event.type === "rule_triggered"
      ) {
        queryClient.invalidateQueries({
          queryKey: ["teams", teamId, "incidents"],
        });
      }
    });
  }, [teamId, onMessage, queryClient]);
};

export const useIncidentRealtime = (teamId: string, incidentId: string) => {
  const { sendMessage, onMessage } = useWebSocket();
  const queryClient = useQueryClient();
  const [watchers, setWatchers] = useState<string[]>([]);

  useEffect(() => {
    if (!incidentId) return;

    sendMessage({ type: "watch", incident_id: incidentId });

    const unsubscribe = onMessage((event) => {
      if (!("incident_id" in event) || event.incident_id !== incidentId) {
        return;
      }

      if (event.type === "presence_update") {
        setWatchers(event.watchers);
        return;
      }

      if (
        event.type === "timeline_entry_added" ||
        event.type === "timeline_entry_edited"
      ) {
        queryClient.invalidateQueries({
          queryKey: ["teams", teamId, "incidents", incidentId, "timeline"],
        });
        return;
      }

      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "incidents", incidentId],
      });
    });

    return () => {
      sendMessage({ type: "unwatch", incident_id: incidentId });
      unsubscribe();
    };
  }, [teamId, incidentId, sendMessage, onMessage, queryClient]);

  return { watchers };
};
