import { invoke, isTauri } from '@tauri-apps/api/core'
export const isDesktop = isTauri()
let csrfToken = ''
export class ApiError extends Error {
  constructor(message, status, errors = {}) {
    super(message)
    this.status = status
    this.errors = errors
  }
}
export async function api(path, options = {}) {
  let response
  try {
    if (isDesktop) {
      const result = await invoke('desktop_request', {
        request: {
          path: `/api${path}`,
          method: options.method || 'GET',
          body: options.body ?? null,
          csrf: csrfToken || null,
        },
      })
      response = {
        status: result.status,
        ok: result.status >= 200 && result.status < 300,
        json: async () => result.data,
      }
    } else
      response = await fetch(`/api${path}`, {
        credentials: 'same-origin',
        ...options,
        headers: {
          Accept: 'application/json',
          ...(options.body ? { 'Content-Type': 'application/json' } : {}),
          ...(csrfToken ? { 'X-CSRF-Token': csrfToken } : {}),
          ...options.headers,
        },
        body: options.body ? JSON.stringify(options.body) : undefined,
      })
  } catch {
    throw new ApiError(
      isDesktop
        ? 'Cannot complete this action. Reopen the application and try again.'
        : 'Cannot reach the server. Check your connection and try again.',
      0,
    )
  }
  const data = response.status === 204 ? {} : await response.json().catch(() => ({}))
  if (!response.ok) {
    if (response.status === 401) window.dispatchEvent(new Event('session-expired'))
    if (response.status === 403 && data.errors?.csrf) {
      // The server rejected this write before the handler. Refresh state, never replay it.
      const session = await api('/auth/session')
      window.dispatchEvent(new CustomEvent('session-refreshed', { detail: session }))
      if (!session.user) window.dispatchEvent(new Event('session-expired'))
    }
    throw new ApiError(
      data.message || `Request failed (${response.status}).`,
      response.status,
      data.errors,
    )
  }
  if ('csrf_token' in data) csrfToken = data.csrf_token
  return data
}
