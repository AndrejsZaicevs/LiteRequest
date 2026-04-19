import { useRef, useEffect, useCallback, useState, lazy, Suspense } from "react";
import type { OnMount, OnChange } from "@monaco-editor/react";
import type * as Monaco from "monaco-editor";
import { generateScriptTypes } from "../../lib/api";

const MonacoEditor = lazy(() => import("@monaco-editor/react"));

type MonacoInstance = typeof Monaco;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type TSDefaults = any;

// Workaround: monaco-editor ships typescript API at runtime but the TS
// types mark it as deprecated. Access via bracket notation.
function getTSDefaults(monaco: MonacoInstance): TSDefaults {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (monaco.languages as any).typescript.typescriptDefaults;
}

// Dark theme matching the app's design
const LITE_THEME: Monaco.editor.IStandaloneThemeData = {
  base: "vs-dark",
  inherit: true,
  rules: [
    { token: "keyword", foreground: "c084fc" },
    { token: "string", foreground: "34d399" },
    { token: "number", foreground: "f59e0b" },
    { token: "comment", foreground: "6b7280", fontStyle: "italic" },
    { token: "type", foreground: "60a5fa" },
    { token: "delimiter", foreground: "6b7280" },
  ],
  colors: {
    "editor.background": "#0d0d0d",
    "editor.foreground": "#d1d5db",
    "editor.lineHighlightBackground": "#1a1a1a80",
    "editor.selectionBackground": "#3b82f655",
    "editorCursor.foreground": "#60a5fa",
    "editorGutter.background": "#0d0d0d",
    "editorLineNumber.foreground": "#374151",
    "editorLineNumber.activeForeground": "#6b7280",
    "editor.inactiveSelectionBackground": "#3b82f622",
    "editorWidget.background": "#1a1a1a",
    "editorSuggestWidget.background": "#1a1a1a",
    "editorSuggestWidget.border": "#2d2d2d",
    "editorSuggestWidget.selectedBackground": "#2d2d2d",
  },
};

interface ScriptEditorProps {
  value: string;
  onChange: (value: string) => void;
  /** "post-exec" enables lr.request/response types, "standalone" enables lr.sleep + imports */
  mode: "post-exec" | "standalone";
  readOnly?: boolean;
}

let typesRegistered = false;
let typesUri: string | null = null;

export function ScriptEditor({ value, onChange, mode, readOnly = false }: ScriptEditorProps) {
  const editorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<typeof Monaco | null>(null);
  const [loading, setLoading] = useState(true);

  // Register type definitions with Monaco's TypeScript worker
  const registerTypes = useCallback(async (monaco: typeof Monaco) => {
    if (typesRegistered) return;
    try {
      const dts = await generateScriptTypes();
      const uri = "ts:lr-types.d.ts";
      getTSDefaults(monaco).addExtraLib(dts, uri);
      typesUri = uri;
      typesRegistered = true;
    } catch (err) {
      console.error("Failed to load script types:", err);
    }
  }, []);

  // Refresh types when mode changes (post-exec vs standalone may have different collections)
  useEffect(() => {
    if (monacoRef.current) {
      typesRegistered = false;
      registerTypes(monacoRef.current);
    }
  }, [mode, registerTypes]);

  const handleMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

    // Define the theme
    monaco.editor.defineTheme("liteTheme", LITE_THEME);
    monaco.editor.setTheme("liteTheme");

    // TypeScript compiler options for script editing
    const tsDefaults = getTSDefaults(monaco);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const ts = (monaco.languages as any).typescript;
    tsDefaults.setCompilerOptions({
      target: ts.ScriptTarget.ES2020,
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.NodeJs,
      allowJs: true,
      strict: false,
      noEmit: true,
      esModuleInterop: true,
      allowNonTsExtensions: true,
      lib: ["es2020"],
    });

    // Disable built-in default lib to avoid browser/node type pollution
    tsDefaults.setDiagnosticsOptions({
      noSemanticValidation: false,
      noSyntaxValidation: false,
    });

    registerTypes(monaco);
    setLoading(false);
  }, [registerTypes]);

  const handleChange: OnChange = useCallback((val) => {
    if (val !== undefined) onChange(val);
  }, [onChange]);

  return (
    <div className="h-full w-full relative">
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center bg-[#0d0d0d] z-10">
          <span className="text-xs text-gray-600">Loading editor…</span>
        </div>
      )}
      <Suspense fallback={
        <div className="h-full flex items-center justify-center bg-[#0d0d0d]">
          <span className="text-xs text-gray-600">Loading editor…</span>
        </div>
      }>
        <MonacoEditor
          height="100%"
          language="typescript"
          value={value}
          onChange={handleChange}
          onMount={handleMount}
          options={{
            readOnly,
            minimap: { enabled: false },
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
            lineHeight: 22,
            padding: { top: 12 },
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            wordWrap: "on",
            renderWhitespace: "none",
            quickSuggestions: true,
            suggestOnTriggerCharacters: true,
            parameterHints: { enabled: true },
            folding: true,
            lineNumbers: "on",
            scrollbar: {
              verticalScrollbarSize: 8,
              horizontalScrollbarSize: 8,
            },
          }}
        />
      </Suspense>
    </div>
  );
}

/**
 * Transpile TypeScript source to JavaScript using the TypeScript compiler API.
 * This is a pure in-process transform — no Monaco workers required.
 */
export async function transpileTS(source: string): Promise<string> {
  const ts = await import("typescript");
  const result = ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2020,
      module: ts.ModuleKind.CommonJS,
    },
  });
  if (!result.outputText) {
    throw new Error("TypeScript transpilation produced no output");
  }
  return result.outputText;
}

/**
 * Refresh the type definitions (call after collection structure changes).
 */
export function refreshScriptTypes() {
  typesRegistered = false;
}
