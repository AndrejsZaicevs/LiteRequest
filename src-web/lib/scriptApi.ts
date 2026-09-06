/**
 * Lightweight parser for the `.d.ts` produced by the Rust `typegen` module.
 *
 * It only understands the subset the generator emits (interfaces, inline
 * object types and `declare const` globals) and turns it into a member tree
 * that the script editor uses for completions. It is deliberately not a
 * TypeScript type checker.
 */

export interface ApiMember {
  name: string;
  kind: "property" | "method";
  /** Type annotation (properties) or call signature (methods), as written. */
  type: string;
  /** Text of a preceding `/** … *\/` doc comment, if any. */
  doc?: string;
  /** Members of an inline object type such as `request: { … }`. */
  members?: ApiMember[];
}

export interface ScriptApi {
  interfaces: Record<string, ApiMember[]>;
  /** `declare const name: Type` globals, name → type text. */
  globals: Record<string, string>;
}

export function parseScriptDts(dts: string): ScriptApi {
  const api: ScriptApi = { interfaces: {}, globals: {} };

  const blockRe = /(?:^|[\s;])(?:declare\s+)?interface\s+([\w$]+)\s*\{/g;
  let m: RegExpExecArray | null;
  while ((m = blockRe.exec(dts)) !== null) {
    const open = m.index + m[0].length - 1;
    const close = findClosingBrace(dts, open);
    if (close < 0) break;
    api.interfaces[m[1]] = parseMembers(dts.slice(open + 1, close));
    blockRe.lastIndex = close + 1;
  }

  const constRe = /declare\s+const\s+([\w$]+)\s*:\s*/g;
  while ((m = constRe.exec(dts)) !== null) {
    const start = m.index + m[0].length;
    const end = findStatementEnd(dts, start);
    api.globals[m[1]] = dts.slice(start, end).trim();
    constRe.lastIndex = end;
  }

  return api;
}

/** Members of a type expression: an inline `{ … }` or an `A & B` intersection of interfaces. */
export function membersOfType(api: ScriptApi, type: string): ApiMember[] {
  const t = type.trim();
  if (t.startsWith("{") && t.endsWith("}")) return parseMembers(t.slice(1, -1));

  const out: ApiMember[] = [];
  const seen = new Set<string>();
  for (const part of t.split("&")) {
    for (const member of api.interfaces[part.trim()] ?? []) {
      if (seen.has(member.name)) continue;
      seen.add(member.name);
      out.push(member);
    }
  }
  return out;
}

/** Members available on a global root such as `lr` or `console`. */
export function rootMembers(api: ScriptApi, root: string): ApiMember[] {
  const type = api.globals[root];
  return type ? membersOfType(api, type) : [];
}

/** Resolve a dotted access path (`["lr", "response"]`) to the members that can follow it. */
export function resolvePath(api: ScriptApi, path: string[]): ApiMember[] | null {
  const [root, ...rest] = path;
  if (!root) return null;

  let members = rootMembers(api, root);
  if (members.length === 0) return null;

  for (const segment of rest) {
    const member = members.find(x => x.name === segment);
    if (!member || member.kind === "method") return null;
    members = member.members ?? membersOfType(api, member.type);
    if (members.length === 0) return null;
  }
  return members;
}

// ── internals ───────────────────────────────────────────────

function parseMembers(body: string): ApiMember[] {
  const members: ApiMember[] = [];
  for (const raw of splitStatements(body)) {
    let stmt = raw.trim();
    if (!stmt) continue;

    let doc: string | undefined;
    const docMatch = stmt.match(/^\/\*\*([\s\S]*?)\*\/\s*/);
    if (docMatch) {
      doc = cleanDoc(docMatch[1]);
      stmt = stmt.slice(docMatch[0].length);
    }
    stmt = stmt.replace(/^\s*\/\/.*$/gm, "").trim();
    stmt = stmt.replace(/^(?:(?:export|declare|readonly)\s+)+/, "");
    if (!stmt) continue;

    const fn = stmt.match(/^function\s+([\w$]+)\s*(\([\s\S]*)$/);
    if (fn) {
      members.push({ name: fn[1], kind: "method", type: fn[2].trim(), doc });
      continue;
    }
    const method = stmt.match(/^([\w$]+)\??\s*(\([\s\S]*)$/);
    if (method) {
      members.push({ name: method[1], kind: "method", type: method[2].trim(), doc });
      continue;
    }
    const prop = stmt.match(/^([\w$]+)\??\s*:\s*([\s\S]+)$/);
    if (prop) {
      const type = prop[2].trim();
      const member: ApiMember = { name: prop[1], kind: "property", type, doc };
      if (type.startsWith("{") && type.endsWith("}")) {
        member.members = parseMembers(type.slice(1, -1));
      }
      members.push(member);
    }
  }
  return members;
}

/** Split on `;` at bracket depth zero. */
function splitStatements(body: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < body.length; i++) {
    const ch = body[i];
    if (ch === "{" || ch === "(" || ch === "[") depth++;
    else if (ch === "}" || ch === ")" || ch === "]") depth--;
    else if (ch === ";" && depth === 0) {
      out.push(body.slice(start, i));
      start = i + 1;
    }
  }
  out.push(body.slice(start));
  return out;
}

function findClosingBrace(text: string, open: number): number {
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    if (text[i] === "{") depth++;
    else if (text[i] === "}" && --depth === 0) return i;
  }
  return -1;
}

function findStatementEnd(text: string, start: number): number {
  let depth = 0;
  for (let i = start; i < text.length; i++) {
    const ch = text[i];
    if (ch === "{" || ch === "(" || ch === "[") depth++;
    else if (ch === "}" || ch === ")" || ch === "]") depth--;
    else if (ch === ";" && depth === 0) return i;
  }
  return text.length;
}

function cleanDoc(raw: string): string {
  return raw
    .split("\n")
    .map(line => line.replace(/^\s*\*\s?/, "").trim())
    .filter(Boolean)
    .join(" ");
}
