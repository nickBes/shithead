<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { states } from '@/game/states'
import { isMatching, P } from 'ts-pattern'
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
    states.gameSocket?.messageHandlers.set("getLobbies", (message) => {
        if (isMatching({lobbies: P.any}, message)) {
            lobbies.value = message.lobbies
        }
    })
    // will get lobbies now and after updateTimeout time
    getLobbies()
    interval = setInterval(getLobbies, updateTimeout)
})

onUnmounted(() => {
    states.gameSocket?.messageHandlers.delete("getLobbies")
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
            <li><RouterLink :to="'/lobby/' + lobby.id">{{lobby.name}}</RouterLink></li>
        </template>
    </ul>
</template>