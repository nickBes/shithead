<script setup lang="ts">
import { notificationSettings, states } from "@/game/states"
import { isMatching, P } from "ts-pattern"
import { onMounted, onUnmounted, ref } from "vue"
import { useRouter } from "vue-router"
import { useNotification } from "naive-ui"

const router = useRouter()
let lobbyName = ref<string>()
const notification = useNotification()

function createLobby(event : SubmitEvent) {
    event?.preventDefault()
    if (lobbyName.value != undefined) {
        states.gameSocket?.send({
            createLobby: {
                lobbyName: lobbyName.value
            }
        })
    }
}

onMounted(() => {
    states.gameSocket?.messageHandlers.set("createdLobby", (message) => {
        if (isMatching({joinLobby: P.any}, message)) {
                states.lobby = message.joinLobby
                states.isOwner.value = true
                router.push(`/lobby/${message.joinLobby}`)
                notification.success({title: "Successfully created a lobby", ...notificationSettings})
        }
    })
})

onUnmounted(() => {
    states.gameSocket?.messageHandlers.delete("createdLobby")
})

</script>
<template>
    <p>This is lobby creator</p>
    <form @submit="(event) => createLobby(event as SubmitEvent)">
        <input v-model.lazy.trim="lobbyName" autocomplete="off" name="lobbyName" type="text"/>
        <button type="submit">Create New Lobby</button>
    </form>
</template>