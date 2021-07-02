pub mod dependencies;
pub mod parser;

use parser::Parsed;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf}
};

pub struct Load<'a> {
    pub root: &'a Path,
    pub parsed: HashMap<PathBuf, Parsed>,
    pub children: HashMap<PathBuf, Vec<Option<PathBuf>>>
}

pub fn load(file: &Path) -> anyhow::Result<Load<'_>> {
    let mut parsed: HashMap<PathBuf, Parsed> = HashMap::new();
    let mut children: HashMap<PathBuf, Vec<Option<PathBuf>>> = HashMap::new();
    let mut que: VecDeque<PathBuf> = vec![file.to_owned()].into();
    while let Some(target) = que.pop_front() {
        if parsed.get(&target).is_some() {
            continue;
        }
        let body = fs::read_to_string(&target)?;
        let (p, cs) = analyze_module(&target, &body)?;
        parsed.insert(target.to_owned(), p);
        children.insert(target.to_owned(), cs.clone());
        que.append(&mut cs.into_iter().flatten().collect());
    }
    Ok(Load {
        root: file,
        parsed,
        children
    })
}

fn analyze_module(path: &Path, source: &str) -> anyhow::Result<(Parsed, Vec<Option<PathBuf>>)> {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let p = parser::parse_module(name, source)?;
    let cs = p
        .dependencies
        .iter()
        .map(|d| dependencies::find(path, d.specifier.as_ref()))
        .collect();
    Ok((p, cs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::{Path, PathBuf},
        process::{Command, Stdio}
    };

    #[test]
    fn can_analyze_module() {
        let tmp = std::env::temp_dir();
        let commit = "e9246089";
        let dir = prepare_target(&tmp, commit);
        {
            let file = dir.join("src/utils/utils.ts");
            let (p, cs) = analyze_module(&file, &fs::read_to_string(&file).unwrap()).unwrap();
            assert_eq!(
                p.dependencies
                    .iter()
                    .map(|d| d.specifier.as_ref())
                    .collect::<Vec<_>>(),
                [
                    "path",
                    "fs",
                    "rimraf",
                    "crypto",
                    "os",
                    "child_process",
                    "proxy-from-env",
                    "url",
                    "https-proxy-agent",
                    "https",
                    "http",
                    "./../../package.json",
                ]
            );
            assert_eq!(
                cs,
                [
                    Some(dir.join("node_modules/@types/node/path.d.ts")),
                    Some(dir.join("node_modules/@types/node/fs.d.ts")),
                    Some(dir.join("node_modules/@types/rimraf/index.d.ts")),
                    Some(dir.join("node_modules/@types/node/crypto.d.ts")),
                    Some(dir.join("node_modules/@types/node/os.d.ts")),
                    Some(dir.join("node_modules/@types/node/child_process.d.ts")),
                    Some(dir.join("node_modules/@types/proxy-from-env/index.d.ts")),
                    Some(dir.join("node_modules/@types/node/url.d.ts")),
                    Some(dir.join("node_modules/https-proxy-agent/dist/index.d.ts")),
                    Some(dir.join("node_modules/@types/node/https.d.ts")),
                    Some(dir.join("node_modules/@types/node/http.d.ts")),
                    None
                ]
            );
        }
        {
            let file = dir.join("src/client/playwright.ts");
            let (p, cs) = analyze_module(&file, &fs::read_to_string(&file).unwrap()).unwrap();
            assert_eq!(
                p.dependencies
                    .iter()
                    .map(|d| d.specifier.as_ref())
                    .collect::<Vec<_>>(),
                [
                    "../protocol/channels",
                    "./browserType",
                    "./channelOwner",
                    "./selectors",
                    "./electron",
                    "../utils/errors",
                    "./types",
                    "./android",
                    "./socksSocket",
                ]
            );
            assert_eq!(
                cs,
                [
                    Some(dir.join("src/protocol/channels.ts")),
                    Some(dir.join("src/client/browserType.ts")),
                    Some(dir.join("src/client/channelOwner.ts")),
                    Some(dir.join("src/client/selectors.ts")),
                    Some(dir.join("src/client/electron.ts")),
                    Some(dir.join("src/utils/errors.ts")),
                    Some(dir.join("src/client/types.ts")),
                    Some(dir.join("src/client/android.ts")),
                    Some(dir.join("src/client/socksSocket.ts")),
                ]
            );
        }
    }

    #[test]
    fn can_load() {
        let tmp = std::env::temp_dir();
        let commit = "master";
        let dir = prepare_target(&tmp, commit);
        let Load {
            parsed, children, ..
        } = load(&dir.join("src/client/playwright.ts")).unwrap();
        let files = parsed.iter().map(|(f, _)| f);
        let bad: Vec<_> = files
            .filter_map(|f| {
                let p = parsed.get(f);
                let cs = children.get(f).into_iter().flatten();
                let dependencies = p.into_iter().map(|p| p.dependencies.iter()).flatten();
                let not_found: Vec<_> = dependencies
                    .zip(cs)
                    .filter(|(d, c)| {
                        c.is_none() && !d.specifier.as_ref().ends_with("/package.json")
                    })
                    .collect();
                if not_found.is_empty() {
                    None
                } else {
                    Some((f, not_found))
                }
            })
            .collect();
        dbg!(&bad);
        assert!(bad.is_empty());
    }

    fn prepare_target(tmp: &Path, commit: &str) -> PathBuf {
        let dir = tmp.join(format!("betterty-playwright-{}", commit));
        Command::new("git")
            .args(&[
                "clone",
                "https://github.com/microsoft/playwright",
                &dir.display().to_string()
            ])
            .stderr(Stdio::null())
            .status()
            .unwrap();
        cmd(&dir, &["git", "checkout", commit]);
        if !dir.join("node_modules").exists() {
            cmd(&dir, &["npm", "install"]);
            cmd(&dir, &["npm", "run-script", "build"]);
        }
        dir
    }

    fn cmd(cd: &Path, cmd: &[&str]) {
        let status = Command::new(cmd[0])
            .args(&cmd[1..])
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .current_dir(cd)
            .status()
            .unwrap();
        if !status.success() {
            panic!("");
        }
    }
}
