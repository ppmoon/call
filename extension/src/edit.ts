import { mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { functionName, LlmComplete, replaceFunction } from "./llm";

export type DiffDecision = "accept" | "reject";

export type DiffGate = (input: {
  originalPath: string;
  proposedPath: string;
}) => Promise<DiffDecision>;

export function autoAcceptGate(): DiffGate {
  return async () => "accept";
}

export function autoRejectGate(): DiffGate {
  return async () => "reject";
}

export async function proposeAndMaybeApply(opts: {
  workspaceRoot: string;
  filePath: string;
  signature: string;
  source: string;
  prompt: string;
  neighbors: string;
  instruction: string;
  complete: LlmComplete;
  gate: DiffGate;
  appendPrompt: (qn: string, prompt: string) => void;
  qualifiedName: string;
}): Promise<{ applied: boolean; proposed: string }> {
  const proposed = await opts.complete({
    prompt: opts.prompt,
    source: opts.source,
    signature: opts.signature,
    neighbors: opts.neighbors,
    instruction: opts.instruction,
  });
  const proposedPath = path.join(opts.workspaceRoot, ".call", "_proposal.py");
  mkdirSync(path.dirname(proposedPath), { recursive: true });
  writeFileSync(proposedPath, proposed);
  const decision = await opts.gate({
    originalPath: opts.filePath,
    proposedPath,
  });
  if (decision !== "accept") {
    return { applied: false, proposed };
  }
  writeFileSync(opts.filePath, proposed);
  opts.appendPrompt(opts.qualifiedName, opts.prompt);
  return { applied: true, proposed };
}

export function ghostFunctionName(prompt: string): string {
  const match = /def\s+([A-Za-z_]\w*)/.exec(prompt);
  if (match) {
    return match[1];
  }
  const words = prompt.toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_|_$/g, "");
  return words.slice(0, 24) || "ghost_fn";
}

export function applyGhost(source: string, callerSignature: string, newFnName: string): string {
  const withCall = replaceFunction(source, functionName(callerSignature), (full) => {
    const trimmed = full.replace(/\s*$/, "");
    return `${trimmed}\n    ${newFnName}()\n`;
  });
  return `${withCall.replace(/\s*$/, "")}\n\ndef ${newFnName}() -> str:\n    return "generated"\n`;
}

export function promptHistoryExists(workspaceRoot: string, hashOrQnsFile: string): boolean {
  return existsSync(hashOrQnsFile) || existsSync(path.join(workspaceRoot, ".call", "symbols"));
}

export { readFileSync };
