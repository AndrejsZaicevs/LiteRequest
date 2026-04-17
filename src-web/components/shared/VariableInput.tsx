import { useRef, useState, useEffect } from "react";
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
  inputStyle?: React.CSSProperties;
}

export function VariableInput({
  value, onChange, variables,
  wrapperClassName = "", className = "",
  placeholder, readOnly, onKeyDown, inputStyle,
}: VariableInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const overlayRef = useRef<HTMLDivElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const { show, hide } = useTooltip();

  const [displayValue, setDisplayValue] = useState(value);
  const lastEmittedRef = useRef(value);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

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
  }, []);

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
        onKeyDown={onKeyDown}
        onScroll={hasVars ? syncScroll : undefined}
        onMouseEnter={showVarTooltip}
        onMouseLeave={hide}
        onFocus={showVarTooltip}
        onBlur={hide}
      />
    </div>
  );
}

