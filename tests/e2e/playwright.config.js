import { defineConfig, devices } from "@playwright/test";

const mobileTestPattern = /@mobile/;
const smokeSpecPaths = [
  "public/about/about.spec.js",
  "public/common/navigation.spec.js",
  "public/jobboard/smoke.spec.js",
  "public/stats/stats.spec.js",
  "workflows/auth/auth.spec.js",
];

export default defineConfig({
  fullyParallel: false,
  retries: 0,
  testDir: ".",
  workers: 1,
  use: {
    baseURL: process.env.BASE_URL || "http://localhost:9000",
    locale: "en-US",
    screenshot: "only-on-failure",
    timezoneId: "UTC",
    trace: "retain-on-failure",
  },
  reporter: "list",
  projects: [
    {
      name: "chromium-smoke",
      grepInvert: mobileTestPattern,
      testMatch: smokeSpecPaths,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "firefox-smoke",
      grepInvert: mobileTestPattern,
      testMatch: smokeSpecPaths,
      use: { ...devices["Desktop Firefox"] },
    },
    {
      name: "webkit-smoke",
      grepInvert: mobileTestPattern,
      testMatch: smokeSpecPaths,
      use: { ...devices["Desktop Safari"] },
    },
    {
      name: "chromium-deep",
      grepInvert: mobileTestPattern,
      testIgnore: smokeSpecPaths,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "chromium-mobile-deep",
      grep: mobileTestPattern,
      use: { ...devices["iPhone 12"] },
    },
  ],
});
