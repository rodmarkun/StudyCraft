use crate::constants::{
    VECDB_CACHE_SIZE, VECDB_EMBEDDING_DIMENSIONS, VECDB_JOURNAL_MODE, VECDB_MMAP_SIZE,
    VECDB_SYNCHRONOUS, VECDB_TEMP_STORE,
};

pub const PRAGMA_SETTINGS: &str = const_format::concatcp!(
    "PRAGMA journal_mode = ",
    VECDB_JOURNAL_MODE,
    ";\n",
    "PRAGMA synchronous = ",
    VECDB_SYNCHRONOUS,
    ";\n",
    "PRAGMA cache_size = ",
    VECDB_CACHE_SIZE,
    ";\n",
    "PRAGMA temp_store = ",
    VECDB_TEMP_STORE,
    ";\n",
    "PRAGMA mmap_size = ",
    VECDB_MMAP_SIZE,
    ";"
);

pub const CREATE_CHUNKS_TABLE: &str = "CREATE TABLE IF NOT EXISTS chunks (
    id TEXT PRIMARY KEY,
    material_id TEXT NOT NULL,
    file_display_name TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    text_length INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)";

pub const CREATE_CHUNKS_FTS_TABLE: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
    id UNINDEXED,
    chunk_text,
    content='chunks',
    content_rowid='id',
    tokenize = 'porter unicode61'
);";

pub const CREATE_CHUNKS_MATERIAL_ID_INDEX: &str =
    "CREATE INDEX IF NOT EXISTS idx_chunks_material_id ON chunks(material_id)";

pub const CREATE_CHUNKS_LENGTH_INDEX: &str =
    "CREATE INDEX IF NOT EXISTS idx_chunks_length ON chunks(text_length)";

pub const COUNT_ALL_CHUNKS: &str = "SELECT COUNT(*) FROM chunks";

pub const COUNT_DISTINCT_MATERIALS: &str = "SELECT COUNT(DISTINCT material_id) FROM chunks";

pub const INSERT_CHUNK: &str =
    "INSERT INTO chunks (id, material_id, file_display_name, chunk_index, chunk_text, text_length) 
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

pub const INSERT_VECTOR: &str = "INSERT INTO vec_chunks (chunk_id, embedding) VALUES (?1, ?2)";

pub const GLOBAL_VECTOR_SEARCH: &str = "SELECT 
    c.id,
    c.material_id,
    c.file_display_name,
    c.chunk_index,
    c.chunk_text,
    vec_chunks.distance
 FROM vec_chunks
 INNER JOIN chunks c ON c.id = vec_chunks.chunk_id
 WHERE vec_chunks.embedding MATCH ?1 AND k = ?2
 ORDER BY vec_chunks.distance";

pub const GET_CHUNK_IDS_BY_MATERIAL: &str = "SELECT id FROM chunks WHERE material_id = ?1";

pub const DELETE_CHUNKS_BY_MATERIAL: &str = "DELETE FROM chunks WHERE material_id = ?1";

pub const DELETE_VECTOR_BY_CHUNK_ID: &str = "DELETE FROM vec_chunks WHERE chunk_id = ?1";

pub const CREATE_VEC_CHUNKS_TABLE: &str = const_format::concatcp!(
    "CREATE VIRTUAL TABLE IF NOT EXISTS vec_chunks USING vec0(
        chunk_id TEXT PRIMARY KEY,
        embedding FLOAT[",
    VECDB_EMBEDDING_DIMENSIONS,
    "]
    )"
);

pub const CREATE_FTS_INSERT_TRIGGER: &str = "
CREATE TRIGGER IF NOT EXISTS t_chunks_after_insert AFTER INSERT ON chunks BEGIN
  INSERT INTO chunks_fts(rowid, id, chunk_text) VALUES (new.rowid, new.id, new.chunk_text);
END";

pub const CREATE_FTS_DELETE_TRIGGER: &str = "
CREATE TRIGGER IF NOT EXISTS t_chunks_after_delete AFTER DELETE ON chunks BEGIN
  INSERT INTO chunks_fts(chunks_fts, rowid, id, chunk_text) VALUES ('delete', old.rowid, old.id, old.chunk_text);
END";

pub const CREATE_FTS_UPDATE_TRIGGER: &str = "
CREATE TRIGGER IF NOT EXISTS t_chunks_after_update AFTER UPDATE ON chunks BEGIN
  INSERT INTO chunks_fts(chunks_fts, rowid, id, chunk_text) VALUES ('delete', old.rowid, old.id, old.chunk_text);
  INSERT INTO chunks_fts(rowid, id, chunk_text) VALUES (new.rowid, new.id, new.chunk_text);
END";

pub const FTS_SEARCH: &str = "SELECT id, rank FROM chunks_fts WHERE chunk_text MATCH ? ORDER BY rank LIMIT ?;";

pub const GET_CHUNKS_BY_IDS: &str = "
SELECT id, material_id, file_display_name, chunk_index, chunk_text
FROM chunks WHERE id IN ";

pub fn build_material_specific_vector_search(material_count: usize) -> String {
    let placeholders = (0..material_count)
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "SELECT 
            c.id,
            c.material_id,
            c.file_display_name,
            c.chunk_index,
            c.chunk_text,
            vec_chunks.distance
         FROM vec_chunks
         INNER JOIN chunks c ON c.id = vec_chunks.chunk_id
         WHERE vec_chunks.embedding MATCH ?1
         AND c.material_id IN ({}) AND k = ?{}
         ORDER BY vec_chunks.distance",
        placeholders,
        material_count + 2 // +1 for embedding param, +1 for limit param
    )
}
