import { useState } from "react";
import { Play, Terminal, Upload, Check, X } from "lucide-react";
import type { RequestData, HttpMethod } from "../../lib/types";
import { methodColor, HTTP_METHODS } from "../../lib/types";
import { VariableInput } from "../shared/VariableInput";

interface UrlBarProps {
  data: RequestData;
  onChange: (data: RequestData) => void;
  onSend: () => void;
  onCancel: () => void;
  onCopyCurl: () => void;
  onImportCurl: (curlStr: string) => void;
  isLoading: boolean;
  basePath: string;
  variables?: Record<string, string>;
}

export function UrlBar({ data, onChange, onSend, onCancel, onCopyCurl, onImportCurl, isLoading, basePath, variables = {} }: UrlBarProps) {
  const [showImport, setShowImport] = useState(false);
  const [importText, setImportText] = useState("");
  const [curlCopied, setCurlCopied] = useState(false);

  const updateField = <K extends keyof RequestData>(field: K, value: RequestData[K]) =>
    onChange({ ...data, [field]: value });

  const handleCopyCurl = () => {
    onCopyCurl();
    setCurlCopied(true);
    setTimeout(() => setCurlCopied(false), 2000);
  };

  const showBasePath = basePath && !(data.url.startsWith("http://") || data.url.startsWith("https://"));

  return (
    <>
      <div className="px-4 py-3 border-b border-gray-800 bg-[#121212] flex-shrink-0">
        <div className="flex items-center gap-2">
          <div className="flex rounded-md overflow-hidden border border-gray-700/60 flex-1 bg-[#1a1a1a]">
            {/* Method selector */}
            <select
              value={data.method}
              onChange={(e) => updateField("method", e.target.value as HttpMethod)}
              className="bg-transparent font-semibold text-sm pl-3 pr-8 py-2 outline-none cursor-pointer"
              style={{ color: methodColor(data.method), appearance: "none", borderRadius: 0, border: "none", borderRight: "1px solid rgba(55,65,81,0.6)" }}
            >
              {HTTP_METHODS.map(m => (
                <option key={m} value={m} style={{ color: "#d1d5db" }}>{m}</option>
              ))}
            </select>

            {/* Base path + URL input */}
            <div className="flex-1 flex items-center px-3 py-2 font-mono text-sm relative overflow-visible">
              {showBasePath && (
                <span className="text-gray-500 shrink-0 select-none mr-px" title={basePath}>
                  {basePath.replace(/\/+$/, "")}
                </span>
              )}
              <VariableInput
                value={data.url}
                onChange={(v) => updateField("url", v)}
                variables={variables}
                wrapperClassName="flex-1 min-w-[100px]"
                className="bg-transparent text-gray-200 outline-none"
                inputStyle={{ border: "none", borderRadius: 0, padding: 0, fontSize: "inherit" }}
                placeholder={showBasePath ? "" : "https://api.example.com/path"}
                onKeyDown={(e) => { if (e.key === "Enter") onSend(); }}
              />
            </div>
          </div>

          {/* Send / Cancel button */}
          {isLoading ? (
            <button
              onClick={onCancel}
              title="Cancel request"
              className="bg-red-700 hover:bg-red-600 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 shadow-sm"
            >
              <X size={14} /> Cancel
            </button>
          ) : (
            <button
              onClick={onSend}
              title="Send (Ctrl+Enter)"
              className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 shadow-sm"
            >
              Send <Play size={14} className="fill-white" />
            </button>
          )}

          {/* cURL actions */}
          <button
            onClick={handleCopyCurl}
            title="Copy as cURL"
            className={`p-2 rounded-md transition-all border ${
              curlCopied
                ? "text-green-400 bg-green-500/10 border-green-500/40"
                : "text-gray-400 hover:text-gray-200 hover:bg-gray-700/50 border-gray-700/60"
            }`}
          >
            <span className={`block transition-all duration-200 ${curlCopied ? "scale-110" : "scale-100"}`}>
              {curlCopied ? <Check size={15} /> : <Terminal size={15} />}
            </span>
          </button>
          <button
            onClick={() => { setImportText(""); setShowImport(true); }}
            title="Import from cURL"
            className="p-2 rounded-md text-gray-400 hover:text-gray-200 hover:bg-gray-700/50 transition-colors border border-gray-700/60"
          >
            <Upload size={15} />
          </button>
        </div>
      </div>

      {/* Import cURL modal */}
      {showImport && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm px-4"
          onClick={() => setShowImport(false)}
        >
          <div
            className="w-full max-w-2xl bg-[#161616] border border-gray-700 rounded-xl shadow-2xl overflow-hidden flex flex-col"
            onClick={e => e.stopPropagation()}
          >
            <div className="px-5 py-4 border-b border-gray-800 flex items-center gap-2">
              <Terminal size={15} className="text-gray-400" />
              <span className="text-sm font-semibold text-gray-200">Import from cURL</span>
            </div>
            <div className="p-5">
              <textarea
                autoFocus
                value={importText}
                onChange={e => setImportText(e.target.value)}
                placeholder={"curl 'https://api.example.com/data' \\\n  -H 'Authorization: Bearer token' \\\n  -H 'Content-Type: application/json' \\\n  --data '{\"key\":\"value\"}'"}
                className="w-full h-40 bg-[#0d0d0d] border border-gray-700 rounded-md p-3 font-mono text-xs text-gray-200 placeholder-gray-600 outline-none focus:border-gray-600 resize-none"
              />
            </div>
            <div className="px-5 pb-5 flex justify-end gap-3">
              <button
                onClick={() => setShowImport(false)}
                className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => { if (importText.trim()) { onImportCurl(importText.trim()); setShowImport(false); } }}
                disabled={!importText.trim()}
                className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded-md font-medium transition-colors disabled:opacity-40"
              >
                Import
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
