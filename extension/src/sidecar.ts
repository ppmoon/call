import { ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { drainFrames, encodeRequest, parsePingResult, PingResult } from "./protocol";

export function defaultSidecarPath(): string {
  if (process.env.CALL_SIDECAR_PATH) {
    return process.env.CALL_SIDECAR_PATH;
  }
  const exe = process.platform === "win32" ? "call-sidecar.exe" : "call-sidecar";
  const packaged = path.join(__dirname, "..", "bin", exe);
  if (existsSync(packaged)) {
    return packaged;
  }
  return path.join(__dirname, "..", "..", "target", "debug", exe);
}

export class SidecarClient {
  private child: ChildProcessWithoutNullStreams;
  private buffer = Buffer.alloc(0);
  private nextId = 1;
  private pending = new Map<number, { resolve: (body: string) => void; reject: (err: Error) => void }>();

  constructor(command: string, args: string[] = []) {
    this.child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
    this.child.stdout.on("data", (chunk: Buffer) => this.onData(chunk));
    this.child.stderr.on("data", (chunk: Buffer) => {
      const text = chunk.toString("utf8").trim();
      if (text) {
        console.error(`[sidecar] ${text}`);
      }
    });
    this.child.on("error", (err) => this.failAll(err));
    this.child.on("exit", (code, signal) => {
      if (this.pending.size > 0) {
        this.failAll(new Error(`Sidecar exited (code=${code}, signal=${signal})`));
      }
    });
  }

  async ping(): Promise<PingResult> {
    const body = await this.request("ping");
    return parsePingResult(body);
  }

  dispose(): void {
    this.failAll(new Error("Sidecar disposed"));
    this.child.kill();
  }

  private request(method: string, params: unknown = {}): Promise<string> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.child.stdin.write(encodeRequest(id, method, params), (err) => {
        if (err) {
          this.pending.delete(id);
          reject(err);
        }
      });
    });
  }

  async rpc<T>(method: string, params: unknown = {}): Promise<T> {
    const body = await this.request(method, params);
    const msg = JSON.parse(body) as { error?: { message?: string }; result?: T };
    if (msg.error) {
      throw new Error(msg.error.message ?? "JSON-RPC error");
    }
    if (msg.result === undefined) {
      throw new Error(`no result for ${method}`);
    }
    return msg.result;
  }

  private onData(chunk: Buffer): void {
    this.buffer = Buffer.from(Buffer.concat([this.buffer, chunk]));
    const drained = drainFrames(this.buffer);
    this.buffer = Buffer.from(drained.rest);
    for (const body of drained.bodies) {
      const id = (JSON.parse(body) as { id?: number }).id;
      const waiter = typeof id === "number" ? this.pending.get(id) : undefined;
      if (waiter) {
        this.pending.delete(id as number);
        waiter.resolve(body);
      }
    }
  }

  private failAll(err: Error): void {
    for (const waiter of this.pending.values()) {
      waiter.reject(err);
    }
    this.pending.clear();
  }
}

export function startSidecar(command = defaultSidecarPath()): SidecarClient {
  if (!existsSync(command)) {
    throw new Error(
      `Sidecar binary not found at ${command}. Build with \`cargo build\` or set CALL_SIDECAR_PATH.`,
    );
  }
  return new SidecarClient(command);
}
