import { describe, expect, it } from "vitest";
import { layoutGraph } from "./layout";
import type { Graph } from "./types";

describe("layoutGraph", () => {
  it("layers nodes downward and honors Pins", () => {
    const graph: Graph = {
      nodes: [
        { id: "a", kind: "function", label: "a", qualifiedName: "a", layer: 0, external: false, expandable: true },
        { id: "b", kind: "function", label: "b", qualifiedName: "b", layer: 1, external: false, expandable: false },
      ],
      edges: [],
      depth: 2,
      showExternal: false,
      empty: false,
      entries: ["a"],
    };
    const pos = layoutGraph(graph, [{ qualifiedName: "b", x: 9, y: 9 }]);
    expect(pos.get("a")).toEqual({ x: 80, y: 80 });
    expect(pos.get("b")).toEqual({ x: 9, y: 9 });
  });
});
