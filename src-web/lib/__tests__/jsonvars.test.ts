import { describe, it, expect } from "vitest";
import { jsonVarsLanguage } from "../jsonvars";

function parse(src: string) {
  return jsonVarsLanguage.parser.parse(src);
}

function nodeNames(src: string): string[] {
  const names: string[] = [];
  parse(src).iterate({ enter: n => { names.push(n.name); } });
  return names;
}

function hasErrors(src: string): boolean {
  let err = false;
  parse(src).iterate({ enter: n => { if (n.type.isError) err = true; } });
  return err;
}

describe("jsonvars grammar", () => {
  it("parses plain JSON without errors", () => {
    expect(hasErrors('{"a": [1, 2.5, -3e2], "b": {"c": null, "d": true}}')).toBe(false);
  });

  it("accepts {{variable}} as a value", () => {
    const src = '{ "external_contract_id": {{contractId}}, "status": "APPROVED" }';
    expect(hasErrors(src)).toBe(false);
    expect(nodeNames(src)).toContain("Variable");
  });

  it("accepts {{variable}} as a property name", () => {
    expect(hasErrors('{ {{key}}: 1 }')).toBe(false);
  });

  it("accepts dynamic variables and dotted names", () => {
    expect(hasErrors('[{{$randomInt}}, {{user.id}}, {{a-b}}]')).toBe(false);
  });

  it("keeps a variable inside a string as part of the string", () => {
    const names = nodeNames('{ "auth": "Bearer {{token}}" }');
    expect(names).toContain("String");
    expect(names).not.toContain("Variable");
  });

  it("does not treat {{ }} braces as object brackets", () => {
    const src = '{ "id": {{contractId}} }';
    const tree = parse(src);
    // Only one Object node, and it spans the whole document.
    const objects: { from: number; to: number }[] = [];
    tree.iterate({ enter: n => { if (n.name === "Object") objects.push({ from: n.from, to: n.to }); } });
    expect(objects).toEqual([{ from: 0, to: src.length }]);
  });

  it("still reports a real error", () => {
    expect(hasErrors('{ "a": }')).toBe(true);
  });
});
