import { authHandlers } from "./handlers/auth";
import { teamsHandlers } from "./handlers/teams";

export const handlers = [...authHandlers, ...teamsHandlers];
