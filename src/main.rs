use std::{
    fs,
    path::{Path, PathBuf}
};
use swc_common::{comments::SingleThreadedComments, sync::Lrc, SourceMap};
use swc_ecma_dep_graph::analyze_dependencies;

fn main() {
    betterty::search_dependencies(
        Path::new("/home/octaltree/storage/repos/others/playwright/src/client/playwright.ts"),
        Path::new("/home/octaltree/storage/repos/others/playwright/node_modules")
    )
    .unwrap();

    // for p in ls_rec("/home/octaltree/storage/repos/others/playwright/src") {
    //    if p.extension() != Some("ts".as_ref()) {
    //        continue;
    //    }
    //    dbg!(&p);
    //    let (name, body): (&str, &str) = (
    //        p.file_name().unwrap().to_str().unwrap(),
    //        &fs::read_to_string(&p).unwrap()
    //    );
    //    if name != "installDeps.ts" {
    //        continue;
    //    }
    //    let (m, s, c): (swc_ecma_ast::Module, Lrc<SourceMap>, SingleThreadedComments) =
    //        betterty::parser::parse_module(name, body).unwrap();
    //    let dependencies = analyze_dependencies(&m, &s, &c);
    //    dbg!(&dependencies);
    //    dbg!(dependencies.len());
    //}
}

fn ls_rec<P: AsRef<Path>>(p: P) -> Vec<PathBuf> {
    fs::read_dir(p)
        .unwrap()
        .filter_map(Result::ok)
        .fold(Vec::new(), |mut a, x| {
            if x.file_type().unwrap().is_dir() {
                a.append(&mut ls_rec(x.path()));
            } else {
                a.push(x.path());
            }
            a
        })
}
