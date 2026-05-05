use super::{
    Path, compute_file_hash, extract_pattern_symbols, is_high_noise_file, is_ignorable_path,
    is_source_code, to_pascal_case, verify_file_stable,
};

#[test]
fn test_is_source_code() {
    assert!(is_source_code(Path::new("src/lib.rs")));
    assert!(is_source_code(Path::new("app/main.py")));
    assert!(is_source_code(Path::new("ui/index.ts")));
    assert!(is_source_code(Path::new("web/app.js")));
    assert!(!is_source_code(Path::new("docs/README.md")));
    assert!(!is_source_code(Path::new("config.toml")));
}

#[test]
fn test_is_ignorable_path() {
    assert!(is_ignorable_path(Path::new(".git/config")));
    assert!(is_ignorable_path(Path::new("target/debug/app")));
    assert!(!is_ignorable_path(Path::new("src/lib.rs")));
}

#[test]
fn test_is_high_noise_file() {
    assert!(is_high_noise_file(Path::new("src/mod.rs")));
    assert!(is_high_noise_file(Path::new("src/lib.rs")));
    assert!(is_high_noise_file(Path::new("bin/main.rs")));
    assert!(is_high_noise_file(Path::new("prelude.rs")));
    assert!(is_high_noise_file(Path::new("types.rs")));
    assert!(is_high_noise_file(Path::new("error.rs")));
    assert!(is_high_noise_file(Path::new("utils.rs")));
    assert!(!is_high_noise_file(Path::new("src/parser.rs")));
    assert!(!is_high_noise_file(Path::new("src/sentinel.rs")));
    assert!(!is_high_noise_file(Path::new("app/models/user.rs")));
}

#[test]
fn test_extract_pattern_symbols_function() {
    let symbols = extract_pattern_symbols("fn process_data($$$)");
    assert_eq!(symbols, vec!["process_data"]);

    let symbols = extract_pattern_symbols("async fn fetch_user(id: u32) -> Result<User, Error>");
    assert!(symbols.contains(&"fetch_user".to_string()));
}

#[test]
fn test_extract_pattern_symbols_struct() {
    let symbols = extract_pattern_symbols("struct User { $$$ }");
    assert_eq!(symbols, vec!["User"]);

    let symbols = extract_pattern_symbols("struct HttpRequest { method: String, path: String }");
    assert!(symbols.contains(&"HttpRequest".to_string()));
}

#[test]
fn test_extract_pattern_symbols_class() {
    let symbols = extract_pattern_symbols("class UserProfile { $$$ }");
    assert_eq!(symbols, vec!["UserProfile"]);
}

#[test]
fn test_extract_pattern_symbols_enum() {
    let symbols = extract_pattern_symbols("enum Status { $$$ }");
    assert_eq!(symbols, vec!["Status"]);
}

#[test]
fn test_extract_pattern_symbols_trait() {
    let symbols = extract_pattern_symbols("trait Handler { $$$ }");
    assert_eq!(symbols, vec!["Handler"]);
}

#[test]
fn test_extract_pattern_symbols_impl() {
    let symbols = extract_pattern_symbols("impl User { $$$ }");
    assert!(symbols.contains(&"User".to_string()));

    let symbols = extract_pattern_symbols("impl Display for User { $$$ }");
    assert!(symbols.contains(&"User".to_string()));
}

#[test]
fn test_extract_pattern_symbols_multiple() {
    let symbols = extract_pattern_symbols("fn create_user() -> User { $$$ }");
    assert!(symbols.contains(&"create_user".to_string()));

    let symbols = extract_pattern_symbols("struct User { } fn create_user() { $$$ }");
    assert!(symbols.contains(&"User".to_string()));
    assert!(symbols.contains(&"create_user".to_string()));
}

#[test]
fn test_extract_pattern_symbols_empty() {
    let symbols = extract_pattern_symbols("$$$");
    assert!(symbols.is_empty());

    let symbols = extract_pattern_symbols("// just a comment");
    assert!(symbols.is_empty());
}

#[test]
fn test_verify_file_stable_with_temp_file() {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("xiuxian_test_stable.rs");

    let Ok(mut file) = std::fs::File::create(&temp_path) else {
        panic!("failed to create temporary sentinel stability file");
    };
    assert!(file.write_all(b"fn main() {}").is_ok());
    drop(file);

    assert!(verify_file_stable(&temp_path));
    std::fs::remove_file(&temp_path).ok();
}

#[test]
fn test_verify_file_stable_nonexistent() {
    assert!(!verify_file_stable(Path::new("/nonexistent/file.rs")));
}

#[test]
fn test_compute_file_hash_with_temp_file() {
    use std::io::Write;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("xiuxian_test_hash.txt");

    let Ok(mut file) = std::fs::File::create(&temp_path) else {
        panic!("failed to create temporary sentinel hash file");
    };
    assert!(file.write_all(b"test content for hashing").is_ok());
    drop(file);

    let hash = compute_file_hash(&temp_path);
    assert!(hash.is_some());
    let Some(hash) = hash else {
        panic!("hash should exist for the temporary file");
    };
    assert_eq!(hash.len(), 64);

    let Some(hash2) = compute_file_hash(&temp_path) else {
        panic!("hash should exist for the same temporary file");
    };
    assert_eq!(hash, hash2);

    std::fs::remove_file(&temp_path).ok();
}

#[test]
fn test_compute_file_hash_nonexistent() {
    let hash = compute_file_hash(Path::new("/nonexistent/file.rs"));
    assert!(hash.is_none());
}

#[test]
fn test_to_pascal_case() {
    assert_eq!(to_pascal_case("user_handler"), "UserHandler");
    assert_eq!(to_pascal_case("process_data"), "ProcessData");
    assert_eq!(to_pascal_case("single"), "Single");
    assert_eq!(to_pascal_case(""), "");
    assert_eq!(to_pascal_case("a_b_c"), "ABC");
}
