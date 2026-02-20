import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 1,
  reporter: process.env.CI
    ? [
        ["json", { outputFile: "playwright-report/results.json" }],
        ["html", { open: "never" }],
        ["line"],
      ]
    : [["html"], ["line"]],
  use: {
    baseURL: "http://localhost:8080",
    headless: true,
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
  webServer: [
    {
      command: "dx build --release && npx serve -s target/dx/log-workout/release/web/public -p 8080",
      url: "http://127.0.0.1:8080",
      timeout: 10 * 60 * 1000,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
    },
  ],
});
