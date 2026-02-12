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
import Users from '../views/Users.vue'
import ActivityLog from '../views/ActivityLog.vue'
import Chat from '../views/Chat.vue'

const routes = [
  { path: '/login', component: Login, name: 'Login', meta: { public: true } },
  { path: '/', component: Dashboard, name: 'Dashboard' },
  { path: '/products', component: Products, name: 'Products', meta: { permission: 'canManageProducts' } },
  { path: '/buying', component: Buying, name: 'Buying', meta: { permission: 'canBuy' } },
  { path: '/selling', component: Selling, name: 'Selling', meta: { permission: 'canSell' } },
  { path: '/stocks', component: Stocks, name: 'Stocks', meta: { permission: 'canViewStock' } },
  { path: '/reports', component: Reports, name: 'Reports', meta: { permission: 'canViewReports' } },
  { path: '/backup', component: Backup, name: 'Backup', meta: { permission: 'canManageBackup' } },
  { path: '/settings', component: Settings, name: 'Settings', meta: { permission: 'canManageSettings' } },
  { path: '/users', component: Users, name: 'Users', meta: { permission: 'canManageUsers' } },
  { path: '/activity-log', component: ActivityLog, name: 'ActivityLog', meta: { permission: 'canViewActivityLog' } },
  { path: '/chat', component: Chat, name: 'Chat' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.beforeEach((to, from, next) => {
  const auth = useAuthStore()
  if (!to.meta.public && !auth.isAuthenticated) {
    next('/login')
  } else if (to.meta.permission && !auth[to.meta.permission]) {
    next('/')
  } else if (to.name === 'Login' && auth.isAuthenticated) {
    next('/')
  } else {
    next()
  }
})

export default router
