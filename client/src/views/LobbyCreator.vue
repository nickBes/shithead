<script setup lang="ts">
import { states } from "@/game/states"

function createLobby(event : SubmitEvent) {
    event?.preventDefault()
    let formData = new FormData(event.target as HTMLFormElement)
    if (formData.has('lobbyName')) {
        let lobbyName = formData.get('lobbyName')
        if (typeof lobbyName == "string") {
            states.gameSocket?.send({
                createLobby: {
                    lobbyName
                }
            })
        }
    }
}

</script>
<template>
    <p>This is lobby creator</p>
    <form @submit="(event) => createLobby(event as SubmitEvent)">
        <input autocomplete="off" name="lobbyName" type="text"/>
        <button type="submit">Create New Lobby</button>
    </form>
</template>