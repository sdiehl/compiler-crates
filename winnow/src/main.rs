use winnow_example::{parse_config, parse_expression, parse_json, parse_sexpr, parse_url};

fn main() {
    println!("=== Winnow Parser Examples ===\n");

    // Arithmetic expressions
    println!("Arithmetic Expression Parsing:");
    let expressions = vec![
        "42",
        "3.14",
        "1 + 2",
        "1 + 2 * 3",
        "(1 + 2) * 3",
        "10 - 5 / 2",
        "100 / 10 + 5 * 2",
    ];

    for expr_str in &expressions {
        match parse_expression(expr_str) {
            Ok(expr) => {
                println!("  {} = {}", expr_str, expr.eval());
            }
            Err(e) => println!("  Error parsing '{}': {}", expr_str, e),
        }
    }

    // JSON parsing
    println!("\nJSON Parsing:");
    let json_inputs = vec![
        "null",
        "true",
        "42",
        r#""hello world""#,
        "[1, 2, 3]",
        r#"{"name": "Alice", "age": 30, "active": true}"#,
    ];

    for json_str in &json_inputs {
        match parse_json(json_str) {
            Ok(json) => {
                let preview = if json_str.len() > 40 {
                    format!("{}...", &json_str[..37])
                } else {
                    json_str.to_string()
                };
                println!("  {} -> {:?}", preview, json);
            }
            Err(e) => println!("  Error parsing JSON: {}", e),
        }
    }

    // S-Expression parsing
    println!("\nS-Expression Parsing:");
    let sexpr_inputs = vec![
        "42",
        "foo",
        r#""hello""#,
        "()",
        "(+ 1 2)",
        "(define (square x) (* x x))",
        "(lambda (x y) (+ x y))",
    ];

    for sexpr_str in &sexpr_inputs {
        match parse_sexpr(sexpr_str) {
            Ok(sexpr) => {
                println!("  {} -> {:?}", sexpr_str, sexpr);
            }
            Err(e) => println!("  Error parsing S-expression: {}", e),
        }
    }

    // Configuration parsing
    println!("\nConfiguration File Parsing:");
    let config_input = r#"server_name = "production"
port = 8080
enable_ssl = true
allowed_hosts = ["example.com", "www.example.com"]
max_connections = 1000
"#;

    match parse_config(config_input) {
        Ok(config) => {
            println!("  Parsed configuration:");
            for entry in &config.entries {
                println!("    {} = {:?}", entry.key, entry.value);
            }
        }
        Err(e) => println!("  Error parsing config: {}", e),
    }

    // URL parsing
    println!("\nURL Parsing:");
    let urls = vec![
        "http://example.com",
        "https://api.example.com:8080/v1/users",
        "http://example.com/search?q=rust&limit=10",
        "https://docs.rust-lang.org/book/ch01-00-introduction.html#getting-started",
    ];

    for url_str in &urls {
        match parse_url(url_str) {
            Ok(url) => {
                println!("  {}", url_str);
                println!("    Scheme: {}", url.scheme);
                println!("    Host: {}", url.host);
                if let Some(port) = url.port {
                    println!("    Port: {}", port);
                }
                println!("    Path: {}", url.path);
                if let Some(query) = &url.query {
                    println!("    Query: {}", query);
                }
                if let Some(fragment) = &url.fragment {
                    println!("    Fragment: {}", fragment);
                }
            }
            Err(e) => println!("  Error parsing URL '{}': {}", url_str, e),
        }
    }
}
