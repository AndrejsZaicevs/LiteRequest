import { describe, it, expect } from "vitest";
import {
  DYNAMIC_VARS,
  DYNAMIC_VAR_NAMES,
  isDynamicVar,
  resolveDynamicVars,
  getDynamicVarPreviews,
} from "../dynamicVars";

describe("DYNAMIC_VARS registry", () => {
  it("has all names starting with $", () => {
    for (const name of Object.keys(DYNAMIC_VARS)) {
      expect(name).toMatch(/^\$/);
    }
  });

  it("every generator returns a string", () => {
    for (const [name, gen] of Object.entries(DYNAMIC_VARS)) {
      const val = gen();
      expect(typeof val).toBe("string");
      expect(val.length).toBeGreaterThan(0);
    }
  });

  it("DYNAMIC_VAR_NAMES is consistent with DYNAMIC_VARS keys", () => {
    expect(DYNAMIC_VAR_NAMES.size).toBe(Object.keys(DYNAMIC_VARS).length);
    for (const name of Object.keys(DYNAMIC_VARS)) {
      expect(DYNAMIC_VAR_NAMES.has(name)).toBe(true);
    }
  });
});

describe("isDynamicVar", () => {
  it("returns true for $ prefixed names", () => {
    expect(isDynamicVar("$randomInt")).toBe(true);
    expect(isDynamicVar("$anything")).toBe(true);
  });

  it("returns false for non-$ names", () => {
    expect(isDynamicVar("host")).toBe(false);
    expect(isDynamicVar("")).toBe(false);
  });
});

describe("resolveDynamicVars", () => {
  it("populates all dynamic vars into result", () => {
    const result = resolveDynamicVars({});
    for (const name of Object.keys(DYNAMIC_VARS)) {
      expect(result).toHaveProperty(name);
      expect(typeof result[name]).toBe("string");
    }
  });

  it("does not overwrite existing entries", () => {
    const result = resolveDynamicVars({ $randomInt: "42" });
    expect(result.$randomInt).toBe("42");
  });

  it("preserves non-dynamic variables", () => {
    const result = resolveDynamicVars({ host: "example.com" });
    expect(result.host).toBe("example.com");
  });
});

describe("getDynamicVarPreviews", () => {
  it("returns stable values across calls", () => {
    const a = getDynamicVarPreviews();
    const b = getDynamicVarPreviews();
    expect(a).toBe(b); // same reference (cached)
  });

  it("has an entry for every dynamic var", () => {
    const previews = getDynamicVarPreviews();
    for (const name of Object.keys(DYNAMIC_VARS)) {
      expect(previews).toHaveProperty(name);
    }
  });
});

describe("specific generators", () => {
  it("$randomUUID matches UUID v4 format", () => {
    const val = DYNAMIC_VARS.$randomUUID();
    expect(val).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/
    );
  });

  it("$guid matches UUID v4 format", () => {
    const val = DYNAMIC_VARS.$guid();
    expect(val).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/
    );
  });

  it("$randomInt is an integer 0-1000", () => {
    for (let i = 0; i < 50; i++) {
      const n = parseInt(DYNAMIC_VARS.$randomInt());
      expect(n).toBeGreaterThanOrEqual(0);
      expect(n).toBeLessThanOrEqual(1000);
    }
  });

  it("$randomEmail looks like an email", () => {
    const val = DYNAMIC_VARS.$randomEmail();
    expect(val).toMatch(/.+@.+\..+/);
  });

  it("$randomIP is a valid IPv4", () => {
    const val = DYNAMIC_VARS.$randomIP();
    const parts = val.split(".");
    expect(parts).toHaveLength(4);
    for (const p of parts) {
      const n = parseInt(p);
      expect(n).toBeGreaterThanOrEqual(0);
      expect(n).toBeLessThanOrEqual(255);
    }
  });

  it("$randomHexColor starts with #", () => {
    const val = DYNAMIC_VARS.$randomHexColor();
    expect(val).toMatch(/^#[0-9a-f]{6}$/);
  });

  it("$timestamp is a Unix timestamp", () => {
    const val = parseInt(DYNAMIC_VARS.$timestamp());
    const now = Math.floor(Date.now() / 1000);
    expect(Math.abs(val - now)).toBeLessThan(5);
  });

  it("$isoTimestamp is valid ISO 8601", () => {
    const val = DYNAMIC_VARS.$isoTimestamp();
    expect(new Date(val).toISOString()).toBe(val);
  });

  it("$randomBoolean is 'true' or 'false'", () => {
    for (let i = 0; i < 20; i++) {
      expect(["true", "false"]).toContain(DYNAMIC_VARS.$randomBoolean());
    }
  });

  it("$randomZipCode is 5 digits", () => {
    const val = DYNAMIC_VARS.$randomZipCode();
    expect(val).toMatch(/^\d{5}$/);
  });

  it("$randomSemVer matches semver format", () => {
    const val = DYNAMIC_VARS.$randomSemVer();
    expect(val).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it("$randomLatitude is in range -90 to 90", () => {
    const val = parseFloat(DYNAMIC_VARS.$randomLatitude());
    expect(val).toBeGreaterThanOrEqual(-90);
    expect(val).toBeLessThanOrEqual(90);
  });

  it("$randomLongitude is in range -180 to 180", () => {
    const val = parseFloat(DYNAMIC_VARS.$randomLongitude());
    expect(val).toBeGreaterThanOrEqual(-180);
    expect(val).toBeLessThanOrEqual(180);
  });

  it("$randomMACAddress matches MAC format", () => {
    const val = DYNAMIC_VARS.$randomMACAddress();
    expect(val).toMatch(/^[0-9a-f]{2}(:[0-9a-f]{2}){5}$/);
  });

  it("$randomIPV6 has 8 groups", () => {
    const val = DYNAMIC_VARS.$randomIPV6();
    expect(val.split(":")).toHaveLength(8);
  });

  it("$randomPrice has dollars and cents", () => {
    const val = DYNAMIC_VARS.$randomPrice();
    expect(val).toMatch(/^\d+\.\d{2}$/);
  });
});
