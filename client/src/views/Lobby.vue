<script setup lang="ts">
import { states } from '@/game/states';
import { match, P } from 'ts-pattern';
import { onMounted } from 'vue';
import { useRouter, useRoute, onBeforeRouteLeave } from 'vue-router';

const router = useRouter()
const route = useRoute()
let rawLobbyId : any = route.params.id
let lobbyId : number

if (rawLobbyId && typeof rawLobbyId == "string") {
    lobbyId = parseInt(rawLobbyId)
    if (isNaN(lobbyId)) {
        router.push("/")
    }
} else {
    router.push("/")
}

onBeforeRouteLeave(() => {
    states.gameSocket?.send("leaveLobby")
})

onMounted(() => {
    if (states.lobby != lobbyId) { // then we either joined or switched
        states.gameSocket?.setOnMessage((message, sk) => {
            match(message)
                .with({joinLobby: P.any}, () => {
                    states.lobby = lobbyId
                    sk.setOnMessage(() => {})
                })
                .otherwise((msg) => {
                    console.warn(`Couldn't join the lobby for the following reason: ${JSON.stringify(msg)}`)
                    router.push("/")
                })
        })
        states.gameSocket?.send({joinLobby: lobbyId})
    }
})

</script>
<template>
    <p>This is lobby #{{rawLobbyId}}</p>
</template>