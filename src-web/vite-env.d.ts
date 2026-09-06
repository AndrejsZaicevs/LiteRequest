/// <reference types="vite/client" />

// Lezer grammars compiled by @lezer/generator/rollup at build time
declare module "*.grammar" {
  import type { LRParser } from "@lezer/lr";
  export const parser: LRParser;
}
