/**
 * Transpile TypeScript source to JavaScript using the TypeScript compiler API.
 * Loaded lazily: the compiler is a large chunk that is only needed when a
 * script is saved or run, never for rendering the editor.
 */
export async function transpileTS(source: string): Promise<string> {
  const ts = await import("typescript");
  const result = ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2020,
      module: ts.ModuleKind.None,
    },
  });
  if (!result.outputText) {
    throw new Error("TypeScript transpilation produced no output");
  }
  return result.outputText;
}
