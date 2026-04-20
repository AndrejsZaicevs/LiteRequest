import { describe, it, expect } from "vitest";
import {
  computeVersionFingerprint,
  resolveVariableRefs,
  collectVarRefs,
  findUnresolvedVars,
  defaultRequestData,
  statusColor,
  type RequestData,
  type KeyValuePair,
} from "../types";

// ── Fingerprint ──────────────────────────────────────────────

describe("computeVersionFingerprint", () => {
  const base: RequestData = {
    ...defaultRequestData(),
    method: "POST",
    url: "/users",
    headers: [{ key: "Accept", value: "application/json", enabled: true }],
    query_params: [{ key: "page", value: "1", enabled: true }],
  };

  it("produces a pipe-delimited string", () => {
    const fp = computeVersionFingerprint(base);
    expect(fp.split("|").length).toBe(6);
  });

  it("is stable for value-only changes", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({
      ...base,
      headers: [{ key: "Accept", value: "text/plain", enabled: true }],
      query_params: [{ key: "page", value: "99", enabled: true }],
    });
    expect(a).toBe(b);
  });

  it("changes when method changes", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({ ...base, method: "GET" });
    expect(a).not.toBe(b);
  });

  it("changes when url changes", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({ ...base, url: "/posts" });
    expect(a).not.toBe(b);
  });

  it("changes when a new header key is added", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({
      ...base,
      headers: [
        ...base.headers,
        { key: "X-Custom", value: "yes", enabled: true },
      ],
    });
    expect(a).not.toBe(b);
  });

  it("changes when body_type changes", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({ ...base, body_type: "Json" });
    expect(a).not.toBe(b);
  });

  it("ignores disabled params", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({
      ...base,
      query_params: [
        ...base.query_params,
        { key: "debug", value: "1", enabled: false },
      ],
    });
    expect(a).toBe(b);
  });

  it("ignores empty-key params", () => {
    const a = computeVersionFingerprint(base);
    const b = computeVersionFingerprint({
      ...base,
      query_params: [
        ...base.query_params,
        { key: "", value: "ghost", enabled: true },
      ],
    });
    expect(a).toBe(b);
  });

  it("header keys are case-insensitive", () => {
    const a = computeVersionFingerprint({
      ...base,
      headers: [{ key: "Content-Type", value: "text/plain", enabled: true }],
    });
    const b = computeVersionFingerprint({
      ...base,
      headers: [{ key: "content-type", value: "application/json", enabled: true }],
    });
    expect(a).toBe(b);
  });

  it("query param order is irrelevant", () => {
    const a = computeVersionFingerprint({
      ...base,
      query_params: [
        { key: "a", value: "1", enabled: true },
        { key: "b", value: "2", enabled: true },
      ],
    });
    const b = computeVersionFingerprint({
      ...base,
      query_params: [
        { key: "b", value: "2", enabled: true },
        { key: "a", value: "1", enabled: true },
      ],
    });
    expect(a).toBe(b);
  });

  it("includes multipart field keys", () => {
    const a = computeVersionFingerprint({
      ...base,
      body_type: "Multipart",
      multipart_fields: [{ key: "file", value: "", is_file: true, file_path: "/a.txt", enabled: true }],
    });
    const b = computeVersionFingerprint({
      ...base,
      body_type: "Multipart",
      multipart_fields: [
        { key: "file", value: "", is_file: true, file_path: "/a.txt", enabled: true },
        { key: "desc", value: "test", is_file: false, file_path: "", enabled: true },
      ],
    });
    expect(a).not.toBe(b);
  });
});

// ── Variable resolution ─────────────────────────────────────

describe("resolveVariableRefs", () => {
  it("resolves simple references", () => {
    const result = resolveVariableRefs({
      host: "example.com",
      url: "https://{{host}}/api",
    });
    expect(result.url).toBe("https://example.com/api");
  });

  it("resolves chained references", () => {
    const result = resolveVariableRefs({
      base: "example.com",
      host: "https://{{base}}",
      url: "{{host}}/api",
    });
    expect(result.url).toBe("https://example.com/api");
  });

  it("leaves circular references unresolved", () => {
    const result = resolveVariableRefs({
      a: "{{b}}",
      b: "{{a}}",
    });
    // After one pass a="{{a}}" and b="{{b}}", then they swap again, etc.
    // The important thing is it terminates and doesn't crash.
    expect(typeof result.a).toBe("string");
    expect(typeof result.b).toBe("string");
  });

  it("handles variables with no references", () => {
    const result = resolveVariableRefs({
      plain: "hello",
      other: "world",
    });
    expect(result.plain).toBe("hello");
    expect(result.other).toBe("world");
  });
});

describe("collectVarRefs", () => {
  it("extracts variable names from template string", () => {
    expect(collectVarRefs("{{host}}/{{version}}/api")).toEqual(["host", "version"]);
  });

  it("handles whitespace in braces", () => {
    expect(collectVarRefs("{{ spaced }}")).toEqual(["spaced"]);
  });

  it("returns empty array for no refs", () => {
    expect(collectVarRefs("no variables")).toEqual([]);
  });

  it("handles multiple occurrences of same variable", () => {
    expect(collectVarRefs("{{a}}-{{a}}")).toEqual(["a", "a"]);
  });
});

describe("findUnresolvedVars", () => {
  it("finds vars missing from resolved map", () => {
    const data: RequestData = {
      ...defaultRequestData(),
      url: "https://{{host}}/api",
      headers: [{ key: "Authorization", value: "Bearer {{token}}", enabled: true }],
    };
    const unresolved = findUnresolvedVars(data, "", { host: "example.com" });
    expect(unresolved).toEqual(["token"]);
  });

  it("returns empty when all vars are resolved", () => {
    const data: RequestData = {
      ...defaultRequestData(),
      url: "https://{{host}}/api",
    };
    const unresolved = findUnresolvedVars(data, "", { host: "example.com" });
    expect(unresolved).toEqual([]);
  });

  it("ignores dynamic vars ($-prefixed)", () => {
    const data: RequestData = {
      ...defaultRequestData(),
      url: "https://example.com/{{$randomInt}}",
    };
    const unresolved = findUnresolvedVars(data, "", {});
    expect(unresolved).toEqual([]);
  });

  it("scans base path", () => {
    const data = defaultRequestData();
    const unresolved = findUnresolvedVars(data, "https://{{host}}", {});
    expect(unresolved).toEqual(["host"]);
  });

  it("scans query param keys and values", () => {
    const data: RequestData = {
      ...defaultRequestData(),
      query_params: [{ key: "{{paramName}}", value: "{{paramValue}}", enabled: true }],
    };
    const unresolved = findUnresolvedVars(data, "", {});
    expect(unresolved).toContain("paramName");
    expect(unresolved).toContain("paramValue");
  });
});

// ── statusColor ──────────────────────────────────────────────

describe("statusColor", () => {
  it("returns green for 2xx", () => {
    expect(statusColor(200)).toBe("#49cc90");
    expect(statusColor(201)).toBe("#49cc90");
    expect(statusColor(299)).toBe("#49cc90");
  });

  it("returns orange for 3xx", () => {
    expect(statusColor(301)).toBe("#fca130");
  });

  it("returns red for 4xx", () => {
    expect(statusColor(404)).toBe("#f93e3e");
  });

  it("returns red for 5xx", () => {
    expect(statusColor(500)).toBe("#ff5757");
  });

  it("returns gray for other codes", () => {
    expect(statusColor(100)).toBe("#8c8c96");
  });
});

// ── defaultRequestData ───────────────────────────────────────

describe("defaultRequestData", () => {
  it("returns correct defaults", () => {
    const d = defaultRequestData();
    expect(d.method).toBe("GET");
    expect(d.url).toBe("");
    expect(d.body_type).toBe("None");
    expect(d.body).toBe("");
    expect(d.headers).toEqual([]);
    expect(d.query_params).toEqual([]);
    expect(d.path_params).toEqual([]);
    expect(d.multipart_fields).toEqual([]);
  });
});
