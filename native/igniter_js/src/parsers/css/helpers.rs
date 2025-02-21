use swc_common::{
    comments::SingleThreadedComments, input::SourceFileInput, sync::Lrc, FileName, SourceMap,
};
use swc_css_ast::Stylesheet;
use swc_css_parser::{
    lexer::Lexer,
    parser::{Parser, ParserConfig},
};

pub fn parse(
    file_content: &str,
) -> Result<(Stylesheet, SingleThreadedComments, Lrc<SourceMap>), String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Anon.into(), file_content.into());

    let comments = SingleThreadedComments::default();

    let lexer = Lexer::new(
        SourceFileInput::from(&*fm),
        Some(&comments),
        ParserConfig::default(),
    );

    let mut parser = Parser::new(lexer, ParserConfig::default());

    let stylesheet = match parser.parse_all() {
        Ok(s) => s,
        Err(_) => {
            return Err("Failed to parse CSS".to_string());
        }
    };

    Ok((stylesheet, comments, cm))
}
