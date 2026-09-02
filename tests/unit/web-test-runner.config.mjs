import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { playwrightLauncher } from "@web/test-runner-playwright";

const configDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRootDirectory = resolve(configDirectory, "../..");
const litDevModeWarningUrl = "lit.dev/msg/dev-mode";
const importMap = JSON.stringify({
  imports: {
    "/static/": "/gitjobs-server/static/",
  },
});

export default {
  rootDir: repositoryRootDirectory,
  files: `${repositoryRootDirectory}/tests/unit/**/*.test.js`,
  filterBrowserLogs: ({ args }) =>
    !args.some((argument) => typeof argument === "string" && argument.includes(litDevModeWarningUrl)),
  hostname: "127.0.0.1",
  nodeResolve: true,
  browsers: [
    playwrightLauncher({
      product: "chromium",
      createBrowserContext: ({ browser }) => browser.newContext({ locale: "en-US" }),
    }),
  ],
  testRunnerHtml: (testFrameworkImport) => `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <script type="importmap">${importMap}</script>
  </head>
  <body>
    <script type="module" src="${testFrameworkImport}"></script>
  </body>
</html>`,
};
