import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '../views/Dashboard.vue'
import Products from '../views/Products.vue'
import Buying from '../views/Buying.vue'
import Selling from '../views/Selling.vue'
import Reports from '../views/Reports.vue'
import Backup from '../views/Backup.vue'
import Settings from '../views/Settings.vue'

const routes = [
  { path: '/', component: Dashboard, name: 'Dashboard' },
  { path: '/products', component: Products, name: 'Products' },
  { path: '/buying', component: Buying, name: 'Buying' },
  { path: '/selling', component: Selling, name: 'Selling' },
  { path: '/reports', component: Reports, name: 'Reports' },
  { path: '/backup', component: Backup, name: 'Backup' },
  { path: '/settings', component: Settings, name: 'Settings' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
