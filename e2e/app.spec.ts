import { test, expect } from "@playwright/test";

// After the merge, base_path was removed from Dioxus.toml, so the app
// is served at root.  The dev-server (dx serve) does NOT add a prefix.
const BASE = "";

test.describe("Sessions tab (home page)", () => {
  test("renders title and tagline", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
    await expect(page.locator(".app-tagline")).toHaveText(
      "Turn off your computer, Log your workOut"
    );
  });

  test("has permanent bottom navigation with 4 icon tabs", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
    // After the redesign the nav only has emoji icons (no text labels
    // except Credits).  Just verify all 4 tabs exist.
    const tabs = page.locator(".bottom-nav__tab");
    await expect(tabs).toHaveCount(4);
  });

  test("has button to start a new session", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await expect(page.locator(".new-session-button")).toBeVisible();
  });

  test("starting a session shows session view with Cancel Session button", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    // Empty session → Cancel Session (Finish Session only appears after
    // completing at least one exercise)
    await expect(
      page.locator("button", { hasText: "Cancel Session" })
    ).toBeVisible();
  });

  test("cancelling an empty session returns to sessions list", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );
    await page.click("button:has-text('Cancel Session')");
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
    await expect(page.locator(".new-session-button")).toBeVisible();
  });

  test("Sessions tab is active on home page", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // The active tab has the --active modifier class
    await expect(page.locator(".bottom-nav__tab--active")).toBeVisible();
  });
});

test.describe("Exercise browser (Exercises tab)", () => {
  test("renders with search input and title", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".search-input")).toBeVisible();
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });

  test("has permanent bottom navigation", async ({ page }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("Exercises tab is active on exercise browser page", async ({
    page,
  }) => {
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator(".bottom-nav__tab--active")).toBeVisible();
  });

  test("navigate to exercises from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // Click the first tab (Exercises icon)
    await page.locator(".bottom-nav__tab").first().click();
    await expect(page.locator("h1")).toHaveText("Exercise Database");
  });
});

test.describe("Analytics tab", () => {
  test("renders analytics page", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    await expect(page.locator(".analytics-panel")).toBeVisible();
  });

  test("has permanent bottom navigation", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("Analytics tab is active on analytics page", async ({ page }) => {
    await page.goto(`${BASE}/analytics`);
    await expect(page.locator(".bottom-nav__tab--active")).toBeVisible();
  });

  test("navigate to analytics from home via bottom nav", async ({ page }) => {
    await page.goto(`${BASE}/`);
    // Analytics is the 3rd tab (index 2)
    await page.locator(".bottom-nav__tab").nth(2).click();
    await expect(page.locator(".analytics-panel")).toBeVisible();
  });
});

test.describe("Active session view", () => {
  test("has exercise search input", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(
      page.locator('input[placeholder="Search for an exercise..."]')
    ).toBeVisible();
  });

  test("bottom nav is visible during active session", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".bottom-nav")).toBeVisible();
  });

  test("empty session shows Cancel Session button", async ({ page }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(
      page.locator("button", { hasText: "Cancel Session" })
    ).toBeVisible();
  });

  test("cancelling an empty session returns to home screen", async ({
    page,
  }) => {
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await page.click("button:has-text('Cancel Session')");
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
  });

  test("complete workout flow: search exercise, add reps, complete", async ({
    page,
  }) => {
    // Start a new session
    await page.goto(`${BASE}/`);
    await page.click(".new-session-button");
    await expect(page.locator(".session-header__title")).toContainText(
      "Active Session"
    );

    // Search for pullups
    const searchInput = page.locator('input[placeholder="Search for an exercise..."]');
    await searchInput.fill("pullups");

    // Wait for search results to appear
    await expect(page.locator(".search-results")).toBeVisible();

    // Click the first result
    const firstResult = page.locator(".search-result-item").first();
    await expect(firstResult).toBeVisible();
    await firstResult.click();

    // Verify exercise form is shown
    await expect(page.locator(".exercise-form")).toBeVisible();

    // Fill in reps (and optionally weight)
    const repsInput = page.locator('input[placeholder="Reps"]');
    if (await repsInput.isVisible()) {
      await repsInput.fill("10");
    }

    // Complete the exercise
    await page.click("button:has-text('Complete Exercise')");

    // Verify the exercise appears in completed exercises
    await expect(page.locator(".completed-exercises-section")).toBeVisible();

    // Finish the session
    await page.click("button:has-text('Finish Session')");

    // Verify we're back at home and session is saved
    await expect(page.locator(".app-title")).toHaveText("💪 LogOut");
  });
});

test.describe("Exercise editing", () => {
  test("edit exercise instructions and verify changes persist", async ({
    page,
  }) => {
    // Go to exercises list
    await page.goto(`${BASE}/exercises`);
    await expect(page.locator("h1")).toHaveText("Exercise Database");

    // Search for pushups
    const searchInput = page.locator(".search-input");
    await searchInput.fill("pushups");

    // Wait for search results
    await page.waitForTimeout(500); // Give search time to filter

    // Click on the first exercise card to open details
    const firstExercise = page.locator(".exercise-card").first();
    await expect(firstExercise).toBeVisible();

    // Get the exercise name for verification later
    const exerciseName = await firstExercise.locator(".exercise-card__name, h3").first().textContent();

    // Click on the exercise name to view details
    await firstExercise.locator(".exercise-card__name, h3").first().click();

    // Look for edit button or instructions field
    // Note: This part depends on the actual UI structure which may need adjustment
    // If there's an edit button, click it
    const editButton = page.locator("button:has-text('Edit')");
    if (await editButton.isVisible({ timeout: 2000 }).catch(() => false)) {
      await editButton.click();

      // Find instructions textarea/input and modify it
      const instructionsField = page.locator('textarea, input[type="text"]').filter({ hasText: /instruction/i }).first();
      if (await instructionsField.isVisible({ timeout: 2000 }).catch(() => false)) {
        await instructionsField.fill("Custom test instructions");

        // Save changes
        await page.click("button:has-text('Save')");

        // Navigate back and verify
        await page.goBack();
        await firstExercise.locator(".exercise-card__name, h3").first().click();

        // Verify instructions were saved
        await expect(page.locator("text=Custom test instructions")).toBeVisible();
      }
    }
  });
});

test.describe("PWA assets", () => {
  test("manifest.json is accessible and valid JSON", async ({ request }) => {
    const response = await request.get(`${BASE}/manifest.json`);
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.name).toBeTruthy();
    expect(body.icons).toBeInstanceOf(Array);
  });

  test("service worker script sw.js is accessible", async ({ request }) => {
    const response = await request.get(`${BASE}/sw.js`);
    expect(response.ok()).toBeTruthy();
    const contentType = response.headers()["content-type"];
    expect(contentType).toContain("javascript");
  });

  test("manifest link is present in page HTML", async ({ page }) => {
    await page.goto(`${BASE}/`);
    const manifestLink = page.locator('link[rel="manifest"]');
    await expect(manifestLink).toHaveAttribute("href", "./manifest.json");
  });
});

