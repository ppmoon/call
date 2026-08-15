use crate::protocol::SymbolInfo;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Tree};

const STDLIB: &[&str] = &[
    "abc", "ast", "asyncio", "base64", "builtins", "collections", "copy", "csv", "datetime",
    "decimal", "enum", "functools", "glob", "hashlib", "heapq", "hmac", "html", "http", "importlib",
    "inspect", "io", "itertools", "json", "logging", "math", "os", "pathlib", "pickle", "platform",
    "pprint", "random", "re", "shutil", "socket", "sqlite3", "ssl", "string", "struct", "subprocess",
    "sys", "tempfile", "textwrap", "threading", "time", "token", "tokenize", "traceback", "typing",
    "unittest", "urllib", "uuid", "venv", "warnings", "weakref", "xml", "zipfile",
];

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub qualified_name: String,
    pub label: String,
    pub file: PathBuf,
    pub line: u32,
    pub signature: String,
    pub module: String,
    pub class_name: Option<String>,
    pub callees: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Index {
    pub functions: HashMap<String, FunctionDef>,
    pub by_file_line: HashMap<(String, u32), String>,
    pub by_signature: HashMap<String, Vec<String>>,
}

impl Index {
    pub fn symbols(&self) -> Vec<SymbolInfo> {
        let mut out: Vec<_> = self
            .functions
            .values()
            .map(|f| SymbolInfo {
                qualified_name: f.qualified_name.clone(),
                label: f.label.clone(),
                file: f.file.to_string_lossy().to_string(),
                line: f.line,
                signature: f.signature.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));
        out
    }
}

pub fn index_workspace(root: &Path) -> Result<Index, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|e| format!("tree-sitter python: {e}"))?;

    let files = python_files(root);
    let mut modules: HashMap<String, ParsedModule> = HashMap::new();
    for file in &files {
        let rel = file.strip_prefix(root).unwrap_or(file);
        let module = module_name(rel);
        let src = std::fs::read_to_string(file).map_err(|e| format!("read {}: {e}", file.display()))?;
        let tree = parser
            .parse(&src, None)
            .ok_or_else(|| format!("parse {}", file.display()))?;
        modules.insert(module.clone(), parse_module(file, &module, &src, &tree));
    }

    let mut index = Index::default();
    for parsed in modules.values() {
        for func in &parsed.functions {
            index.functions.insert(func.qualified_name.clone(), func.clone());
            index.by_file_line.insert(
                (func.file.to_string_lossy().to_string(), func.line),
                func.qualified_name.clone(),
            );
            index
                .by_signature
                .entry(func.signature.clone())
                .or_default()
                .push(func.qualified_name.clone());
        }
    }

    for parsed in modules.values() {
        for func in &parsed.functions {
            let callees = resolve_callees(func, parsed, &index.functions);
            if let Some(slot) = index.functions.get_mut(&func.qualified_name) {
                slot.callees = callees;
            }
        }
    }
    Ok(index)
}

pub fn detect_entries(index: &Index) -> Vec<String> {
    let mut mains: Vec<_> = index
        .functions
        .keys()
        .filter(|qn| qn.ends_with(".main") || qn.ends_with(".__main__"))
        .cloned()
        .collect();
    mains.sort();
    let mut preferred: Vec<_> = mains
        .iter()
        .filter(|qn| qn.ends_with(".__main__.main") || qn.ends_with(".main.main"))
        .cloned()
        .collect();
    if preferred.is_empty() {
        preferred = mains;
    }
    preferred
}

fn python_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    visit(root, &mut out);
    out.sort();
    out
}

fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name.starts_with('.')
                || name == "venv"
                || name == ".venv"
                || name == "node_modules"
                || name == "__pycache__"
                || name == "target"
                || name == "dist"
            {
                continue;
            }
            visit(&path, out);
        } else if name.ends_with(".py") {
            out.push(path);
        }
    }
}

fn module_name(rel: &Path) -> String {
    let mut parts: Vec<String> = rel
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    if let Some(last) = parts.last_mut() {
        if let Some(stripped) = last.strip_suffix(".py") {
            *last = stripped.to_string();
        }
    }
    if parts.last().map(String::as_str) == Some("__init__") {
        parts.pop();
    }
    parts.join(".")
}

#[derive(Debug, Clone)]
struct ParsedModule {
    functions: Vec<FunctionDef>,
    imports: HashMap<String, String>,
    #[allow(dead_code)]
    classes: HashSet<String>,
}

#[derive(Debug, Clone)]
struct RawFunc {
    name: String,
    class_name: Option<String>,
    line: u32,
    signature: String,
    call_names: Vec<CallName>,
}

#[derive(Debug, Clone)]
enum CallName {
    Name(String),
    Attr { object: String, attr: String },
    CtorMethod { ctor: String, method: String },
}

fn parse_module(file: &Path, module: &str, src: &str, tree: &Tree) -> ParsedModule {
    let root = tree.root_node();
    let bytes = src.as_bytes();
    let mut imports = HashMap::new();
    let mut classes = HashSet::new();
    let mut raw = Vec::new();
    collect(root, bytes, None, &mut imports, &mut classes, &mut raw);

    let functions = raw
        .into_iter()
        .map(|f| {
            let qualified_name = match &f.class_name {
                Some(class) => format!("{module}.{class}.{}", f.name),
                None => format!("{module}.{}", f.name),
            };
            FunctionDef {
                qualified_name,
                label: f.name.clone(),
                file: file.to_path_buf(),
                line: f.line,
                signature: f.signature,
                module: module.to_string(),
                class_name: f.class_name,
                callees: Vec::new(),
            }
            .with_calls(f.call_names)
        })
        .collect();
    ParsedModule {
        functions,
        imports,
        classes,
    }
}

trait WithCalls {
    fn with_calls(self, calls: Vec<CallName>) -> FunctionDef;
}

impl WithCalls for FunctionDef {
    fn with_calls(mut self, calls: Vec<CallName>) -> FunctionDef {
        self.callees = calls
            .into_iter()
            .map(|c| match c {
                CallName::Name(n) => format!("name:{n}"),
                CallName::Attr { object, attr } => format!("attr:{object}.{attr}"),
                CallName::CtorMethod { ctor, method } => format!("ctor:{ctor}.{method}"),
            })
            .collect();
        // stash raw in callees temporarily — replaced in resolve. Keep raw parallel?
        self
    }
}

// Store raw call names alongside FunctionDef via a side channel during parse.
// Simpler: put raw calls on FunctionDef as callees using a private encoding, resolve later.

fn collect(
    node: Node,
    src: &[u8],
    class: Option<String>,
    imports: &mut HashMap<String, String>,
    classes: &mut HashSet<String>,
    funcs: &mut Vec<RawFunc>,
) {
    match node.kind() {
        "import_statement" => parse_import(node, src, imports),
        "import_from_statement" => parse_import_from(node, src, imports),
        "class_definition" => {
            if let Some(name) = child_named(node, "name").map(|n| text(n, src)) {
                classes.insert(name.clone());
                if let Some(body) = child_named(node, "body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        collect(child, src, Some(name.clone()), imports, classes, funcs);
                    }
                }
                return;
            }
        }
        "function_definition" => {
            if let Some(name_node) = child_named(node, "name") {
                let name = text(name_node, src);
                let params = child_named(node, "parameters").map(|n| text(n, src)).unwrap_or_else(|| "()".into());
                let mut calls = Vec::new();
                if let Some(body) = child_named(node, "body") {
                    collect_calls(body, src, &mut calls);
                }
                funcs.push(RawFunc {
                    name,
                    class_name: class.clone(),
                    line: node.start_position().row as u32 + 1,
                    signature: format!("def {}{}", text(name_node, src), params),
                    call_names: calls,
                });
            }
            return;
        }
        "decorated_definition" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect(child, src, class.clone(), imports, classes, funcs);
            }
            return;
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, src, class.clone(), imports, classes, funcs);
    }
}

fn parse_import(node: Node, src: &[u8], imports: &mut HashMap<String, String>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "dotted_name" | "identifier" => {
                let name = text(child, src);
                let alias = name.split('.').last().unwrap_or(&name).to_string();
                imports.insert(alias, name);
            }
            "aliased_import" => {
                let dotted = child.child_by_field_name("name").map(|n| text(n, src));
                let alias = child.child_by_field_name("alias").map(|n| text(n, src));
                if let (Some(dotted), Some(alias)) = (dotted, alias) {
                    imports.insert(alias, dotted);
                }
            }
            _ => {}
        }
    }
}

fn parse_import_from(node: Node, src: &[u8], imports: &mut HashMap<String, String>) {
    let module = node.child_by_field_name("module_name").map(|n| text(n, src));
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "dotted_name" | "identifier"
                if Some(child.id()) != node.child_by_field_name("module_name").map(|n| n.id()) =>
            {
                if let Some(module) = &module {
                    let name = text(child, src);
                    imports.insert(name.clone(), format!("{module}.{name}"));
                }
            }
            "aliased_import" => {
                let name = child.child_by_field_name("name").map(|n| text(n, src));
                let alias = child.child_by_field_name("alias").map(|n| text(n, src));
                if let (Some(module), Some(name), Some(alias)) = (&module, name, alias) {
                    imports.insert(alias, format!("{module}.{name}"));
                }
            }
            _ => {}
        }
    }
}

fn collect_calls(node: Node, src: &[u8], out: &mut Vec<CallName>) {
    if node.kind() == "call" {
        if let Some(func) = node.child_by_field_name("function") {
            match func.kind() {
                "identifier" => out.push(CallName::Name(text(func, src))),
                "attribute" => {
                    let attr = func.child_by_field_name("attribute").map(|n| text(n, src));
                    let obj = func.child_by_field_name("object");
                    if let (Some(attr), Some(obj)) = (attr, obj) {
                        if obj.kind() == "identifier" {
                            out.push(CallName::Attr {
                                object: text(obj, src),
                                attr,
                            });
                        } else if obj.kind() == "call" {
                            if let Some(inner) = obj.child_by_field_name("function") {
                                if inner.kind() == "identifier" {
                                    out.push(CallName::CtorMethod {
                                        ctor: text(inner, src),
                                        method: attr,
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_calls(child, src, out);
    }
}

fn resolve_callees(
    func: &FunctionDef,
    module: &ParsedModule,
    all: &HashMap<String, FunctionDef>,
) -> Vec<String> {
    let mut out = Vec::new();
    for encoded in &func.callees {
        if let Some(resolved) = resolve_one(encoded, func, module, all) {
            if !out.contains(&resolved) {
                out.push(resolved);
            }
        }
    }
    out
}

fn resolve_one(
    encoded: &str,
    func: &FunctionDef,
    module: &ParsedModule,
    all: &HashMap<String, FunctionDef>,
) -> Option<String> {
    if let Some(name) = encoded.strip_prefix("name:") {
        if name == "print" || name == "len" || name == "str" || name == "int" || name == "range" {
            return None;
        }
        if let Some(mapped) = module.imports.get(name) {
            return Some(resolve_import_target(mapped, all));
        }
        let local = format!("{}.{}", func.module, name);
        if all.contains_key(&local) {
            return Some(local);
        }
        return Some(classify_unknown(name));
    }
    if let Some(rest) = encoded.strip_prefix("attr:") {
        let (object, attr) = rest.split_once('.')?;
        if object == "self" {
            if let Some(class) = &func.class_name {
                let qn = format!("{}.{class}.{attr}", func.module);
                if all.contains_key(&qn) {
                    return Some(qn);
                }
            }
        }
        if let Some(mapped) = module.imports.get(object) {
            return Some(resolve_import_target(&format!("{mapped}.{attr}"), all));
        }
        let qn = format!("{}.{object}.{attr}", func.module);
        if all.contains_key(&qn) {
            return Some(qn);
        }
        return Some(classify_unknown(object));
    }
    if let Some(rest) = encoded.strip_prefix("ctor:") {
        let (ctor, method) = rest.split_once('.')?;
        if let Some(mapped) = module.imports.get(ctor) {
            let class_qn = resolve_import_target(mapped, all);
            let candidate = if all.contains_key(&format!("{class_qn}.{method}")) {
                format!("{class_qn}.{method}")
            } else {
                format!("{class_qn}.{ctor}.{method}")
            };
            if all.contains_key(&candidate) {
                return Some(candidate);
            }
            // mapped is module.Class
            let direct = format!("{mapped}.{method}");
            if all.contains_key(&direct) {
                return Some(direct);
            }
        }
        let local = format!("{}.{ctor}.{method}", func.module);
        if all.contains_key(&local) {
            return Some(local);
        }
    }
    None
}

fn resolve_import_target(mapped: &str, all: &HashMap<String, FunctionDef>) -> String {
    if all.contains_key(mapped) {
        return mapped.to_string();
    }
    classify_unknown(mapped)
}

fn classify_unknown(name: &str) -> String {
    let root = name.split('.').next().unwrap_or(name);
    if STDLIB.contains(&root) {
        format!("ext:stdlib.{name}")
    } else {
        format!("ext:third.{name}")
    }
}

pub fn is_external(qn: &str) -> bool {
    qn.starts_with("ext:")
}

fn child_named<'a>(node: Node<'a>, field: &str) -> Option<Node<'a>> {
    node.child_by_field_name(field)
}

fn text(node: Node, src: &[u8]) -> String {
    node.utf8_text(src).unwrap_or_default().to_string()
}
