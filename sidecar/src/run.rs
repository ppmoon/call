use crate::protocol::RunResult;
use crate::python::index_workspace;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub trait Runner {
    fn run_python(&self, python: &Path, args: &[String], cwd: &Path) -> Result<RunResult, String>;
}

pub struct NativeRunner;

impl Runner for NativeRunner {
    fn run_python(&self, python: &Path, args: &[String], cwd: &Path) -> Result<RunResult, String> {
        let output = Command::new(python)
            .args(args)
            .current_dir(cwd)
            .env("PYTHONPATH", {
                let extra = cwd.display().to_string();
                match std::env::var("PYTHONPATH") {
                    Ok(existing) if !existing.is_empty() => format!("{extra}:{existing}"),
                    _ => extra,
                }
            })
            .output()
            .map_err(|e| format!("spawn {}: {e}", python.display()))?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let exit_code = output.status.code().unwrap_or(-1);
        Ok(RunResult {
            ok: output.status.success(),
            stdout,
            stderr,
            exit_code,
            hits: vec![],
            failed_at: None,
        })
    }
}

pub fn default_python() -> PathBuf {
    PathBuf::from("python3")
}

pub fn generate_node_harness(qualified_name: &str) -> String {
    format!(
        r#"import importlib, json, sys, traceback
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

qn = {qn:?}
args = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {{}}
parts = qn.split(".")
err = None
mod = None
obj = None
for i in range(len(parts) - 1, 0, -1):
    try:
        mod = importlib.import_module(".".join(parts[:i]))
        obj = mod
        for attr in parts[i:]:
            obj = getattr(obj, attr)
        break
    except Exception as e:
        err = e
        mod = None
if obj is None:
    raise SystemExit(f"cannot import {{qn}}: {{err}}")
try:
    if isinstance(args, dict):
        result = obj(**args)
    elif isinstance(args, list):
        result = obj(*args)
    else:
        result = obj(args)
    print(result)
except Exception:
    traceback.print_exc()
    raise
"#,
        qn = qualified_name
    )
}

pub fn run_node<R: Runner>(
    runner: &R,
    workspace: &Path,
    qualified_name: &str,
    args_json: &str,
    python: &Path,
) -> Result<RunResult, String> {
    serde_json::from_str::<serde_json::Value>(args_json).map_err(|e| format!("invalid JSON args: {e}"))?;
    let harness = generate_node_harness(qualified_name);
    let path = workspace.join(".call").join("_harness_node.py");
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&path, harness).map_err(|e| e.to_string())?;
    runner.run_python(python, &[path.to_string_lossy().to_string(), args_json.to_string()], workspace)
}

pub fn generate_program_harness(script: &Path, hits_path: &Path) -> String {
    format!(
        r#"import json, runpy, sys, traceback
hits_path = {hits:?}
script = {script:?}
hits = []

def tracer(frame, event, arg):
    if event == "call":
        code = frame.f_code
        hits.append({{"file": code.co_filename, "name": code.co_name, "line": code.co_firstlineno}})
    return tracer

sys.settrace(tracer)
failed = None
try:
    runpy.run_path(script, run_name="__main__")
except Exception:
    traceback.print_exc()
    failed = True
finally:
    sys.settrace(None)
    open(hits_path, "w").write(json.dumps(hits))
if failed:
    sys.exit(1)
"#,
        hits = hits_path,
        script = script
    )
}

pub fn run_program<R: Runner>(
    runner: &R,
    workspace: &Path,
    entry: &str,
    python: &Path,
) -> Result<RunResult, String> {
    let index = index_workspace(workspace)?;
    let func = index
        .functions
        .get(entry)
        .ok_or_else(|| format!("unknown Entry: {entry}"))?;
    let hits_path = workspace.join(".call").join("_hits.json");
    let harness_path = workspace.join(".call").join("_harness_program.py");
    std::fs::create_dir_all(harness_path.parent().unwrap()).map_err(|e| e.to_string())?;
    let harness = generate_program_harness(&func.file, &hits_path);
    let mut file = std::fs::File::create(&harness_path).map_err(|e| e.to_string())?;
    file.write_all(harness.as_bytes()).map_err(|e| e.to_string())?;
    let mut result = runner.run_python(
        python,
        &[harness_path.to_string_lossy().to_string()],
        workspace,
    )?;
    if let Ok(text) = std::fs::read_to_string(&hits_path) {
        if let Ok(raw) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
            let mut hits = Vec::new();
            for item in raw {
                let file = item.get("file").and_then(|v| v.as_str()).unwrap_or("");
                let line = item.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(qn) = index.by_file_line.get(&(file.to_string(), line)) {
                    if !hits.contains(qn) {
                        hits.push(qn.clone());
                    }
                } else {
                    let suffix = format!(".{name}");
                    if let Some(qn) = index.functions.keys().find(|k| k.ends_with(&suffix) && file.ends_with(&index.functions[*k].file.to_string_lossy().to_string())) {
                        if !hits.contains(qn) {
                            hits.push(qn.clone());
                        }
                    }
                }
            }
            result.hits = hits;
            if !result.ok {
                result.failed_at = result.hits.last().cloned();
            }
        }
    }
    Ok(result)
}

pub fn extract_function_source(workspace: &Path, qualified_name: &str) -> Result<serde_json::Value, String> {
    let index = index_workspace(workspace)?;
    let func = index
        .functions
        .get(qualified_name)
        .ok_or_else(|| format!("unknown function: {qualified_name}"))?;
    let src = std::fs::read_to_string(&func.file).map_err(|e| e.to_string())?;
    let neighbors: Vec<_> = func
        .callees
        .iter()
        .filter_map(|c| index.functions.get(c))
        .map(|f| serde_json::json!({"qualifiedName": f.qualified_name, "signature": f.signature}))
        .collect();
    Ok(serde_json::json!({
        "qualifiedName": func.qualified_name,
        "file": func.file,
        "line": func.line,
        "signature": func.signature,
        "source": src,
        "neighbors": neighbors,
    }))
}
