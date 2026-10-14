import { reactive } from 'vue'
import { api } from './api'

export const auth = reactive({ user: null, ready: false, appName: 'RustMVC' })
export async function loadSession() {
  const data = await api('/auth/session')
  auth.user = data.user
  auth.appName = data.app_name
  auth.ready = true
}
export async function login(credentials) {
  await loadSession()
  const data = await api('/auth/login', { method: 'POST', body: credentials })
  auth.user = data.user
}
export async function logout() {
  await api('/auth/logout', { method: 'POST' })
  auth.user = null
}
