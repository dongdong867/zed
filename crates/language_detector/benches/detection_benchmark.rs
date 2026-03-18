use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
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

struct Sample {
    name: &'static str,
    #[allow(dead_code)]
    expected_language: &'static str,
    code: &'static str,
}

fn corpus() -> Vec<Sample> {
    vec![
        // === JSON ===
        Sample {
            name: "json_object",
            expected_language: "JSON",
            code: r#"{
  "name": "zed-editor",
  "version": "1.0.0",
  "dependencies": {
    "serde": "1.0",
    "tokio": { "version": "1.0", "features": ["full"] }
  },
  "scripts": {
    "build": "cargo build --release",
    "test": "cargo test"
  }
}"#,
        },
        Sample {
            name: "json_array",
            expected_language: "JSON",
            code: r#"[
  {"id": 1, "name": "Alice", "email": "alice@example.com", "active": true},
  {"id": 2, "name": "Bob", "email": "bob@example.com", "active": false},
  {"id": 3, "name": "Charlie", "email": "charlie@example.com", "active": true}
]"#,
        },
        Sample {
            name: "json_nested",
            expected_language: "JSON",
            code: r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "credentials": {
      "username": "admin",
      "password": null
    },
    "options": {
      "ssl": true,
      "timeout": 30,
      "pool_size": 10
    }
  }
}"#,
        },
        // === Python ===
        Sample {
            name: "python_function",
            expected_language: "Python",
            code: r#"import os
from pathlib import Path
from typing import Optional, List

def find_files(directory: str, pattern: str = "*.py") -> List[Path]:
    """Find all files matching a pattern in a directory tree."""
    results = []
    root = Path(directory)
    if not root.exists():
        raise FileNotFoundError(f"Directory not found: {directory}")
    for path in root.rglob(pattern):
        if path.is_file():
            results.append(path)
    return sorted(results)

class FileProcessor:
    def __init__(self, base_dir: str):
        self.base_dir = base_dir
        self._cache: dict = {}

    def process(self, filename: str) -> Optional[str]:
        if filename in self._cache:
            return self._cache[filename]
        content = Path(self.base_dir, filename).read_text()
        self._cache[filename] = content
        return content
"#,
        },
        Sample {
            name: "python_short",
            expected_language: "Python",
            code: r#"def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

for i in range(10):
    print(f"fib({i}) = {fibonacci(i)}")
"#,
        },
        // === Rust ===
        Sample {
            name: "rust_struct_impl",
            expected_language: "Rust",
            code: r#"use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Registry {
    entries: HashMap<String, Arc<Entry>>,
    capacity: usize,
}

impl Registry {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn insert(&mut self, key: String, value: Entry) -> Option<Arc<Entry>> {
        if self.entries.len() >= self.capacity {
            return None;
        }
        self.entries.insert(key, Arc::new(value))
    }

    pub fn get(&self, key: &str) -> Option<&Arc<Entry>> {
        self.entries.get(key)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub version: u32,
    pub data: Vec<u8>,
}
"#,
        },
        Sample {
            name: "rust_async",
            expected_language: "Rust",
            code: r#"use anyhow::Result;
use tokio::sync::mpsc;

async fn process_messages(mut rx: mpsc::Receiver<String>) -> Result<Vec<String>> {
    let mut results = Vec::new();
    while let Some(msg) = rx.recv().await {
        if msg == "quit" {
            break;
        }
        let processed = msg.to_uppercase();
        results.push(processed);
    }
    Ok(results)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handle = tokio::spawn(process_messages(rx));
    tx.send("hello".to_string()).await?;
    tx.send("world".to_string()).await?;
    tx.send("quit".to_string()).await?;
    drop(tx);
    let results = handle.await??;
    println!("{:?}", results);
    Ok(())
}
"#,
        },
        // === Go ===
        Sample {
            name: "go_http_server",
            expected_language: "Go",
            code: r#"package main

import (
	"encoding/json"
	"fmt"
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
	fmt.Println("Server starting on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
"#,
        },
        // === TypeScript ===
        Sample {
            name: "typescript_interface",
            expected_language: "TypeScript",
            code: r#"interface User {
  id: number;
  name: string;
  email: string;
  roles: Role[];
  createdAt: Date;
}

type Role = "admin" | "editor" | "viewer";

async function fetchUsers(apiUrl: string): Promise<User[]> {
  const response = await fetch(apiUrl);
  if (!response.ok) {
    throw new Error(`HTTP error: ${response.status}`);
  }
  const data: User[] = await response.json();
  return data.filter((user) => user.roles.includes("admin"));
}

class UserService {
  private cache: Map<number, User> = new Map();

  async getUser(id: number): Promise<User | undefined> {
    if (this.cache.has(id)) {
      return this.cache.get(id);
    }
    const users = await fetchUsers(`/api/users/${id}`);
    const user = users[0];
    if (user) {
      this.cache.set(id, user);
    }
    return user;
  }
}
"#,
        },
        // === C ===
        Sample {
            name: "c_linked_list",
            expected_language: "C",
            code: r#"#include <stdio.h>
#include <stdlib.h>

typedef struct Node {
    int data;
    struct Node *next;
} Node;

Node *create_node(int data) {
    Node *node = (Node *)malloc(sizeof(Node));
    if (node == NULL) {
        fprintf(stderr, "Memory allocation failed\n");
        return NULL;
    }
    node->data = data;
    node->next = NULL;
    return node;
}

void push(Node **head, int data) {
    Node *new_node = create_node(data);
    if (new_node == NULL) return;
    new_node->next = *head;
    *head = new_node;
}

void print_list(Node *head) {
    Node *current = head;
    while (current != NULL) {
        printf("%d -> ", current->data);
        current = current->next;
    }
    printf("NULL\n");
}

void free_list(Node *head) {
    Node *temp;
    while (head != NULL) {
        temp = head;
        head = head->next;
        free(temp);
    }
}
"#,
        },
        // === C++ ===
        Sample {
            name: "cpp_class",
            expected_language: "C++",
            code: r#"#include <iostream>
#include <vector>
#include <memory>
#include <string>

template<typename T>
class Stack {
public:
    void push(T value) {
        data_.push_back(std::move(value));
    }

    std::optional<T> pop() {
        if (data_.empty()) {
            return std::nullopt;
        }
        T value = std::move(data_.back());
        data_.pop_back();
        return value;
    }

    [[nodiscard]] bool empty() const { return data_.empty(); }
    [[nodiscard]] size_t size() const { return data_.size(); }

private:
    std::vector<T> data_;
};

int main() {
    auto stack = std::make_unique<Stack<std::string>>();
    stack->push("hello");
    stack->push("world");
    while (!stack->empty()) {
        auto val = stack->pop();
        if (val) {
            std::cout << *val << std::endl;
        }
    }
    return 0;
}
"#,
        },
        // === Bash ===
        Sample {
            name: "bash_script",
            expected_language: "Shell Script",
            code: r#"#!/bin/bash
set -euo pipefail

readonly LOG_DIR="/var/log/myapp"
readonly BACKUP_DIR="/backups"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "${LOG_DIR}/deploy.log"
}

check_dependencies() {
    local deps=("docker" "curl" "jq")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &>/dev/null; then
            log "ERROR: Missing dependency: $dep"
            exit 1
        fi
    done
}

backup_database() {
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="${BACKUP_DIR}/db_${timestamp}.sql.gz"
    pg_dump mydb | gzip > "$backup_file"
    log "Database backed up to $backup_file"
}

main() {
    check_dependencies
    backup_database
    log "Deploy complete"
}

main "$@"
"#,
        },
        // === CSS ===
        Sample {
            name: "css_styles",
            expected_language: "CSS",
            code: r#":root {
  --primary-color: #3b82f6;
  --bg-color: #1a1a2e;
  --text-color: #e2e8f0;
  --border-radius: 8px;
}

body {
  margin: 0;
  padding: 0;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  background-color: var(--bg-color);
  color: var(--text-color);
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: var(--border-radius);
  padding: 1.5rem;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.3);
}

@media (max-width: 768px) {
  .container {
    grid-template-columns: 1fr;
    padding: 1rem;
  }
}
"#,
        },
        // === YAML ===
        Sample {
            name: "yaml_config",
            expected_language: "YAML",
            code: r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  namespace: production
  labels:
    app: web-app
    version: v2.1.0
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
    spec:
      containers:
        - name: web
          image: registry.example.com/web-app:v2.1.0
          ports:
            - containerPort: 8080
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: db-credentials
                  key: url
          resources:
            limits:
              memory: "256Mi"
              cpu: "500m"
            requests:
              memory: "128Mi"
              cpu: "250m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 15
            periodSeconds: 10
"#,
        },
        // === Markdown ===
        Sample {
            name: "markdown_readme",
            expected_language: "Markdown",
            code: r#"# Project Name

A brief description of the project.

## Installation

```bash
npm install my-package
```

## Usage

```javascript
const lib = require('my-package');
lib.doSomething();
```

## Features

- **Fast**: Optimized for performance
- **Simple**: Easy to use API
- **Extensible**: Plugin system included

## API Reference

### `doSomething(options)`

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `verbose` | `boolean` | `false` | Enable verbose output |
| `timeout` | `number` | `5000` | Timeout in milliseconds |

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing`)
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

## License

MIT
"#,
        },
        // === Diff ===
        Sample {
            name: "diff_patch",
            expected_language: "Diff",
            code: r#"diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,7 +10,9 @@ fn main() {
     let config = Config::load();
-    let server = Server::new(config.port);
+    let server = Server::builder()
+        .port(config.port)
+        .workers(config.workers)
+        .build();
     server.start();
 }

diff --git a/src/config.rs b/src/config.rs
index 111aaa..222bbb 100644
--- a/src/config.rs
+++ b/src/config.rs
@@ -5,6 +5,7 @@ pub struct Config {
     pub port: u16,
+    pub workers: usize,
     pub database_url: String,
 }
"#,
        },
        // === Edge cases ===
        Sample {
            name: "short_json",
            expected_language: "JSON",
            code: r#"{"key": "value", "count": 42, "active": true}"#,
        },
        Sample {
            name: "short_python",
            expected_language: "Python",
            code: r#"def greet(name: str) -> str:
    return f"Hello, {name}!"

print(greet("world"))
"#,
        },
        Sample {
            name: "short_rust",
            expected_language: "Rust",
            code: r#"fn main() {
    let numbers: Vec<i32> = (1..=10).collect();
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {sum}");
}
"#,
        },
    ]
}

fn bench_best_match(c: &mut Criterion) {
    let candidates = load_languages();
    let samples = corpus();

    let mut group = c.benchmark_group("best_match");
    for sample in &samples {
        group.bench_with_input(
            BenchmarkId::new("detect", sample.name),
            &sample.code,
            |b, code| {
                b.iter(|| {
                    language_detector::best_match(code.as_bytes(), &candidates);
                });
            },
        );
    }
    group.finish();
}

fn bench_score_language(c: &mut Criterion) {
    let candidates = load_languages();
    let json_sample = r#"{"name": "test", "version": "1.0.0", "dependencies": {"serde": "1.0"}}"#;

    let mut group = c.benchmark_group("score_language");
    for lang in &candidates {
        let name = lang.name().to_string();
        group.bench_with_input(BenchmarkId::new("score", &name), &json_sample, |b, code| {
            b.iter(|| {
                language_detector::score_language(lang, code.as_bytes());
            });
        });
    }
    group.finish();
}

#[cfg(feature = "magika")]
fn bench_magika_detect(c: &mut Criterion) {
    let samples = corpus();
    let detector = match language_detector::MagikaDetector::new() {
        Ok(detector) => detector,
        Err(error) => {
            eprintln!("Skipping Magika benchmark: failed to initialize session: {error:#}");
            return;
        }
    };
    let detector = std::cell::RefCell::new(detector);

    let mut group = c.benchmark_group("magika_detect");
    for sample in &samples {
        group.bench_with_input(
            BenchmarkId::new("detect", sample.name),
            &sample.code,
            |b, code| {
                b.iter(|| {
                    let _ = detector.borrow_mut().detect_language_name(code.as_bytes());
                });
            },
        );
    }
    group.finish();
}

#[cfg(feature = "magika")]
fn bench_magika_init(c: &mut Criterion) {
    c.bench_function("magika_init/new_session", |b| {
        b.iter(|| {
            let _ = language_detector::MagikaDetector::new();
        });
    });
}

#[cfg(not(feature = "magika"))]
fn bench_magika_detect(_c: &mut Criterion) {}

#[cfg(not(feature = "magika"))]
fn bench_magika_init(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_best_match,
    bench_score_language,
    bench_magika_detect,
    bench_magika_init
);
criterion_main!(benches);
