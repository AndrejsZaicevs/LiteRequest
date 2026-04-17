import { createContext, useContext, useState, useCallback, useRef, useEffect, type ReactNode } from "react";
import { createPortal } from "react-dom";

interface TooltipState {
  content: ReactNode;
  x: number;
  y: number;
  /** true = above anchor, false = below */
  above: boolean;
  /** true = right-aligned to anchor, false = left-aligned */
  alignRight: boolean;
}

interface TooltipContextValue {
  show: (anchor: DOMRect, content: ReactNode) => void;
  hide: () => void;
}

const TooltipContext = createContext<TooltipContextValue>({
  show: () => {},
  hide: () => {},
});

export function useTooltip() {
  return useContext(TooltipContext);
}

const TOOLTIP_GAP = 6;
const SCREEN_MARGIN = 8;
// Estimated max width — actual flip decision is made after mount via ref
const ESTIMATE_WIDTH = 220;
const ESTIMATE_HEIGHT = 80;

export function TooltipProvider({ children }: { children: ReactNode }) {
  const [tip, setTip] = useState<TooltipState | null>(null);
  const tipRef = useRef<HTMLDivElement>(null);

  const show = useCallback((anchor: DOMRect, content: ReactNode) => {
    const above = anchor.bottom + TOOLTIP_GAP + ESTIMATE_HEIGHT > window.innerHeight - SCREEN_MARGIN;
    const alignRight = anchor.left + ESTIMATE_WIDTH > window.innerWidth - SCREEN_MARGIN;

    setTip({
      content,
      x: alignRight ? anchor.right : anchor.left,
      y: above ? anchor.top : anchor.bottom,
      above,
      alignRight,
    });
  }, []);

  const hide = useCallback(() => setTip(null), []);

  // After render, correct position using actual measured dimensions
  useEffect(() => {
    if (!tip || !tipRef.current) return;
    const rect = tipRef.current.getBoundingClientRect();
    const above = tip.above || rect.bottom > window.innerHeight - SCREEN_MARGIN;
    const alignRight = tip.alignRight || rect.right > window.innerWidth - SCREEN_MARGIN;
    if (above !== tip.above || alignRight !== tip.alignRight) {
      setTip(prev => prev ? { ...prev, above, alignRight } : null);
    }
  });

  return (
    <TooltipContext.Provider value={{ show, hide }}>
      {children}
      {tip && createPortal(
        <div
          ref={tipRef}
          className="bg-[#1a1a1a] border border-gray-700 rounded shadow-xl p-2 min-w-[160px] text-xs pointer-events-none"
          style={{
            position: "fixed",
            zIndex: 9999,
            top: tip.above ? undefined : tip.y + TOOLTIP_GAP,
            bottom: tip.above ? window.innerHeight - tip.y + TOOLTIP_GAP : undefined,
            left: tip.alignRight ? undefined : tip.x,
            right: tip.alignRight ? window.innerWidth - tip.x : undefined,
          }}
        >
          {tip.content}
        </div>,
        document.body,
      )}
    </TooltipContext.Provider>
  );
}
