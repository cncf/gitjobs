import { expect } from "@playwright/test";

/**
 * Fills a markdown editor through its visible CodeMirror control.
 * @param {import("@playwright/test").Page} page - Page containing the editor.
 * @param {string} fieldName - Submitted textarea name.
 * @param {string} value - Markdown content to enter.
 * @returns {Promise<void>}
 */
export const fillMarkdownEditor = async (page, fieldName, value) => {
  const submittedTextarea = page.locator(`markdown-editor textarea[name="${fieldName}"]`);
  await submittedTextarea.waitFor({ state: "attached", timeout: 10000 });

  const markdownEditor = page.locator("markdown-editor").filter({
    has: page.locator(`textarea[name="${fieldName}"]`),
  });
  const codeMirror = markdownEditor.locator(".CodeMirror");
  await codeMirror.waitFor({ state: "visible", timeout: 10000 });
  await codeMirror.click();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.insertText(value);

  await expect(submittedTextarea).toHaveValue(value);
};
