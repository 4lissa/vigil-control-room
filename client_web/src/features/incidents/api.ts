import { apiClient } from "@/shared/lib/api-client";
import {
  AddTimelineEntryRequest,
  AssignResponderRequest,
  CreateIncidentRequest,
  EditTimelineEntryRequest,
  EscalateIncidentRequest,
  IncidentResponse,
  TimelineEntryResponse,
} from "./types";

export const getIncidents = (
  teamId: string,
  token: string,
): Promise<IncidentResponse[]> =>
  apiClient.get<IncidentResponse[]>(`/teams/${teamId}/incidents`, token);

export const getIncident = (
  teamId: string,
  incidentId: string,
  token: string,
): Promise<IncidentResponse> =>
  apiClient.get<IncidentResponse>(
    `/teams/${teamId}/incidents/${incidentId}`,
    token,
  );

export const createIncident = (
  teamId: string,
  body: CreateIncidentRequest,
  token: string,
): Promise<IncidentResponse> =>
  apiClient.post<IncidentResponse>(`/teams/${teamId}/incidents`, body, token);

export const acknowledgeIncident = (
  teamId: string,
  incidentId: string,
  token: string,
): Promise<IncidentResponse> =>
  apiClient.post<IncidentResponse>(
    `/teams/${teamId}/incidents/${incidentId}/acknowledge`,
    {},
    token,
  );

export const escalateIncident = (
  teamId: string,
  incidentId: string,
  body: EscalateIncidentRequest,
  token: string,
): Promise<IncidentResponse> =>
  apiClient.post<IncidentResponse>(
    `/teams/${teamId}/incidents/${incidentId}/escalate`,
    body,
    token,
  );

export const resolveIncident = (
  teamId: string,
  incidentId: string,
  token: string,
): Promise<IncidentResponse> =>
  apiClient.post<IncidentResponse>(
    `/teams/${teamId}/incidents/${incidentId}/resolve`,
    {},
    token,
  );

export const assignResponder = (
  teamId: string,
  incidentId: string,
  body: AssignResponderRequest,
  token: string,
): Promise<IncidentResponse> =>
  apiClient.post<IncidentResponse>(
    `/teams/${teamId}/incidents/${incidentId}/assign`,
    body,
    token,
  );

export const getTimelineEntries = (
  teamId: string,
  incidentId: string,
  token: string,
): Promise<TimelineEntryResponse[]> =>
  apiClient.get<TimelineEntryResponse[]>(
    `/teams/${teamId}/incidents/${incidentId}/timeline`,
    token,
  );

export const addTimelineEntry = (
  teamId: string,
  incidentId: string,
  body: AddTimelineEntryRequest,
  token: string,
): Promise<TimelineEntryResponse> =>
  apiClient.post<TimelineEntryResponse>(
    `/teams/${teamId}/incidents/${incidentId}/timeline`,
    body,
    token,
  );

export const editTimelineEntry = (
  teamId: string,
  incidentId: string,
  entryId: string,
  body: EditTimelineEntryRequest,
  token: string,
): Promise<TimelineEntryResponse> =>
  apiClient.patch<TimelineEntryResponse>(
    `/teams/${teamId}/incidents/${incidentId}/timeline/${entryId}`,
    body,
    token,
  );
