import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      component: () => import("@/views/Home.vue")
    },
    {
      path: '/lobbyCreator',
      component: () => import("@/views/LobbyCreator.vue"),
    },
    {
      path: '/lobby/:id',
      component: () => import("@/views/Lobby.vue")
    }
  ]
})

export default router