import { describe, it, expect } from "vitest";
import { renderMarkdown } from "./markdown";

describe("renderMarkdown", () => {
  it("renders headings", () => {
    expect(renderMarkdown("# Titulo")).toBe("<h1>Titulo</h1>");
    expect(renderMarkdown("## Sub")).toBe("<h2>Sub</h2>");
  });

  it("renders a table", () => {
    const md = "| A | B |\n|---|---|\n| 1 | 2 |";
    const html = renderMarkdown(md);
    expect(html).toContain("<table>");
    expect(html).toContain("<th>A</th>");
    expect(html).toContain("<td>1</td>");
  });

  it("renders a fenced code block without interpreting its contents", () => {
    const md = "```python\nprint('a < b')\n```";
    const html = renderMarkdown(md);
    expect(html).toBe("<pre><code>print('a &lt; b')</code></pre>");
  });

  it("renders inline bold and code", () => {
    expect(renderMarkdown("Usa **negrita** y `codigo`")).toBe(
      "<p>Usa <strong>negrita</strong> y <code>codigo</code></p>",
    );
  });

  it("renders a list", () => {
    const html = renderMarkdown("- uno\n- dos");
    expect(html).toBe("<ul><li>uno</li><li>dos</li></ul>");
  });

  it("escapes raw html in text", () => {
    expect(renderMarkdown("<script>")).toBe("<p>&lt;script&gt;</p>");
  });
});
