import { useRef, useState, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";
import { useTooltip } from "./TooltipPortal";

interface Segment {
  type: "text" | "var";
  content: string;
  name: string;
}

function parseSegments(text: string): Segment[] {
  const segments: Segment[] = [];
  const re = /\{\{([^}]+)\}\}/g;
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    if (m.index > last) segments.push({ type: "text", content: text.slice(last, m.index), name: "" });
    segments.push({ type: "var", content: m[0], name: m[1].trim() });
    last = m.index + m[0].length;
  }
  if (last < text.length) segments.push({ type: "text", content: text.slice(last), name: "" });
  return segments;
}

interface VariableInputProps {
  value: string;
  onChange: (v: string) => void;
  variables: Record<string, string>;
  wrapperClassName?: string;
  className?: string;
  placeholder?: string;
  readOnly?: boolean;
  onKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  onBlur?: () => void;
  inputStyle?: React.CSSProperties;
}

export function VariableInput({
  value, onChange, variables,
  wrapperClassName = "", className = "",
  placeholder, readOnly, onKeyDown, onBlur, inputStyle,
}: VariableInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const { show, hide } = useTooltip();

  const [displayValue, setDisplayValue] = useState(value);
  const lastEmittedRef = useRef(value);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  // Autocomplete state
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [dropdownPos, setDropdownPos] = useState<{ top: number; left: number; width: number } | null>(null);
  const variablesRef = useRef(variables);
  variablesRef.current = variables;

  const checkSuggestions = useCallback(() => {
    const el = inputRef.current;
    if (!el || document.activeElement !== el) { setSuggestions([]); return; }

    const pos = el.selectionStart ?? 0;
    const text = el.value;
    const before = text.slice(0, pos);

    const lastOpen = before.lastIndexOf("{{");
    if (lastOpen === -1) { setSuggestions([]); return; }

    const between = before.slice(lastOpen + 2);
    if (between.includes("}}")) { setSuggestions([]); return; }

    const partial = between.toLowerCase().trim();
    const allVars = Object.keys(variablesRef.current).sort();
    const filtered = partial
      ? allVars.filter(v => v.toLowerCase().includes(partial))
      : allVars;

    if (filtered.length === 0) { setSuggestions([]); return; }

    setSuggestions(filtered.slice(0, 12));
    setSelectedIdx(0);
    hide();

    if (wrapperRef.current) {
      const rect = wrapperRef.current.getBoundingClientRect();
      setDropdownPos({ top: rect.bottom + 2, left: rect.left, width: Math.max(rect.width, 240) });
    }
  }, [hide]);

  const applySuggestion = useCallback((varName: string) => {
    const el = inputRef.current;
    if (!el) return;

    const pos = el.selectionStart ?? 0;
    const text = el.value;
    const before = text.slice(0, pos);
    const lastOpen = before.lastIndexOf("{{");
    if (lastOpen === -1) return;

    const after = text.slice(pos);
    const closingMatch = after.match(/^\s*\}\}/);
    const skipLen = closingMatch ? closingMatch[0].length : 0;

    const newText = text.slice(0, lastOpen) + `{{${varName}}}` + text.slice(pos + skipLen);
    const newPos = lastOpen + varName.length + 4;

    el.value = newText;
    el.setSelectionRange(newPos, newPos);
    lastEmittedRef.current = newText;
    setDisplayValue(newText);
    onChangeRef.current(newText);
    setSuggestions([]);
    el.focus();
  }, []);

  useEffect(() => {
    const el = inputRef.current;
    if (!el) return;
    const handler = () => {
      const v = el.value;
      lastEmittedRef.current = v;
      setDisplayValue(v);
      onChangeRef.current(v);
      checkSuggestions();
    };
    el.addEventListener("input", handler);
    return () => el.removeEventListener("input", handler);
  }, [checkSuggestions]);

  useEffect(() => {
    if (value !== lastEmittedRef.current) {
      lastEmittedRef.current = value;
      setDisplayValue(value);
      if (inputRef.current) inputRef.current.value = value;
    }
  }, [value]);

  const hasVars = /\{\{[^}]+\}\}/.test(displayValue);
  const segments = hasVars ? parseSegments(displayValue) : [];
  const varSegments = segments.filter(s => s.type === "var");

  const syncScroll = () => {
    if (overlayRef.current && inputRef.current) {
      overlayRef.current.scrollLeft = inputRef.current.scrollLeft;
    }
  };

  const showVarTooltip = () => {
    if (suggestions.length > 0) return;
    if (varSegments.length === 0 || !wrapperRef.current) return;
    const rect = wrapperRef.current.getBoundingClientRect();
    show(rect, (
      <>
        {varSegments.map(s => (
          <div key={s.name} className="flex items-center gap-1.5 py-0.5">
            <span className={`font-mono ${
              s.name.startsWith("$") ? "text-purple-400"
              : variables[s.name] !== undefined ? "text-emerald-400"
              : "text-orange-400"
            }`}>
              {`{{${s.name}}}`}
            </span>
            <span className="text-gray-600">→</span>
            {s.name.startsWith("$")
              ? <span className="text-purple-300/70 italic">dynamic</span>
              : variables[s.name] !== undefined
                ? <span className="text-gray-200 font-mono max-w-[150px] truncate">{variables[s.name]}</span>
                : <span className="text-orange-400/70 italic">not set</span>
            }
          </div>
        ))}
      </>
    ));
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (suggestions.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault(); e.stopPropagation();
        setSelectedIdx(prev => Math.min(prev + 1, suggestions.length - 1));
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault(); e.stopPropagation();
        setSelectedIdx(prev => Math.max(prev - 1, 0));
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault(); e.stopPropagation();
        applySuggestion(suggestions[selectedIdx]);
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        setSuggestions([]);
        return;
      }
    }
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      setTimeout(checkSuggestions, 0);
    }
    onKeyDown?.(e);
  };

  return (
    <div ref={wrapperRef} className={`relative ${wrapperClassName}`}>
      {hasVars && (
        <div
          ref={overlayRef}
          aria-hidden
          className={`absolute inset-0 pointer-events-none overflow-hidden flex items-center ${className}`}
          style={{ background: "transparent", border: "transparent" }}
        >
          <span className="whitespace-pre">
            {segments.map((seg, i) =>
              seg.type === "text"
                ? <span key={i} className="text-gray-200">{seg.content}</span>
                : (
                  <span
                    key={i}
                    className={`rounded-sm ${
                      seg.name.startsWith("$")
                        ? "text-purple-400 bg-purple-400/10"
                        : variables[seg.name] !== undefined
                          ? "text-emerald-400 bg-emerald-400/10"
                          : "text-orange-400 bg-orange-400/10"
                    }`}
                  >
                    {seg.content}
                  </span>
                )
            )}
          </span>
        </div>
      )}

      <input
        ref={inputRef}
        defaultValue={value}
        placeholder={placeholder}
        readOnly={readOnly}
        className={`${className} w-full ${hasVars ? "text-transparent placeholder-transparent" : ""}`}
        style={{ ...inputStyle, ...(hasVars ? { caretColor: "#9ca3af" } : {}) }}
        onKeyDown={handleKeyDown}
        onScroll={hasVars ? syncScroll : undefined}
        onMouseEnter={showVarTooltip}
        onMouseLeave={hide}
        onFocus={() => { showVarTooltip(); checkSuggestions(); }}
        onBlur={() => { hide(); setSuggestions([]); onBlur?.(); }}
        onClick={checkSuggestions}
      />

      {suggestions.length > 0 && dropdownPos && createPortal(
        <div
          className="bg-[#1a1a1a] border border-gray-700 rounded-md shadow-2xl overflow-hidden text-xs"
          style={{
            position: "fixed",
            zIndex: 10000,
            top: dropdownPos.top,
            left: dropdownPos.left,
            width: dropdownPos.width,
            maxHeight: 200,
            overflowY: "auto",
          }}
          onMouseDown={(e) => e.preventDefault()}
        >
          {suggestions.map((name, i) => (
            <div
              key={name}
              className={`px-3 py-1.5 cursor-pointer flex items-center gap-2 ${
                i === selectedIdx ? "bg-blue-600/30 text-gray-100" : "text-gray-300 hover:bg-gray-800"
              }`}
              onMouseDown={(e) => { e.preventDefault(); applySuggestion(name); }}
              onMouseEnter={() => setSelectedIdx(i)}
            >
              <span className={`font-mono ${name.startsWith("$") ? "text-purple-400" : "text-emerald-400"}`}>
                {`{{${name}}}`}
              </span>
              {!name.startsWith("$") && variablesRef.current[name] !== undefined && (
                <span className="text-gray-500 truncate max-w-[120px] ml-auto">{variablesRef.current[name]}</span>
              )}
              {name.startsWith("$") && (
                <span className="text-purple-300/60 italic ml-auto">dynamic</span>
              )}
            </div>
          ))}
        </div>,
        document.body
      )}
    </div>
  );
}

