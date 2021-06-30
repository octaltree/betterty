use std::path::{Path, PathBuf};

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
        dbg!(&a);
        return find_file(&a);
    }
    if s.starts_with('#') {
        todo!()
    }
    search_node_modules(dir, s)
}

fn get_core_module(s: &str) -> Option<PathBuf> { None }

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
    None
}

fn search_node_modules(dir: &Path, s: &str) -> Option<PathBuf> {
    // for d in dir.ancestors() {
    //    let p = format!("{}", d.join("node_modules").join(s).display());
    //    if let Some(p) = find_file(&p) {
    //        return Some(p);
    //    }
    //}
    None
}

// require(X) from module at path Y
// 1. If X is a core module,
//   a. return the core module
//   b. STOP
// 2. If X begins with '/'
//   a. set Y to be the filesystem root
// 3. If X begins with './' or '/' or '../'
//   a. LOAD_AS_FILE(Y + X)
//   b. LOAD_AS_DIRECTORY(Y + X)
//   c. THROW "not found"
// 4. If X begins with '#'
//   a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
// 5. LOAD_PACKAGE_SELF(X, dirname(Y))
// 6. LOAD_NODE_MODULES(X, dirname(Y))
// 7. THROW "not found"
//
// LOAD_AS_FILE(X)
// 1. If X is a file, load X as its file extension format. STOP
// 2. If X.js is a file, load X.js as JavaScript text. STOP
// 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
// 4. If X.node is a file, load X.node as binary addon. STOP
//
// LOAD_INDEX(X)
// 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
// 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
// 3. If X/index.node is a file, load X/index.node as binary addon. STOP
//
// LOAD_AS_DIRECTORY(X)
// 1. If X/package.json is a file,
//   a. Parse X/package.json, and look for "main" field.
//   b. If "main" is a falsy value, GOTO 2.
//   c. let M = X + (json main field)
//   d. LOAD_AS_FILE(M)
//   e. LOAD_INDEX(M)
//   f. LOAD_INDEX(X) DEPRECATED
//   g. THROW "not found"
// 2. LOAD_INDEX(X)
//
// LOAD_NODE_MODULES(X, START)
// 1. let DIRS = NODE_MODULES_PATHS(START)
// 2. for each DIR in DIRS:
//   a. LOAD_PACKAGE_EXPORTS(X, DIR)
//   b. LOAD_AS_FILE(DIR/X)
//   c. LOAD_AS_DIRECTORY(DIR/X)
//
// NODE_MODULES_PATHS(START)
// 1. let PARTS = path split(START)
// 2. let I = count of PARTS - 1
// 3. let DIRS = [GLOBAL_FOLDERS]
// 4. while I >= 0,
//   a. if PARTS[I] = "node_modules" CONTINUE
//   b. DIR = path join(PARTS[0 .. I] + "node_modules")
//   c. DIRS = DIRS + DIR
//   d. let I = I - 1
// 5. return DIRS
//
// LOAD_PACKAGE_IMPORTS(X, DIR)
// 1. Find the closest package scope SCOPE to DIR.
// 2. If no scope was found, return.
// 3. If the SCOPE/package.json "imports" is null or undefined, return.
// 4. let MATCH = PACKAGE_IMPORTS_RESOLVE(X, pathToFileURL(SCOPE),
//  ["node", "require"]) defined in the ESM resolver.
// 5. RESOLVE_ESM_MATCH(MATCH).
//
// LOAD_PACKAGE_EXPORTS(X, DIR)
// 1. Try to interpret X as a combination of NAME and SUBPATH where the name
//   may have a @scope/ prefix and the subpath begins with a slash (`/`).
// 2. If X does not match this pattern or DIR/NAME/package.json is not a file,
//   return.
// 3. Parse DIR/NAME/package.json, and look for "exports" field.
// 4. If "exports" is null or undefined, return.
// 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(DIR/NAME), "." + SUBPATH,
//   `package.json` "exports", ["node", "require"]) defined in the ESM resolver.
// 6. RESOLVE_ESM_MATCH(MATCH)
//
// LOAD_PACKAGE_SELF(X, DIR)
// 1. Find the closest package scope SCOPE to DIR.
// 2. If no scope was found, return.
// 3. If the SCOPE/package.json "exports" is null or undefined, return.
// 4. If the SCOPE/package.json "name" is not the first segment of X, return.
// 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(SCOPE),
//   "." + X.slice("name".length), `package.json` "exports", ["node", "require"])
//   defined in the ESM resolver.
// 6. RESOLVE_ESM_MATCH(MATCH)
//
// RESOLVE_ESM_MATCH(MATCH)
// 1. let { RESOLVED, EXACT } = MATCH
// 2. let RESOLVED_PATH = fileURLToPath(RESOLVED)
// 3. If EXACT is true,
//   a. If the file at RESOLVED_PATH exists, load RESOLVED_PATH as its extension
//      format. STOP
// 4. Otherwise, if EXACT is false,
//   a. LOAD_AS_FILE(RESOLVED_PATH)
//   b. LOAD_AS_DIRECTORY(RESOLVED_PATH)
// 5. THROW "not found"
