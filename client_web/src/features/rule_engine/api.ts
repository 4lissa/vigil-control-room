import { apiClient } from "@/shared/lib/api-client";
import { CreateRuleRequest, RuleResponse, UpdateRuleRequest } from "./types";

export const createRule = (
  teamId: string,
  body: CreateRuleRequest,
  token: string,
): Promise<RuleResponse> =>
  apiClient.post<RuleResponse>(`/teams/${teamId}/rules`, body, token);

export const getRules = (
  teamId: string,
  token: string,
): Promise<RuleResponse[]> =>
  apiClient.get<RuleResponse[]>(`/teams/${teamId}/rules`, token);

export const updateRule = (
  teamId: string,
  ruleId: string,
  body: UpdateRuleRequest,
  token: string,
): Promise<RuleResponse> =>
  apiClient.patch<RuleResponse>(
    `/teams/${teamId}/rules/${ruleId}`,
    body,
    token,
  );

export const deleteRule = (
  teamId: string,
  ruleId: string,
  token: string,
): Promise<void> =>
  apiClient.delete<void>(`/teams/${teamId}/rules/${ruleId}`, token);
