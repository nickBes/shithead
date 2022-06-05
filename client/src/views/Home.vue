<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { states } from '@/game/states'
import { match, P } from 'ts-pattern'
import type types from '@/bindings/bindings'

const updateTimeout = 1000
let lobbies = ref<types.ExposedLobbyInfo[]>()
let username = ref<string>()
let interval : number

function getLobbies() {
    states.gameSocket?.send("getLobbies")
}

function updateUsername() {
    if (username.value != undefined) {
        states.gameSocket?.send({setUsername: username.value})
        states.name = username.value
    }
}

onMounted(() => {
    states.gameSocket?.setOnMessage(message => {
        match(message)
            .with({lobbies: P.any}, (msg) => lobbies.value = msg.lobbies)
            .otherwise(msg => console.warn(`Recieved a non related message on lobby query: ${JSON.stringify(msg)}`))
    })

    // will get lobbies now and after updateTimeout time
    getLobbies()
    interval = setInterval(getLobbies, updateTimeout)
})

onUnmounted(() => {
    clearInterval(interval)
})

</script>

<template>
    <p>Hello, {{states.name ?? ('user#' + states.id)}}.
    <br/>This is the home page</p>
    <form @submit.prevent="updateUsername">
        <input v-model.lazy.trim="username" type="text" placeholder="new username"/>
        <button type="submit">change username</button>
    </form>
    <ul>
        <template v-for="lobby in lobbies">
            <RouterLink :to="'/lobby/' + lobby.id">{{lobby.name}}</RouterLink>
        </template>
    </ul>
</template>