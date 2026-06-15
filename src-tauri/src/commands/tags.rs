use rusqlite::Connection;
use crate::error::AppError;
use crate::models::{Tag, TagInput};

/// Creates a tag if it does not exist, then inserts the snippet_tag join row.
/// INSERT OR IGNORE makes both operations idempotent.
pub fn attach_tags(conn: &Connection, snippet_id: i64, tags: &[TagInput]) -> Result<(), AppError> {
    for tag in tags {
        let name = tag.name.to_lowercase();
        conn.execute(
            "INSERT OR IGNORE INTO tag (name) VALUES (?1)",
            [&name],
        )?;
        let tag_id: i64 = conn.query_row(
            "SELECT id FROM tag WHERE name = ?1",
            [&name],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO snippet_tag (snippet_id, tag_id, source) VALUES (?1, ?2, ?3)",
            rusqlite::params![snippet_id, tag_id, &tag.source],
        )?;
    }
    Ok(())
}

fn db_list_tags(conn: &Connection) -> Result<Vec<Tag>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, count(st.snippet_id) \
         FROM tag t \
         LEFT JOIN snippet_tag st ON st.tag_id = t.id AND st.source != 'suppressed' \
         GROUP BY t.id \
         ORDER BY t.name",
    )?;
    let tags = stmt
        .query_map([], |r| {
            Ok(Tag { id: r.get(0)?, name: r.get(1)?, source: String::new(), count: r.get(2)? })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

#[tauri::command]
pub fn list_tags(
    db: tauri::State<std::sync::Mutex<Connection>>,
) -> Result<Vec<Tag>, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_list_tags(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::apply_migrations;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        apply_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn attach_tags_creates_tag_if_missing() {
        let conn = setup();
        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('t', 'c', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let snippet_id = conn.last_insert_rowid();

        attach_tags(
            &conn,
            snippet_id,
            &[TagInput { name: "rust".to_string(), source: "user".to_string() }],
        )
        .unwrap();

        let tag_count: i64 = conn
            .query_row("SELECT count(*) FROM tag WHERE name='rust'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tag_count, 1);

        let link_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_tag WHERE snippet_id=?1",
                [snippet_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(link_count, 1);
    }

    #[test]
    fn attach_tags_reuses_existing_tag() {
        let conn = setup();
        conn.execute("INSERT INTO tag (name) VALUES ('rust')", []).unwrap();
        let existing_tag_id: i64 = conn
            .query_row("SELECT id FROM tag WHERE name='rust'", [], |r| r.get(0))
            .unwrap();

        conn.execute(
            "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
             VALUES ('t', 'c', 'text', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let snippet_id = conn.last_insert_rowid();

        attach_tags(
            &conn,
            snippet_id,
            &[TagInput { name: "rust".to_string(), source: "system".to_string() }],
        )
        .unwrap();

        let tag_count: i64 = conn
            .query_row("SELECT count(*) FROM tag", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tag_count, 1);

        let linked_id: i64 = conn
            .query_row(
                "SELECT tag_id FROM snippet_tag WHERE snippet_id=?1",
                [snippet_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(linked_id, existing_tag_id);
    }

    #[test]
    fn db_list_tags_returns_all_sorted_by_name() {
        let conn = setup();
        conn.execute("INSERT INTO tag (name) VALUES ('zebra')", []).unwrap();
        conn.execute("INSERT INTO tag (name) VALUES ('apple')", []).unwrap();

        let tags = db_list_tags(&conn).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "apple");
        assert_eq!(tags[1].name, "zebra");
    }
}
