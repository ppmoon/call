use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NodeKind {
    Function,
    Container,
    Cluster,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub kind: NodeKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualified_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub layer: u32,
    pub external: bool,
    pub expandable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Graph {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub depth: u32,
    pub show_external: bool,
    pub empty: bool,
    #[serde(default)]
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildParams {
    pub workspace_root: String,
    pub entry: Option<String>,
    #[serde(default = "default_depth")]
    pub depth: u32,
    #[serde(default)]
    pub show_external: bool,
    #[serde(default)]
    pub expanded: Vec<String>,
    #[serde(default)]
    pub collapsed_containers: Vec<String>,
    #[serde(default)]
    pub unclustered_layers: Vec<u32>,
}

fn default_depth() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub qualified_name: String,
    pub label: String,
    pub file: String,
    pub line: u32,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphMeta {
    #[serde(default)]
    pub pins: Vec<Pin>,
    #[serde(default)]
    pub run_args: Vec<RunArg>,
    #[serde(default)]
    pub identities: Vec<Identity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Pin {
    pub qualified_name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunArg {
    pub qualified_name: String,
    pub args_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub qualified_name: String,
    pub file: String,
    pub line: u32,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileHit {
    pub requested: String,
    pub bound: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    #[serde(default)]
    pub hits: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<String>,
}

pub fn canonical_graph(graph: &Graph) -> serde_json::Value {
    let mut nodes: Vec<_> = graph
        .nodes
        .iter()
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "kind": n.kind,
                "qn": n.qualified_name,
                "parent": n.parent,
                "layer": n.layer,
                "external": n.external,
            })
        })
        .collect();
    nodes.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let mut edges: Vec<_> = graph
        .edges
        .iter()
        .map(|e| serde_json::json!({"source": e.source, "target": e.target}))
        .collect();
    edges.sort_by(|a, b| {
        let ak = format!("{}->{}", a["source"], a["target"]);
        let bk = format!("{}->{}", b["source"], b["target"]);
        ak.cmp(&bk)
    });
    serde_json::json!({
        "entry": graph.entry,
        "empty": graph.empty,
        "nodes": nodes,
        "edges": edges,
    })
}
