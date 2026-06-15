use std::path::Path;
use rusqlite::Connection;
use crate::error::AppError;

const MIGRATION_1: &str = "
    CREATE TABLE snippet (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        title        TEXT    NOT NULL DEFAULT '',
        content      TEXT    NOT NULL DEFAULT '',
        content_type TEXT    NOT NULL DEFAULT 'text',
        created_at   TEXT    NOT NULL,
        updated_at   TEXT    NOT NULL
    );

    CREATE TABLE tag (
        id   INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE
    );

    CREATE TABLE snippet_tag (
        snippet_id INTEGER NOT NULL REFERENCES snippet(id) ON DELETE CASCADE,
        tag_id     INTEGER NOT NULL REFERENCES tag(id)    ON DELETE CASCADE,
        source     TEXT    NOT NULL DEFAULT 'user',
        PRIMARY KEY (snippet_id, tag_id)
    );

    CREATE VIRTUAL TABLE snippet_fts USING fts5(
        title,
        content,
        content='snippet',
        content_rowid='id'
    );

    CREATE TRIGGER snippet_ai AFTER INSERT ON snippet BEGIN
        INSERT INTO snippet_fts(rowid, title, content)
        VALUES (new.id, new.title, new.content);
    END;

    CREATE TRIGGER snippet_ad AFTER DELETE ON snippet BEGIN
        INSERT INTO snippet_fts(snippet_fts, rowid, title, content)
        VALUES ('delete', old.id, old.title, old.content);
    END;

    CREATE TRIGGER snippet_au AFTER UPDATE ON snippet BEGIN
        INSERT INTO snippet_fts(snippet_fts, rowid, title, content)
        VALUES ('delete', old.id, old.title, old.content);
        INSERT INTO snippet_fts(rowid, title, content)
        VALUES (new.id, new.title, new.content);
    END;
";

const MIGRATION_2: &str = "
    ALTER TABLE snippet ADD COLUMN copy_count   INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE snippet ADD COLUMN last_used_at TEXT;

    -- Deduplicate tags case-insensitively before lowercasing:
    -- 1. Reassign snippet_tag rows that point to a non-canonical (non-min-id) tag
    UPDATE snippet_tag
    SET tag_id = (
        SELECT MIN(t2.id) FROM tag t2
        WHERE LOWER(t2.name) = LOWER((SELECT t3.name FROM tag t3 WHERE t3.id = snippet_tag.tag_id))
    )
    WHERE tag_id NOT IN (SELECT MIN(id) FROM tag GROUP BY LOWER(name));

    -- 2. Delete orphaned duplicates
    DELETE FROM tag WHERE id NOT IN (SELECT MIN(id) FROM tag GROUP BY LOWER(name));

    -- 3. Lowercase all tag names
    UPDATE tag SET name = LOWER(name);
";

/// Open (or create) the database at `path`, enable WAL + foreign keys, run migrations.
pub fn open_db(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    apply_migrations(&conn)?;
    Ok(conn)
}

pub(crate) fn apply_migrations(conn: &Connection) -> Result<(), AppError> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(&format!("BEGIN;\n{MIGRATION_1}\nCOMMIT;"))?;
        conn.execute_batch("PRAGMA user_version = 1;")?;
    }
    if version < 2 {
        conn.execute_batch(&format!("BEGIN;\n{MIGRATION_2}\nCOMMIT;"))?;
        conn.execute_batch("PRAGMA user_version = 2;")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        super::apply_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn migration_sets_user_version_to_2() {
        let conn = setup();
        let ver: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 2);
    }

    #[test]
    fn migration_2_adds_copy_columns() {
        let conn = setup();
        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('t', 'c', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        let (copy_count, last_used_at): (i64, Option<String>) = conn
            .query_row(
                "SELECT copy_count, last_used_at FROM snippet WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(copy_count, 0);
        assert!(last_used_at.is_none());
    }

    #[test]
    fn migration_2_normalizes_tag_case() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        let m1 = super::MIGRATION_1;
        conn.execute_batch(&format!("BEGIN;\n{m1}\nCOMMIT;")).unwrap();
        conn.execute_batch("PRAGMA user_version = 1;").unwrap();
        conn.execute("INSERT INTO tag (name) VALUES ('Rust')", []).unwrap();
        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('t', 'c', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let snippet_id = conn.last_insert_rowid();
        let tag_id: i64 = conn
            .query_row("SELECT id FROM tag WHERE name = 'Rust'", [], |r| r.get(0))
            .unwrap();
        conn.execute(
            "INSERT INTO snippet_tag (snippet_id, tag_id, source) VALUES (?1, ?2, 'user')",
            rusqlite::params![snippet_id, tag_id],
        )
        .unwrap();
        // Now run migration 2
        super::apply_migrations(&conn).unwrap();
        let name: String = conn
            .query_row("SELECT name FROM tag WHERE id = ?1", [tag_id], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "rust");
    }

    #[test]
    fn snippet_and_tag_round_trip() {
        let conn = setup();

        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
            rusqlite::params!["My snippet", "fn main() {}", "code"],
        )
        .unwrap();
        let snippet_id = conn.last_insert_rowid();

        conn.execute("INSERT INTO tag (name) VALUES (?1)", ["rust"])
            .unwrap();
        let tag_id: i64 = conn
            .query_row("SELECT id FROM tag WHERE name = ?1", ["rust"], |r| r.get(0))
            .unwrap();

        conn.execute(
            "INSERT INTO snippet_tag (snippet_id, tag_id, source) VALUES (?1, ?2, 'user')",
            rusqlite::params![snippet_id, tag_id],
        )
        .unwrap();

        let title: String = conn
            .query_row(
                "SELECT title FROM snippet WHERE id = ?1",
                [snippet_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title, "My snippet");

        let tag_name: String = conn
            .query_row(
                "SELECT t.name FROM tag t \
                 JOIN snippet_tag st ON st.tag_id = t.id \
                 WHERE st.snippet_id = ?1",
                [snippet_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tag_name, "rust");
    }

    #[test]
    fn fts5_indexes_on_insert() {
        let conn = setup();

        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('Hello World', 'fn greet() {}', 'code', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_fts WHERE snippet_fts MATCH 'Hello'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn fts5_deindexes_on_delete() {
        let conn = setup();

        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('Delete me', 'ephemeral', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();

        let before: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_fts WHERE snippet_fts MATCH 'Delete'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(before, 1);

        conn.execute("DELETE FROM snippet WHERE id = ?1", [id])
            .unwrap();

        let after: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_fts WHERE snippet_fts MATCH 'Delete'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(after, 0);
    }

    #[test]
    fn fts5_reindexes_on_update() {
        let conn = setup();

        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('Old title', 'body', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();

        conn.execute(
            "UPDATE snippet SET title = 'New title', updated_at = datetime('now') WHERE id = ?1",
            [id],
        )
        .unwrap();

        let old_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_fts WHERE snippet_fts MATCH 'Old'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(old_count, 0);

        let new_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_fts WHERE snippet_fts MATCH 'New'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(new_count, 1);
    }

    #[test]
    fn snippet_tag_cascade_deletes_with_snippet() {
        let conn = setup();

        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('t', 'c', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let snippet_id = conn.last_insert_rowid();

        conn.execute("INSERT INTO tag (name) VALUES ('x')", [])
            .unwrap();
        let tag_id: i64 = conn
            .query_row("SELECT id FROM tag WHERE name = 'x'", [], |r| r.get(0))
            .unwrap();

        conn.execute(
            "INSERT INTO snippet_tag (snippet_id, tag_id, source) VALUES (?1, ?2, 'user')",
            rusqlite::params![snippet_id, tag_id],
        )
        .unwrap();

        conn.execute("DELETE FROM snippet WHERE id = ?1", [snippet_id])
            .unwrap();

        let orphan_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_tag WHERE snippet_id = ?1",
                [snippet_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(orphan_count, 0, "snippet_tag rows must be cascade-deleted");
    }
}
