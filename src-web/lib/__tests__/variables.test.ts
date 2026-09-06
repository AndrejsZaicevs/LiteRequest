import { describe, it, expect } from "vitest";
import { mergeVariables, resolveParentName, maskSecretVariables, collectSecretKeys, SECRET_MASK } from "../variables";
import { resolveVariableRefs } from "../types";
import type { Collection, Folder, Request } from "../types";

describe("mergeVariables", () => {
  it("collection variables override environment variables", () => {
    const merged = mergeVariables(
      [{ key: "host", value: "env.example.com" }, { key: "version", value: "v1" }],
      { host: "collection.example.com" },
      {},
    );
    expect(merged).toEqual({ host: "collection.example.com", version: "v1" });
  });

  it("built-ins override both scopes", () => {
    const merged = mergeVariables(
      [{ key: "collectionName", value: "from-env" }],
      { collectionName: "from-collection", requestName: "from-collection" },
      { collectionName: "Billing", requestName: "Create charge" },
    );
    expect(merged.collectionName).toBe("Billing");
    expect(merged.requestName).toBe("Create charge");
  });

  it("omits built-ins when not provided", () => {
    const merged = mergeVariables([{ key: "a", value: "1" }], {}, {});
    expect(merged).toEqual({ a: "1" });
  });

  it("keeps intentionally empty values", () => {
    const merged = mergeVariables([{ key: "token", value: "" }], {}, {});
    expect(merged).toEqual({ token: "" });
  });

  it("sets parentName when provided", () => {
    const merged = mergeVariables([], {}, { parentName: "parent" });
    expect(merged).toEqual({ parentName: "parent" });
  });
});

describe("resolveParentName", () => {
  const collection = { id: "c1", name: "testing" } as Collection;
  const folders = [
    { id: "f1", collection_id: "c1", parent_folder_id: null, name: "parent" },
  ] as Folder[];
  const request = (folder_id: string | null) =>
    ({ id: "r1", collection_id: "c1", folder_id, name: "request" }) as Request;

  it("returns the folder name when the request is in a folder", () => {
    expect(resolveParentName(request("f1"), folders, collection)).toBe("parent");
  });

  it("falls back to the collection name at collection root", () => {
    expect(resolveParentName(request(null), folders, collection)).toBe("testing");
  });

  it("falls back to the collection name when the folder is unknown", () => {
    expect(resolveParentName(request("missing"), folders, collection)).toBe("testing");
  });

  it("returns undefined without folder or collection", () => {
    expect(resolveParentName(request(null), [], undefined)).toBeUndefined();
  });
});

describe("collectSecretKeys", () => {
  it("collects secret env variables", () => {
    const keys = collectSecretKeys(
      [{ key: "API_KEY", is_secret: true }, { key: "host", is_secret: false }],
    );
    expect([...keys]).toEqual(["API_KEY"]);
  });

  it("collection secret values are included", () => {
    const keys = collectSecretKeys(
      [],
      [{ key: "token", is_secret: true, value_id: "v1" }],
    );
    expect(keys.has("token")).toBe(true);
  });

  it("a non-secret collection value un-hides an env secret of the same key", () => {
    const keys = collectSecretKeys(
      [{ key: "API_KEY", is_secret: true }],
      [{ key: "API_KEY", is_secret: false, value_id: "v1" }],
    );
    expect(keys.has("API_KEY")).toBe(false);
  });

  it("ignores collection rows with no value in the active env", () => {
    const keys = collectSecretKeys(
      [{ key: "API_KEY", is_secret: true }],
      [{ key: "API_KEY", is_secret: false, value_id: null }],
    );
    expect(keys.has("API_KEY")).toBe(true);
  });
});

describe("maskSecretVariables", () => {
  it("replaces secret values with the mask and leaves others intact", () => {
    const masked = maskSecretVariables(
      { API_KEY: "sk-live-123", host: "api.example.com" },
      new Set(["API_KEY"]),
    );
    expect(masked).toEqual({ API_KEY: SECRET_MASK, host: "api.example.com" });
  });

  it("supports a custom mask", () => {
    const masked = maskSecretVariables({ a: "1" }, new Set(["a"]), "<hidden>");
    expect(masked.a).toBe("<hidden>");
  });

  it("returns the same map when there are no secrets", () => {
    const vars = { a: "1" };
    expect(maskSecretVariables(vars, new Set())).toBe(vars);
  });

  it("masks secrets referenced from other variables when applied before ref resolution", () => {
    const vars = { API_KEY: "sk-live-123", auth: "Bearer {{API_KEY}}" };
    const resolved = resolveVariableRefs(maskSecretVariables(vars, new Set(["API_KEY"])));
    expect(resolved.auth).toBe(`Bearer ${SECRET_MASK}`);
    expect(JSON.stringify(resolved)).not.toContain("sk-live-123");
  });
});
