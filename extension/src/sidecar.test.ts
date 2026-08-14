import { spawnSync } from "node:child_process";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { defaultSidecarPath, startSidecar } from "./sidecar";

function ensureSidecarBuilt(): string {
  const bin = defaultSidecarPath();
  const cargo = spawnSync("cargo", ["build", "-p", "call-sidecar"], {
    cwd: path.join(__dirname, "..", ".."),
    encoding: "utf8",
  });
  if (cargo.status !== 0) {
    throw new Error(cargo.stderr || cargo.stdout || "cargo build failed");
  }
  return bin;
}

describe("Sidecar handshake", () => {
  it("spawns the Sidecar and ping returns its crate version", async () => {
    const bin = ensureSidecarBuilt();
    const client = startSidecar(bin);
    try {
      const result = await client.ping();
      expect(result.version).toBe("0.1.0");
      expect(result.protocolVersion).toBe(1);
    } finally {
      client.dispose();
    }
  });
});
