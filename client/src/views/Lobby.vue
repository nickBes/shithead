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
    console.log('leaving lobby....')
    // will be added later
})

onMounted(() => {
    if (states.lobby != lobbyId) { // then we either joined or switched
        states.gameSocket?.setOnMessage((message) => {
            match(message)
                .with(P.not({joinLobby: P.any}), (msg) => {
                    console.warn(`Couldn't join the lobby for the following reason: ${JSON.stringify(msg)}`)
                    router.push("/")
                })
                .run()
        })
        states.gameSocket?.send({joinLobby: lobbyId})
    }
})

</script>
<template>
    <p>This is lobby #{{rawLobbyId}}</p>
</template>