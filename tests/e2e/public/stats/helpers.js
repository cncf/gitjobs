/**
 * Counts the visible empty-data messages rendered for stats charts.
 * @param {import("@playwright/test").Page} page - Stats page.
 * @returns {Promise<number>}
 */
export const countVisibleNoDataMessages = async (page) => {
  return page.getByText("No data available yet").evaluateAll((nodes) => {
    return nodes.filter((node) => {
      if (!(node instanceof HTMLElement)) {
        return false;
      }

      const style = window.getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    }).length;
  });
};
