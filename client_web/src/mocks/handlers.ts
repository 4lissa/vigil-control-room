import { authHandlers } from "./handlers/auth";
import { teamsHandlers } from "./handlers/teams";
import { incidentsHandlers } from "./handlers/incidents";
import { releasesHandlers } from "./handlers/releases";
import { messagesHandlers } from "./handlers/messages";

export const handlers = [
  ...authHandlers,
  ...teamsHandlers,
  ...incidentsHandlers,
  ...releasesHandlers,
  ...messagesHandlers,
];
