export interface SendMessageRequest {
  content: string;
}

export interface MessageResponse {
  id: string;
  sender_id: string;
  recipient_id: string;
  content: string;
  created_at: number;
}
