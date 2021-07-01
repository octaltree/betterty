use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf}
};

// fn search_dependencies(path: &Path) -> anyhow::Result<()> {
//    let body = fs::read_to_string(path)?;
//    let mut parsed = HashMap::new();
//    let (m, s, c) = parser::parse_module(path.file_name().unwrap().to_str().unwrap(), &body)?;
//    let mut que: VecDeque<_> = analyze_dependencies(&m, &s, &c)
//        .into_iter()
//        .map(|x| (path.to_owned(), x))
//        .collect();
//    parsed.insert(path.to_owned(), (m, s, c));
//    while let Some((p, d)) = que.pop_front() {
//        if let Some(p) = dependencies::find(&p, d.specifier.as_ref()) {
//            if parsed.get(&p).is_none() {
//                let body = fs::read_to_string(&p)?;
//                let (m, s, c) =
//                    parser::parse_module(path.file_name().unwrap().to_str().unwrap(), &body)?;
//                let mut ds = analyze_dependencies(&m, &s, &c)
//                    .into_iter()
//                    .map(|x| (p.to_owned(), x))
//                    .collect();
//                que.append(&mut ds);
//                parsed.insert(p.to_owned(), (m, s, c));
//            }
//        } else {
//            dbg!(Err::<(), _>((p, d.specifier.as_ref())));
//        }
//    }
//    Ok(())
//}

/// Only for posix path
/// https://github.com/Microsoft/TypeScript-Handbook/blob/master/pages/Module%20Resolution.md
/// https://nodejs.org/api/modules.html#modules_all_together
/// https://github.com/microsoft/TypeScript/blob/837ed9669718fa3515aabc99974abe91f7254a3e/src/jsTyping/jsTyping.ts#L32
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
    if p.ends_with(".ts") || p.ends_with(".d.ts") {
        if let Some(p) = Path::new(p).canonicalize().ok() {
            return Some(p.to_owned());
        }
    }
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
        let p = format!("{}", d.join("node_modules/@types/node").join(s).display());
        if let Some(p) = find_file(&p) {
            return Some(p);
        }
        let p = format!("{}", d.join("node_modules/@types").join(s).display());
        if let Some(p) = find_file(&p) {
            return Some(p);
        }
        let p = format!("{}", d.join("node_modules").join(s).display());
        if let Some(p) = find_file(&p) {
            return Some(p);
        }
        if let Some(p) = analyze_package_json(&p) {
            return Some(p);
        }
    }
    None
}

fn analyze_package_json(p: &str) -> Option<PathBuf> {
    let json = Path::new(p).join("package.json").canonicalize().ok()?;
    let json = File::open(json).ok()?;
    let data: PackageJson = serde_json::from_reader(json).ok()?;
    if let Some(types) = data.types.or(data.typings) {
        return find_file(&format!("{}/{}", p, types));
    }
    None
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    types: Option<String>,
    typings: Option<String>
}
