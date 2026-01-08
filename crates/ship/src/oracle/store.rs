use crate::oracle::edges::GraphEdge;
use crate::oracle::graph::GraphNode;
use crate::oracle::schema::init_db;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct OracleStore {
    conn: Connection,
}

impl OracleStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        init_db(&conn)?;
        Ok(Self { conn })
    }

    pub fn insert_node(&mut self, node: &GraphNode) -> Result<i64> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO nodes (path, type, name, start_line, end_line, signature_hash, docstring)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(path, type, name, start_line) DO UPDATE SET
             signature_hash=excluded.signature_hash, end_line=excluded.end_line",
            params![
                node.path,
                node.node_type,
                node.name,
                node.start_line,
                node.end_line,
                node.signature_hash,
                node.docstring
            ],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    pub fn insert_edge(&mut self, edge: &GraphEdge) -> Result<()> {
        // Find IDs first (simplified logic: assumes uniqueness by name for MVP)
        // In reality, would need path resolution.
        let source_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM nodes WHERE name = ?1 LIMIT 1",
                params![edge.source_node_name],
                |row| row.get(0),
            )
            .optional()?;

        let target_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM nodes WHERE name = ?1 LIMIT 1",
                params![edge.target_node_name],
                |row| row.get(0),
            )
            .optional()?;

        if let (Some(sid), Some(tid)) = (source_id, target_id) {
            self.conn.execute(
                "INSERT INTO edges (source_id, target_id, type) VALUES (?1, ?2, ?3)",
                params![sid, tid, edge.edge_type],
            )?;
        }
        Ok(())
    }

    /// Get a node by its ID
    pub fn get_node_by_id(&self, id: i64) -> Option<GraphNode> {
        self.conn.query_row(
            "SELECT path, type, name, start_line, end_line, signature_hash, docstring FROM nodes WHERE id = ?1",
            params![id],
            |row| {
                Ok(GraphNode {
                    path: row.get(0)?,
                    node_type: row.get(1)?,
                    name: row.get(2)?,
                    start_line: row.get(3)?,
                    end_line: row.get(4)?,
                    signature_hash: row.get(5)?,
                    docstring: row.get(6)?,
                })
            },
        ).ok()
    }

    /// Get function signatures from the same file or nearby lines
    pub fn get_related_signatures(&self, file_path: &str, near_line: Option<u32>) -> Vec<String> {
        let line = near_line.unwrap_or(0) as i64;

        // Query functions/structs in the same file, ordered by proximity to the error line
        let mut stmt = match self.conn.prepare(
            "SELECT name, type, start_line, end_line FROM nodes 
             WHERE path = ?1 AND type IN ('function', 'struct', 'impl')
             ORDER BY ABS(start_line - ?2) LIMIT 5",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let results = stmt.query_map(params![file_path, line], |row| {
            let name: String = row.get(0)?;
            let node_type: String = row.get(1)?;
            let start: i64 = row.get(2)?;
            let end: i64 = row.get(3)?;
            Ok(format!("{} {} (L{}-L{})", node_type, name, start, end))
        });

        match results {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        }
    }
}

// Add optional helper trait
use rusqlite::OptionalExtension;
