import type { Text } from "@codemirror/state";
import { foldService } from "@codemirror/language";

export interface FoldRange { from: number; to: number }

/**
 * Compute fold ranges for pretty-printed JSON by matching opening and
 * closing brackets line by line. Works on the raw text, so it does not
 * depend on the syntax tree — CodeMirror parses large documents lazily
 * around the viewport, which leaves fold markers missing on in-view lines
 * whose matching bracket has not been parsed yet.
 *
 * Assumes the layout produced by `JSON.stringify(value, null, indent)`:
 * one bracket opens at the end of a line and its closing bracket starts a
 * later line. Strings never contain raw newlines in that output, so a
 * line ending in `{`/`[` is always an opener and a line starting with
 * `}`/`]` is always a closer.
 *
 * The result maps each opener line's start position to the range that
 * folds (just after the bracket → just before its closing bracket),
 * matching CodeMirror's built-in `foldInside` behaviour.
 */
export function computeJsonFolds(doc: Text): Map<number, FoldRange> {
  const folds = new Map<number, FoldRange>();
  const stack: { lineFrom: number; foldFrom: number }[] = [];
  const iter = doc.iterLines();
  let pos = 0;
  for (let step = iter.next(); !step.done; step = iter.next()) {
    const text = step.value;
    const trimmedEnd = text.trimEnd();
    const lead = text.length - text.trimStart().length;
    const first = text.charCodeAt(lead);
    const last = trimmedEnd.charCodeAt(trimmedEnd.length - 1);

    // Closer: line starts with `}` or `]`
    if ((first === 125 /* } */ || first === 93 /* ] */) && stack.length > 0) {
      const opener = stack.pop()!;
      const to = pos + lead;
      if (to > opener.foldFrom) folds.set(opener.lineFrom, { from: opener.foldFrom, to });
    }
    // Opener: line ends with `{` or `[`
    if (last === 123 /* { */ || last === 91 /* [ */) {
      stack.push({ lineFrom: pos, foldFrom: pos + trimmedEnd.length });
    }
    pos += text.length + 1;
  }
  return folds;
}

const cache = new WeakMap<Text, Map<number, FoldRange>>();

function foldsFor(doc: Text): Map<number, FoldRange> {
  let folds = cache.get(doc);
  if (!folds) {
    folds = computeJsonFolds(doc);
    cache.set(doc, folds);
  }
  return folds;
}

/**
 * Fold service for pretty-printed JSON. Consulted before syntax-tree
 * folding, so fold markers appear immediately for every line in view
 * regardless of how much of the document the parser has reached.
 */
export const jsonBracketFolding = foldService.of((state, lineStart) =>
  foldsFor(state.doc).get(lineStart) ?? null,
);
