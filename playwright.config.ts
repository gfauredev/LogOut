import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 1,
  use: {
    baseURL: "http://localhost:8080",
    headless: true,
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
  webServer: [
    {
      command: "dx serve --port 8080",
      port: 8080,
      timeout: 10 * 60 * 1000,
      reuseExistingServer: !process.env.CI,
      stdout: "pipe",
    },
  ],
});
