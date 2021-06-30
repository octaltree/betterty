use once_cell::sync::Lazy;
use std::{
    collections::HashSet,
    path::{Path, PathBuf}
};

/// https://github.com/microsoft/TypeScript/blob/837ed9669718fa3515aabc99974abe91f7254a3e/src/jsTyping/jsTyping.ts#L32
static CORE_MODULES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "assert",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "https",
        "http2",
        "inspector",
        "net",
        "os",
        "path",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "string_decoder",
        "timers",
        "tls",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "zlib"
    ]
    .iter()
    .map(|&s| s)
    .collect()
});

/// https://github.com/Microsoft/TypeScript-Handbook/blob/master/pages/Module%20Resolution.md
/// https://nodejs.org/api/modules.html#modules_all_together
pub(super) fn resolve_module_path(file: &Path, s: &str, node_modules: &Path) -> Option<PathBuf> {
    let dir = file.parent().unwrap();
    let ts = dir.join(format!("{}.ts", s)).canonicalize();
    let dts = dir.join(format!("{}.d.ts", s)).canonicalize();
    if let Ok(p) = ts {
        Some(p)
    } else if let Ok(p) = dts {
        Some(p)
    } else {
        None
    }
}

/// Only for posix path
pub fn find(file: &Path, s: &str) -> Option<PathBuf> {
    let dir = file.parent()?;
    if let Some(m) = get_core_module(s) {
        return Some(m);
    }
    if s.starts_with('/') {
        return find_file(s);
    }
    if s.starts_with("./") || s.starts_with("../") {
        let a = format!("{}/{}", dir.display(), s);
        return find_file(&a);
    }
    if s.starts_with('#') {
        todo!()
    }
    search_node_modules(dir, s)
}

fn get_core_module(_s: &str) -> Option<PathBuf> { None }

fn find_file(p: &str) -> Option<PathBuf> {
    // if let Some(p) = Path::new(p).canonicalize().ok() {
    //    return Some(p.to_owned());
    //}
    if let Some(p) = Path::new(&format!("{}.ts", p)).canonicalize().ok() {
        return Some(p.to_owned());
    }
    if let Some(p) = Path::new(&format!("{}.d.ts", p)).canonicalize().ok() {
        return Some(p.to_owned());
    }
    if let Some(p) = Path::new(p).join("index.ts").canonicalize().ok() {
        return Some(p.to_owned());
    }
    if let Some(p) = Path::new(p).join("index.d.ts").canonicalize().ok() {
        return Some(p.to_owned());
    }
    None
}

fn search_node_modules(dir: &Path, s: &str) -> Option<PathBuf> {
    for d in dir.ancestors() {
        if CORE_MODULES.get(&s).is_some() {
            let p = format!("{}", d.join("node_modules/@types/node").join(s).display());
            if let Some(p) = find_file(&p) {
                return Some(p);
            }
        }
        let p = format!("{}", d.join("node_modules/@types").join(s).display());
        if let Some(p) = find_file(&p) {
            return Some(p);
        }
        let p = format!("{}", d.join("node_modules").join(s).display());
        if let Some(p) = find_file(&p) {
            return Some(p);
        }
    }
    None
}
