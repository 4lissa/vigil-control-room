import { apiClient } from "@/shared/lib/api-client";
import { MessageResponse, SendMessageRequest } from "./types";

export const listConversations = (token: string): Promise<MessageResponse[]> =>
  apiClient.get<MessageResponse[]>("/messages", token);

export const sendMessage = (
  userId: string,
  body: SendMessageRequest,
  token: string,
): Promise<MessageResponse> =>
  apiClient.post<MessageResponse>(`/messages/${userId}`, body, token);

export const getConversation = (
  userId: string,
  token: string,
): Promise<MessageResponse[]> =>
  apiClient.get<MessageResponse[]>(`/messages/${userId}`, token);
