pub mod parser;
pub mod parser_py;
pub mod graph;
pub mod edges;
pub mod store;
pub mod hash;
pub mod embed;
pub mod vector_store;
pub mod schema;

use anyhow::Result;
use std::path::Path;
use ignore::WalkBuilder; // Add 'ignore' crate for .gitignore support
use indicatif::{ProgressBar, ProgressStyle}; // Add 'indicatif'

pub use store::OracleStore;
pub use vector_store::VectorStore;
pub mod query;

pub async fn scan_codebase(root: &Path) -> Result<()> {
    println!("Scanning codebase at {:?}", root);
    
    // 1. Init Stores
    let db_path = root.join(".arqon/graph.db");
    let mut store = OracleStore::open(db_path)?;
    
    let vector_path = root.join(".arqon/vectors.lance");
    let vector_uri = vector_path.to_str().unwrap();
    let mut vector_store = VectorStore::new(vector_uri).await?;
    vector_store.create_table_if_not_exists().await?;

    // 2. Walk Files
    let walker = WalkBuilder::new(root)
        // .hidden(false) // config overrides
        .build();

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner());
    
    let mut graph_builder = graph::GraphBuilder::new()?;
    let mut edge_builder = edges::EdgeBuilder::new()?;
    // let embedding_model = embed::MiniLM::new()?; // Heavy load

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy();
                        if ext_str == "rs" || ext_str == "py" {
                            pb.set_message(format!("Processing {:?}", path.file_name().unwrap()));
                            
                            let content = std::fs::read_to_string(path)?;
                            let relative_path = path.strip_prefix(root)?.to_string_lossy();
                            
                            // 3. Extract Nodes
                            let nodes = graph_builder.extract_nodes(&relative_path, &content);
                            for node in &nodes {
                                store.insert_node(node)?;
                                // TODO: Embed and insert into LanceDB
                            }
                            
                            // 4. Extract Edges
                            let edges = edge_builder.extract_edges(&relative_path, &content);
                            for edge in edges {
                                store.insert_edge(&edge)?;
                            }
                        }
                    }
                }
            }
            Err(err) => eprintln!("Error walking path: {}", err),
        }
    }
    
    pb.finish_with_message("Scan complete.");
    Ok(())
}
