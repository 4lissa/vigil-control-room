export interface CreateIncidentRequest {
  title: string;
  description?: string;
  severity: Severity;
  release_id?: string;
}

export interface EscalateIncidentRequest {
  severity?: Severity;
}

export interface AssignResponderRequest {
  user_id: string | null;
}

export interface AddTimelineEntryRequest {
  content: string;
}

export interface EditTimelineEntryRequest {
  content: string;
}

export type Severity = "low" | "medium" | "high" | "critical";

export type IncidentState = "open" | "acknowledged" | "escalated" | "resolved";

export interface IncidentResponse {
  id: string;
  team_id: string;
  title: string;
  description: string;
  state: IncidentState;
  severity: Severity;
  created_by: string | null;
  assigned_to: string | null;
  release_id: string | null;
  created_at: number;
  resolved_at: number | null;
}

export interface TimelineEntryResponse {
  id: string;
  incident_id: string;
  author_id: string | null;
  content: string;
  created_at: number;
  edited_at: number | null;
}
