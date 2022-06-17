<script setup lang="ts">
import type types from "@/bindings/bindings";
import cards from "@/game/cards"
import { states } from '@/game/states';
import { isMatching, P } from "ts-pattern";
import { onMounted, ref } from "vue";

let cardsInHand = ref<types.Card[]>([])

onMounted(() => {
    states.gameSocket?.messageHandlers.set("getInitialcards", (message) => {
        if (isMatching({initialCards: P.any}, message)) {
            message.initialCards.cards_in_hand.forEach(cardId => {
                cardsInHand.value.push(cards[cardId])
            })
            states.gameSocket?.messageHandlers.delete("getInitialcards")
        }
    })
})

</script>

<template>
    <p>This is the game page.</p>
    <ol>
       <li v-for="card in cardsInHand">{{card.rank}} of {{card.suit}}</li> 
    </ol>
</template>