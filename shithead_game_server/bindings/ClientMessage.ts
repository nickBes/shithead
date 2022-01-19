
export type ClientMessage = { t: "setUsername", new_username: string, } | { t: "getLobbies" } | { t: "joinLobby", id: number, } | { t: "createLobby", name: string, } | { t: "startGame" };