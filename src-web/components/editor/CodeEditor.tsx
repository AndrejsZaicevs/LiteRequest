import CodeMirror from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorView } from "@codemirror/view";
import { useMemo } from "react";

const baseTheme = EditorView.theme({
  "&": {
    height: "100%",
    fontSize: "13px",
    backgroundColor: "transparent",
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
  ".cm-editor": { backgroundColor: "transparent" },
  ".cm-gutters": {
    backgroundColor: "#0d0d0d",
    border: "none",
    color: "#4b5563",
    paddingRight: "8px",
  },
  ".cm-activeLineGutter": { backgroundColor: "#1a1a1a" },
  ".cm-activeLine": { backgroundColor: "#1a1a1a80" },
  ".cm-selectionBackground, .cm-focused .cm-selectionBackground": {
    backgroundColor: "#3b82f620 !important",
  },
  ".cm-matchingBracket": {
    backgroundColor: "#3b82f640",
    outline: "1px solid #3b82f660",
  },
});

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  language?: "json" | "text";
  placeholder?: string;
}

export function CodeEditor({ value, onChange, language = "json", placeholder }: CodeEditorProps) {
  const extensions = useMemo(() => {
    const exts = [baseTheme, EditorView.lineWrapping];
    if (language === "json") exts.push(json());
    return exts;
  }, [language]);

  return (
    <CodeMirror
      value={value}
      height="100%"
      theme={oneDark}
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
