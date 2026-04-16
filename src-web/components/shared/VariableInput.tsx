import { useRef, useState, useEffect } from "react";

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
  inputStyle?: React.CSSProperties;
}

export function VariableInput({
  value, onChange, variables,
  wrapperClassName = "", className = "",
  placeholder, readOnly, onKeyDown, inputStyle,
}: VariableInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);
  const [showTooltip, setShowTooltip] = useState(false);

  // displayValue drives the overlay only. The actual <input> DOM value is
  // managed imperatively — React never touches input.value after mount.
  const [displayValue, setDisplayValue] = useState(value);
  const lastEmittedRef = useRef(value);
  // Keep onChange in a ref so the native listener never goes stale.
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  // Register a native DOM input listener so React's reconciler is completely
  // out of the picture for value updates — this is the only reliable way to
  // prevent React 19 from resetting the cursor position on re-renders.
  useEffect(() => {
    const el = inputRef.current;
    if (!el) return;
    const handler = () => {
      const v = el.value;
      lastEmittedRef.current = v;
      setDisplayValue(v);
      onChangeRef.current(v);
    };
    el.addEventListener("input", handler);
    return () => el.removeEventListener("input", handler);
  }, []); // register once on mount

  // Sync prop → DOM only for genuine external changes (e.g. switching requests).
  useEffect(() => {
    if (value !== lastEmittedRef.current) {
      lastEmittedRef.current = value;
      setDisplayValue(value);
      if (inputRef.current) {
        inputRef.current.value = value;
      }
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

  return (
    <div className={`relative ${wrapperClassName}`}>
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
                      variables[seg.name] !== undefined
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

      {/* Fully uncontrolled — no value/onChange props on the element itself.
          The native "input" listener above handles all user edits without
          going through React's controlled-input pipeline. */}
      <input
        ref={inputRef}
        defaultValue={value}
        placeholder={placeholder}
        readOnly={readOnly}
        className={`${className} w-full ${hasVars ? "text-transparent placeholder-transparent" : ""}`}
        style={{ ...inputStyle, ...(hasVars ? { caretColor: "#9ca3af" } : {}) }}
        onKeyDown={onKeyDown}
        onScroll={hasVars ? syncScroll : undefined}
        onMouseEnter={() => varSegments.length > 0 && setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
        onFocus={() => varSegments.length > 0 && setShowTooltip(true)}
        onBlur={() => setShowTooltip(false)}
      />

      {showTooltip && varSegments.length > 0 && (
        <div className="absolute top-full left-0 z-50 mt-1 bg-[#1a1a1a] border border-gray-700 rounded shadow-xl p-2 min-w-[160px] text-xs pointer-events-none">
          {varSegments.map(s => (
            <div key={s.name} className="flex items-center gap-1.5 py-0.5">
              <span className={`font-mono ${variables[s.name] !== undefined ? "text-emerald-400" : "text-orange-400"}`}>
                {`{{${s.name}}}`}
              </span>
              <span className="text-gray-600">→</span>
              {variables[s.name] !== undefined
                ? <span className="text-gray-200 font-mono max-w-[150px] truncate">{variables[s.name]}</span>
                : <span className="text-orange-400/70 italic">not set</span>
              }
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
