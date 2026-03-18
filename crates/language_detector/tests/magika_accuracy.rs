#![cfg(feature = "magika")]

struct TestCase {
    name: &'static str,
    expected: &'static str,
    code: &'static str,
}

fn corpus() -> Vec<TestCase> {
    vec![
        TestCase {
            name: "json",
            expected: "JSON",
            code: r#"{"name":"zed","version":"1.0","active":true}"#,
        },
        TestCase {
            name: "python",
            expected: "Python",
            code: "def greet(name):\n    return f'Hello {name}'\n",
        },
        TestCase {
            name: "rust",
            expected: "Rust",
            code: "fn main() {\n    println!(\"hello\");\n}\n",
        },
        TestCase {
            name: "go",
            expected: "Go",
            code: "package main\n\nfunc main() {\n}\n",
        },
        TestCase {
            name: "typescript",
            expected: "TypeScript",
            code: "interface User { id: number; name: string }\nconst user: User = { id: 1, name: 'a' };\n",
        },
        TestCase {
            name: "c",
            expected: "C",
            code: "#include <stdio.h>\nint main(void) { return 0; }\n",
        },
        TestCase {
            name: "cpp",
            expected: "C++",
            code: "#include <iostream>\nint main() { std::cout << \"hi\"; }\n",
        },
        TestCase {
            name: "bash",
            expected: "Shell Script",
            code: "#!/bin/bash\necho hello\nif [ -f foo ]; then\n  cat foo\nfi\n",
        },
        TestCase {
            name: "css",
            expected: "CSS",
            code: "body { margin: 0; color: #333; }\n.container { display: grid; }\n",
        },
        TestCase {
            name: "yaml",
            expected: "YAML",
            code: "apiVersion: v1\nkind: Pod\nmetadata:\n  name: demo\n",
        },
        TestCase {
            name: "markdown",
            expected: "Markdown",
            code: "# Title\n\n- item 1\n- item 2\n\n```rust\nfn main() {}\n```\n",
        },
        TestCase {
            name: "diff",
            expected: "Diff",
            code: "diff --git a/a.rs b/a.rs\n--- a/a.rs\n+++ b/a.rs\n@@ -1 +1 @@\n-fn a() {}\n+fn b() {}\n",
        },
    ]
}

#[test]
fn test_magika_accuracy() {
    let mut detector = language_detector::MagikaDetector::new()
        .expect("Magika session should initialize for benchmark comparison");

    let corpus = corpus();
    let total = corpus.len();
    let mut correct = 0;

    println!("\n=== Magika Accuracy ===\n");
    println!(
        "{:<14} {:<14} {:<14}",
        "Test Case", "Expected", "Detected"
    );
    println!("{}", "-".repeat(46));

    for case in &corpus {
        let detected = detector
            .detect_language_name(case.code.as_bytes())
            .unwrap_or("(none)");
        println!("{:<14} {:<14} {:<14}", case.name, case.expected, detected);
        if detected == case.expected {
            correct += 1;
        }
    }

    let accuracy = correct as f64 / total as f64 * 100.0;
    println!("\nCorrect: {correct}/{total}");
    println!("Accuracy: {accuracy:.1}%\n");

    assert!(accuracy >= 60.0, "Magika accuracy too low: {accuracy:.1}%");
}

#[test]
fn test_magika_cold_start_latency() {
    let init_start = std::time::Instant::now();
    let mut detector =
        language_detector::MagikaDetector::new().expect("Magika session should initialize");
    let init_elapsed = init_start.elapsed();

    let detect_start = std::time::Instant::now();
    let detected = detector.detect_language_name(br#"{"kind":"cold-start-check"}"#);
    let detect_elapsed = detect_start.elapsed();

    println!("\n=== Magika Cold Start ===");
    println!("Init:   {} µs", init_elapsed.as_micros());
    println!("Detect: {} µs", detect_elapsed.as_micros());
    println!("Result: {:?}\n", detected);

    assert!(
        init_elapsed.as_millis() < 1000,
        "Magika init exceeded 1 second: {} ms",
        init_elapsed.as_millis()
    );
}
