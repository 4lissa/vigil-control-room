import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getToken } from "@/features/auth/token";
import { useWebSocket } from "@/shared/providers/WebSocketProvider";
import { createRule, deleteRule, getRules, updateRule } from "./api";
import { CreateRuleRequest, UpdateRuleRequest } from "./types";

export const useRules = (teamId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "rules"],
    queryFn: () => getRules(teamId, getToken()!),
    enabled: !!teamId,
  });

export const useCreateRule = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateRuleRequest) =>
      createRule(teamId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "rules"] });
    },
  });
};

export const useUpdateRule = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      ruleId,
      body,
    }: {
      ruleId: string;
      body: UpdateRuleRequest;
    }) => updateRule(teamId, ruleId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "rules"] });
    },
  });
};

export const useDeleteRule = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (ruleId: string) => deleteRule(teamId, ruleId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams", teamId, "rules"] });
    },
  });
};

export interface RuleEvent {
  id: string;
  ruleName: string;
  status: "triggered" | "failed";
  detail: string;
  at: number;
}

const MAX_FEED_EVENTS = 20;

export const useRuleEventsFeed = (teamId: string) => {
  const { onMessage } = useWebSocket();
  const [events, setEvents] = useState<RuleEvent[]>([]);
  const [lastTeamId, setLastTeamId] = useState(teamId);

  if (teamId !== lastTeamId) {
    setLastTeamId(teamId);
    setEvents([]);
  }

  useEffect(() => {
    return onMessage((event) => {
      if (event.type === "rule_triggered") {
        setEvents((prev) =>
          [
            {
              id: `${Date.now()}-${prev.length}`,
              ruleName: event.rule_name,
              status: "triggered" as const,
              detail: event.result,
              at: Date.now(),
            },
            ...prev,
          ].slice(0, MAX_FEED_EVENTS),
        );
      } else if (event.type === "rule_failed") {
        setEvents((prev) =>
          [
            {
              id: `${Date.now()}-${prev.length}`,
              ruleName: event.rule_name,
              status: "failed" as const,
              detail: event.error,
              at: Date.now(),
            },
            ...prev,
          ].slice(0, MAX_FEED_EVENTS),
        );
      }
    });
  }, [onMessage]);

  return events;
};
