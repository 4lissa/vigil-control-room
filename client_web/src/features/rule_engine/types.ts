export interface TriggerDto {
  service: string;
  event: string;
  filters: Record<string, string>;
}

export interface ReactionDto {
  type: string;
  payload: Record<string, string>;
}

export interface CreateRuleRequest {
  name: string;
  enabled: boolean;
  trigger: TriggerDto;
  webhook_secret: string;
  reaction: ReactionDto;
}

export interface UpdateRuleRequest {
  enabled: boolean;
}

export interface RuleResponse {
  id: string;
  team_id: string;
  name: string;
  enabled: boolean;
  trigger: TriggerDto;
  reaction: ReactionDto;
  created_by: string | null;
  created_at: number;
}
