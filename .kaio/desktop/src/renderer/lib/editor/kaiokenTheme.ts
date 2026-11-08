// src/renderer/lib/editor/kaiokenTheme.ts
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

// ─── VSCode Dark+ inspired color palette ────────────────────────────────────
const dark = {
  bg:          "#1e1e1e",
  bgPanel:     "#252526",
  bgSelect:    "#264f78",
  bgActiveLine:"#2a2d2e",
  cursor:      "#aeafad",
  fg:          "#d4d4d4",
  comment:     "#6a9955",
  keyword:     "#569cd6",
  string:      "#ce9178",
  number:      "#b5cea8",
  type:        "#4ec9b0",
  function:    "#dcdcaa",
  variable:    "#9cdcfe",
  tag:         "#4ec9b0",
  attribute:   "#9cdcfe",
  operator:    "#d4d4d4",
  punctuation: "#d4d4d4",
  meta:        "#c586c0",
  constant:    "#4fc1ff",
  lineNum:     "#858585",
  lineNumAct:  "#c6c6c6",
  rule:        "#404040",
  matchBg:     "rgba(255,255,255,0.08)",
  matchBorder: "#F38518"
};

// ─── VSCode Light+ inspired color palette ───────────────────────────────────
const light = {
  bg:          "#ffffff",
  bgPanel:     "#f3f3f3",
  bgSelect:    "#add6ff",
  bgActiveLine:"#f0f0f0",
  cursor:      "#000000",
  fg:          "#000000",
  comment:     "#008000",
  keyword:     "#0000ff",
  string:      "#a31515",
  number:      "#098658",
  type:        "#267f99",
  function:    "#795e26",
  variable:    "#001080",
  tag:         "#800000",
  attribute:   "#ff0000",
  operator:    "#000000",
  punctuation: "#000000",
  meta:        "#af00db",
  constant:    "#0070c1",
  lineNum:     "#237893",
  lineNumAct:  "#0b216f",
  rule:        "#e5e5e5",
  matchBg:     "rgba(0,0,0,0.07)",
  matchBorder: "#F38518"
};

function buildTheme(c: typeof dark) {
  const theme = EditorView.theme({
    "&": {
      color: c.fg,
      background: c.bg,
      height: "100%",
      fontSize: "13px"
    },
    ".cm-content": {
      caretColor: c.cursor,
      padding: "8px 0",
      fontFamily: "'Geist Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace",
      lineHeight: "1.6"
    },
    "&.cm-focused .cm-cursor": { borderLeftColor: c.cursor },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
      background: c.bgSelect
    },
    ".cm-panels": { background: c.bgPanel, color: c.fg },
    ".cm-panels.cm-panels-top": { borderBottom: `2px solid ${c.rule}` },
    ".cm-panels.cm-panels-bottom": { borderTop: `2px solid ${c.rule}` },
    ".cm-searchMatch": { background: c.matchBg, outline: `1px solid ${c.matchBorder}` },
    ".cm-searchMatch.cm-searchMatch-selected": { background: c.bgSelect },
    ".cm-activeLine": { background: c.bgActiveLine },
    ".cm-selectionMatch": { background: c.matchBg },
    "&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket": {
      background: "rgba(127,127,127,0.25)"
    },
    ".cm-gutters": {
      background: c.bgPanel,
      color: c.lineNum,
      border: "none",
      borderRight: `1px solid ${c.rule}`
    },
    ".cm-activeLineGutter": { background: c.bgActiveLine, color: c.lineNumAct },
    ".cm-foldPlaceholder": {
      background: "transparent",
      border: "none",
      color: c.fg
    },
    ".cm-tooltip": {
      border: `1px solid ${c.rule}`,
      background: c.bgPanel
    },
    ".cm-tooltip-autocomplete": {
      "& > ul > li[aria-selected]": {
        background: c.bgSelect,
        color: c.fg
      }
    },
    ".cm-lineNumbers .cm-gutterElement": {
      padding: "0 12px 0 8px",
      minWidth: "44px",
      fontSize: "12px"
    }
  });

  const highlight = syntaxHighlighting(
    HighlightStyle.define([
      { tag: t.keyword, color: c.keyword },
      { tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName], color: c.variable },
      { tag: [t.function(t.variableName), t.labelName], color: c.function },
      { tag: [t.color, t.constant(t.name), t.standard(t.name)], color: c.constant },
      { tag: [t.definition(t.name), t.separator], color: c.fg },
      {
        tag: [t.typeName, t.className, t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace],
        color: c.type
      },
      {
        tag: [t.operator, t.operatorKeyword, t.url, t.escape, t.regexp, t.link, t.special(t.string)],
        color: c.meta
      },
      { tag: [t.meta, t.comment], color: c.comment },
      { tag: t.strong, fontWeight: "bold" },
      { tag: t.emphasis, fontStyle: "italic" },
      { tag: t.strikethrough, textDecoration: "line-through" },
      { tag: t.link, color: c.constant, textDecoration: "underline" },
      { tag: t.heading, fontWeight: "bold", color: c.keyword },
      { tag: [t.atom, t.bool, t.special(t.variableName)], color: c.constant },
      { tag: [t.processingInstruction, t.string, t.inserted], color: c.string },
      { tag: t.number, color: c.number },
      { tag: t.invalid, color: "#ff0000" },
      { tag: t.tagName, color: c.tag },
      { tag: t.attributeName, color: c.attribute }
    ])
  );

  return [theme, highlight];
}

export const kaiokenDarkTheme = buildTheme(dark);
export const kaiokenLightTheme = buildTheme(light);
