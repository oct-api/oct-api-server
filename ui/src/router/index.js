import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import LoginConfirm from '../views/login-confirm.vue'
import Dashboard from '../views/dashboard.vue'
import AppOverview from '../views/app-overview.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/feedback',
    name: 'Feedback',
    component: () => import('../views/feedback.vue')
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: Dashboard
  },
  {
    path: '/app/:appname',
    name: 'AppOverview',
    props: true,
    component: AppOverview
  },
  {
    path: '/login/confirm/:token',
    name: 'LoginConfirm',
    component: LoginConfirm
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
