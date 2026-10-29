use auto_uml::{
    diagram::{Class, Diagram},
    graphviz,
    lang_config::LangConfig,
    mermaid,
    stitcher::{Directory, Stitcher},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tree_sitter::Parser;

fn parser(language: &str) -> Parser {
    let mut parser = Parser::new();
    let grammar = match language {
        "rust" => tree_sitter_rust::LANGUAGE,
        "kotlin" => tree_sitter_kotlin::LANGUAGE,
        "javascript" => tree_sitter_javascript::LANGUAGE,
        "dart" => tree_sitter_dart::LANGUAGE,
        "cpp" => tree_sitter_cpp::LANGUAGE,
        "java" => tree_sitter_java::LANGUAGE,
        _ => unreachable!(),
    };
    parser.set_language(&grammar.into()).unwrap();
    parser
}

fn free_functions(diagram: &Diagram) -> Vec<String> {
    diagram
        .files
        .iter()
        .flat_map(|file| &file.functions)
        .map(|function| function.render(diagram.lang()))
        .collect()
}

fn methods(diagram: &Diagram) -> Vec<String> {
    diagram
        .classes
        .iter()
        .flat_map(|class| {
            class
                .functions
                .iter()
                .map(|function| format!("{}::{}", class.name, function.render(diagram.lang())))
        })
        .collect()
}

fn resolved_directory<'a>(root: &Path, config: &'a LangConfig, no_ignore: bool) -> Directory<'a> {
    let mut stitcher = Stitcher::new(root.to_path_buf(), config, parser("rust"), no_ignore);
    let mut directory = stitcher.build();
    directory.merge_all();
    directory.resolve_types(&stitcher.type_map);
    assert_eq!(stitcher.type_map.types.len(), 1);
    directory
}

#[test]
fn rust_namespaces_methods_and_callable_boundaries() {
    let config = LangConfig::load("rust");
    let mut diagram = Diagram::new(&config);
    diagram.build(
        br#"
        struct User {}
        impl User { fn method() { fn local_method() {} } }
        fn top() -> User { fn nested() {} User {} }
        mod auth { pub fn login(user: User) -> Vec<User> { vec![] }
            mod inner { fn helper() {} }
        }
        const CALLBACK: fn() = || { fn hidden() {} };
        trait Service { fn default_method() {} }
        "#,
        &mut parser("rust"),
        Some("api.rs".into()),
    );
    assert_eq!(
        free_functions(&diagram),
        [
            "top() User",
            "auth::login(user: User) Vec~User~",
            "auth_inner::helper() void"
        ]
    );
    assert_eq!(methods(&diagram), ["User::method() void"]);
    let mermaid = mermaid::generate(&diagram);
    assert!(mermaid.contains("class auto_uml_file_0[\"api.rs\"]"));
    assert!(mermaid.contains("<<file>>"));
    assert!(mermaid.contains("+auth::login(user: User) Vec~User~"));
    assert_eq!(mermaid.matches("auto_uml_file_0 ..> User").count(), 2);
    let dot = graphviz::generate(&diagram);
    assert!(dot.contains("auth::login(user: User) Vec\\<User\\>"));
    assert_eq!(dot.matches("\"auto_uml_file_0\" -> \"User\"").count(), 2);
}

#[test]
fn kotlin_package_overloads_and_anonymous_callables() {
    let config = LangConfig::load("kotlin");
    let mut diagram = Diagram::new(&config);
    diagram.build(
        br#"
        package demo.api
        class User { fun method() { fun localMethod() {} } }
        fun load(): User { fun nested() {} ; return User() }
        fun load(id: Int): User = User()
        val callback = { fun lambdaLocal() {} }
        val anonymous = fun() { fun anonymousLocal() {} }
        "#,
        &mut parser("kotlin"),
        None,
    );
    assert_eq!(
        free_functions(&diagram),
        ["demo.api::load() User", "demo.api::load(id: Int) User"]
    );
    assert_eq!(methods(&diagram), ["User::method() void"]);
    for output in [mermaid::generate(&diagram), graphviz::generate(&diagram)] {
        assert!(output.contains("(unnamed source)"));
        assert_eq!(output.matches("demo.api::load(").count(), 2);
    }
}

#[test]
fn javascript_top_level_anonymous_initializers_are_boundaries() {
    let config = LangConfig::load("javascript");
    let mut diagram = Diagram::new(&config);
    diagram.build(
        br#"
        function top() { function local() {} }
        const arrow = () => { function arrowLocal() {} };
        const expr = function() { function expressionLocal() {} };
        const object = { objectMethod() { function objectLocal() {} } };
        class Owner { method() { function methodLocal() {} } }
        "#,
        &mut parser("javascript"),
        None,
    );
    assert_eq!(free_functions(&diagram), ["top()"]);
    assert_eq!(methods(&diagram), ["Owner::method()"]);
}

#[test]
fn local_class_and_impl_methods_reset_callable_scope() {
    for (language, source, expected_methods, expected_free) in [
        (
            "java",
            "class Outer { void run() { class Local { void work() {} } } }",
            vec!["Outer::run() void", "Local::work() void"],
            vec![],
        ),
        (
            "rust",
            "fn outer() { struct Local {} impl Local { fn work() { fn nested() {} } } }",
            vec!["Local::work() void"],
            vec!["outer() void"],
        ),
        (
            "rust",
            "struct Local {} fn outer() { impl Local { fn work() {} } }",
            vec!["Local::work() void"],
            vec!["outer() void"],
        ),
    ] {
        let config = LangConfig::load(language);
        let mut diagram = Diagram::new(&config);
        diagram.build(source.as_bytes(), &mut parser(language), None);
        assert_eq!(methods(&diagram), expected_methods, "{language}");
        assert_eq!(free_functions(&diagram), expected_free, "{language}");
    }
}

#[test]
fn broad_declarations_and_sibling_function_bodies() {
    for (language, source) in [
        ("cpp", "int global; int top(int input) { return input; }"),
        (
            "dart",
            "int global = 0; int top(int input) { int local() { return 0; } return input; }",
        ),
    ] {
        let config = LangConfig::load(language);
        let mut diagram = Diagram::new(&config);
        diagram.build(source.as_bytes(), &mut parser(language), None);
        assert_eq!(
            free_functions(&diagram),
            ["top(input: int) int"],
            "{language}"
        );
    }
}

#[test]
fn file_ids_labels_empty_units_and_repeated_builds() {
    let config = LangConfig::load("rust");
    let mut diagram = Diagram::new(&config);
    diagram.classes.push(Class::new("auto_uml_file_0".into()));
    let mut parser = parser("rust");
    diagram.build(b"fn a() {}", &mut parser, Some("a/shared.rs".into()));
    diagram.build(b"fn b() {}", &mut parser, Some("b/shared.rs".into()));
    diagram.build(
        b"fn c() {}",
        &mut parser,
        Some("odd/quote\"[x]{y}|<z>&#~.rs".into()),
    );
    diagram.build(b"struct Empty {}", &mut parser, Some("empty.rs".into()));
    let mmd = mermaid::generate(&diagram);
    assert!(mmd.contains("auto_uml_file_1[\"a/shared.rs\"]"));
    assert!(mmd.contains("auto_uml_file_2[\"b/shared.rs\"]"));
    assert!(mmd.contains("quote#34;#91;x#93;#123;y#125;|#60;z#62;#38;#35;~.rs"));
    assert!(!mmd.contains("empty.rs"));
    let dot = graphviz::generate(&diagram);
    assert!(dot.contains("quote\\\"[x]\\{y\\}\\|\\<z\\>&#~.rs"));
    assert!(!dot.contains("empty.rs"));
    assert_eq!(dot.matches("\\<\\<file\\>\\>").count(), 3);
    assert_eq!(mermaid::generate(&diagram), mmd);
}

struct Fixture(PathBuf);
impl Fixture {
    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn directory_merge_resolution_paths_and_ignore_rules() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("free-function-test-{}", std::process::id()));
    let fixture = Fixture(root.clone());
    fixture.write(".gitignore", "ignored/\n");
    for dir in ["a", "b"] {
        fixture.write(
            &format!("{dir}/types.rs"),
            "mod domain { pub struct User {} }",
        );
        fixture.write(
            &format!("{dir}/shared.rs"),
            "mod domain { fn load(user: User, users: Vec<User>) -> Vec<User> { vec![] } }",
        );
    }
    fixture.write("ignored/skip.rs", "fn hidden() {}");
    let config = LangConfig::load("rust");
    for no_ignore in [false, true] {
        let directory = resolved_directory(&root, &config, no_ignore);
        let diagram = &directory.merged_diagram;
        let mut summaries = free_functions(diagram);
        summaries.sort();
        let mut expected = vec![
            "domain::load(user: domain_a_User, users: Vec~domain_a_User~) Vec~domain_a_User~",
            "domain::load(user: domain_b_User, users: Vec~domain_b_User~) Vec~domain_b_User~",
        ];
        if no_ignore {
            expected.push("hidden() void");
        }
        assert_eq!(summaries, expected);
        let output = mermaid::generate(diagram);
        assert!(output.contains("a/shared.rs"));
        assert!(output.contains("b/shared.rs"));
        assert!(output.contains(" ..> domain_a_User"));
        assert!(output.contains(" ..> domain_b_User"));
        assert!(!output.contains(" --> "));
        assert_eq!(output.contains("ignored/skip.rs"), no_ignore);
    }
}
