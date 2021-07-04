pub mod typescript;

use proc_macro2::Span;
use std::{
    collections::{HashMap, VecDeque},
    path,
    path::{Path, PathBuf}
};

pub fn convert(ts: typescript::Load<'_>, dir: &Path) -> anyhow::Result<Vec<(PathBuf, syn::File)>> {
    let typescript::Load {
        root,
        parsed,
        children
    } = ts;
    let files: Vec<_> = children.iter().map(|(k, _)| -> &Path { k }).collect();
    let dests = destinations(root, &files, dir);
    if files.len() != 1 {
        return Ok(vec![]);
    }
    let dest = dests.get(root).unwrap();
    let p = tmp(parsed.get(root).unwrap());
    Ok(vec![(dest.to_owned(), p)])
}

fn tmp(parsed: &typescript::Parsed) -> syn::File {
    use swc_ecma_ast as ast;
    let typescript::Parsed { ast, .. } = parsed;
    let items = ast.body.iter().fold(Vec::new(), |mut items, item| {
        match item {
            ast::ModuleItem::Stmt(ast::Stmt::Decl(ast::Decl::Fn(f))) => {
                let attrs = vec![];
                items.push(syn::Item::Fn(syn::ItemFn {
                    attrs,
                    vis: syn::Visibility::Inherited,
                    sig: syn::Signature {
                        constness: None,
                        asyncness: None,
                        unsafety: None,
                        abi: None,
                        fn_token: syn::token::Fn::default(),
                        ident: ident(&f.ident),
                        generics: syn::Generics {
                            lt_token: None,
                            params: syn::punctuated::Punctuated::default(),
                            gt_token: None,
                            where_clause: None
                        },
                        paren_token: syn::token::Paren::default(),
                        inputs: syn::punctuated::Punctuated::default(),
                        variadic: None,
                        output: syn::ReturnType::Default
                    },
                    block: Box::new(syn::Block {
                        brace_token: syn::token::Brace::default(),
                        stmts: vec![]
                    })
                }));
            }
            _ => {}
        }
        items
    });
    syn::File {
        shebang: None,
        attrs: Vec::new(),
        items
    }
}

fn ident(x: &swc_ecma_ast::Ident) -> syn::Ident { syn::Ident::new(x.as_ref(), Span::call_site()) }

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

fn relative(root: &Path, file: &Path) -> PathBuf {
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

pub mod test_utils {
    use std::{
        env::temp_dir,
        fs,
        path::{Path, PathBuf},
        process::{Command, Stdio}
    };

    pub fn prepare_playwright(id: &str, commit: &str) -> PathBuf {
        let tmp = temp_dir();
        let betterty = tmp.join("betterty");
        fs::create_dir_all(&betterty).unwrap();
        let dir = betterty.join(id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::prepare_playwright;
    use std::{collections::HashSet, fs::File, io::Write};
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

    //#[test]
    // fn tmp() {
    //    let root = Path::new("/home/octaltree/storage/repos/others/playwright/src/inprocess.ts");
    //    let loaded = typescript::load(root).unwrap();
    //    dbg!(destinations(
    //        root,
    //        &loaded
    //            .children
    //            .iter()
    //            .filter(|(_, v)| v.is_empty())
    //            .map(|(k, _)| -> &Path { k })
    //            .collect::<Vec<_>>(),
    //        Path::new("/tmp/foo")
    //    ));
    //}

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

    #[test]
    fn can_convert_basic() -> anyhow::Result<()> {
        let dir = no_dependences("can_convert_basic")?;
        {
            let root = dir.join("src/client/events.ts");
            let loaded = typescript::load(&root).unwrap();
            let result = convert(loaded, Path::new("/"))?;
            assert_eq!(result.len(), 1);
        }
        Ok(())
    }

    fn no_dependences(id: &str) -> anyhow::Result<PathBuf> {
        let dir = prepare_playwright(id, "fe32d384");
        let root = dir.join("src/inprocess.ts");
        let loaded = typescript::load(&root).unwrap();
        let empties: HashSet<_> = loaded
            .children
            .iter()
            .filter(|(_, v)| v.is_empty())
            .map(|(k, _)| -> &Path { k })
            .collect();
        dbg!(&empties);
        for p in [
            dir.join("src/client/events.ts"),
            dir.join("src/common/types.ts"),
            dir.join("src/generated/consoleApiSource.ts"),
            dir.join("src/generated/injectedScriptSource.ts"),
            dir.join("src/generated/recorderSource.ts"),
            dir.join("src/generated/utilityScriptSource.ts"),
            dir.join("src/server/common/domErrors.ts"),
            dir.join("src/server/common/utilityScriptSerializers.ts"),
            dir.join("src/server/injected/selectorEngine.ts"),
            dir.join("src/server/macEditingCommands.ts"),
            dir.join("src/server/snapshot/snapshotTypes.ts"),
            dir.join("src/server/supplements/har/har.ts"),
            dir.join("src/server/supplements/recorder/recorderActions.ts"),
            dir.join("src/server/usKeyboardLayout.ts"),
            dir.join("src/utils/errors.ts")
        ] {
            let is_empty = empties.get(&p as &Path).is_some();
            assert!(is_empty, "{} has dependencies", p.display());
        }
        Ok(dir)
    }
}
