use crate::protocol::{BuildParams, Graph, GraphEdge, GraphNode, NodeKind};
use crate::python::{detect_entries, index_workspace, is_external, Index};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

pub const CLUSTER_THRESHOLD: usize = 30;

pub fn build(params: &BuildParams) -> Result<Graph, String> {
    let root = Path::new(&params.workspace_root);
    let index = index_workspace(root)?;
    let entries = detect_entries(&index);
    if index.functions.is_empty() {
        return Ok(Graph {
            entry: None,
            nodes: vec![],
            edges: vec![],
            depth: params.depth,
            show_external: params.show_external,
            empty: true,
            entries,
        });
    }
    let entry = params
        .entry
        .clone()
        .or_else(|| entries.first().cloned())
        .ok_or_else(|| "no Entry found; pick one manually".to_string())?;
    if !index.functions.contains_key(&entry) && !is_external(&entry) {
        return Err(format!("unknown Entry: {entry}"));
    }
    Ok(assemble(&index, &entry, params, entries))
}

pub fn list_symbols(workspace_root: &str) -> Result<Vec<crate::protocol::SymbolInfo>, String> {
    let index = index_workspace(Path::new(workspace_root))?;
    Ok(index.symbols())
}

fn assemble(index: &Index, entry: &str, params: &BuildParams, entries: Vec<String>) -> Graph {
    let expanded: HashSet<&str> = params.expanded.iter().map(String::as_str).collect();
    let unclustered: HashSet<u32> = params.unclustered_layers.iter().copied().collect();
    let collapsed: HashSet<&str> = params
        .collapsed_containers
        .iter()
        .map(String::as_str)
        .collect();

    let mut layer_of: HashMap<String, u32> = HashMap::new();
    let mut parent_edge: HashMap<String, String> = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back((entry.to_string(), 0_u32));
    layer_of.insert(entry.to_string(), 0);

    while let Some((qn, layer)) = queue.pop_front() {
        let extra = expanded.contains(qn.as_str());
        let limit = if extra {
            params.depth.max(layer + 1)
        } else {
            params.depth
        };
        if layer >= limit {
            continue;
        }
        let Some(func) = index.functions.get(&qn) else {
            continue;
        };
        for callee in &func.callees {
            if !params.show_external && is_external(callee) {
                continue;
            }
            if layer_of.contains_key(callee) {
                parent_edge.entry(callee.clone()).or_insert(qn.clone());
                continue;
            }
            layer_of.insert(callee.clone(), layer + 1);
            parent_edge.insert(callee.clone(), qn.clone());
            if !is_external(callee) {
                queue.push_back((callee.clone(), layer + 1));
            }
        }
    }

    // Include callees of expanded nodes one extra hop.
    for (qn, layer) in layer_of.clone() {
        if !expanded.contains(qn.as_str()) {
            continue;
        }
        if let Some(func) = index.functions.get(&qn) {
            for callee in &func.callees {
                if !params.show_external && is_external(callee) {
                    continue;
                }
                layer_of.entry(callee.clone()).or_insert(layer + 1);
            }
        }
    }

    let mut by_layer: HashMap<u32, Vec<String>> = HashMap::new();
    for (qn, layer) in &layer_of {
        by_layer.entry(*layer).or_default().push(qn.clone());
    }
    for list in by_layer.values_mut() {
        list.sort();
    }

    let mut clustered_away: HashSet<String> = HashSet::new();
    let mut cluster_nodes: Vec<GraphNode> = Vec::new();
    let mut cluster_for: HashMap<String, String> = HashMap::new();
    for (layer, list) in &by_layer {
        if *layer == 0 || unclustered.contains(layer) {
            continue;
        }
        let functions: Vec<_> = list
            .iter()
            .filter(|qn| !is_external(qn) && index.functions.contains_key(*qn))
            .cloned()
            .collect();
        if functions.len() <= CLUSTER_THRESHOLD {
            continue;
        }
        let cluster_id = format!("cluster:layer{layer}");
        let overflow: Vec<_> = functions.iter().skip(CLUSTER_THRESHOLD).cloned().collect();
        for qn in &overflow {
            clustered_away.insert(qn.clone());
            cluster_for.insert(qn.clone(), cluster_id.clone());
        }
        cluster_nodes.push(GraphNode {
            id: cluster_id,
            kind: NodeKind::Cluster,
            label: format!("{} more functions", overflow.len()),
            qualified_name: None,
            file: None,
            line: None,
            signature: None,
            parent: None,
            layer: *layer,
            external: false,
            expandable: true,
        });
    }

    let mut nodes = Vec::new();
    let mut containers: HashMap<String, GraphNode> = HashMap::new();
    for (qn, layer) in &layer_of {
        if clustered_away.contains(qn) {
            continue;
        }
        if is_external(qn) {
            nodes.push(GraphNode {
                id: qn.clone(),
                kind: NodeKind::Function,
                label: qn.rsplit('.').next().unwrap_or(qn).to_string(),
                qualified_name: Some(qn.clone()),
                file: None,
                line: None,
                signature: None,
                parent: None,
                layer: *layer,
                external: true,
                expandable: false,
            });
            continue;
        }
        let Some(func) = index.functions.get(qn) else {
            continue;
        };
        let container_id = match &func.class_name {
            Some(class) => format!("container:{}.{}", func.module, class),
            None => format!("container:{}", func.module),
        };
        let container_label = func
            .class_name
            .clone()
            .unwrap_or_else(|| func.module.clone());
        containers.entry(container_id.clone()).or_insert(GraphNode {
            id: container_id.clone(),
            kind: NodeKind::Container,
            label: container_label,
            qualified_name: Some(container_id.trim_start_matches("container:").to_string()),
            file: Some(func.file.to_string_lossy().to_string()),
            line: None,
            signature: None,
            parent: None,
            layer: *layer,
            external: false,
            expandable: true,
        });
        let hidden = collapsed.contains(container_id.as_str());
        if hidden {
            continue;
        }
        let expandable = func
            .callees
            .iter()
            .any(|c| params.show_external || !is_external(c));
        nodes.push(GraphNode {
            id: qn.clone(),
            kind: NodeKind::Function,
            label: func.label.clone(),
            qualified_name: Some(qn.clone()),
            file: Some(func.file.to_string_lossy().to_string()),
            line: Some(func.line),
            signature: Some(func.signature.clone()),
            parent: Some(container_id),
            layer: *layer,
            external: false,
            expandable,
        });
    }
    nodes.extend(containers.into_values());
    nodes.extend(cluster_nodes);

    let mut edges = Vec::new();
    let mut seen = HashSet::new();
    for (qn, _layer) in &layer_of {
        let Some(caller) = index.functions.get(qn) else {
            continue;
        };
        for callee in &caller.callees {
            if !layer_of.contains_key(callee) && !cluster_for.contains_key(callee) {
                continue;
            }
            if !params.show_external && is_external(callee) {
                continue;
            }
            let mut source = qn.clone();
            let mut target = cluster_for.get(callee).cloned().unwrap_or_else(|| callee.clone());
            if clustered_away.contains(&source) {
                if let Some(cid) = cluster_for.get(&source) {
                    source = cid.clone();
                } else {
                    continue;
                }
            }
            if let Some(func) = index.functions.get(qn) {
                let container_id = match &func.class_name {
                    Some(class) => format!("container:{}.{}", func.module, class),
                    None => format!("container:{}", func.module),
                };
                if collapsed.contains(container_id.as_str()) {
                    source = container_id;
                }
            }
            if let Some(func) = index.functions.get(callee) {
                let container_id = match &func.class_name {
                    Some(class) => format!("container:{}.{}", func.module, class),
                    None => format!("container:{}", func.module),
                };
                if collapsed.contains(container_id.as_str()) {
                    target = container_id;
                }
            }
            if source == target {
                continue;
            }
            let key = format!("{source}->{target}");
            if seen.insert(key.clone()) {
                edges.push(GraphEdge {
                    id: key,
                    source,
                    target,
                });
            }
        }
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    Graph {
        entry: Some(entry.to_string()),
        nodes,
        edges,
        depth: params.depth,
        show_external: params.show_external,
        empty: false,
        entries,
    }
}
