use std::fmt;
use swc_common::{comments::SingleThreadedComments, sync::Lrc, FileName, SourceMap};
use swc_ecma_ast as ast;
use swc_ecma_dep_graph::{analyze_dependencies, DependencyDescriptor};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

pub struct Parsed {
    pub ast: ast::Module,
    pub source_map: Lrc<SourceMap>,
    pub comments: SingleThreadedComments,
    pub dependencies: Vec<DependencyDescriptor>
}

#[derive(Debug)]
pub struct ParseError(swc_ecma_parser::error::Error);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) }
}

impl From<swc_ecma_parser::error::Error> for ParseError {
    fn from(x: swc_ecma_parser::error::Error) -> Self { ParseError(x) }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None }

    fn description(&self) -> &str { "description() is deprecated; use Display" }

    fn cause(&self) -> Option<&dyn std::error::Error> { self.source() }
}

pub fn parse_module(file_name: &str, source: &str) -> Result<Parsed, ParseError> {
    let sm: Lrc<SourceMap> = Default::default();
    let fm = sm.new_source_file(FileName::Custom(file_name.to_string()), source.to_string());

    let comments = SingleThreadedComments::default();
    let lexer: Lexer<StringInput<'_>> = Lexer::new(
        Syntax::Typescript(TsConfig {
            dts: file_name.ends_with(".d.ts"),
            tsx: file_name.contains("tsx"),
            dynamic_import: true,
            decorators: true,
            no_early_errors: true,
            import_assertions: true
        }),
        Default::default(),
        (&*fm).into(),
        Some(&comments)
    );

    let mut p = Parser::new_from(lexer);
    let m = p.parse_module()?;
    if let Some(err) = p.take_errors().into_iter().next() {
        return Err(err.into());
    }
    let ds = analyze_dependencies(&m, &sm, &comments);
    Ok(Parsed {
        ast: m,
        source_map: sm,
        comments,
        dependencies: ds
    })
}
