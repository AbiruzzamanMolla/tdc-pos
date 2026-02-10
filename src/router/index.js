import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import Dashboard from '../views/Dashboard.vue'
import Products from '../views/Products.vue'
import Buying from '../views/Buying.vue'
import Selling from '../views/Selling.vue'
import Stocks from '../views/Stocks.vue'
import Reports from '../views/Reports.vue'
import Backup from '../views/Backup.vue'
import Settings from '../views/Settings.vue'
import Login from '../views/Login.vue'

const routes = [
  { path: '/login', component: Login, name: 'Login', meta: { public: true } },
  { path: '/', component: Dashboard, name: 'Dashboard' },
  { path: '/products', component: Products, name: 'Products' },
  { path: '/buying', component: Buying, name: 'Buying' },
  { path: '/selling', component: Selling, name: 'Selling' },
  { path: '/stocks', component: Stocks, name: 'Stocks' },
  { path: '/reports', component: Reports, name: 'Reports' },
  { path: '/backup', component: Backup, name: 'Backup', meta: { requiresAdmin: true } },
  { path: '/settings', component: Settings, name: 'Settings', meta: { requiresAdmin: true } },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to, from, next) => {
  const auth = useAuthStore()
  if (!to.meta.public && !auth.isAuthenticated) {
    next('/login')
  } else if (to.meta.requiresAdmin && !auth.isAdmin) {
    next('/')
  } else if (to.name === 'Login' && auth.isAuthenticated) {
    next('/')
  } else {
    next()
  }
})

export default router
