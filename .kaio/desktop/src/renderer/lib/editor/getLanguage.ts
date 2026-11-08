// src/renderer/lib/editor/getLanguage.ts
import { LanguageSupport } from "@codemirror/language";

export async function getLanguageForFile(filename: string): Promise<LanguageSupport | null> {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";

  try {
    switch (ext) {
      case "js":
      case "jsx": {
        const { javascript } = await import("@codemirror/lang-javascript");
        return javascript({ jsx: true });
      }
      case "ts":
      case "tsx": {
        const { javascript } = await import("@codemirror/lang-javascript");
        return javascript({ typescript: true, jsx: ext === "tsx" });
      }
      case "py": {
        const { python } = await import("@codemirror/lang-python");
        return python();
      }
      case "md":
      case "mdx": {
        const { markdown } = await import("@codemirror/lang-markdown");
        return markdown();
      }
      case "css": {
        const { css } = await import("@codemirror/lang-css");
        return css();
      }
      case "json": {
        const { json } = await import("@codemirror/lang-json");
        return json();
      }
      case "html":
      case "htm": {
        const { html } = await import("@codemirror/lang-html");
        return html();
      }
      case "rs": {
        const { rust } = await import("@codemirror/lang-rust");
        return rust();
      }
      case "go": {
        const { go } = await import("@codemirror/lang-go");
        return go();
      }
      case "java": {
        const { java } = await import("@codemirror/lang-java");
        return java();
      }
      default:
        return null;
    }
  } catch {
    return null;
  }
}
