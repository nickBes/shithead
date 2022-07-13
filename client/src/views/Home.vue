<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { states } from '@/game/states'
import { isMatching, P } from 'ts-pattern'
import FuzzySearch from 'fuzzy-search'
import { NInput, NList, NListItem, NButton, NThing, NScrollbar } from 'naive-ui'
import type types from '@/bindings/bindings'

const updateTimeout = 1000
let lobbies = ref<types.ExposedLobbyInfo[]>([])
let username = ref<string>()
let interval : number
let lobbySearchInput = ref<string>()

function getLobbies() {
    states.gameSocket?.send("getLobbies")
}

function updateUsername() {
    if (username.value != undefined) {
        states.gameSocket?.send({setUsername: username.value})
        states.name.value = username.value
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

const filteredLobbies = computed(() => {
    if (lobbySearchInput.value == undefined) return lobbies.value
    // sorting lobbies by their names from the most relevant to the least
    const fuzzy = new FuzzySearch(lobbies.value, ['name'], {sort: true})
    return fuzzy.search(lobbySearchInput.value)
})

</script>

<template>
    <p>Hello, {{states.name.value}}
    <br/>This is the home page</p>
    <form @submit.prevent="updateUsername">
        <input v-model.lazy.trim="username" type="text" placeholder="new username"/>
        <button type="submit">change username</button>
    </form>
    <n-list bordered>
        <template #header>
            <n-input placeholder="Filter lobbies by name" v-model:value="lobbySearchInput"/>
        </template>
            <!-- the height limit is temporary just to demonstrate the scrollbar -->
            <n-scrollbar style="max-height: 150px">
                    <n-list-item v-for="lobby in filteredLobbies">
                        <n-thing :title="lobby.name"/>
                        <template #suffix>
                            <RouterLink :to="'/lobby/' + lobby.id">
                                <n-button>Join</n-button>
                            </RouterLink>
                        </template>
                    </n-list-item>
            </n-scrollbar>

    </n-list>
</template>