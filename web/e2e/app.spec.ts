import { test, expect } from "@playwright/test";

/**
 * E2E tests for zk-pool frontend.
 *
 * Test only UI (rendering, presence of elements).
 * Real deposit/withdraw is not tested — requires Phantom + devnet + ZK proof.
 */

test.describe("zk-pool app", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("renders main page with header", async ({ page }) => {
    await expect(page.locator("h1")).toContainText("zk-pool");
  });

  test("shows tagline", async ({ page }) => {
    await expect(page.getByText("Private SOL transfers")).toBeVisible();
  });

  test("shows wallet connect section", async ({ page }) => {
    const walletSection = page.locator("text=/Connect Wallet|Wallet:/i").first();
    await expect(walletSection).toBeVisible();
  });

  test("renders Deposit form", async ({ page }) => {
    await expect(page.getByText("Deposit SOL")).toBeVisible();
  });

  test("renders Withdraw form", async ({ page }) => {
    await expect(page.getByText("Withdraw SOL")).toBeVisible();
  });

  test("renders footer with tech stack", async ({ page }) => {
    await expect(page.getByText(/Noir/)).toBeVisible();
    await expect(page.getByText(/Sunspot/)).toBeVisible();
  });
});
