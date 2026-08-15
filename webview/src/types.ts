export type NodeKind = "function" | "container" | "cluster";

export type GraphNode = {
  id: string;
  kind: NodeKind;
  label: string;
  qualifiedName?: string;
  file?: string;
  line?: number;
  signature?: string;
  parent?: string;
  layer: number;
  external: boolean;
  expandable: boolean;
};

export type GraphEdge = { id: string; source: string; target: string };

export type Graph = {
  entry?: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  depth: number;
  showExternal: boolean;
  empty: boolean;
  entries: string[];
};

export type Pin = { qualifiedName: string; x: number; y: number };

export type RunResult = {
  ok: boolean;
  stdout: string;
  stderr: string;
  exitCode: number;
  hits: string[];
  failedAt?: string;
};

export type SymbolInfo = {
  qualifiedName: string;
  label: string;
  file: string;
  line: number;
  signature: string;
};
