import type { ExposedLobbyPlayerInfo } from "./ExposedLobbyPlayerInfo";

export interface ExposedLobbyInfo { name: string, id: number, players: Array<ExposedLobbyPlayerInfo>, owner_id: number, }