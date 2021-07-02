use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf
};
use structopt::StructOpt;
use syn::__private::ToTokens;

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let loaded = betterty::typescript::load(&opt.ts_file)?;
    let rs = betterty::convert(loaded, &opt.dir)?;
    dbg!(&rs);
    write(rs)?;
    Ok(())
}

fn write(rs: Vec<(PathBuf, syn::File)>) -> anyhow::Result<()> {
    for (p, f) in rs.into_iter() {
        if let Some(d) = p.parent() {
            fs::create_dir_all(&d)?;
        }
        let mut file = File::open(&p)?;
        format(&mut file, f)?;
    }
    Ok(())
}

fn format<W: Write>(w: &mut W, f: syn::File) -> std::io::Result<()> {
    let s = f
        .into_token_stream()
        .into_iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    w.write_all(&s.into_bytes())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "TS", parse(from_os_str))]
    ts_file: PathBuf,
    #[structopt(name = "DIR", parse(from_os_str))]
    dir: PathBuf
}
