import { apiClient } from "@/shared/lib/api-client";
import {
  CreateReleaseRequest,
  ReleaseResponse,
  ReleaseStepResponse,
} from "./types";

export const createRelease = (
  teamId: string,
  body: CreateReleaseRequest,
  token: string,
): Promise<ReleaseResponse> =>
  apiClient.post<ReleaseResponse>(`/teams/${teamId}/releases`, body, token);

export const getReleases = (
  teamId: string,
  token: string,
): Promise<ReleaseResponse[]> =>
  apiClient.get<ReleaseResponse[]>(`/teams/${teamId}/releases`, token);

export const getRelease = (
  teamId: string,
  releaseId: string,
  token: string,
): Promise<ReleaseResponse> =>
  apiClient.get<ReleaseResponse>(
    `/teams/${teamId}/releases/${releaseId}`,
    token,
  );

export const getReleaseSteps = (
  teamId: string,
  releaseId: string,
  token: string,
): Promise<ReleaseStepResponse[]> =>
  apiClient.get<ReleaseStepResponse[]>(
    `/teams/${teamId}/releases/${releaseId}/steps`,
    token,
  );

export const validateStep = (
  teamId: string,
  releaseId: string,
  stepId: string,
  token: string,
): Promise<ReleaseStepResponse> =>
  apiClient.post<ReleaseStepResponse>(
    `/teams/${teamId}/releases/${releaseId}/steps/${stepId}/validate`,
    {},
    token,
  );

export const cancelRelease = (
  teamId: string,
  releaseId: string,
  token: string,
): Promise<ReleaseResponse> =>
  apiClient.post<ReleaseResponse>(
    `/teams/${teamId}/releases/${releaseId}/cancel`,
    {},
    token,
  );
