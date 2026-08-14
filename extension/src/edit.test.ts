import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { autoAcceptGate, autoRejectGate, proposeAndMaybeApply } from "./edit";
import { mockLlmComplete } from "./llm";

function tmpWorkspace(): string {
  const dir = path.join(os.tmpdir(), `call-edit-${Date.now()}`);
  mkdirSync(dir, { recursive: true });
  return dir;
}

describe("Diff Gate edit pipeline", () => {
  it("accept writes the file and records the prompt", async () => {
    const root = tmpWorkspace();
    const filePath = path.join(root, "fetch.py");
    const source = `def fetch(url: str) -> str:\n    return url\n`;
    writeFileSync(filePath, source);
    const recorded: string[] = [];
    const result = await proposeAndMaybeApply({
      workspaceRoot: root,
      filePath,
      signature: "def fetch(url: str)",
      source,
      prompt: "加重试逻辑",
      neighbors: "",
      instruction: "edit",
      complete: mockLlmComplete(),
      gate: autoAcceptGate(),
      appendPrompt: (qn, prompt) => recorded.push(`${qn}:${prompt}`),
      qualifiedName: "httpcli.fetch.fetch",
    });
    expect(result.applied).toBe(true);
    expect(readFileSync(filePath, "utf8")).toContain("_attempt");
    expect(recorded).toEqual(["httpcli.fetch.fetch:加重试逻辑"]);
    rmSync(root, { recursive: true, force: true });
  });

  it("reject leaves the file and prompt history unchanged", async () => {
    const root = tmpWorkspace();
    const filePath = path.join(root, "fetch.py");
    const source = `def fetch(url: str) -> str:\n    return url\n`;
    writeFileSync(filePath, source);
    const recorded: string[] = [];
    const result = await proposeAndMaybeApply({
      workspaceRoot: root,
      filePath,
      signature: "def fetch(url: str)",
      source,
      prompt: "加重试逻辑",
      neighbors: "",
      instruction: "edit",
      complete: mockLlmComplete(),
      gate: autoRejectGate(),
      appendPrompt: (_qn, prompt) => recorded.push(prompt),
      qualifiedName: "httpcli.fetch.fetch",
    });
    expect(result.applied).toBe(false);
    expect(readFileSync(filePath, "utf8")).toBe(source);
    expect(recorded).toEqual([]);
    rmSync(root, { recursive: true, force: true });
  });
});
