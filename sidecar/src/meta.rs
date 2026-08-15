use crate::protocol::{GraphMeta, Identity, Pin, ReconcileHit, RunArg};
use crate::python::index_workspace;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn call_dir(root: &Path) -> PathBuf {
    root.join(".call")
}

pub fn graph_json_path(root: &Path) -> PathBuf {
    call_dir(root).join("graph.json")
}

pub fn symbols_dir(root: &Path) -> PathBuf {
    call_dir(root).join("symbols")
}

pub fn symbol_file(root: &Path, qualified_name: &str) -> PathBuf {
    symbols_dir(root).join(format!("{}.md", hash_qn(qualified_name)))
}

pub fn hash_qn(qualified_name: &str) -> String {
    let digest = Sha256::digest(qualified_name.as_bytes());
    hex::encode(&digest[..16])
}

pub fn load_meta(root: &Path) -> GraphMeta {
    let path = graph_json_path(root);
    let Ok(text) = std::fs::read_to_string(path) else {
        return GraphMeta::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_meta(root: &Path, meta: &GraphMeta) -> Result<(), String> {
    let dir = call_dir(root);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(symbols_dir(root)).map_err(|e| e.to_string())?;
    let path = graph_json_path(root);
    let text = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn upsert_pin(root: &Path, pin: Pin) -> Result<GraphMeta, String> {
    let mut meta = load_meta(root);
    meta.pins.retain(|p| p.qualified_name != pin.qualified_name);
    meta.pins.push(pin);
    save_meta(root, &meta)?;
    Ok(meta)
}

pub fn upsert_run_args(root: &Path, arg: RunArg) -> Result<GraphMeta, String> {
    let mut meta = load_meta(root);
    meta.run_args.retain(|a| a.qualified_name != arg.qualified_name);
    meta.run_args.push(arg);
    save_meta(root, &meta)?;
    Ok(meta)
}

pub fn snapshot_identities(root: &Path) -> Result<GraphMeta, String> {
    let index = index_workspace(root)?;
    let mut meta = load_meta(root);
    meta.identities = index
        .functions
        .values()
        .map(|f| Identity {
            qualified_name: f.qualified_name.clone(),
            file: f.file.to_string_lossy().to_string(),
            line: f.line,
            signature: f.signature.clone(),
        })
        .collect();
    save_meta(root, &meta)?;
    Ok(meta)
}

/// Bind a stored qualified name back to the live index.
pub fn reconcile(root: &Path, requested: &str) -> Result<ReconcileHit, String> {
    let index = index_workspace(root)?;
    if index.functions.contains_key(requested) {
        return Ok(ReconcileHit {
            requested: requested.to_string(),
            bound: Some(requested.to_string()),
        });
    }
    let meta = load_meta(root);
    let Some(identity) = meta.identities.iter().find(|i| i.qualified_name == requested) else {
        return Ok(ReconcileHit {
            requested: requested.to_string(),
            bound: None,
        });
    };
    if let Some(qn) = index
        .by_file_line
        .get(&(identity.file.clone(), identity.line))
    {
        return Ok(ReconcileHit {
            requested: requested.to_string(),
            bound: Some(qn.clone()),
        });
    }
    if let Some(qns) = index.by_signature.get(&identity.signature) {
        if let Some(qn) = qns.first() {
            return Ok(ReconcileHit {
                requested: requested.to_string(),
                bound: Some(qn.clone()),
            });
        }
    }
    Ok(ReconcileHit {
        requested: requested.to_string(),
        bound: None,
    })
}

pub fn append_prompt(root: &Path, qualified_name: &str, prompt: &str) -> Result<PathBuf, String> {
    let dir = symbols_dir(root);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = symbol_file(root, qualified_name);
    let mut body = if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| e.to_string())?
    } else {
        format!("# {qualified_name}\n\n")
    };
    body.push_str(&format!("## Prompt\n\n{prompt}\n\n"));
    std::fs::write(&path, body).map_err(|e| e.to_string())?;
    Ok(path)
}
