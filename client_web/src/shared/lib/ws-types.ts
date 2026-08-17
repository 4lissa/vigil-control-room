export type WsEvent =
  | { type: "pong" }
  | {
      type: "incident_state_changed";
      incident_id: string;
      new_state: string;
      by: string;
    }
  | {
      type: "incident_escalated";
      incident_id: string;
      new_severity: string;
      by: string;
    }
  | {
      type: "incident_assigned";
      incident_id: string;
      assigned_to: string | null;
      by: string;
    }
  | {
      type: "timeline_entry_added";
      incident_id: string;
      entry_id: string;
      author: string;
      content: string;
      created_at: number;
    }
  | {
      type: "timeline_entry_edited";
      incident_id: string;
      entry_id: string;
      new_content: string;
      edited_at: number;
    }
  | {
      type: "presence_update";
      incident_id: string;
      watchers: string[];
    }
  | {
      type: "release_step_validated";
      release_id: string;
      step: string;
      by: string;
    }
  | {
      type: "release_state_changed";
      release_id: string;
      new_state: string;
    }
  | {
      type: "private_message_received";
      from: string;
      to: string;
      content: string;
      at: number;
    }
  | {
      type: "member_joined";
      team_id: string;
      member: string;
      role: string;
    }
  | {
      type: "member_kicked";
      team_id: string;
      member: string;
      by: string;
    }
  | {
      type: "member_banned";
      team_id: string;
      member: string;
      until: number | null;
      by: string;
    };

export type ClientMessage =
  | { type: "ping" }
  | { type: "watch"; incident_id: string }
  | { type: "unwatch"; incident_id: string };
