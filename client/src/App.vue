<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { onMounted } from 'vue';
import { states } from './game/states'
import Socket from './game/socket'

onMounted(() => {
    // connect when mounted
    states.gameSocket = new Socket('ws://localhost:7522', () => {
        console.log('Connected to server')
    })
})

</script>

<template>
    <nav>
        <RouterLink to="/">Home</RouterLink>
        <RouterLink to="/lobbyCreator">Create Lobby</RouterLink>
    </nav>
    <RouterView/>
</template>

<style scoped>
    nav {
        display: flex;
        flex-direction: column;
    }
</style>