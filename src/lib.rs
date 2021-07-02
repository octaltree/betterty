pub mod typescript;

use std::{
    collections::HashMap,
    path,
    path::{Path, PathBuf}
};

pub fn convert(ts: typescript::Load<'_>, dir: &Path) -> anyhow::Result<Vec<(PathBuf, syn::File)>> {
    let entry = dir.join("lib.rs");
    Ok(vec![(
        dir.join("lib.rs"),
        syn::File {
            shebang: None,
            attrs: Vec::new(),
            items: Vec::new()
        }
    )])
}

fn destinations(root: &Path, files: &[&Path], dir: &Path) -> HashMap<PathBuf, PathBuf> {
    files
        .iter()
        .map(|f| {
            let mut cs = f.components();
            consume_node_modules(&mut cs);
            todo!()
        })
        .collect()
}

fn consume_node_modules(components: &mut path::Components<'_>) {
    for c in components {
        if c == path::Component::Normal("node_modules".as_ref()) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write};
    use tempdir::TempDir;

    #[test]
    fn can_consume() {
        {
            let mut c = Path::new("/foo/node_modules/bar/hoge").components();
            consume_node_modules(&mut c);
            let p: PathBuf = c.collect();
            assert_eq!(p, Path::new("bar/hoge"));
        }
        {
            let mut c = Path::new("/foo/bar/hoge").components();
            consume_node_modules(&mut c);
            let p: PathBuf = c.collect();
            assert_eq!(p, Path::new(""));
        }
    }

    #[test]
    fn can_convert_blank() -> anyhow::Result<()> {
        let tmp = TempDir::new("betterty")?;
        let p = tmp.path().join("index.ts");
        File::create(&p)?;
        let loaded = typescript::load(&p)?;
        dbg!(&loaded.parsed.get(&p).unwrap().ast);
        dbg!(&loaded.parsed.get(&p).unwrap().comments);
        let rs = convert(loaded, Path::new("/"))?;
        assert_eq!(
            rs,
            [(
                Path::new("/lib.rs").into(),
                syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: Vec::new()
                }
            )]
        );
        Ok(())
    }

    #[test]
    fn can_convert_function() -> anyhow::Result<()> {
        let tmp = TempDir::new("betterty")?;
        let p = tmp.path().join("index.ts");
        let mut f = File::create(&p)?;
        f.write_all(r#"function foo(){}"#.as_bytes())?;
        let loaded = typescript::load(&p)?;
        dbg!(&loaded.parsed.get(&p).unwrap().ast);
        dbg!(&loaded.parsed.get(&p).unwrap().comments);
        let rs = convert(loaded, Path::new("/"))?;
        assert_eq!(
            rs,
            [(
                Path::new("/lib.rs").into(),
                syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: vec![syn::Item::Fn(syn::parse_str::<syn::ItemFn>("fn foo(){}")?)]
                }
            )]
        );
        Ok(())
    }
}
