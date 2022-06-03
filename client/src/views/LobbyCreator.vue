<script setup lang="ts">
import { states } from "@/game/states"
import { match, P } from "ts-pattern";
import { onMounted } from "vue";
import { useRouter } from "vue-router";

const router = useRouter()

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

onMounted(() => {
    states.gameSocket?.setOnMessage(message => {
        match(message)
            .with({joinLobby: P.any}, msg => {
                router.push(`/lobby/${msg.joinLobby}`)
            })
            .otherwise(msg => console.warn(`Recieved a non related message on lobby creator: ${JSON.stringify(msg)}`))
    })
})

</script>
<template>
    <p>This is lobby creator</p>
    <form @submit="(event) => createLobby(event as SubmitEvent)">
        <input autocomplete="off" name="lobbyName" type="text"/>
        <button type="submit">Create New Lobby</button>
    </form>
</template>