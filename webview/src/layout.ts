import type { Graph, Pin } from "./types";

/** Layered layout (ELK-style: layered, downward). Pins override coordinates. */
export function layoutGraph(graph: Graph, pins: Pin[]): Map<string, { x: number; y: number }> {
  const pinMap = new Map(pins.map((p) => [p.qualifiedName, { x: p.x, y: p.y }]));
  const byLayer = new Map<number, string[]>();
  for (const node of graph.nodes) {
    const list = byLayer.get(node.layer) ?? [];
    list.push(node.id);
    byLayer.set(node.layer, list);
  }
  const pos = new Map<string, { x: number; y: number }>();
  const layers = [...byLayer.keys()].sort((a, b) => a - b);
  for (const layer of layers) {
    const ids = (byLayer.get(layer) ?? []).sort();
    ids.forEach((id, i) => {
      const node = graph.nodes.find((n) => n.id === id);
      const qn = node?.qualifiedName;
      if (qn && pinMap.has(qn)) {
        pos.set(id, pinMap.get(qn)!);
      } else {
        pos.set(id, { x: 80 + i * 220, y: 80 + layer * 140 });
      }
    });
  }
  return pos;
}
