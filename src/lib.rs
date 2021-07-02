pub mod typescript;

use std::{
    collections::{HashMap, VecDeque},
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
    let res: HashMap<_, _> = files
        .iter()
        .map(|&f| {
            if f == root {
                return (f.to_owned(), Path::new("lib.rs").to_owned());
            }
            let tail: PathBuf = {
                let mut cs = f.components();
                consume_node_modules(&mut cs);
                cs.collect()
            };
            if tail == Path::new("") {
                (f.to_owned(), relative(root, f))
            } else {
                (f.to_owned(), tail)
            }
        })
        .map(|(f, r)| {
            let r: PathBuf = eat_dots(r.components()).into_iter().collect();
            let abs = dir.join(r);
            (f, abs)
        })
        .collect();
    // TODO: rename if conflict
    assert_eq!(files.len(), res.iter().len());
    res
}

fn relative<'a>(root: &Path, file: &'a Path) -> PathBuf {
    let l: Vec<_> = root.components().into_iter().collect();
    let r: Vec<_> = file.components().into_iter().collect();
    let (cnt, _) = root
        .components()
        .zip(file.components())
        .map(|(a, b)| a == b)
        .fold((0, true), |(cnt, success), same| {
            let success = success && same;
            let cnt = if success { cnt + 1 } else { cnt };
            (cnt, success)
        });
    use std::borrow::Cow;
    let prefix = if l.len() - cnt <= 1 {
        Cow::Borrowed("")
    } else {
        Cow::Owned((0..(l.len() - cnt - 1)).map(|_| "../").collect::<String>())
    };
    Path::new(&*prefix).join(r[cnt..].iter().collect::<PathBuf>())
}

fn eat_dots<'a>(components: impl Iterator<Item = path::Component<'a>>) -> Vec<path::Component<'a>> {
    let mut que = components.collect::<VecDeque<_>>();
    dbg!(&que);
    while let Some(front) = que.front() {
        if front == &path::Component::CurDir || front == &path::Component::ParentDir {
            que.pop_front();
        } else {
            break;
        }
    }
    que.into()
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
    fn can_eat_dots() {
        assert_eq!(
            eat_dots(Path::new("../utils/st.ts").components()),
            [
                path::Component::Normal("utils".as_ref()),
                path::Component::Normal("st.ts".as_ref()),
            ]
        );
    }

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
    fn can_relative() {
        assert_eq!(
            relative(Path::new("/foo.ts"), Path::new("/bar.ts")),
            Path::new("bar.ts")
        );
        assert_eq!(
            relative(Path::new("/foo.ts"), Path::new("/foo/bar.ts")),
            Path::new("foo/bar.ts")
        );
        assert_eq!(
            relative(Path::new("/foo/bar.ts"), Path::new("/bar.ts")),
            Path::new("../bar.ts")
        );
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
