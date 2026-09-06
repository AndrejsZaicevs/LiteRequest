import type { Completion, CompletionContext, CompletionResult } from "@codemirror/autocomplete";
import { snippetCompletion } from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { generateScriptTypes } from "./api";
import { parseScriptDts, resolvePath, type ApiMember, type ScriptApi } from "./scriptApi";

let apiCache: Promise<ScriptApi | null> | null = null;

/** Fetch and parse the generated `.d.ts`; cached until `refreshScriptTypes()`. */
export function loadScriptApi(): Promise<ScriptApi | null> {
  if (!apiCache) {
    apiCache = generateScriptTypes()
      .then(parseScriptDts)
      .catch(err => {
        console.error("Failed to load script types:", err);
        apiCache = null;
        return null;
      });
  }
  return apiCache;
}

/** Drop the cached types. */
export function refreshScriptTypes() {
  apiCache = null;
}

const SKIP_NODES = new Set(["LineComment", "BlockComment", "TemplateString"]);
const IDENT = /^[\w$]*$/;
const VAR_CALL = /\.(?:setVariable|getVariable)\(\s*$|\.variables\s*\[\s*$/;
const VAR_SET_RE = /\.setVariable\(\s*["']([^"']+)["']/g;

export interface ScriptCompletionOptions {
  /** Fetch the parsed API; defaults to the cached Tauri call. */
  load?: () => Promise<ScriptApi | null>;
  /** Variables known to the app, used inside `setVariable("…")` and friends. */
  variables?: () => Record<string, string>;
}

/**
 * Completion source for post-execution scripts: `lr.…` members, nested paths
 * such as `lr.response.`, and known variable names inside `setVariable("…")`.
 */
export function scriptCompletionSource(options: ScriptCompletionOptions = {}) {
  const load = options.load ?? loadScriptApi;
  const variables = options.variables ?? (() => ({}));

  return async (context: CompletionContext): Promise<CompletionResult | null> => {
    const node = syntaxTree(context.state).resolveInner(context.pos, -1);
    if (SKIP_NODES.has(node.name)) return null;
    if (node.name === "String") return completeVariableName(context, variables());

    const api = await load();
    if (!api) return null;

    const access = context.matchBefore(/(?:[\w$]+\.)+[\w$]*/);
    if (access) {
      const parts = access.text.split(".");
      const typed = parts.pop() ?? "";
      const members = resolvePath(api, parts);
      if (!members) return null;
      return { from: context.pos - typed.length, options: members.map(toCompletion), validFor: IDENT };
    }

    const ident = context.matchBefore(/[\w$]+/);
    if (!ident && !context.explicit) return null;
    const options: Completion[] = Object.keys(api.globals).map(name => ({ label: name, type: "variable", boost: 10 }));
    return { from: ident?.from ?? context.pos, options, validFor: IDENT };
  };
}

/** Inside `lr.setVariable("…")`, `lr.getVariable("…")` or `lr.variables["…"]`, offer known variable names. */
function completeVariableName(
  context: CompletionContext,
  known: Record<string, string>,
): CompletionResult | null {
  const quoted = context.matchBefore(/["'][^"']*/);
  if (!quoted) return null;
  const lead = context.state.sliceDoc(Math.max(0, quoted.from - 40), quoted.from);
  if (!VAR_CALL.test(lead)) return null;

  const options: Completion[] = Object.entries(known).map(([name, value]) => ({
    label: name,
    type: "variable",
    detail: truncate(value, 40),
    boost: 10,
  }));

  // Names assigned earlier in this script are valid targets too.
  const seen = new Set(Object.keys(known));
  const source = context.state.doc.toString();
  let m: RegExpExecArray | null;
  VAR_SET_RE.lastIndex = 0;
  while ((m = VAR_SET_RE.exec(source)) !== null) {
    if (seen.has(m[1])) continue;
    seen.add(m[1]);
    options.push({ label: m[1], type: "variable", detail: "set in this script" });
  }

  if (options.length === 0) return null;
  return { from: quoted.from + 1, options, validFor: /^[^"']*$/ };
}

function toCompletion(member: ApiMember): Completion {
  const detail = member.type.replace(/\s+/g, " ");
  if (member.kind === "method") {
    const hasParams = !/^\(\s*\)/.test(member.type);
    return snippetCompletion(hasParams ? `${member.name}(#{})` : `${member.name}()`, {
      label: member.name,
      type: "method",
      detail: truncate(detail, 48),
      info: member.doc,
    });
  }
  return {
    label: member.name,
    type: "property",
    detail: truncate(detail, 48),
    info: member.doc,
  };
}

function truncate(text: string, max: number): string {
  return text.length > max ? text.slice(0, max - 1) + "…" : text;
}
