use std::collections::HashSet;
use swc_common::comments::Comments;
use swc_common::Spanned;
use swc_css_codegen::{writer::basic::BasicCssWriter, CodeGenerator, CodegenConfig, Emit};

use crate::parsers::css::helpers::*;
use swc_css_ast::*;
use swc_css_visit::{Visit, VisitMut, VisitMutWith, VisitWith};

/// Visitor struct to check if a specific `@import` exists
struct ImportChecker<'a> {
    search_import: &'a str,
    found: bool,
}

impl<'a> Visit for ImportChecker<'a> {
    fn visit_import_prelude(&mut self, node: &ImportPrelude) {
        match &*node.href {
            // Case for `@import url("file.css");`
            ImportHref::Url(url) => {
                if let Some(boxed_value) = &url.value {
                    if let UrlValue::Str(s) = &**boxed_value {
                        if s.value == self.search_import {
                            self.found = true;
                        }
                    }
                }
            }
            // Case for `@import "file.css";`
            ImportHref::Str(s) => {
                if s.value == self.search_import {
                    self.found = true;
                }
            }
        }
        node.visit_children_with(self)
    }
}

/// Function to check if a given `@import` exists in the CSS code
pub fn is_imported_from_ast(file_content: &str, import: &str) -> Result<bool, String> {
    // Parse CSS
    let (stylesheet, _comments, _cm) = match parse(file_content) {
        Ok(result) => result,
        Err(_) => return Err("Failed to parse CSS content".to_string()),
    };

    // Use visitor to check for the import
    let mut checker = ImportChecker {
        search_import: import,
        found: false,
    };

    stylesheet.visit_with(&mut checker);

    if checker.found {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Visitor struct to insert new `@import` rules
struct ImportInserter {
    new_import_strings: Vec<String>,
    new_imports: Vec<ImportPrelude>,
    existing_imports: HashSet<String>,
}

impl VisitMut for ImportInserter {
    fn visit_mut_stylesheet(&mut self, stylesheet: &mut Stylesheet) {
        // Collect existing `@import` rules
        for rule in &stylesheet.rules {
            if let Rule::AtRule(at_rule) = rule {
                if let Some(prelude) = &at_rule.prelude {
                    if let AtRulePrelude::ImportPrelude(import) = &**prelude {
                        match &*import.href {
                            ImportHref::Url(url) => {
                                if let Some(boxed_value) = &url.value {
                                    if let UrlValue::Str(s) = &**boxed_value {
                                        self.existing_imports.insert(s.value.to_string());
                                    }
                                }
                            }
                            ImportHref::Str(s) => {
                                self.existing_imports.insert(s.value.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Process `new_import_strings` and convert them into `ImportPrelude`
        for import_path in self.new_import_strings.drain(..) {
            let cleaned_import = import_path
                .trim_start_matches("@import")
                .trim()
                .trim_end_matches(';')
                .trim_matches('"');

            let import_prelude = ImportPrelude {
                span: Default::default(),
                href: Box::new(ImportHref::Str(Str {
                    span: Default::default(),
                    value: cleaned_import.into(),
                    raw: None,
                })),
                layer_name: None,
                import_conditions: None,
            };

            if !self.existing_imports.contains(cleaned_import) {
                self.new_imports.push(import_prelude);
            }
        }

        // Convert new `@import` rules into `AtRule` objects, avoiding duplicates
        let mut new_rules: Vec<Rule> = self
            .new_imports
            .drain(..)
            .map(|import| {
                Rule::AtRule(Box::new(AtRule {
                    span: Default::default(),
                    name: swc_css_ast::AtRuleName::Ident(Ident {
                        span: Default::default(),
                        value: "import".into(),
                        raw: None,
                    }),
                    prelude: Some(Box::new(AtRulePrelude::ImportPrelude(import))),
                    block: None,
                }))
            })
            .collect();

        // Add new `@import` rules at the beginning
        new_rules.extend(stylesheet.rules.clone());
        stylesheet.rules = new_rules;
    }
}

pub fn insert_import_to_ast(file_content: &str, import_lines: &str) -> Result<String, String> {
    // Parse the existing CSS into an AST and collect comments
    let (mut stylesheet, comments, _cm) = parse(file_content)?;

    // Initialize the visitor with raw `import_lines` and empty `existing_imports`
    let mut inserter = ImportInserter {
        new_import_strings: import_lines
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect(),
        new_imports: Vec::new(),
        existing_imports: HashSet::new(),
    };

    // Visit and modify the AST
    stylesheet.visit_mut_with(&mut inserter);
    let output = generate_css_with_comments(&stylesheet, &comments);
    Ok(output)
}

struct InlineCommentCollector<'a> {
    comments: &'a dyn Comments,
    // Stores tuples of (property name, inline comment)
    collected_comments: Vec<(String, String)>,
}

impl<'a> Visit for InlineCommentCollector<'a> {
    fn visit_declaration(&mut self, decl: &Declaration) {
        let property = match &decl.name {
            DeclarationName::Ident(ident) => ident.value.to_string(),
            DeclarationName::DashedIdent(dashed) => dashed.value.to_string(),
        };
        let pos = decl.span().hi();
        if let Some(comment_list) = self.comments.get_trailing(pos) {
            for comment in comment_list {
                let inline_comment = format!(" /* {} */", comment.text);
                self.collected_comments
                    .push((property.clone(), inline_comment));
            }
        }
        decl.value.visit_with(self);
    }

    fn visit_component_value(&mut self, node: &ComponentValue) {
        let pos = node.span().hi();
        if let Some(trailing_comments) = self.comments.get_trailing(pos) {
            for comment in trailing_comments {
                let inline_comment = format!(" /* {} */", comment.text);
                self.collected_comments
                    .push(("".to_string(), inline_comment));
            }
        }
    }

    fn visit_pseudo_element_selector(&mut self, node: &PseudoElementSelector) {
        let pos = node.span().hi();
        let pos_lo = node.span().lo();

        if let Some(comment_list) = self.comments.get_leading(pos_lo) {
            for comment in comment_list {
                let inline_comment = format!(" /* {} */", comment.text);
                self.collected_comments
                    .push((format!("::{}", node.name.value), inline_comment));
            }
        }

        if let Some(comment_list) = self.comments.get_trailing(pos) {
            for comment in comment_list {
                let inline_comment = format!(" /* {} */", comment.text);
                self.collected_comments
                    .push((format!("::{}", node.name.value), inline_comment));
            }
        }
        node.visit_children_with(self);
    }

    fn visit_pseudo_class_selector(&mut self, node: &PseudoClassSelector) {
        let pos = node.span().hi();
        if let Some(comment_list) = self.comments.get_trailing(pos) {
            for comment in comment_list {
                let inline_comment = format!(" /* {} */", comment.text);
                self.collected_comments
                    .push((format!(":{}", node.name.value), inline_comment));
            }
        }
        node.visit_children_with(self);
    }
}

fn generate_css_with_comments(stylesheet: &Stylesheet, comments: &dyn Comments) -> String {
    let mut output = String::new();
    let mut formatted_output = String::new();

    {
        let mut writer = BasicCssWriter::new(&mut output, None, Default::default());
        let mut _generator = CodeGenerator::new(&mut writer, CodegenConfig { minify: false });

        for rule in &stylesheet.rules {
            if let Some(comment_list) = comments.get_leading(rule.span().lo()) {
                for comment in comment_list {
                    formatted_output.push_str(&format!("{}\n", comment.text));
                }
            }

            let mut comment_collector = InlineCommentCollector {
                comments,
                collected_comments: Vec::new(),
            };
            rule.visit_with(&mut comment_collector);

            let mut rule_output = String::new();
            {
                let mut rule_writer =
                    BasicCssWriter::new(&mut rule_output, None, Default::default());
                let mut rule_gen =
                    CodeGenerator::new(&mut rule_writer, CodegenConfig { minify: false });
                rule_gen.emit(rule).expect("Failed to generate CSS");
            }

            let mut rule_lines: Vec<String> = rule_output.lines().map(String::from).collect();
            for (prop, inline_comment) in comment_collector.collected_comments.iter().rev() {
                if prop.starts_with("::") || prop.starts_with(":") {
                    if let Some(idx) = rule_lines.iter().position(|line| line.contains(prop)) {
                        if !rule_lines[idx].contains(inline_comment.trim()) {
                            rule_lines[idx].push_str(inline_comment);
                        }
                    }
                } else {
                    if let Some(idx) = rule_lines
                        .iter()
                        .position(|line| line.contains(&format!("{}:", prop)))
                    {
                        if !rule_lines[idx].contains(inline_comment.trim()) {
                            rule_lines[idx].push_str(inline_comment);
                        }
                    }
                }
            }
            formatted_output.push_str(&rule_lines.join("\n"));
            formatted_output.push('\n');

            if let Some(comment_list) = comments.get_trailing(rule.span().hi()) {
                for comment in comment_list {
                    formatted_output.push_str(&format!("{}\n", comment.text));
                }
            }
        }
    }

    formatted_output.trim().to_string()
}

pub fn remove_import_from_ast() {}

pub fn statistics_from_ast() {}

pub fn contains_class_from_ast() {}

pub fn contains_id_from_ast() {}

pub fn extend_class_to_ast() {}

pub fn extend_id_to_ast() {}

pub fn remove_class_from_ast() {}

pub fn remove_id_from_ast() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_is_imported_from_ast() {
        let css_code = r#"
                @import "reset.css";
                @import url("theme.css");
                body { color: black; }
            "#;

        assert_eq!(
            is_imported_from_ast(css_code, "reset.css").is_ok(),
            true,
            "reset.css should be imported"
        );
        assert_eq!(
            is_imported_from_ast(css_code, "theme.css").is_ok(),
            true,
            "theme.css should be imported"
        );
        assert_eq!(
            is_imported_from_ast(css_code, "main.css").is_err(),
            false,
            "main.css should not be imported"
        );

        let css_code = r#"
            @import "tailwindcss/base";
            @import "tailwindcss/components";
            @import "tailwindcss/utilities";
            "#;

        assert_eq!(
            is_imported_from_ast(css_code, "tailwindcss/base").is_ok(),
            true,
            "tailwindcss/base should be imported"
        );
        assert_eq!(
            is_imported_from_ast(css_code, "tailwindcss/components").is_ok(),
            true,
            "tailwindcss/components should be imported"
        );
        assert_eq!(
            is_imported_from_ast(css_code, "tailwindcss/utilities").is_ok(),
            true,
            "tailwindcss/utilities should be imported"
        );
    }

    #[test]
    fn test_css_insert_import_to_ast() {
        let css_code = r#"
            @import "reset.css";
            /* This file is for your main application CSS */

            body { color: black; }
            /* Another comment */

            .hide-scrollbar{
                -ms-overflow-style: none;  /* Internet Explorer 10+ */
                scrollbar-width: none;  /* Firefox */
            }
            /* Another last comment */
            .hide-scrollbar::-webkit-scrollbar {
                display: none;  /* Safari and Chrome */
            }
        "#;

        let new_imports = r#"
            @import "theme.css";
            @import "theme.css";
            @import "reset.css";
            @import "reset.css";
            @import "custom.css";
        "#;

        let result = insert_import_to_ast(css_code, new_imports).unwrap();
        println!("{}", result);
    }

    #[test]
    fn test_css_remove_import_from_ast() {}

    #[test]
    fn test_css_statistics_from_ast() {}

    #[test]
    fn test_css_contains_class_from_ast() {}

    #[test]
    fn test_css_contains_id_from_ast() {}

    #[test]
    fn test_css_extend_class_to_ast() {}

    #[test]
    fn test_css_extend_id_to_ast() {}

    #[test]
    fn test_css_remove_class_from_ast() {}

    #[test]
    fn test_css_remove_id_from_ast() {}
}
