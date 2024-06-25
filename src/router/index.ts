import type { RouteRecordRaw, Router } from 'vue-router'
import { createRouter, createWebHashHistory } from 'vue-router'

import Backup from '../views/Backup/Backup.vue'
import Config from '../views/Config/Config.vue'
import Mission from '../views/Mission/Mission.vue'
import Statistic from '../views/Statistic/Statistic.vue'
import Procedure from '../views/Procedure/Procedure.vue'
import Screensaver from '../views/Screensaver/Screensaver.vue'

/**
 * Router record array
 */
const routes: RouteRecordRaw[] = [
  { path: '/', component: Mission },
  { path: '/backup', component: Backup },
  { path: '/config', component: Config },
  { path: '/mission', component: Mission },
  { path: '/statistic', component: Statistic },
  { path: '/procedure', component: Procedure },
  { path: '/screensaver', component: Screensaver },
]

/**
 * Create router instance
 */
const router: Router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior(_to: any, _from: any, _savedPosition: any) {
    return {
      top: 0,
      behavior: 'smooth',
    }
  },
})

export default router
