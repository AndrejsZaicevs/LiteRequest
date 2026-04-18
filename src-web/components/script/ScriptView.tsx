import { useState, lazy, Suspense, useCallback } from "react";
import type { Script, ScriptVersion, ScriptResult } from "../../lib/types";
import { Play } from "lucide-react";

const ScriptEditorLazy = lazy(() =>
  import("../editor/ScriptEditor").then(m => ({ default: m.ScriptEditor }))
);

interface ScriptViewProps {
  script: Script;
  content: string;
  versions: ScriptVersion[];
  dirty: boolean;
  onContentChange: (content: string) => void;
  onRun: () => Promise<void>;
  runResult: ScriptResult | null;
}

export function ScriptView({ script, content, versions, dirty, onContentChange, onRun, runResult }: ScriptViewProps) {
  const [isRunning, setIsRunning] = useState(false);
  const [outputExpanded, setOutputExpanded] = useState(true);

  const handleRun = useCallback(async () => {
    setIsRunning(true);
    try {
      await onRun();
    } finally {
      setIsRunning(false);
    }
  }, [onRun]);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Top bar */}
      <div className="h-10 border-b border-gray-800 flex items-center px-4 gap-3 shrink-0 bg-[#161616]">
        <span className="text-sm text-gray-200 font-medium truncate">{script.name}</span>
        {dirty && <span className="text-[10px] text-yellow-400">unsaved</span>}
        <div className="flex-1" />
        <span className="text-[10px] text-gray-600">
          {versions.length} version{versions.length !== 1 ? "s" : ""}
        </span>
        <button
          onClick={handleRun}
          disabled={isRunning}
          className="flex items-center gap-1.5 px-3 py-1 rounded text-xs font-medium bg-green-600 hover:bg-green-500 disabled:opacity-50 text-white transition-colors"
        >
          <Play size={12} />
          {isRunning ? "Running..." : "Run"}
        </button>
      </div>

      {/* Editor */}
      <div
        className={`flex-1 overflow-hidden ${runResult ? "border-b border-gray-800" : ""}`}
        style={{ minHeight: runResult ? "50%" : "100%" }}
      >
        <Suspense
          fallback={
            <div className="flex-1 flex items-center justify-center text-gray-600 text-sm">
              Loading editor...
            </div>
          }
        >
          <ScriptEditorLazy
            value={content}
            onChange={onContentChange}
            mode="standalone"
          />
        </Suspense>
      </div>

      {/* Output panel */}
      {runResult && (
        <div className="shrink-0 overflow-auto" style={{ maxHeight: "50%" }}>
          <div
            className="h-8 border-b border-gray-800 flex items-center px-4 gap-2 cursor-pointer hover:bg-[#1a1a1a]"
            onClick={() => setOutputExpanded(!outputExpanded)}
          >
            <span className="text-xs text-gray-400 font-medium">Output</span>
            <span
              className={`text-[10px] px-1.5 py-0.5 rounded ${
                runResult.status === "success"
                  ? "bg-green-500/15 text-green-400"
                  : "bg-red-500/15 text-red-400"
              }`}
            >
              {runResult.status}
            </span>
            <span className="text-[10px] text-gray-600">{runResult.duration_ms}ms</span>
          </div>
          {outputExpanded && (
            <div className="p-4 space-y-3 text-xs font-mono">
              {runResult.error && (
                <div className="bg-red-500/10 border border-red-500/20 rounded p-3">
                  <pre className="text-red-300 whitespace-pre-wrap break-all">{runResult.error}</pre>
                </div>
              )}
              {Object.keys(runResult.variables_set).length > 0 && (
                <div>
                  <div className="text-[10px] text-gray-500 font-semibold uppercase tracking-wider mb-1">
                    Variables Set
                  </div>
                  {Object.entries(runResult.variables_set).map(([k, v]) => (
                    <div key={k} className="flex gap-2 px-2 py-0.5 rounded bg-[#1a1a1a]">
                      <span className="text-blue-400">{k}</span>
                      <span className="text-gray-600">=</span>
                      <span className="text-gray-300 break-all">{v}</span>
                    </div>
                  ))}
                </div>
              )}
              {runResult.logs.length > 0 && (
                <div>
                  <div className="text-[10px] text-gray-500 font-semibold uppercase tracking-wider mb-1">
                    Console ({runResult.logs.length})
                  </div>
                  <div className="bg-[#0a0a0a] rounded border border-gray-800 p-3 max-h-[200px] overflow-auto">
                    {runResult.logs.map((line, i) => (
                      <div key={i} className="text-gray-400 py-0.5 whitespace-pre-wrap break-all">
                        <span className="text-gray-600 mr-2 select-none">{i + 1}</span>
                        {line}
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {runResult.logs.length === 0 &&
                !runResult.error &&
                Object.keys(runResult.variables_set).length === 0 && (
                  <div className="text-gray-600 text-center py-4">
                    Script ran successfully with no output
                  </div>
                )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
