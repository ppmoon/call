export type JsonRpcId = string | number;

export type PingResult = {
  version: string;
  protocolVersion: number;
};

export function encodeFrame(body: string): Buffer {
  return Buffer.from(`Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`, "utf8");
}

export function encodeRequest(id: JsonRpcId, method: string, params: unknown = {}): Buffer {
  return encodeFrame(
    JSON.stringify({ jsonrpc: "2.0", id, method, params }),
  );
}

/**
 * Pull complete JSON-RPC bodies out of a growing stdout buffer.
 * Returns remaining unparsed bytes.
 */
export function drainFrames(buffer: Buffer): { bodies: string[]; rest: Buffer } {
  const bodies: string[] = [];
  let rest = buffer;
  while (true) {
    const headerEnd = indexOfHeaderEnd(rest);
    if (headerEnd < 0) {
      break;
    }
    const header = rest.subarray(0, headerEnd).toString("utf8");
    const match = /content-length:\s*(\d+)/i.exec(header);
    if (!match) {
      throw new Error("JSON-RPC frame missing Content-Length");
    }
    const length = Number(match[1]);
    const bodyStart = headerEnd + 4;
    if (rest.length < bodyStart + length) {
      break;
    }
    bodies.push(rest.subarray(bodyStart, bodyStart + length).toString("utf8"));
    rest = rest.subarray(bodyStart + length);
  }
  return { bodies, rest };
}

function indexOfHeaderEnd(buffer: Buffer): number {
  const needle = Buffer.from("\r\n\r\n");
  return buffer.indexOf(needle);
}

export function parsePingResult(body: string): PingResult {
  const msg = JSON.parse(body) as {
    error?: { message?: string };
    result?: { version?: string; protocolVersion?: number };
  };
  if (msg.error) {
    throw new Error(msg.error.message ?? "JSON-RPC error");
  }
  const version = msg.result?.version;
  const protocolVersion = msg.result?.protocolVersion;
  if (typeof version !== "string" || typeof protocolVersion !== "number") {
    throw new Error("ping result missing version");
  }
  return { version, protocolVersion };
}
