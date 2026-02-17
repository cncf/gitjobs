import { expect, Page } from "@playwright/test";

export const navigateHomeWithRetries = async (page: Page): Promise<void> => {
  let lastError: unknown;
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      await page.goto("/", { timeout: 9000 });
      await page.waitForLoadState("domcontentloaded");
      return;
    } catch (error) {
      lastError = error;
    }
  }

  throw new Error(
    `Failed to navigate to home page after 3 attempts: ${String(lastError)}`,
  );
};

export const countVisibleNoDataMessages = async (
  page: Page,
): Promise<number> => {
  return page.getByText("No data available yet").evaluateAll((nodes) => {
    return nodes.filter((node) => {
      const element = node;
      if (!(element instanceof HTMLElement)) {
        return false;
      }

      const style = window.getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return (
        style.display !== "none" &&
        style.visibility !== "hidden" &&
        rect.width > 0 &&
        rect.height > 0
      );
    }).length;
  });
};

export const waitForOnlyJobTypeResults = async (
  page: Page,
  expectedType: string,
): Promise<void> => {
  const expectedTypePattern = expectedType
    .trim()
    .split(/\s+/)
    .map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join("\\s+");
  const jobTypePattern = new RegExp(
    `Job\\s*type\\s*${expectedTypePattern}`,
    "i",
  );

  await expect
    .poll(
      async () => {
        const cards = page.locator('[data-preview-job="true"]');
        const totalCards = await cards.count();
        if (totalCards === 0) {
          return false;
        }

        const matchingCards = await page
          .locator('[data-preview-job="true"]', { hasText: jobTypePattern })
          .count();
        return totalCards === matchingCards;
      },
      { timeout: 10000 },
    )
    .toBe(true);
};

export const waitForOnlyJobTypeSetResults = async (
  page: Page,
  expectedTypes: string[],
): Promise<void> => {
  const typePatterns = expectedTypes.map((expectedType) => {
    const normalizedExpectedType = expectedType
      .trim()
      .split(/\s+/)
      .map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
      .join("\\s+");

    return new RegExp(`Job\\s*type\\s*${normalizedExpectedType}`, "i");
  });

  await expect
    .poll(
      async () => {
        const cards = page.locator('[data-preview-job="true"]');
        const totalCards = await cards.count();
        if (totalCards === 0) {
          return false;
        }

        const matchesByType = await Promise.all(
          typePatterns.map((pattern) =>
            page
              .locator('[data-preview-job="true"]', { hasText: pattern })
              .count(),
          ),
        );
        const totalMatches = matchesByType.reduce(
          (sum, value) => sum + value,
          0,
        );
        return totalMatches === totalCards;
      },
      { timeout: 10000 },
    )
    .toBe(true);
};
