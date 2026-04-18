import { useState, useEffect, useRef, useCallback } from "react";

function readLS(key: string, fallback: number, min: number, max: number): number {
  const v = localStorage.getItem(key);
  return v ? Math.max(min, Math.min(max, Number(v))) : fallback;
}

export function useLayoutState() {
  const [sidebarWidth, setSidebarWidth] = useState(() => readLS("lr.sidebarWidth", 240, 160, 500));
  const [inspectorWidth, setInspectorWidth] = useState(() => readLS("lr.inspectorWidth", 280, 200, 600));
  const [splitRatio, setSplitRatio] = useState(() => readLS("lr.splitRatio", 0.5, 0.15, 0.85));
  const [splitOverride, setSplitOverride] = useState<"auto" | "request" | "response" | "split">("auto");

  const dragging = useRef<"sidebar" | "inspector" | "split" | null>(null);
  const splitContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!dragging.current) return;
      e.preventDefault();
      if (dragging.current === "sidebar") {
        setSidebarWidth(Math.max(180, Math.min(400, e.clientX)));
      } else if (dragging.current === "inspector") {
        setInspectorWidth(Math.max(200, Math.min(500, window.innerWidth - e.clientX)));
      } else if (dragging.current === "split") {
        const container = splitContainerRef.current;
        if (!container) return;
        const rect = container.getBoundingClientRect();
        const ratio = (e.clientY - rect.top) / rect.height;
        setSplitRatio(Math.max(0.2, Math.min(0.8, ratio)));
      }
    };
    const onMouseUp = () => {
      if (dragging.current === "sidebar") {
        setSidebarWidth(w => { localStorage.setItem("lr.sidebarWidth", String(w)); return w; });
      } else if (dragging.current === "inspector") {
        setInspectorWidth(w => { localStorage.setItem("lr.inspectorWidth", String(w)); return w; });
      } else if (dragging.current === "split") {
        setSplitRatio(r => { localStorage.setItem("lr.splitRatio", String(r)); return r; });
      }
      dragging.current = null;
      document.body.classList.remove("resizing");
    };
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("mouseup", onMouseUp);
    };
  }, []);

  const startDrag = useCallback((panel: "sidebar" | "inspector" | "split") => {
    dragging.current = panel;
    document.body.classList.add("resizing");
  }, []);

  return {
    sidebarWidth,
    inspectorWidth,
    splitRatio,
    splitOverride,
    setSplitOverride,
    splitContainerRef,
    startDrag,
  };
}
