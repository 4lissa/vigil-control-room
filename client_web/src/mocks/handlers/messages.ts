import { http, HttpResponse } from "msw";

const mockMessage = {
  id: "message-123",
  sender_id: "123",
  recipient_id: "456",
  content: "Hey, got a minute?",
  created_at: 1755000000,
};

export const messagesHandlers = [
  http.get("http://localhost:8080/messages", () => {
    return HttpResponse.json([mockMessage]);
  }),

  http.get("http://localhost:8080/messages/:userId", () => {
    return HttpResponse.json([mockMessage]);
  }),

  http.post("http://localhost:8080/messages/:userId", () => {
    return HttpResponse.json(mockMessage, { status: 201 });
  }),
];
