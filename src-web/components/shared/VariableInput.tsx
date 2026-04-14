import { useRef, useState } from "react";

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
  /** Applied to the outer wrapper div (use for sizing: flex-1, w-0, min-w, etc.) */
  wrapperClassName?: string;
  /** Applied to both the input and the overlay (padding, font-size, bg, border, focus styles, etc.) */
  className?: string;
  placeholder?: string;
  readOnly?: boolean;
  onKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void;
  /** Extra style applied only to the input element */
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

  const hasVars = /\{\{[^}]+\}\}/.test(value);
  const segments = hasVars ? parseSegments(value) : [];
  const varSegments = segments.filter(s => s.type === "var");

  const syncScroll = () => {
    if (overlayRef.current && inputRef.current) {
      overlayRef.current.scrollLeft = inputRef.current.scrollLeft;
    }
  };

  return (
    <div className={`relative ${wrapperClassName}`}>
      {/* Color overlay — only rendered when {{vars}} are present */}
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

      {/* Input — text goes transparent when overlay is active */}
      <input
        ref={inputRef}
        value={value}
        onChange={e => onChange(e.target.value)}
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

      {/* Tooltip */}
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
