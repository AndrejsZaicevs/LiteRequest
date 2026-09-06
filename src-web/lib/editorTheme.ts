import { EditorView } from "@codemirror/view";
import { HighlightStyle } from "@codemirror/language";
import { tags } from "@lezer/highlight";

/** Shared dark editor chrome used by every CodeMirror instance in the app. */
export const liteEditorTheme = EditorView.theme(
  {
    "&": {
      height: "100%",
      fontSize: "13px",
      backgroundColor: "#0d0d0d",
      color: "#d1d5db",
    },
    ".cm-scroller": {
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      lineHeight: "1.6",
      overflow: "auto",
    },
    ".cm-content": {
      padding: "16px",
      caretColor: "#60a5fa",
    },
    ".cm-focused": { outline: "none" },
    ".cm-editor": { backgroundColor: "#0d0d0d" },
    ".cm-gutters": {
      backgroundColor: "#0d0d0d",
      borderRight: "1px solid #1f2937",
      color: "#4b5563",
      paddingRight: "8px",
    },
    ".cm-activeLineGutter": { backgroundColor: "#1a1a1a" },
    ".cm-activeLine": { backgroundColor: "#1a1a1a80" },
    ".cm-selectionBackground": { backgroundColor: "#3b82f655 !important" },
    ".cm-focused .cm-selectionBackground": { backgroundColor: "#3b82f655 !important" },
    ".cm-matchingBracket": {
      backgroundColor: "#3b82f640",
      outline: "1px solid #3b82f660",
    },
    ".cm-cursor": { borderLeftColor: "#60a5fa" },
    ".cm-lineNumbers .cm-gutterElement": { color: "#374151" },
  },
  { dark: true }
);

/** Shared syntax colours for JSON and JavaScript/TypeScript. */
export const liteHighlightStyle = HighlightStyle.define([
  // JSON keys / object properties — blue-400
  { tag: tags.propertyName,                          color: "#60a5fa" },
  { tag: [tags.string, tags.special(tags.string)],   color: "#34d399" },   // emerald-400
  { tag: tags.number,                                color: "#f59e0b" },   // amber-400
  { tag: tags.bool,                                  color: "#f59e0b" },
  { tag: tags.null,                                  color: "#9ca3af" },   // gray-400
  { tag: [tags.keyword, tags.controlKeyword, tags.moduleKeyword,
          tags.definitionKeyword, tags.operatorKeyword, tags.self], color: "#c084fc" }, // purple-400
  { tag: tags.comment,                               color: "#6b7280", fontStyle: "italic" },
  { tag: tags.punctuation,                           color: "#6b7280" },
  { tag: tags.bracket,                               color: "#9ca3af" },
  { tag: tags.operator,                              color: "#9ca3af" },
  { tag: [tags.typeName, tags.className, tags.namespace], color: "#22d3ee" }, // cyan-400
  { tag: [tags.function(tags.variableName), tags.function(tags.propertyName)], color: "#fbbf24" }, // amber-300
  { tag: tags.definition(tags.variableName),         color: "#e5e7eb" },   // gray-200
  { tag: tags.variableName,                          color: "#d1d5db" },
  { tag: tags.regexp,                                color: "#f472b6" },   // pink-400
  { tag: tags.atom,                                  color: "#f59e0b" },
]);
