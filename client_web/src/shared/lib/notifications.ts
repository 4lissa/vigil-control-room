"use client";

import { useEffect } from "react";
import { useTranslations } from "next-intl";
import { isTauri } from "@tauri-apps/api/core";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { useWebSocket } from "@/shared/providers/WebSocketProvider";

const notify = async (title: string, body: string) => {
  if (!isTauri()) return;

  let granted = await isPermissionGranted();
  if (!granted) {
    granted = (await requestPermission()) === "granted";
  }
  if (granted) sendNotification({ title, body });
};

export const useNativeNotifications = (userId: string | undefined) => {
  const { onMessage } = useWebSocket();
  const incidentsT = useTranslations("incidents");
  const releasesT = useTranslations("releases");

  useEffect(() => {
    if (!userId) return;

    return onMessage((event) => {
      if (event.type === "incident_assigned" && event.assigned_to === userId) {
        notify(
          incidentsT("notificationAssignedTitle"),
          incidentsT("notificationAssignedBody"),
        );
      }

      if (
        event.type === "incident_escalated" &&
        event.new_severity === "critical"
      ) {
        notify(
          incidentsT("notificationCriticalTitle"),
          incidentsT("notificationCriticalBody"),
        );
      }

      if (
        event.type === "release_state_changed" &&
        event.new_state === "blocked"
      ) {
        notify(
          releasesT("notificationBlockedTitle"),
          releasesT("notificationBlockedBody"),
        );
      }
    });
  }, [userId, onMessage, incidentsT, releasesT]);
};
