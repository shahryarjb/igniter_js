// Based on:
//
// Tasks:
// https://github.com/ash-project/igniter/issues/106
// https://github.com/mishka-group/mishka_chelekom/issues/181
//
// My Questions:
// https://users.rust-lang.org/t/122689/
// https://users.rust-lang.org/t/122507/
// https://users.rust-lang.org/t/122153/
// https://users.rust-lang.org/t/121861/

use oxc::{
    allocator::{Allocator, Box as OXCBox, Vec as OXCVec},
    ast::ast::{
        Argument, Expression, IdentifierName, IdentifierReference, NewExpression, ObjectExpression,
        ObjectProperty, ObjectPropertyKind, Program, PropertyKey, PropertyKind, Statement,
        VariableDeclarator,
    },
    codegen::Codegen,
    parser::{ParseOptions, Parser},
    span::{Atom, SourceType, Span},
};

use std::cell::Cell;

// TODO: Done
pub fn source_to_ast<'a>(
    file_content: &'a str,
    allocator: &'a Allocator,
) -> Result<Program<'a>, String> {
    let source_type = SourceType::default();
    let parser = Parser::new(allocator, file_content, source_type).with_options(ParseOptions {
        parse_regular_expression: true,
        ..ParseOptions::default()
    });

    let parse_result = parser.parse();
    Ok(parse_result.program)
}

// TODO: Done
pub fn is_module_imported_from_ast<'a>(
    file_content: &str,
    module_name: &str,
    allocator: &Allocator,
) -> Result<bool, String> {
    let program = source_to_ast(file_content, allocator)?;

    for node in program.body {
        if let Statement::ImportDeclaration(import_decl) = node {
            if import_decl.source.value == module_name {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

// TODO: Done
pub fn insert_import_to_ast<'a>(
    file_content: &str,
    import_lines: &str,
    allocator: &Allocator,
) -> Result<String, String> {
    let mut program = source_to_ast(file_content, allocator)?;

    for import_line in import_lines.lines() {
        let import_source_type = SourceType::default();
        let parser =
            Parser::new(allocator, import_line, import_source_type).with_options(ParseOptions {
                parse_regular_expression: true,
                ..ParseOptions::default()
            });

        let parsed_result = parser.parse();
        if let Some(errors) = parsed_result.errors.first() {
            return Err(format!("Failed to parse import line: {:?}", errors));
        }

        let new_import = parsed_result
            .program
            .body
            .into_iter()
            .find(|node| matches!(node, Statement::ImportDeclaration(_)))
            .ok_or_else(|| "No import declaration found in parsed import line".to_string())?;

        if program.body.iter().any(|node| {
            matches!(
                (node, &new_import),
                (
                    Statement::ImportDeclaration(existing_import),
                    Statement::ImportDeclaration(new_import_node)
                ) if existing_import.source.value == new_import_node.source.value
            )
        }) {
            continue; // Skip duplicate imports
        }

        let position = program
            .body
            .iter()
            .rposition(|node| matches!(node, Statement::ImportDeclaration(_)))
            .map(|index| index + 1)
            .unwrap_or(0);

        program.body.insert(position, new_import);
    }

    let codegen = Codegen::new();
    let generated_code = codegen.build(&program).code;

    Ok(generated_code)
}

// TODO: Done
pub fn remove_import_from_ast<'a>(
    file_content: &str,
    modules: impl IntoIterator<Item = impl AsRef<str>>,
    allocator: &'a Allocator,
) -> Result<String, String> {
    // Parse the source file into AST
    let mut program = source_to_ast(file_content, allocator)?;

    // Find and remove the specified import declaration
    let modules: Vec<String> = modules
        .into_iter()
        .map(|n| n.as_ref().to_string())
        .collect();

    program.body.retain(|node| {
        if let Statement::ImportDeclaration(import_decl) = node {
            let source_value = import_decl.source.value.to_string();
            !modules.contains(&source_value)
        } else {
            true
        }
    });

    let codegen = Codegen::new();
    let generated_code = codegen.build(&program).code;
    Ok(generated_code)
}

// TODO: Done
pub fn find_live_socket_node_from_ast<'a>(program: &'a Program<'a>) -> Result<bool, bool> {
    if program.body.iter().any(|node| {
        if let Statement::VariableDeclaration(var_decl) = node {
            var_decl.declarations.iter().any(|decl| {
                decl.id
                    .get_identifier()
                    .map_or(false, |ident| ident == "liveSocket")
            })
        } else {
            false
        }
    }) {
        Ok(true)
    } else {
        Err(false)
    }
}

// These are different ways
// names: impl IntoIterator<Item = &'a (impl AsRef<str> + 'a + ?Sized)>,
// names: impl IntoIterator<Item = impl AsRef<str>>,
// under program: let names2: Vec<String>;
// let new_property = create_and_import_object_into_hook(name, allocator);
// obj_expr.properties.push(new_property);

// use the outer one
// names2 = names.into_iter().map(|n| n.as_ref().to_string()).collect();
// for name in &names2 {
//     let new_property = create_and_import_object_into_hook(name, allocator);
//     obj_expr.properties.push(new_property);
// }

// TODO: Done
pub fn extend_hook_object_to_ast<'a>(
    file_content: &str,
    names: impl IntoIterator<Item = &'a str>,
    allocator: &Allocator,
) -> Result<String, String> {
    let mut program = source_to_ast(file_content, allocator)?;

    if find_live_socket_node_from_ast(&program).is_err() {
        return Err("liveSocket not found.".to_string());
    }

    let maybe_properties = get_properties(&mut program.body);
    if let Some(properties) = maybe_properties {
        let hooks_property = match find_hooks_property(properties) {
            Some(prop) => prop,
            None => {
                let new_hooks_property = create_init_hooks(allocator);
                properties.push(new_hooks_property);
                get_property_by_key(properties.last_mut().unwrap(), "hooks").unwrap()
            }
        };

        if let Expression::ObjectExpression(obj_expr) = hooks_property {
            for name in names {
                let new_property = create_and_import_object_into_hook(name.as_ref(), allocator);
                obj_expr.properties.push(new_property)
            }
        }
    } else {
        return Err("properties not found in the AST.".to_string());
    }

    let codegen = Codegen::new();
    let generated_code = codegen.build(&program).code;
    Ok(generated_code)
}

// TODO: Done
pub fn remove_objects_of_hooks_from_ast(
    file_content: &str,
    object_names: impl IntoIterator<Item = impl AsRef<str>>,
    allocator: &Allocator,
) -> Result<String, String> {
    let mut program = source_to_ast(file_content, allocator)?;

    if find_live_socket_node_from_ast(&program).is_err() {
        return Err("liveSocket not found.".to_string());
    }

    let maybe_properties = get_properties(&mut program.body);
    if let Some(properties) = maybe_properties {
        let hooks_property = match find_hooks_property(properties) {
            Some(prop) => prop,
            None => {
                let new_hooks_property = create_init_hooks(allocator);

                properties.push(new_hooks_property);
                get_property_by_key(properties.last_mut().unwrap(), "hooks").unwrap()
            }
        };

        if let Expression::ObjectExpression(hooks_obj) = hooks_property {
            let object_names: Vec<String> = object_names
                .into_iter()
                .map(|n| n.as_ref().to_string())
                .collect();
            hooks_obj.properties.retain(|property| {
                match property {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        !matches!(&prop.key, PropertyKey::StaticIdentifier(key) if object_names.iter().any(|name| name == key.name.as_str()))
                    }
                    _ => true,
                }
            });
        }
    } else {
        return Err("properties not found in the AST.".to_string());
    }

    let codegen = Codegen::new();
    let generated_code = codegen.build(&program).code;
    Ok(generated_code)
}

fn get_properties<'short, 'long>(
    body: &'short mut OXCVec<'long, Statement<'long>>,
) -> Option<&'short mut OXCVec<'long, ObjectPropertyKind<'long>>> {
    body.iter_mut().find_map(|node| match node {
        Statement::VariableDeclaration(var_decl) => {
            var_decl.declarations.iter_mut().find_map(|decl| {
                let obj_expr = get_new_expression(decl)?;
                obj_expr.arguments.iter_mut().find_map(|arg| {
                    let obj_expr_inner = get_object_expression(arg)?;
                    Some(&mut obj_expr_inner.properties)
                })
            })
        }
        _ => None,
    })
}

fn find_hooks_property<'short, 'long>(
    properties: &'short mut OXCVec<'long, ObjectPropertyKind<'long>>,
) -> Option<&'short mut Expression<'long>> {
    properties
        .iter_mut()
        .find_map(|prop| get_property_by_key(prop, "hooks"))
}

fn create_and_import_object_into_hook<'a>(
    name: &'a str,
    allocator: &Allocator,
) -> ObjectPropertyKind<'a> {
    ObjectPropertyKind::ObjectProperty(OXCBox::new_in(
        ObjectProperty {
            span: Span::default(),
            kind: PropertyKind::Init,
            key: PropertyKey::StaticIdentifier(OXCBox::new_in(
                IdentifierName {
                    span: Span::default(),
                    name: Atom::from(name),
                },
                allocator,
            )),
            value: Expression::Identifier(OXCBox::new_in(
                IdentifierReference {
                    span: Span::default(),
                    name: Atom::from(name),
                    reference_id: Cell::new(None),
                },
                allocator,
            )),
            method: false,
            shorthand: true,
            computed: false,
        },
        allocator,
    ))
}

fn create_init_hooks(allocator: &Allocator) -> ObjectPropertyKind {
    ObjectPropertyKind::ObjectProperty(OXCBox::new_in(
        ObjectProperty {
            span: Span::default(),
            kind: PropertyKind::Init,
            key: PropertyKey::StaticIdentifier(OXCBox::new_in(
                IdentifierName {
                    span: Span::default(),
                    name: Atom::from("hooks"),
                },
                allocator,
            )),
            value: Expression::ObjectExpression(OXCBox::new_in(
                ObjectExpression {
                    span: Span::default(),
                    properties: OXCVec::new_in(allocator),
                    trailing_comma: None,
                },
                allocator,
            )),
            method: false,
            shorthand: false,
            computed: false,
        },
        allocator,
    ))
}

fn get_new_expression<'short, 'long>(
    decl: &'short mut VariableDeclarator<'long>,
) -> Option<&'short mut NewExpression<'long>> {
    match decl.init.as_mut()? {
        Expression::NewExpression(expr) => Some(expr),
        _ => None,
    }
}

fn get_object_expression<'short, 'long>(
    arg: &'short mut Argument<'long>,
) -> Option<&'short mut ObjectExpression<'long>> {
    arg.as_expression_mut().and_then(|expr| match expr {
        Expression::ObjectExpression(boxed_obj_expr) => Some(boxed_obj_expr.as_mut()),
        _ => None,
    })
}

fn get_property_by_key<'short, 'long>(
    property: &'short mut ObjectPropertyKind<'long>,
    key_name: &str,
) -> Option<&'short mut Expression<'long>> {
    match property {
        ObjectPropertyKind::ObjectProperty(prop) => match &prop.key {
            PropertyKey::StaticIdentifier(key) if key.as_ref().name == key_name => {
                Some(&mut prop.value)
            }
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::path::Path;

    fn create_allocator<'a>() -> &'a Allocator {
        let allocator = Box::new(Allocator::default());
        Box::leak(allocator)
    }

    #[test]
    fn test_parse_and_display_ast() {
        let js_content = r#"
               import { foo } from 'module-name';
               import bar from 'another-module';

               console.log('Testing AST parsing');
           "#;

        // Test the AST parsing
        let allocator = create_allocator();

        match source_to_ast(js_content, allocator) {
            Ok(ast) => {
                println!("{:#?}", ast.body);
                assert!(!ast.body.is_empty(), "AST body should not be empty");
            }
            Err(e) => panic!("Error while parsing AST: {}", e),
        }
    }

    #[test]
    fn test_is_module_imported_from_ast() {
        // Write a test JavaScript file
        let js_content = r#"
               import { foo } from 'module-name';
               import bar from 'another-module';
           "#;

        // Test the function with a valid module
        let allocator = create_allocator();
        match is_module_imported_from_ast(js_content, "module-name", allocator) {
            Ok(true) => println!("Module 'module-name' is imported as expected."),
            Ok(false) => panic!("Module 'module-name' should be imported but was not detected."),
            Err(e) => panic!("Error while checking module: {}", e),
        }

        // Test the function with another valid module
        match is_module_imported_from_ast(js_content, "another-module", allocator) {
            Ok(true) => println!("Module 'another-module' is imported as expected."),
            Ok(false) => panic!("Module 'another-module' should be imported but was not detected."),
            Err(e) => panic!("Error while checking module: {}", e),
        }

        // Test the function with a non-existent module
        match is_module_imported_from_ast(js_content, "non-existent-module", allocator) {
            Ok(true) => panic!("Module 'non-existent-module' should not be imported."),
            Ok(false) => println!("Module 'non-existent-module' is correctly not imported."),
            Err(e) => panic!("Error while checking module: {}", e),
        }
    }

    #[test]
    fn test_insert_duplicate_import() {
        let js_content = r#"
            import { foo } from "module-name";
            console.log("Duplicate import test");
        "#;

        let duplicate_import = r#"import { foo } from "module-name";"#;
        let allocator = create_allocator();
        let result = insert_import_to_ast(js_content, duplicate_import, allocator);

        match result {
            Ok(updated_content) => {
                println!("Updated Content:\n{}", updated_content);
                // Ensure the duplicate import is not added
                let import_count = updated_content.matches(duplicate_import).count();
                assert_eq!(
                    import_count, 1,
                    "Duplicate import should not be added, but it was found multiple times"
                );
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_insert_import_to_ast_with_existing_imports() {
        let js_content = r#"
            import bar from "another-module";
            console.log("Some imports here!");
        "#;

        let new_import = r#"import { foo } from "module-name";"#;
        let allocator = create_allocator();
        let result = insert_import_to_ast(js_content, new_import, allocator);

        match result {
            Ok(updated_content) => {
                println!(
                    "Updated Content::test_insert_import_to_ast_with_existing_imports:::\n{}",
                    updated_content
                );
                let lines: Vec<&str> = updated_content.lines().collect();
                let last_import_position =
                    lines.iter().rposition(|&line| line.starts_with("import"));
                assert_eq!(
                    lines[last_import_position.unwrap() + 1],
                    "console.log(\"Some imports here!\");",
                    "New import should be added after the last import"
                );
            }
            Err(e) => panic!("Error while inserting import: {}", e),
        }
    }

    #[test]
    fn test_insert_multiple_imports() {
        let js_content = r#"
            console.log("Starting with no imports!");
        "#;

        let imports = vec![
            r#"import { foo } from "module-one";"#,
            r#"import bar from "module-two";"#,
            r#"import * as namespace from "module-three";"#,
            r#"import something, { foo as bar } from "module-four";"#,
        ];

        let allocator = create_allocator();
        for import in &imports {
            let result = insert_import_to_ast(js_content, import, allocator);
            match result {
                Ok(updated_content) => {
                    println!(
                        "Updated Content::test_insert_multiple_imports::\n{}",
                        updated_content
                    );

                    assert!(
                        updated_content.contains(import),
                        "Import not added: {}",
                        import
                    );
                }
                Err(e) => panic!("Error while inserting import '{}': {}", import, e),
            }
        }
    }

    #[test]
    fn test_insert_import_to_ast_with_alert_only() {
        // Write a test JavaScript file with only an alert
        let js_content = r#"
            alert('Hello, world!');
        "#;

        // Insert a new import
        let new_import = r#"import { foo } from "module-name";"#;
        let allocator = create_allocator();
        let result = insert_import_to_ast(js_content, new_import, allocator);

        match result {
            Ok(updated_content) => {
                println!("Updated Content:\n{}", updated_content);
                assert!(updated_content.contains(new_import), "New import not added");
                assert!(
                    updated_content.starts_with(new_import),
                    "New import should be at the top"
                );
            }
            Err(e) => panic!("Error while inserting import: {}", e),
        }
    }

    #[test]
    fn test_remove_import_from_ast() {
        // Write a test JavaScript file
        let js_content = r#"
                import { foo } from "module-name";
                import bar from "another-module";

                console.log("Testing remove import");
            "#;

        // Test the function to remove an existing module
        let allocator = create_allocator();
        let module_names = vec!["module-name"];
        match remove_import_from_ast(js_content, module_names, allocator) {
            Ok(updated_code) => {
                let expected_snippet = "module-name";

                assert!(
                    !updated_code.contains(expected_snippet),
                    "The updated code is missing expected content: '{}'",
                    expected_snippet
                );
            }
            Err(e) => panic!("Error while removing import: {}", e),
        }
    }

    #[test]
    fn test_find_live_socket_variable() {
        // Set up a test JavaScript file
        let js_content = r#"
            const someVar = 42;
            let liveSocket = new LiveSocket("/live", Socket, {
              hooks: { ...Hooks, CopyMixInstallationHook },
              longPollFallbackMs: 2500,
              params: { _csrf_token: csrfToken },
            });
            const anotherVar = "hello";
        "#;

        let allocator = create_allocator();
        let program = source_to_ast(js_content, allocator).expect("Failed to parse AST");

        // Test the function
        let result = find_live_socket_node_from_ast(&program);
        println!("Result for test_find_live_socket_variable: {:?}", result);

        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_find_live_socket_variable_not_found() {
        // Set up a test JavaScript file
        let js_content = r#"
            const someVar = 42;
            const anotherVar = "hello";
        "#;

        let allocator = create_allocator();
        let program = source_to_ast(js_content, allocator).expect("Failed to parse AST");

        // Test the function
        let result = find_live_socket_node_from_ast(&program);
        println!(
            "Result for test_find_live_socket_variable_not_found: {:?}",
            result
        );

        assert_eq!(result, Err(false));
    }

    #[test]
    fn test_new_extend_hook_object_to_ast() {
        let js_content = r#"
            let liveSocket = new LiveSocket("/live", Socket, {
              hooks: { ...Hooks, CopyMixInstallationHook },
              longPollFallbackMs: 2500,
              params: { _csrf_token: csrfToken },
            });
        "#;

        let allocator = create_allocator();
        let object_names = vec!["OXCTestHook", "MishkaHooks"];
        match extend_hook_object_to_ast(js_content, object_names, allocator) {
            Ok(ast) => {
                println!("Hook object extended successfully. ==> {}", ast);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("Failed to extend hook object.");
            }
        }
    }

    #[test]
    fn test_remove_an_object_from_ast() {
        let js_content = r#"
            let liveSocket = new LiveSocket("/live", Socket, {
              hooks: { ...Hooks, CopyMixInstallationHook },
              longPollFallbackMs: 2500,
              params: { _csrf_token: csrfToken },
            });
        "#;

        let allocator = create_allocator();
        let object_names = vec!["CopyMixInstallationHook"];

        let expected_snippet = "hooks: { ...Hooks }";

        match remove_objects_of_hooks_from_ast(js_content, object_names, &allocator) {
            Ok(updated_code) => {
                println!("Updated Code:\n{}", updated_code);

                assert!(
                    updated_code.contains(expected_snippet),
                    "The updated code is missing expected content: '{}'",
                    expected_snippet
                );
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
