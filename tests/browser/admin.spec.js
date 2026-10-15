import { test, expect } from '@playwright/test'

async function login(page) {
  await page.goto('/login')
  await page.getByLabel('Email address', { exact: true }).fill('admin@example.com')
  await page.getByLabel('Password', { exact: true }).fill('e2e-test-password-123')
  await page.getByRole('button', { name: 'Sign in to workspace' }).click()
  await expect(
    page.getByRole('heading', { name: 'A little clarity. A lot of possibility.' }),
  ).toBeVisible()
}

test('admin can manage users and sessions with a persistent backend', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1050 })
  await login(page)
  await expect(page.getByText('Loading your workspace…')).not.toBeVisible()
  await page.screenshot({ path: 'test-results/dashboard-desktop.png', fullPage: true })
  await page.getByRole('link', { name: 'Add user', exact: true }).click()
  await page.getByLabel('Full name').fill('Browser Test User')
  await page.getByLabel('Email address', { exact: true }).fill('browser@example.com')
  await page.getByLabel('Password', { exact: true }).fill('browser-test-password')
  await page.getByRole('button', { name: 'Create user' }).click()
  await expect(page.getByText('Your new user is ready.')).toBeVisible()
  await page.getByRole('searchbox', { name: 'Search users' }).fill('browser@example.com')
  await expect(
    page.getByRole('cell', { name: 'Browser Test User browser@example.com' }),
  ).toBeVisible()
  await page.reload()
  await page.getByRole('button', { name: 'Edit Browser Test User' }).click()
  await page.getByLabel('Full name').fill('Updated Browser User')
  await page.getByRole('button', { name: 'Save changes' }).click()
  await expect(page.getByText('User updated successfully.')).toBeVisible()
  await page.getByRole('button', { name: 'Delete Updated Browser User' }).click()
  await page.getByRole('button', { name: 'Delete user', exact: true }).click()
  await expect(page.getByText('User deleted.', { exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Edit Updated Browser User' })).toHaveCount(0)
  await page.getByRole('button', { name: 'Sign out', exact: true }).click()
  await expect(page).toHaveURL(/\/login$/)
  await page.goto('/users')
  await expect(page).toHaveURL(/\/login$/)
})

test('mobile navigation, profile persistence, and maximum corner radius', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await login(page)
  await expect(page.getByText('Loading your workspace…')).not.toBeVisible()
  await page.screenshot({ path: 'test-results/dashboard-mobile.png', fullPage: true })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  )
  await page.getByRole('button', { name: 'Open navigation' }).click()
  await page.getByRole('link', { name: 'Settings', exact: true }).click()
  await page.getByLabel('Full name').fill('Workspace Admin Updated')
  await page.getByRole('button', { name: 'Save changes' }).click()
  await expect(page.getByText('Your profile has been updated.')).toBeVisible()
  await page.reload()
  await expect(page.getByLabel('Full name')).toHaveValue('Workspace Admin Updated')
  const excessiveCorners = await page.evaluate(() =>
    [...document.querySelectorAll('body *')]
      .filter((el) => {
        if (!el.getClientRects().length) return false
        const css = getComputedStyle(el)
        return [
          'borderTopLeftRadius',
          'borderTopRightRadius',
          'borderBottomLeftRadius',
          'borderBottomRightRadius',
        ].some((prop) => parseFloat(css[prop]) > 4)
      })
      .map((el) => el.className),
  )
  expect(excessiveCorners).toEqual([])
})

test('production document routes preserve HTTP status and accessibility', async ({ page }) => {
  const { default: axe } = await import('axe-core')
  async function accessible() {
    await page.addScriptTag({ content: axe.source })
    const violations = await page.evaluate(async () =>
      (
        await window.axe.run(document, {
          runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] },
        })
      ).violations.map(({ id, nodes }) => ({
        id,
        nodes: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })),
      })),
    )
    expect(violations).toEqual([])
  }
  expect((await page.goto('/login')).status()).toBe(200)
  await accessible()
  await login(page)
  await expect(page.getByText('Loading your workspace…')).not.toBeVisible()
  await accessible()
  for (const path of ['/users', '/settings']) {
    expect((await page.goto(path)).status()).toBe(200)
    if (path === '/users') await expect(page.getByText('Loading users…')).not.toBeVisible()
    await accessible()
  }
  await page.goto('/users')
  await page.getByRole('button', { name: 'Add user', exact: true }).click()
  await accessible()
  await page.keyboard.press('Escape')
  await expect(page.getByRole('dialog')).not.toBeVisible()
  expect((await page.request.get('/assets/does-not-exist.js')).status()).toBe(404)
  expect((await page.request.get('/api/does-not-exist')).status()).toBe(404)
  expect((await page.request.get('/does-not-exist')).status()).toBe(404)
})

test('mobile navigation hides closed controls and manages keyboard focus', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await login(page)
  await page.evaluate(() => {
    document.body.tabIndex = -1
    document.body.focus()
  })
  await page.keyboard.press('Tab')
  await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused()
  await page.keyboard.press('Enter')
  await expect(page.getByRole('button', { name: 'Close menu' })).toBeFocused()
  await page.keyboard.press('Shift+Tab')
  await expect(page.getByRole('button', { name: 'Sign out' })).toBeFocused()
  await page.keyboard.press('Tab')
  await expect(page.getByRole('button', { name: 'Close menu' })).toBeFocused()
  await page.keyboard.press('Escape')
  await expect(page.getByRole('button', { name: 'Open navigation' })).toBeFocused()
  await expect(page.locator('.sidebar')).toHaveAttribute('inert', '')
  await page.setViewportSize({ width: 1280, height: 900 })
  await expect(page.locator('.sidebar')).not.toHaveAttribute('inert', '')
  await expect(page.getByRole('link', { name: 'Users', exact: true })).toBeVisible()
})

test('cross-tab cookie rotation refreshes CSRF without replaying writes', async ({
  page,
  context,
}) => {
  await login(page)
  await page.goto('/settings')
  const original = await page.getByLabel('Full name').inputValue()
  const second = await context.newPage()
  await second.goto('/settings')
  await second.getByRole('button', { name: 'Sign out', exact: true }).click()
  await login(second)
  await page.getByLabel('Full name').fill('Must not auto-submit')
  await page.getByRole('button', { name: 'Save changes' }).click()
  await expect(page.getByRole('alert')).toContainText('session')
  await second.goto('/settings')
  await expect(second.getByLabel('Full name')).toHaveValue(original)
  await page.getByRole('button', { name: 'Save changes' }).click()
  await expect(page.getByText('Your profile has been updated.')).toBeVisible()
  await second.reload()
  await expect(second.getByLabel('Full name')).toHaveValue('Must not auto-submit')
})

test('component catalogue interactions, production routes, mobile layout and accessibility', async ({
  page,
}) => {
  const { default: axe } = await import('axe-core')
  await login(page)
  for (const path of [
    '/guide',
    '/examples/elements',
    '/examples/cards',
    '/examples/tables',
    '/examples/forms',
  ]) {
    expect((await page.goto(path)).status()).toBe(200)
    await expect(page.locator('h1')).toBeVisible()
    await page.addScriptTag({ content: axe.source })
    const violations = await page.evaluate(async () =>
      (
        await window.axe.run(document, {
          runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] },
        })
      ).violations.map(({ id, nodes }) => ({
        id,
        nodes: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })),
      })),
    )
    expect(violations, path).toEqual([])
    await page.setViewportSize({ width: 390, height: 844 })
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      path,
    ).toBe(true)
    const corners = await page.evaluate(() =>
      [...document.querySelectorAll('body *')]
        .filter(
          (el) =>
            el.getClientRects().length &&
            [
              'borderTopLeftRadius',
              'borderTopRightRadius',
              'borderBottomLeftRadius',
              'borderBottomRightRadius',
            ].some((prop) => parseFloat(getComputedStyle(el)[prop]) > 4),
        )
        .map((el) => el.className),
    )
    expect(corners, path).toEqual([])
    await page.screenshot({
      path: `test-results/example-${path.split('/').at(-1)}-mobile.png`,
      fullPage: true,
    })
    await page.setViewportSize({ width: 1440, height: 1050 })
  }
  await page.goto('/examples/elements')
  await page.getByRole('button', { name: 'Open dialog', exact: true }).click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await page.getByRole('button', { name: 'Confirm example' }).click()
  await expect(page.getByRole('status')).toContainText(['Example confirmed.', 'Overview:'])
  await page.getByRole('button', { name: 'Activity', exact: true }).click()
  await expect(page.getByText('Activity: No recent activity in this demo.')).toBeVisible()
  await page.goto('/examples/cards')
  await page.getByRole('button', { name: 'Save project', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Saved to collection' })).toHaveAttribute(
    'aria-pressed',
    'true',
  )
  await page.goto('/examples/tables')
  await page.getByRole('button', { name: 'Next page' }).click()
  await expect(page.getByText('Page 2 of 2')).toBeVisible()
  await page.getByLabel('Search projects').fill('workspace')
  await expect(page.getByRole('rowheader', { name: 'Workspace redesign' })).toBeVisible()
  await page.getByLabel('Select Workspace redesign', { exact: true }).check()
  await expect(page.getByRole('status')).toContainText('1 selected')
  await page.getByLabel('Project status', { exact: true }).selectOption('Done')
  await expect(page.getByText('No projects match.', { exact: false })).toBeVisible()
  await page.getByRole('button', { name: 'Reset table' }).click()
  await expect(page.getByRole('status')).toContainText('7 results · 0 selected')
  const first = await page.getByRole('rowheader').first().textContent()
  await page.getByRole('button', { name: 'Project ↑', exact: true }).click()
  expect(await page.getByRole('rowheader').first().textContent()).not.toBe(first)
  await page.goto('/examples/forms')
  await page.getByRole('button', { name: 'Save demo' }).click()
  await expect(page.getByText('Enter at least two characters.')).toBeVisible()
  await page.getByLabel('Project name', { exact: false }).fill('Example project')
  await page.getByLabel('Contact email', { exact: false }).fill('example@example.com')
  await page.getByRole('button', { name: 'Save demo' }).click()
  await expect(page.getByRole('status')).toContainText('Nothing was sent to the server.')
  await page.getByRole('button', { name: 'Reset form' }).click()
  await expect(page.getByLabel('Project name', { exact: true })).toHaveValue('')
})
