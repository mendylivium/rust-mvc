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
