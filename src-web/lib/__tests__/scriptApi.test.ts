import { describe, it, expect } from "vitest";
import { parseScriptDts, resolvePath, rootMembers, membersOfType } from "../scriptApi";

const DTS = `
interface LrPostExec {
  /** The request as sent. */
  request: {
    method: string;
    url: string;
  };
  response: {
    status: number;
    json(): any;
  };
  variables: Record<string, string>;
  setVariable(name: string, value: unknown): void;
  getVariable(name: string): string | undefined;
  log(...args: any[]): void;
}

interface Extra {
  log(...args: any[]): void;
  extra?: string;
}

declare const lr: LrPostExec;
declare const both: LrPostExec & Extra;
declare const console: { log(...args: any[]): void };
`;

describe("parseScriptDts", () => {
  const api = parseScriptDts(DTS);

  it("collects interfaces and globals", () => {
    expect(Object.keys(api.interfaces).sort()).toEqual(["Extra", "LrPostExec"]);
    expect(api.globals.lr).toBe("LrPostExec");
    expect(api.globals.console).toBe("{ log(...args: any[]): void }");
  });

  it("distinguishes properties, methods, optional members and docs", () => {
    const byName = Object.fromEntries(api.interfaces.LrPostExec.map(m => [m.name, m]));
    expect(byName.variables).toMatchObject({ kind: "property", type: "Record<string, string>" });
    expect(byName.getVariable).toMatchObject({ kind: "method", type: "(name: string): string | undefined" });
    expect(byName.request.doc).toBe("The request as sent.");
    const extra = Object.fromEntries(api.interfaces.Extra.map(m => [m.name, m]));
    expect(extra.extra).toMatchObject({ kind: "property", type: "string" });
  });

  it("parses nested inline object types", () => {
    const request = api.interfaces.LrPostExec.find(m => m.name === "request")!;
    expect(request.members?.map(m => m.name)).toEqual(["method", "url"]);
    const response = api.interfaces.LrPostExec.find(m => m.name === "response")!;
    expect(response.members?.find(m => m.name === "json")?.kind).toBe("method");
  });
});

describe("resolution", () => {
  const api = parseScriptDts(DTS);

  it("resolves globals, intersections and inline object types", () => {
    expect(rootMembers(api, "lr").map(m => m.name)).toContain("setVariable");
    const both = rootMembers(api, "both").map(m => m.name);
    expect(both).toContain("extra");
    expect(both.filter(n => n === "log")).toHaveLength(1);
    expect(membersOfType(api, api.globals.console).map(m => m.name)).toEqual(["log"]);
  });

  it("walks dotted paths", () => {
    expect(resolvePath(api, ["lr", "request"])?.map(m => m.name)).toEqual(["method", "url"]);
    expect(resolvePath(api, ["lr", "response"])?.map(m => m.name)).toEqual(["status", "json"]);
  });

  it("returns null for unknown roots, opaque types and methods", () => {
    expect(resolvePath(api, ["nope"])).toBeNull();
    expect(resolvePath(api, ["lr", "variables"])).toBeNull();
    expect(resolvePath(api, ["lr", "log"])).toBeNull();
    expect(resolvePath(api, ["lr", "request", "method"])).toBeNull();
  });
});
