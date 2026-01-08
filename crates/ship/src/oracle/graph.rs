use tree_sitter::{Node, TreeCursor};
use anyhow::Result;
use crate::oracle::parser::RustParser;
use crate::oracle::parser_py::PythonParser;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub path: String,
    pub node_type: String,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub signature_hash: String,
    pub docstring: Option<String>,
}

pub struct GraphBuilder {
    rust_parser: RustParser,
    _python_parser: PythonParser,
}

impl GraphBuilder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            rust_parser: RustParser::new()?,
            python_parser: PythonParser::new()?,
        })
    }

    pub fn extract_nodes(&mut self, path: &str, content: &str) -> Vec<GraphNode> {
        if path.ends_with(".rs") {
            self.extract_rust(path, content)
        } else if path.ends_with(".py") {
            self.extract_python(path, content)
        } else {
            vec![]
        }
    }

    fn extract_rust(&mut self, path: &str, content: &str) -> Vec<GraphNode> {
        let mut nodes = Vec::new();
        if let Some(tree) = self.rust_parser.parse(content) {
            let mut cursor = tree.walk();
            self.visit_rust_node(&mut cursor, path, content, &mut nodes);
        }
        nodes
    }

    fn visit_rust_node(&self, cursor: &mut TreeCursor, path: &str, content: &str, nodes: &mut Vec<GraphNode>) {
        let node = cursor.node();
        let kind = node.kind();
        
        if kind == "function_item" || kind == "struct_item" || kind == "impl_item" {
             if let Some(name_node) = node.child_by_field_name("name") {
                let name = extract_text(name_node, content);
                let signature = extract_text(node, content); // Simplified: full text as signature hash source
                let hash = compute_hash(&signature);
                
                nodes.push(GraphNode {
                    path: path.to_string(),
                    node_type: kind.replace("_item", ""), // "function", "struct", "impl"
                    name,
                    start_line: node.start_position().row,
                    end_line: node.end_position().row,
                    signature_hash: hash,
                    docstring: None, // TODO: Extract doc comments
                });
             }
        }

        if cursor.goto_first_child() {
            loop {
                self.visit_rust_node(cursor, path, content, nodes);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    fn extract_python(&mut self, _path: &str, _content: &str) -> Vec<GraphNode> {
        // Todo: Implement Python node extraction similar to Rust
        vec![] 
    }
}

fn extract_text(node: Node, content: &str) -> String {
    content[node.byte_range()].to_string()
}

fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text);
    format!("{:x}", hasher.finalize())
}
