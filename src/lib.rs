mod module_resolver;
mod parser;

use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::Path
};
use swc_ecma_dep_graph::analyze_dependencies;

pub fn search_dependencies(path: &Path) -> anyhow::Result<()> {
    let body = fs::read_to_string(path)?;
    let mut parsed = HashMap::new();
    let (m, s, c) = parser::parse_module(path.file_name().unwrap().to_str().unwrap(), &body)?;
    let mut que: VecDeque<_> = analyze_dependencies(&m, &s, &c)
        .into_iter()
        .map(|x| (path.to_owned(), x))
        .collect();
    parsed.insert(path.to_owned(), (m, s, c));
    while let Some((p, d)) = que.pop_front() {
        dbg!((&p, d.specifier.as_ref()));
        if let Some(p) = module_resolver::find(&p, d.specifier.as_ref()) {
            dbg!(Ok::<_, ()>(&p));
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
            dbg!(Err::<(), _>(p));
        }
    }
    Ok(())
}
