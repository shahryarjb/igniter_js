use swc_common::{input::SourceFileInput, sync::Lrc, FileName, SourceMap};
use swc_css_ast::Stylesheet;
use swc_css_parser::{
    lexer::Lexer,
    parser::{Parser, ParserConfig},
};

pub fn parse(file_content: &str) -> Result<(Stylesheet, Lrc<SourceMap>), String> {
    // Initialize source map
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Anon.into(), file_content.into());

    // Create a lexer for the parser
    let lexer = Lexer::new(SourceFileInput::from(&*fm), None, ParserConfig::default());

    // Initialize parser
    let mut parser = Parser::new(lexer, ParserConfig::default());

    // Parse CSS into AST
    let stylesheet = match parser.parse_all() {
        Ok(s) => s,
        Err(_) => {
            return Err("Failed to parse CSS".to_string());
        }
    };

    Ok((stylesheet, cm))
}
