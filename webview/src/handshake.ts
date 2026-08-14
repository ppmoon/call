export type Handshake =
  | { status: "connecting" }
  | { status: "ok"; version: string }
  | { status: "error"; message: string };

export type HandshakeMessage = {
  type: "handshake";
  version?: string;
  error?: string;
};
