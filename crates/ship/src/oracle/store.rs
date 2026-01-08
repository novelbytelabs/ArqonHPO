use rusqlite::{Connection, Result, params};
use crate::oracle::graph::GraphNode;
use crate::oracle::edges::GraphEdge;
use crate::oracle::schema::init_db;
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
                node.path, node.node_type, node.name, 
                node.start_line, node.end_line, node.signature_hash, node.docstring
            ],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    pub fn insert_edge(&mut self, edge: &GraphEdge) -> Result<()> {
        // Find IDs first (simplified logic: assumes uniqueness by name for MVP)
        // In reality, would need path resolution.
        let source_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM nodes WHERE name = ?1 LIMIT 1",
            params![edge.source_node_name],
            |row| row.get(0),
        ).optional()?;

        let target_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM nodes WHERE name = ?1 LIMIT 1",
            params![edge.target_node_name],
            |row| row.get(0),
        ).optional()?;

        if let (Some(sid), Some(tid)) = (source_id, target_id) {
            self.conn.execute(
                "INSERT INTO edges (source_id, target_id, type) VALUES (?1, ?2, ?3)",
                params![sid, tid, edge.edge_type],
            )?;
        }
        Ok(())
    }
}

// Add optional helper trait
use rusqlite::OptionalExtension;
