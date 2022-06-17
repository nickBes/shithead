import type Socket from "./socket"
import type types from "@/bindings/bindings"
import { ref } from "vue"
import type { NotificationOptions, NotificationPlacement } from "naive-ui"

export const notificationPlacement : NotificationPlacement = "bottom-right"

export const notificationSettings : NotificationOptions = {
    duration: 3000,
}

type LobbyState = 'inGame' | 'inLobby'

// global states which can be accessed and modified from every component
export const states = {
    // game socket is not a ref as it shouldn't update other components
    // when created or re-assigned
    gameSocket: undefined as (Socket | undefined),
    lobbyId: undefined as (types.LobbyId | undefined),
    lobbyState: ref<LobbyState | null>(null),
    players: ref<Map<types.ClientId, string>>(new Map()),
    id: undefined as (types.ClientId | undefined),
    isOwner: ref<boolean>(false),
    name: undefined as (string | undefined),
}