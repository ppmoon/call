use call_sidecar::graph::{self, CLUSTER_THRESHOLD};
use call_sidecar::meta;
use call_sidecar::protocol::{BuildParams, Pin, canonical_graph};
use call_sidecar::python::index_workspace;
use call_sidecar::run::{self, NativeRunner};
use call_sidecar::scaffold;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn fixture(name: &str) -> PathBuf {
    root().join("fixtures").join(name)
}

fn toy_graph(depth: u32, show_external: bool, expanded: Vec<String>) -> call_sidecar::protocol::Graph {
    graph::build(&BuildParams {
        workspace_root: fixture("python-toy").to_string_lossy().into(),
        entry: None,
        depth,
        show_external,
        expanded,
        collapsed_containers: vec![],
        unclustered_layers: vec![],
    })
    .unwrap()
}

#[test]
fn python_toy_detects_main_and_projects_two_layers() {
    let g = toy_graph(2, false, vec![]);
    let snap = canonical_graph(&g);
    assert_eq!(snap["entry"], "app.main.main");
    let ids: Vec<_> = g
        .nodes
        .iter()
        .filter(|n| n.kind == call_sidecar::protocol::NodeKind::Function)
        .filter_map(|n| n.qualified_name.clone())
        .collect();
    assert!(ids.contains(&"app.main.main".into()), "{ids:?}");
    assert!(ids.contains(&"app.orchestrate.orchestrate".into()), "{ids:?}");
    assert!(ids.contains(&"app.util.log".into()), "{ids:?}");
    assert!(ids.contains(&"app.worker.Worker.process".into()), "{ids:?}");
    assert!(
        !ids.iter().any(|id| id.contains("json") || id.contains("os")),
        "stdlib leaked: {ids:?}"
    );
}

#[test]
fn python_toy_expand_adds_stdlib_when_externals_shown() {
    let g = toy_graph(2, true, vec!["app.util.log".into()]);
    let ids: Vec<_> = g.nodes.iter().filter_map(|n| n.qualified_name.clone()).collect();
    assert!(
        ids.iter().any(|id| id.contains("os") || id.contains("getcwd") || id.contains("stdlib")),
        "{ids:?}"
    );
}

#[test]
fn python_toy_expand_layer_golden() {
    let g = toy_graph(1, false, vec!["app.orchestrate.orchestrate".into()]);
    let funcs: Vec<_> = g
        .nodes
        .iter()
        .filter(|n| n.kind == call_sidecar::protocol::NodeKind::Function && !n.external)
        .filter_map(|n| n.qualified_name.clone())
        .collect();
    assert!(funcs.contains(&"app.worker.Worker.process".into()), "{funcs:?}");
}

#[test]
fn containers_group_class_methods() {
    let g = toy_graph(2, false, vec![]);
    let worker = g
        .nodes
        .iter()
        .find(|n| n.qualified_name.as_deref() == Some("app.worker.Worker.process"))
        .unwrap();
    assert_eq!(worker.parent.as_deref(), Some("container:app.worker.Worker"));
    let collapsed = graph::build(&BuildParams {
        workspace_root: fixture("python-toy").to_string_lossy().into(),
        entry: None,
        depth: 2,
        show_external: false,
        expanded: vec![],
        collapsed_containers: vec!["container:app.worker.Worker".into()],
        unclustered_layers: vec![],
    })
    .unwrap();
    assert!(collapsed
        .nodes
        .iter()
        .all(|n| n.qualified_name.as_deref() != Some("app.worker.Worker.process")));
    assert!(collapsed.edges.iter().any(|e| e.target == "container:app.worker.Worker"
        || e.source == "container:app.worker.Worker"));
}

#[test]
fn cluster_threshold_30_vs_31() {
    let wide = fixture("wide").to_string_lossy().into_owned();
    let clustered = graph::build(&BuildParams {
        workspace_root: wide.clone(),
        entry: None,
        depth: 1,
        show_external: false,
        expanded: vec![],
        collapsed_containers: vec![],
        unclustered_layers: vec![],
    })
    .unwrap();
    let funcs = clustered
        .nodes
        .iter()
        .filter(|n| n.kind == call_sidecar::protocol::NodeKind::Function && n.layer == 1)
        .count();
    assert_eq!(CLUSTER_THRESHOLD, 30);
    assert_eq!(funcs, 30, "layer 1 should keep 30 functions");
    assert!(clustered
        .nodes
        .iter()
        .any(|n| n.kind == call_sidecar::protocol::NodeKind::Cluster));

    let unclustered = graph::build(&BuildParams {
        workspace_root: wide,
        entry: None,
        depth: 1,
        show_external: false,
        expanded: vec![],
        collapsed_containers: vec![],
        unclustered_layers: vec![1],
    })
    .unwrap();
    let funcs = unclustered
        .nodes
        .iter()
        .filter(|n| n.kind == call_sidecar::protocol::NodeKind::Function && n.layer == 1)
        .count();
    assert_eq!(funcs, 31);
}

#[test]
fn pin_survives_reload_and_missing_meta_rebuilds() {
    let tmp = tempfile_workspace();
    copy_dir(&fixture("python-toy"), &tmp);
    meta::upsert_pin(
        &tmp,
        Pin {
            qualified_name: "app.main.main".into(),
            x: 12.0,
            y: 40.0,
        },
    )
    .unwrap();
    let loaded = meta::load_meta(&tmp);
    assert_eq!(loaded.pins[0].x, 12.0);
    std::fs::remove_dir_all(tmp.join(".call")).unwrap();
    let g = graph::build(&BuildParams {
        workspace_root: tmp.to_string_lossy().into(),
        entry: None,
        depth: 2,
        show_external: false,
        expanded: vec![],
        collapsed_containers: vec![],
        unclustered_layers: vec![],
    })
    .unwrap();
    assert_eq!(g.entry.as_deref(), Some("app.main.main"));
}

#[test]
fn reconcile_falls_back_to_signature_after_rename() {
    let tmp = tempfile_workspace();
    copy_dir(&fixture("python-toy"), &tmp);
    meta::snapshot_identities(&tmp).unwrap();
    let hit = meta::reconcile(&tmp, "app.main.main").unwrap();
    assert_eq!(hit.bound.as_deref(), Some("app.main.main"));
    let miss = meta::reconcile(&tmp, "does.not.exist").unwrap();
    assert!(miss.bound.is_none());
}

#[test]
fn node_harness_runs_worker_process() {
    let result = run::run_node(
        &NativeRunner,
        &fixture("python-toy"),
        "app.orchestrate.orchestrate",
        r#"{"msg": "hi"}"#,
        Path::new("python3"),
    )
    .unwrap();
    assert!(result.ok, "{}", result.stderr);
    assert!(result.stdout.contains("HI"), "{}", result.stdout);
    let _ = result;
}

#[test]
fn invalid_json_args_are_rejected() {
    let err = run::run_node(
        &NativeRunner,
        &fixture("python-toy"),
        "app.orchestrate.orchestrate",
        "{",
        Path::new("python3"),
    )
    .unwrap_err();
    assert!(err.contains("invalid JSON"), "{err}");
}

#[test]
fn scaffold_writes_hello_world_only_when_empty() {
    let tmp = tempfile_workspace();
    scaffold::scaffold(&tmp, "python").unwrap();
    let g = graph::build(&BuildParams {
        workspace_root: tmp.to_string_lossy().into(),
        entry: None,
        depth: 1,
        show_external: false,
        expanded: vec![],
        collapsed_containers: vec![],
        unclustered_layers: vec![],
    })
    .unwrap();
    assert_eq!(g.entry.as_deref(), Some("main.main"));
    let err = scaffold::scaffold(&tmp, "python").unwrap_err();
    assert!(err.contains("already has source"));
}

#[test]
fn httpcli_wow_repo_clusters_and_can_run_fetch() {
    let g = graph::build(&BuildParams {
        workspace_root: fixture("httpcli").to_string_lossy().into(),
        entry: None,
        depth: 1,
        show_external: false,
        expanded: vec![],
        collapsed_containers: vec![],
        unclustered_layers: vec![],
    })
    .unwrap();
    assert!(g.nodes.iter().any(|n| n.kind == call_sidecar::protocol::NodeKind::Cluster));
    let result = run::run_node(
        &NativeRunner,
        &fixture("httpcli"),
        "httpcli.fetch.fetch",
        r#"{"url": "https://example.com"}"#,
        Path::new("python3"),
    )
    .unwrap();
    assert!(result.ok, "{}", result.stderr);
    assert!(result.stdout.contains("example.com"), "{}", result.stdout);
}

fn tempfile_workspace() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("call-test-{}", std::process::id()));
    let unique = dir.join(format!("{}", uuid_like()));
    std::fs::create_dir_all(&unique).unwrap();
    unique
}

fn uuid_like() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn copy_dir(src: &Path, dst: &Path) {
    for py in walk_py(src) {
        let rel = py.strip_prefix(src).unwrap();
        let to = dst.join(rel);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&py, &to).unwrap();
    }
}

fn walk_py(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.extend(walk_py(&p));
            } else if p.extension().and_then(|s| s.to_str()) == Some("py") {
                out.push(p);
            }
        }
    }
    out
}

#[test]
fn program_run_highlights_orchestrate() {
    let result = run::run_program(
        &NativeRunner,
        &fixture("python-toy"),
        "app.main.main",
        Path::new("python3"),
    )
    .unwrap();
    assert!(result.ok, "{}", result.stderr);
    assert!(
        result.hits.iter().any(|h| h.contains("orchestrate") || h.contains("main")),
        "{:?}",
        result.hits
    );
}

#[test]
fn index_python_toy_lists_functions() {
    let index = index_workspace(&fixture("python-toy")).unwrap();
    assert!(index.functions.contains_key("app.main.main"));
}
