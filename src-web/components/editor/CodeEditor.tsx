import CodeMirror from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { EditorView, ViewPlugin, Decoration } from "@codemirror/view";
import type { DecorationSet, ViewUpdate } from "@codemirror/view";
import { RangeSetBuilder } from "@codemirror/state";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { useMemo } from "react";

const varHighlightTheme = EditorView.theme({
  ".cm-var-resolved":   { color: "#34d399 !important", backgroundColor: "rgba(52,211,153,0.12)", borderRadius: "3px" },
  ".cm-var-unresolved": { color: "#fb923c !important", backgroundColor: "rgba(251,146,60,0.12)",  borderRadius: "3px" },
});

function makeVarPlugin(variables: Record<string, string>) {
  const VAR_RE = /\{\{([^}]+)\}\}/g;
  function buildDecos(doc: { toString(): string }): DecorationSet {
    const builder = new RangeSetBuilder<Decoration>();
    const text = doc.toString();
    VAR_RE.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = VAR_RE.exec(text)) !== null) {
      const name = m[1].trim();
      const cls = variables[name] !== undefined ? "cm-var-resolved" : "cm-var-unresolved";
      builder.add(m.index, m.index + m[0].length, Decoration.mark({ class: cls }));
    }
    return builder.finish();
  }
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) { this.decorations = buildDecos(view.state.doc); }
      update(u: ViewUpdate) { if (u.docChanged) this.decorations = buildDecos(u.state.doc); }
    },
    { decorations: v => v.decorations }
  );
}

const liteTheme = EditorView.theme(
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
    ".cm-selectionBackground": { backgroundColor: "#3b82f625 !important" },
    ".cm-focused .cm-selectionBackground": { backgroundColor: "#3b82f625 !important" },
    ".cm-matchingBracket": {
      backgroundColor: "#3b82f640",
      outline: "1px solid #3b82f660",
    },
    ".cm-cursor": { borderLeftColor: "#60a5fa" },
    ".cm-lineNumbers .cm-gutterElement": { color: "#374151" },
  },
  { dark: true }
);

const liteSyntax = HighlightStyle.define([
  { tag: tags.propertyName,       color: "#60a5fa" },   // JSON keys — blue-400
  { tag: tags.string,             color: "#34d399" },   // strings — emerald-400
  { tag: tags.number,             color: "#f59e0b" },   // numbers — amber-400
  { tag: tags.bool,               color: "#f59e0b" },   // booleans — amber-400
  { tag: tags.null,               color: "#9ca3af" },   // null — gray-400
  { tag: tags.keyword,            color: "#c084fc" },   // keywords — purple-400
  { tag: tags.comment,            color: "#6b7280", fontStyle: "italic" },
  { tag: tags.punctuation,        color: "#6b7280" },
  { tag: tags.bracket,            color: "#9ca3af" },
]);

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  language?: "json" | "text";
  placeholder?: string;
  variables?: Record<string, string>;
}

export function CodeEditor({ value, onChange, language = "json", placeholder, variables = {} }: CodeEditorProps) {
  const extensions = useMemo(() => {
    const exts = [liteTheme, syntaxHighlighting(liteSyntax), EditorView.lineWrapping, varHighlightTheme, makeVarPlugin(variables)];
    if (language === "json") exts.push(json());
    return exts;
  }, [language, variables]);

  return (
    <CodeMirror
      value={value}
      height="100%"
      theme="none"
      extensions={extensions}
      onChange={onChange}
      placeholder={placeholder}
      basicSetup={{
        lineNumbers: true,
        foldGutter: true,
        bracketMatching: true,
        closeBrackets: true,
        autocompletion: true,
        highlightActiveLine: true,
        indentOnInput: true,
        tabSize: 2,
      }}
      style={{ height: "100%" }}
    />
  );
}
