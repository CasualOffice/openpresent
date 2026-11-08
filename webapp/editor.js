// The editor's entry point, and the only file the page names directly.
//
// Copied from OpenCalc's `webapp/editor.js` at `b23c67c` (webapp/README.md). Its
// reason holds here: a module's sub-imports carry no query string, so the page
// must name exactly one module and let it import the rest, or a stamped URL and
// an unstamped one become two instances of the same module, each with its own
// command registry.
globalThis.__openpresentBuild =
  new URL(import.meta.url).searchParams.get("v") || "dev";

export * from "./editor.core.js";
