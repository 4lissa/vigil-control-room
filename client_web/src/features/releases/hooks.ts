import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getToken } from "@/features/auth/token";
import { useWebSocket } from "@/shared/providers/WebSocketProvider";
import {
  cancelRelease,
  createRelease,
  getRelease,
  getReleases,
  getReleaseSteps,
  validateStep,
} from "./api";
import { CreateReleaseRequest } from "./types";

export const useReleases = (teamId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "releases"],
    queryFn: () => getReleases(teamId, getToken()!),
    enabled: !!teamId,
  });

export const useRelease = (teamId: string, releaseId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "releases", releaseId],
    queryFn: () => getRelease(teamId, releaseId, getToken()!),
    enabled: !!teamId && !!releaseId,
  });

export const useReleaseSteps = (teamId: string, releaseId: string) =>
  useQuery({
    queryKey: ["teams", teamId, "releases", releaseId, "steps"],
    queryFn: () => getReleaseSteps(teamId, releaseId, getToken()!),
    enabled: !!teamId && !!releaseId,
  });

export const useCreateRelease = (teamId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateReleaseRequest) =>
      createRelease(teamId, body, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "releases"],
      });
    },
  });
};

export const useValidateStep = (teamId: string, releaseId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (stepId: string) =>
      validateStep(teamId, releaseId, stepId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "releases"],
      });
    },
  });
};

export const useCancelRelease = (teamId: string, releaseId: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => cancelRelease(teamId, releaseId, getToken()!),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "releases"],
      });
    },
  });
};

export const useReleasesRealtime = (teamId: string) => {
  const { onMessage } = useWebSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    return onMessage((event) => {
      if (
        event.type === "release_step_validated" ||
        event.type === "release_state_changed"
      ) {
        queryClient.invalidateQueries({
          queryKey: ["teams", teamId, "releases"],
        });
      }
    });
  }, [teamId, onMessage, queryClient]);
};

export const useReleaseRealtime = (teamId: string, releaseId: string) => {
  const { onMessage } = useWebSocket();
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!releaseId) return;

    return onMessage((event) => {
      if (!("release_id" in event) || event.release_id !== releaseId) {
        return;
      }

      if (event.type === "release_step_validated") {
        queryClient.invalidateQueries({
          queryKey: ["teams", teamId, "releases", releaseId, "steps"],
        });
      }

      queryClient.invalidateQueries({
        queryKey: ["teams", teamId, "releases", releaseId],
      });
    });
  }, [teamId, releaseId, onMessage, queryClient]);
};
