import type Socket from "./socket"
import type types from "@/bindings/bindings"
import { ref } from "vue"
import type { NotificationOptions, NotificationPlacement } from "naive-ui"

export const notificationPlacement : NotificationPlacement = "bottom-right"

export const notificationSettings : NotificationOptions = {
    duration: 3000,
}

// global states which can be accessed and modified from every component
export const states = {
    // game socket is not a ref as it shouldn't update other components
    // when created or re-assigned
    gameSocket: undefined as (Socket | undefined),
    lobby: undefined as (types.LobbyId | undefined),
    players: ref<Map<types.ClientId, string>>(new Map()),
    id: undefined as (types.ClientId | undefined),
    isOwner: ref<boolean>(false),
    isInGame: false,
    name: undefined as (string | undefined),
}