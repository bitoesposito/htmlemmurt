export type { EmmurtError, CompileResult, Vocabulary } from './index.js';
import type { CompileOptions, CompileResult, Vocabulary, EmmurtError } from './index.js';

/** Loads the WebAssembly module (by default the .wasm file next to this one). Call once, await it. */
export function init(input?: string | URL | Request | Response | BufferSource | WebAssembly.Module): Promise<void>;
/** Compiles htmlemmurt to HTML. Without `read`, includes cannot be resolved. */
export function compile(source: string, options?: Omit<CompileOptions, 'root'>): CompileResult;
export function vocabulary(): Vocabulary;
export function version(): string;
export function runtimeError(error: unknown): string;
export function formatError(error: EmmurtError): string;
