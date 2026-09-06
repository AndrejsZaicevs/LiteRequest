import { describe, it, expect } from "vitest";
import { EditorState } from "@codemirror/state";
import { CompletionContext } from "@codemirror/autocomplete";
import { javascript } from "@codemirror/lang-javascript";
import { scriptCompletionSource } from "../scriptCompletions";
import { parseScriptDts } from "../scriptApi";

const api = parseScriptDts(`
interface LrPostExec {
  request: { method: string; url: string };
  response: { status: number; json(): any };
  setVariable(name: string, value: unknown): void;
  getVariable(name: string): string | undefined;
}
declare const lr: LrPostExec;
declare const console: { log(...args: any[]): void };
`);

const KNOWN_VARS = { token: "abc123", baseUrl: "https://example.test" };

async function complete(doc: string, pos = doc.length) {
  const state = EditorState.create({ doc, extensions: [javascript({ typescript: true })] });
  const source = scriptCompletionSource({ load: async () => api, variables: () => KNOWN_VARS });
  return source(new CompletionContext(state, pos, false));
}

const labels = (r: Awaited<ReturnType<typeof complete>>) => r?.options.map(o => o.label) ?? null;

describe("scriptCompletionSource", () => {
  it("completes lr members", async () => {
    expect(labels(await complete("lr."))).toEqual(["request", "response", "setVariable", "getVariable"]);
  });

  it("completes nested members and reports the typed prefix range", async () => {
    const r = await complete("const s = lr.response.st");
    expect(labels(r)).toEqual(["status", "json"]);
    expect(r?.from).toBe("const s = lr.response.".length);
  });

  it("offers globals for a bare identifier", async () => {
    expect(labels(await complete("l"))).toEqual(["lr", "console"]);
  });

  it("completes known variable names inside setVariable / getVariable / variables[]", async () => {
    expect(labels(await complete('lr.setVariable("'))).toEqual(["token", "baseUrl"]);
    expect(labels(await complete("lr.getVariable('ba"))).toEqual(["token", "baseUrl"]);
    expect(labels(await complete('lr.variables["'))).toEqual(["token", "baseUrl"]);
    const r = await complete('lr.setVariable("to');
    expect(r?.from).toBe('lr.setVariable("'.length);
    expect(r?.options[0]).toMatchObject({ label: "token", detail: "abc123" });
  });

  it("also offers names set earlier in the same script", async () => {
    const doc = 'lr.setVariable("userId", "1");\nlr.getVariable("';
    expect(labels(await complete(doc))).toEqual(["token", "baseUrl", "userId"]);
  });

  it("does not treat other strings as variable names", async () => {
    expect(await complete('lr.log("')).toBeNull();
    expect(await complete('const s = "to')).toBeNull();
  });

  it("stays quiet inside comments", async () => {
    expect(await complete("// lr.")).toBeNull();
  });
});
