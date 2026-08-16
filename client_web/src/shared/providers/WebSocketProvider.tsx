"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
} from "react";
import { getToken } from "@/features/auth/token";
import { WsClient } from "@/shared/lib/ws-client";
import { ClientMessage, WsEvent } from "@/shared/lib/ws-types";

interface WebSocketContextValue {
  sendMessage: (msg: ClientMessage) => void;
  onMessage: (handler: (event: WsEvent) => void) => () => void;
}

const WebSocketContext = createContext<WebSocketContextValue | null>(null);

export const useWebSocket = () => {
  const ctx = useContext(WebSocketContext);
  if (!ctx)
    throw new Error("useWebSocket must be used within WebSocketProvider");
  return ctx;
};

export const WebSocketProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const clientRef = useRef<WsClient | null>(null);
  if (clientRef.current === null) {
    clientRef.current = new WsClient(getToken);
  }

  useEffect(() => {
    const client = clientRef.current!;
    client.connect();

    return () => {
      client.disconnect();
    };
  }, []);

  const sendMessage = useCallback((msg: ClientMessage) => {
    clientRef.current?.send(msg);
  }, []);

  const onMessage = useCallback((handler: (event: WsEvent) => void) => {
    if (!clientRef.current) return () => {};
    return clientRef.current.onMessage(handler);
  }, []);

  return (
    <WebSocketContext.Provider value={{ sendMessage, onMessage }}>
      {children}
    </WebSocketContext.Provider>
  );
};
