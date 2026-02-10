import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '../views/Dashboard.vue'
import Products from '../views/Products.vue'
import Purchases from '../views/Purchases.vue'
import Orders from '../views/Orders.vue'
import Reports from '../views/Reports.vue'
import Backup from '../views/Backup.vue'
import Settings from '../views/Settings.vue'

const routes = [
  { path: '/', component: Dashboard, name: 'Dashboard' },
  { path: '/products', component: Products, name: 'Products' },
  { path: '/purchases', component: Purchases, name: 'Purchases' },
  { path: '/orders', component: Orders, name: 'Orders' },
  { path: '/reports', component: Reports, name: 'Reports' },
  { path: '/backup', component: Backup, name: 'Backup' },
  { path: '/settings', component: Settings, name: 'Settings' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
