import { useEffect, useMemo, useRef } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { javascript, javascriptLanguage } from "@codemirror/lang-javascript";
import { EditorView } from "@codemirror/view";
import { syntaxHighlighting } from "@codemirror/language";
import { autocompletion } from "@codemirror/autocomplete";
import { liteEditorTheme, liteHighlightStyle } from "../../lib/editorTheme";
import { scriptCompletionSource, refreshScriptTypes } from "../../lib/scriptCompletions";

interface ScriptEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  /** Known variables, offered inside `lr.setVariable("…")` / `lr.getVariable("…")`. */
  variables?: Record<string, string>;
}

/** TypeScript editor for a request's post-execution script. */
export function ScriptEditor({ value, onChange, readOnly = false, variables }: ScriptEditorProps) {
  // Read through a ref so a variables change does not rebuild the editor.
  const variablesRef = useRef(variables ?? {});
  variablesRef.current = variables ?? {};

  // Types are fetched lazily on the first completion request.
  useEffect(() => {
    refreshScriptTypes();
  }, []);

  const extensions = useMemo(
    () => [
      liteEditorTheme,
      syntaxHighlighting(liteHighlightStyle),
      EditorView.lineWrapping,
      javascript({ typescript: true }),
      javascriptLanguage.data.of({
        autocomplete: scriptCompletionSource({ variables: () => variablesRef.current }),
      }),
      autocompletion({ activateOnTyping: true }),
    ],
    [],
  );

  return (
    <CodeMirror
      value={value}
      height="100%"
      theme="none"
      extensions={extensions}
      onChange={onChange}
      readOnly={readOnly}
      editable={!readOnly}
      basicSetup={{
        lineNumbers: true,
        foldGutter: true,
        bracketMatching: true,
        closeBrackets: true,
        autocompletion: false,
        highlightActiveLine: true,
        indentOnInput: true,
        tabSize: 2,
      }}
      style={{ height: "100%" }}
    />
  );
}
