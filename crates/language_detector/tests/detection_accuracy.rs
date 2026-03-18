use language::Language;
use std::sync::Arc;

fn load_languages() -> Vec<Arc<Language>> {
    vec![
        languages::language("json", tree_sitter_json::LANGUAGE.into()),
        languages::language("python", tree_sitter_python::LANGUAGE.into()),
        languages::language("rust", tree_sitter_rust::LANGUAGE.into()),
        languages::language("go", tree_sitter_go::LANGUAGE.into()),
        languages::language("typescript", tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        languages::language("tsx", tree_sitter_typescript::LANGUAGE_TSX.into()),
        languages::language("c", tree_sitter_c::LANGUAGE.into()),
        languages::language("cpp", tree_sitter_cpp::LANGUAGE.into()),
        languages::language("bash", tree_sitter_bash::LANGUAGE.into()),
        languages::language("css", tree_sitter_css::LANGUAGE.into()),
        languages::language("yaml", tree_sitter_yaml::LANGUAGE.into()),
        languages::language("markdown", tree_sitter_md::LANGUAGE.into()),
        languages::language("diff", tree_sitter_diff::LANGUAGE.into()),
    ]
}

struct TestCase {
    name: &'static str,
    expected: &'static str,
    code: &'static str,
}

fn test_corpus() -> Vec<TestCase> {
    vec![
        // === JSON ===
        TestCase {
            name: "json_package",
            expected: "JSON",
            code: r#"{
  "name": "zed-editor",
  "version": "1.0.0",
  "dependencies": {
    "serde": "1.0",
    "tokio": { "version": "1.0", "features": ["full"] }
  }
}"#,
        },
        TestCase {
            name: "json_array",
            expected: "JSON",
            code: r#"[
  {"id": 1, "name": "Alice", "active": true},
  {"id": 2, "name": "Bob", "active": false}
]"#,
        },
        TestCase {
            name: "json_single_line",
            expected: "JSON",
            code: r#"{"key": "value", "count": 42, "active": true}"#,
        },
        // === Python ===
        TestCase {
            name: "python_class",
            expected: "Python",
            code: r#"import os
from typing import Optional, List

class FileProcessor:
    def __init__(self, base_dir: str):
        self.base_dir = base_dir
        self._cache: dict = {}

    def process(self, filename: str) -> Optional[str]:
        if filename in self._cache:
            return self._cache[filename]
        with open(os.path.join(self.base_dir, filename)) as f:
            content = f.read()
        self._cache[filename] = content
        return content
"#,
        },
        TestCase {
            name: "python_function",
            expected: "Python",
            code: r#"def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

for i in range(10):
    print(f"fib({i}) = {fibonacci(i)}")
"#,
        },
        TestCase {
            name: "python_comprehension",
            expected: "Python",
            code: r#"numbers = [x ** 2 for x in range(20) if x % 3 == 0]
result = {k: v for k, v in zip(names, scores) if v > 80}
print(sum(numbers))
"#,
        },
        // === Rust ===
        TestCase {
            name: "rust_struct",
            expected: "Rust",
            code: r#"use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Registry {
    entries: HashMap<String, Entry>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: Entry) -> Option<Entry> {
        self.entries.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.get(key)
    }
}
"#,
        },
        TestCase {
            name: "rust_async",
            expected: "Rust",
            code: r#"async fn fetch_data(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data = fetch_data("https://example.com").await?;
    println!("{data}");
    Ok(())
}
"#,
        },
        TestCase {
            name: "rust_enum_match",
            expected: "Rust",
            code: r#"enum Command {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
}

fn execute(cmd: Command) {
    match cmd {
        Command::Quit => std::process::exit(0),
        Command::Echo(msg) => println!("{msg}"),
        Command::Move { x, y } => println!("Moving to ({x}, {y})"),
    }
}
"#,
        },
        // === Go ===
        TestCase {
            name: "go_http",
            expected: "Go",
            code: r#"package main

import (
	"encoding/json"
	"log"
	"net/http"
)

type Response struct {
	Message string `json:"message"`
	Status  int    `json:"status"`
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	resp := Response{Message: "OK", Status: 200}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}

func main() {
	http.HandleFunc("/health", healthHandler)
	log.Fatal(http.ListenAndServe(":8080", nil))
}
"#,
        },
        TestCase {
            name: "go_goroutine",
            expected: "Go",
            code: r#"package main

import (
	"fmt"
	"sync"
)

func worker(id int, wg *sync.WaitGroup, ch chan<- int) {
	defer wg.Done()
	result := id * id
	ch <- result
}

func main() {
	var wg sync.WaitGroup
	ch := make(chan int, 10)

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go worker(i, &wg, ch)
	}

	go func() {
		wg.Wait()
		close(ch)
	}()

	for result := range ch {
		fmt.Println(result)
	}
}
"#,
        },
        // === TypeScript ===
        TestCase {
            name: "typescript_interface",
            expected: "TypeScript",
            code: r#"interface User {
  id: number;
  name: string;
  email: string;
  roles: Role[];
}

type Role = "admin" | "editor" | "viewer";

async function fetchUsers(url: string): Promise<User[]> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP error: ${response.status}`);
  }
  return response.json();
}
"#,
        },
        // === C ===
        TestCase {
            name: "c_pointers",
            expected: "C",
            code: r#"#include <stdio.h>
#include <stdlib.h>

typedef struct Node {
    int data;
    struct Node *next;
} Node;

Node *create_node(int data) {
    Node *node = (Node *)malloc(sizeof(Node));
    if (!node) return NULL;
    node->data = data;
    node->next = NULL;
    return node;
}

void push(Node **head, int data) {
    Node *new_node = create_node(data);
    new_node->next = *head;
    *head = new_node;
}

void print_list(Node *head) {
    while (head) {
        printf("%d -> ", head->data);
        head = head->next;
    }
    printf("NULL\n");
}
"#,
        },
        // === C++ ===
        TestCase {
            name: "cpp_template",
            expected: "C++",
            code: r#"#include <iostream>
#include <vector>
#include <memory>

template<typename T>
class Stack {
public:
    void push(T value) {
        data_.push_back(std::move(value));
    }

    std::optional<T> pop() {
        if (data_.empty()) return std::nullopt;
        T value = std::move(data_.back());
        data_.pop_back();
        return value;
    }

    [[nodiscard]] bool empty() const { return data_.empty(); }
    [[nodiscard]] size_t size() const { return data_.size(); }

private:
    std::vector<T> data_;
};
"#,
        },
        // === Bash ===
        TestCase {
            name: "bash_script",
            expected: "Shell Script",
            code: r#"#!/bin/bash
set -euo pipefail

readonly LOG_DIR="/var/log/myapp"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

check_deps() {
    local deps=("docker" "curl" "jq")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            log "Missing: $dep"
            exit 1
        fi
    done
}

main() {
    check_deps
    log "All dependencies found"
}

main "$@"
"#,
        },
        // === CSS ===
        TestCase {
            name: "css_styles",
            expected: "CSS",
            code: r#":root {
  --primary: #3b82f6;
  --bg: #1a1a2e;
}

body {
  margin: 0;
  font-family: 'Inter', sans-serif;
  background: var(--bg);
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.3);
}

@media (max-width: 768px) {
  .container { grid-template-columns: 1fr; }
}
"#,
        },
        // === YAML ===
        TestCase {
            name: "yaml_k8s",
            expected: "YAML",
            code: r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-app
  template:
    spec:
      containers:
        - name: web
          image: registry.example.com/web:v2
          ports:
            - containerPort: 8080
          resources:
            limits:
              memory: "256Mi"
              cpu: "500m"
"#,
        },
        // === Markdown ===
        TestCase {
            name: "markdown_doc",
            expected: "Markdown",
            code: r#"# Project Name

A brief description of the project.

## Installation

```bash
npm install my-package
```

## Features

- **Fast**: Optimized for performance
- **Simple**: Easy to use API

| Parameter | Type | Default |
|-----------|------|---------|
| `verbose` | `boolean` | `false` |

## License

MIT
"#,
        },
        // === Diff ===
        TestCase {
            name: "diff_patch",
            expected: "Diff",
            code: r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,7 +10,9 @@ fn main() {
     let config = Config::load();
-    let server = Server::new(config.port);
+    let server = Server::builder()
+        .port(config.port)
+        .build();
     server.start();
 }
"#,
        },
    ]
}

#[test]
fn test_detection_accuracy() {
    let candidates = load_languages();
    let corpus = test_corpus();
    let total = corpus.len();
    let mut correct = 0;
    let mut wrong = 0;
    let mut none_count = 0;
    let mut failures: Vec<String> = Vec::new();

    println!("\n=== Language Detection Accuracy Test ===\n");
    println!("{:<25} {:<20} {:<20} {:>8}", "Test Case", "Expected", "Detected", "Score");
    println!("{}", "-".repeat(78));

    for case in &corpus {
        let result = language_detector::best_match(case.code.as_bytes(), &candidates);
        let (detected_name, score) = match &result {
            Some(lang) => {
                let score = language_detector::score_language(lang, case.code.as_bytes());
                (lang.name().to_string(), score)
            }
            None => ("(none)".to_string(), 0.0),
        };

        let is_correct = detected_name == case.expected;
        let marker = if is_correct {
            correct += 1;
            "✓"
        } else if result.is_none() {
            none_count += 1;
            "○"
        } else {
            wrong += 1;
            "✗"
        };

        println!(
            "{} {:<23} {:<20} {:<20} {:>7.4}",
            marker, case.name, case.expected, detected_name, score
        );

        if !is_correct {
            failures.push(format!(
                "{}: expected '{}', got '{}'",
                case.name, case.expected, detected_name
            ));
        }
    }

    println!("{}", "-".repeat(78));
    println!(
        "\nResults: {correct}/{total} correct, {wrong} wrong, {none_count} no detection"
    );
    let accuracy = correct as f64 / total as f64 * 100.0;
    println!("Accuracy: {accuracy:.1}%\n");

    if !failures.is_empty() {
        println!("Failures:");
        for f in &failures {
            println!("  - {f}");
        }
        println!();
    }

    // Print individual scores for debugging
    println!("\n=== Per-Language Score Matrix (first test case per language) ===\n");
    let debug_cases = vec![
        ("JSON", r#"{"name": "test", "version": "1.0"}"#),
        ("Python", "def hello():\n    print('hello')\n"),
        ("Rust", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("Go", "package main\n\nfunc main() {\n}\n"),
        (
            "TypeScript",
            "interface Foo { bar: string }\nconst x: Foo = { bar: 'baz' };\n",
        ),
        ("Bash", "#!/bin/bash\necho hello\nif [ -f test ]; then\n  cat test\nfi\n"),
    ];

    for (label, code) in &debug_cases {
        println!("--- {label} snippet ---");
        for lang in &candidates {
            let score = language_detector::score_language(lang, code.as_bytes());
            if score > 0.01 {
                println!("  {:<15} {:.4}", lang.name(), score);
            }
        }
        println!();
    }

    assert!(
        accuracy >= 70.0,
        "Accuracy {accuracy:.1}% is below the 70% minimum threshold"
    );
}

#[test]
fn test_detection_latency() {
    let candidates = load_languages();
    let corpus = test_corpus();

    println!("\n=== Language Detection Latency Test ===\n");
    println!("{:<25} {:>10} {:>10}", "Test Case", "Time (µs)", "Score");
    println!("{}", "-".repeat(50));

    let mut total_us = 0u128;
    let mut max_us = 0u128;

    for case in &corpus {
        let start = std::time::Instant::now();
        let result = language_detector::best_match(case.code.as_bytes(), &candidates);
        let elapsed = start.elapsed();

        let us = elapsed.as_micros();
        total_us += us;
        if us > max_us {
            max_us = us;
        }

        let score = match &result {
            Some(lang) => language_detector::score_language(lang, case.code.as_bytes()),
            None => 0.0,
        };

        println!("{:<25} {:>10} {:>10.4}", case.name, us, score);
    }

    let avg_us = total_us / corpus.len() as u128;
    println!("{}", "-".repeat(50));
    println!("Average: {avg_us} µs");
    println!("Max: {max_us} µs");
    println!("Total ({} samples): {total_us} µs", corpus.len());

    assert!(
        max_us < 500_000,
        "Max latency {max_us}µs exceeds 500ms budget"
    );
}
