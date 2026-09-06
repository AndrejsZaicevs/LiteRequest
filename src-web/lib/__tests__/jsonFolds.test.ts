import { describe, it, expect } from "vitest";
import { Text } from "@codemirror/state";
import { computeJsonFolds } from "../jsonFolds";

function docOf(value: unknown): { doc: Text; text: string } {
  const text = JSON.stringify(value, null, 2);
  return { doc: Text.of(text.split("\n")), text };
}

describe("computeJsonFolds", () => {
  it("folds a top-level array from after `[` to before `]`", () => {
    const { doc, text } = docOf([{ a: 1 }, { b: 2 }]);
    const folds = computeJsonFolds(doc);
    const top = folds.get(0)!;
    expect(top).toBeDefined();
    expect(text.slice(0, top.from)).toBe("[");
    expect(text.slice(top.to)).toBe("]");
  });

  it("folds nested objects to just before their closing bracket", () => {
    const { doc, text } = docOf({ user: { id: 1, tags: ["x", "y"] } });
    const folds = computeJsonFolds(doc);
    const userLine = doc.line(2);
    const range = folds.get(userLine.from)!;
    expect(text.slice(userLine.from, range.from)).toBe('  "user": {');
    expect(text.slice(range.to, range.to + 1)).toBe("}");
    const tagsLine = doc.line(4);
    const tagsRange = folds.get(tagsLine.from)!;
    expect(text.slice(tagsLine.from, tagsRange.from)).toBe('    "tags": [');
    expect(text.slice(tagsRange.to, tagsRange.to + 1)).toBe("]");
  });

  it("does not fold empty containers or scalar lines", () => {
    const { doc } = docOf({ empty: {}, list: [], s: "a{", n: 1 });
    const folds = computeJsonFolds(doc);
    // Only the outer object is foldable.
    expect([...folds.keys()]).toEqual([0]);
  });

  it("ignores brackets inside string values", () => {
    const { doc } = docOf({ a: "[", b: "{", c: "}" });
    const folds = computeJsonFolds(doc);
    expect(folds.size).toBe(1);
    expect(folds.has(0)).toBe(true);
  });

  it("returns one range per opener for a large document", () => {
    const items = Array.from({ length: 5000 }, (_, i) => ({ id: i, meta: { k: "v" } }));
    const { doc } = docOf(items);
    const folds = computeJsonFolds(doc);
    // outer array + 5000 items + 5000 meta objects
    expect(folds.size).toBe(1 + 5000 * 2);
  });
});
