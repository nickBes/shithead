
export type ClientMessage = { username: string } | "getLobbies" | { joinLobby: number } | { createLobby: { name: string, } };