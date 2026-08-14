import { describe, expect, it } from "vitest";
import { drainFrames, encodeFrame, encodeRequest, parsePingResult } from "./protocol";

describe("JSON-RPC framing", () => {
  it("round-trips a ping request body through Content-Length frames", () => {
    const frame = encodeRequest(1, "ping");
    const { bodies, rest } = drainFrames(frame);
    expect(bodies).toHaveLength(1);
    expect(JSON.parse(bodies[0])).toMatchObject({
      jsonrpc: "2.0",
      id: 1,
      method: "ping",
    });
    expect(rest.length).toBe(0);
  });

  it("holds back a partial frame until Content-Length bytes arrive", () => {
    const full = encodeFrame('{"id":1}');
    const partial = full.subarray(0, full.length - 3);
    const first = drainFrames(partial);
    expect(first.bodies).toHaveLength(0);
    const second = drainFrames(Buffer.concat([first.rest, full.subarray(full.length - 3)]));
    expect(second.bodies).toEqual(['{"id":1}']);
  });

  it("parses a Sidecar ping result", () => {
    expect(
      parsePingResult(
        JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          result: { version: "0.1.0", protocolVersion: 1 },
        }),
      ),
    ).toEqual({ version: "0.1.0", protocolVersion: 1 });
  });
});
