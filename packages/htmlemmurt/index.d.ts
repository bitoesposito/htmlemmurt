export interface EmmurtError {
  /** Error message, in Neapolitan */
  message: string;
  /** File the error is in: the page or a file it includes */
  file: string;
  /** 1-based line */
  line: number;
  /** 1-based column */
  column: number;
  /** A hint on how to fix it, when there is one */
  suggestion: string | null;
}

export interface CompileOptions {
  /** Name of the page, for errors and to resolve relative paths (default: "input.emmurt"). */
  filename?: string;
  /**
   * Reads a file named by `<chiamm da>` or an inlined `<fratm da>`: its text, or null when it
   * does not exist. Default in Node: from disk, `/…` paths from `root`.
   */
  read?: (path: string) => string | null;
  /** Folder for paths that start with `/` (Node only, default: the working directory). */
  root?: string;
  /**
   * Write `<fratm da="file.fratm">` files into the page instead of leaving
   * `<script type="module" src>` for a bundler (default: false).
   */
  inline?: boolean;
}

export interface CompileResult {
  /** The HTML, or null when there are errors */
  html: string | null;
  /** Files read while compiling */
  dependencies: string[];
  errors: EmmurtError[];
}

export interface FratmResult {
  /** The JavaScript, or null when there are errors */
  code: string | null;
  /** Source map (v3) back to the .fratm file */
  map: { version: number; file: string; sources: string[]; sourcesContent: string[]; names: string[]; mappings: string } | null;
  errors: EmmurtError[];
}

export interface Vocabulary {
  tags: [string, string][];
  attributes: { nap: string; html: string; only?: string[]; values?: [string, string][] }[];
  events: [string, string][];
}

/** Compiles htmlemmurt to HTML. Never throws on invalid source: check `errors`. */
export function compile(source: string, options?: CompileOptions): CompileResult;
/** Compiles a FratmScript module to JavaScript. */
export function compileFratm(source: string, options?: { filename?: string }): FratmResult;
/** Every Neapolitan name and its HTML. */
export function vocabulary(): Vocabulary;
/** Version of the compiler. */
export function version(): string;
/** A JavaScript runtime error in Neapolitan, with the original in brackets. */
export function runtimeError(error: unknown): string;
/** `file:line:column: message`, then the hint. */
export function formatError(error: EmmurtError): string;
