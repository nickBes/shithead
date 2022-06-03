<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';

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

</script>
<template>
    <p>This is lobby #{{rawLobbyId}}</p>
</template>