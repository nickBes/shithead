<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { states } from '@/game/states'
import { match, P } from 'ts-pattern'
import type types from '@/bindings/bindings'

const delta = 1000
let lobbies = ref<types.ExposedLobbyInfo[]>()
let interval = ref<number>()

onMounted(() => {
    states.gameSocket?.setOnMessage(message => {
        match(message)
            .with({lobbies: P.any}, (msg) => lobbies.value = msg.lobbies)
            .otherwise(msg => console.warn(`Recieved a non related message on lobby query: ${JSON.stringify(msg)}`))
    })

    interval.value = setInterval(() => {states.gameSocket?.send("getLobbies")}, delta)
})

onUnmounted(() => {
    clearInterval(interval.value)
})

</script>

<template>
    <p>This is the home page</p>
    <ul>
        <li v-for="lobby in lobbies">{{lobby.name}}</li>
    </ul>
</template>