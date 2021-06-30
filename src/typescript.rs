mod dependencies;
mod parser;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::{HashMap, VecDeque},
        fs,
        path::{Path, PathBuf},
        process::{Command, Stdio}
    };
    use swc_ecma_dep_graph::analyze_dependencies;

    #[test]
    fn dependencies() {
        let commit = "e9246089";
        let dir = prepare_target(commit);
        let file = dir.join("src/client/playwright.ts");
        search_dependencies(Path::new(&file)).unwrap();
    }

    fn prepare_target(commit: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("betterty-playwright");
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
        }
        dir
    }

    fn cmd(cd: &Path, cmd: &[&str]) {
        let status = Command::new(cmd[0])
            .args(&cmd[1..])
            .stderr(Stdio::null())
            .current_dir(cd)
            .status()
            .unwrap();
        if !status.success() {
            panic!("");
        }
    }

    fn search_dependencies(path: &Path) -> anyhow::Result<()> {
        let body = fs::read_to_string(path)?;
        let mut parsed = HashMap::new();
        let (m, s, c) = parser::parse_module(path.file_name().unwrap().to_str().unwrap(), &body)?;
        let mut que: VecDeque<_> = analyze_dependencies(&m, &s, &c)
            .into_iter()
            .map(|x| (path.to_owned(), x))
            .collect();
        parsed.insert(path.to_owned(), (m, s, c));
        while let Some((p, d)) = que.pop_front() {
            // dbg!((&p, d.specifier.as_ref()));
            if let Some(p) = dependencies::find(&p, d.specifier.as_ref()) {
                if parsed.get(&p).is_none() {
                    let body = fs::read_to_string(&p)?;
                    let (m, s, c) =
                        parser::parse_module(path.file_name().unwrap().to_str().unwrap(), &body)?;
                    let mut ds = analyze_dependencies(&m, &s, &c)
                        .into_iter()
                        .map(|x| (p.to_owned(), x))
                        .collect();
                    que.append(&mut ds);
                    parsed.insert(p.to_owned(), (m, s, c));
                }
            } else {
                dbg!(Err::<(), _>((p, d.specifier.as_ref())));
            }
        }
        Ok(())
    }
}
