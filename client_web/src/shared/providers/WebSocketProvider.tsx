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

  useEffect(() => {
    const client = new WsClient(getToken);
    clientRef.current = client;
    client.connect();

    return () => {
      client.disconnect();
      clientRef.current = null;
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
