export interface CreateReleaseRequest {
  name: string;
  steps: string[];
}

export type ReleaseState =
  "created" | "in_progress" | "completed" | "cancelled" | "blocked";

export interface ReleaseResponse {
  id: string;
  team_id: string;
  name: string;
  state: ReleaseState;
  created_by: string | null;
  created_at: number;
  completed_at: number | null;
}

export interface ReleaseStepResponse {
  id: string;
  release_id: string;
  name: string;
  position: number;
  validated_at: number | null;
  validated_by: string | null;
}
