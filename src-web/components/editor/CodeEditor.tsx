import CodeMirror from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { useMemo } from "react";

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
}

export function CodeEditor({ value, onChange, language = "json", placeholder }: CodeEditorProps) {
  const extensions = useMemo(() => {
    const exts = [liteTheme, syntaxHighlighting(liteSyntax), EditorView.lineWrapping];
    if (language === "json") exts.push(json());
    return exts;
  }, [language]);

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
