use crate::parsers::css::helpers::*;
use swc_css_ast::*;
use swc_css_visit::{Visit, VisitWith};

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
    let (stylesheet, _cm) = match parse(file_content) {
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

pub fn insert_import_to_ast() {}

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
    fn test_css_insert_import_to_ast() {}

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
