use std::sync::Mutex;
use rusqlite::Connection;
use tauri::State;
use crate::error::AppError;
use crate::models::{ContentType, Snippet, Tag, TagInput};
use crate::commands::tags::attach_tags;

// ── Private DB helpers ───────────────────────────────────────────────────────

fn load_tags(conn: &Connection, snippet_id: i64) -> Result<Vec<Tag>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, st.source \
         FROM tag t \
         JOIN snippet_tag st ON st.tag_id = t.id \
         WHERE st.snippet_id = ?1 \
         ORDER BY t.name",
    )?;
    let tags = stmt
        .query_map([snippet_id], |r| {
            Ok(Tag { id: r.get(0)?, name: r.get(1)?, source: r.get(2)?, count: 0 })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

fn load_snippet(conn: &Connection, id: i64) -> Result<Snippet, AppError> {
    let mut snippet = conn.query_row(
        "SELECT id, title, content, content_type, copy_count, created_at, updated_at \
         FROM snippet WHERE id = ?1",
        [id],
        |r| {
            let ct: String = r.get(3)?;
            Ok(Snippet {
                id: r.get(0)?,
                title: r.get(1)?,
                content: r.get(2)?,
                content_type: ct.parse::<ContentType>().unwrap_or(ContentType::Text),
                copy_count: r.get(4)?,
                created_at: r.get(5)?,
                updated_at: r.get(6)?,
                tags: vec![],
            })
        },
    )?;
    snippet.tags = load_tags(conn, snippet.id)?;
    Ok(snippet)
}

fn db_create_snippet(
    conn: &Connection,
    title: &str,
    content: &str,
    content_type: &str,
    tags: &[TagInput],
) -> Result<Snippet, AppError> {
    conn.execute(
        "INSERT INTO snippet (title, content, content_type, created_at, updated_at) \
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
        rusqlite::params![title, content, content_type],
    )?;
    let id = conn.last_insert_rowid();
    attach_tags(conn, id, tags)?;
    load_snippet(conn, id)
}

fn load_snippets_for_ids(conn: &Connection, ids: &[i64]) -> Result<Vec<Snippet>, AppError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: String = std::iter::repeat("?")
        .take(ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let snippet_sql = format!(
        "SELECT id, title, content, content_type, copy_count, created_at, updated_at \
         FROM snippet WHERE id IN ({placeholders})"
    );
    let mut stmt = conn.prepare(&snippet_sql)?;
    let mut map: std::collections::HashMap<i64, Snippet> = stmt
        .query_map(rusqlite::params_from_iter(ids.iter().copied()), |r| {
            let ct: String = r.get(3)?;
            Ok(Snippet {
                id: r.get(0)?,
                title: r.get(1)?,
                content: r.get(2)?,
                content_type: ct.parse::<ContentType>().unwrap_or(ContentType::Text),
                copy_count: r.get(4)?,
                created_at: r.get(5)?,
                updated_at: r.get(6)?,
                tags: vec![],
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|s| (s.id, s))
        .collect();
    let tag_sql = format!(
        "SELECT st.snippet_id, t.id, t.name, st.source \
         FROM tag t JOIN snippet_tag st ON st.tag_id = t.id \
         WHERE st.snippet_id IN ({placeholders}) \
         ORDER BY t.name"
    );
    let mut tag_stmt = conn.prepare(&tag_sql)?;
    let tag_rows: Vec<(i64, Tag)> = tag_stmt
        .query_map(rusqlite::params_from_iter(ids.iter().copied()), |r| {
            Ok((r.get::<_, i64>(0)?, Tag { id: r.get(1)?, name: r.get(2)?, source: r.get(3)?, count: 0 }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (snippet_id, tag) in tag_rows {
        if let Some(s) = map.get_mut(&snippet_id) {
            s.tags.push(tag);
        }
    }
    Ok(ids.iter().filter_map(|id| map.remove(id)).collect())
}

fn db_list_snippets(
    conn: &Connection,
    tag_filter: Option<&str>,
) -> Result<Vec<Snippet>, AppError> {
    let ids: Vec<i64> = match tag_filter {
        None => {
            let mut stmt = conn
                .prepare("SELECT id FROM snippet ORDER BY copy_count DESC, created_at DESC, id DESC")?;
            let rows = stmt.query_map([], |r| r.get(0))?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        }
        Some(tag) => {
            let mut stmt = conn.prepare(
                "SELECT s.id FROM snippet s \
                 JOIN snippet_tag st ON st.snippet_id = s.id \
                 JOIN tag t ON t.id = st.tag_id \
                 WHERE t.name = ?1 AND st.source != 'suppressed' \
                 ORDER BY s.copy_count DESC, s.created_at DESC, s.id DESC",
            )?;
            let rows = stmt.query_map([tag], |r| r.get(0))?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        }
    };
    load_snippets_for_ids(conn, &ids)
}

fn db_get_snippet(conn: &Connection, id: i64) -> Result<Snippet, AppError> {
    load_snippet(conn, id)
}

fn db_update_snippet(
    conn: &Connection,
    id: i64,
    title: &str,
    content: &str,
    content_type: &str,
    tags: &[TagInput],
) -> Result<Snippet, AppError> {
    let rows = conn.execute(
        "UPDATE snippet SET title=?1, content=?2, content_type=?3, \
         updated_at=datetime('now') WHERE id=?4",
        rusqlite::params![title, content, content_type, id],
    )?;
    if rows == 0 {
        return Err(AppError::Other(format!("snippet {id} not found")));
    }
    conn.execute("DELETE FROM snippet_tag WHERE snippet_id=?1", [id])?;
    attach_tags(conn, id, tags)?;
    load_snippet(conn, id)
}

fn db_delete_snippet(conn: &Connection, id: i64) -> Result<(), AppError> {
    let rows = conn.execute("DELETE FROM snippet WHERE id=?1", [id])?;
    if rows == 0 {
        return Err(AppError::Other(format!("snippet {id} not found")));
    }
    Ok(())
}

fn fts_prefix_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| {
            if token.ends_with('*') { token.to_string() } else { format!("{token}*") }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn db_search_snippets(conn: &Connection, query: &str) -> Result<Vec<Snippet>, AppError> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let fts_query = fts_prefix_query(query);
    let mut stmt = conn.prepare(
        "SELECT s.id \
         FROM snippet s \
         JOIN snippet_fts ON snippet_fts.rowid = s.id \
         WHERE snippet_fts MATCH ?1 \
         ORDER BY bm25(snippet_fts)",
    )?;
    let ids: Vec<i64> = stmt
        .query_map([fts_query], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    load_snippets_for_ids(conn, &ids)
}

fn db_list_snippets_by_period(
    conn: &Connection,
    period: &str,
) -> Result<Vec<Snippet>, AppError> {
    let date_filter = match period {
        "today"     => "date(created_at, 'localtime') = date('now', 'localtime')",
        "yesterday" => "date(created_at, 'localtime') = date('now', 'localtime', '-1 day')",
        "this-week" => "date(created_at, 'localtime') >= date('now', 'localtime', '-6 days') AND date(created_at, 'localtime') < date('now', 'localtime', '-1 day')",
        "last-week" => "date(created_at, 'localtime') >= date('now', 'localtime', '-13 days') AND date(created_at, 'localtime') < date('now', 'localtime', '-6 days')",
        "older"     => "date(created_at, 'localtime') < date('now', 'localtime', '-13 days')",
        _           => return Err(AppError::Other(format!("unknown period: {period}"))),
    };
    let sql = format!(
        "SELECT id FROM snippet WHERE {} ORDER BY created_at DESC, id DESC",
        date_filter
    );
    let mut stmt = conn.prepare(&sql)?;
    let ids: Vec<i64> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    load_snippets_for_ids(conn, &ids)
}

fn db_record_copy(conn: &Connection, id: i64) -> Result<(), AppError> {
    let rows = conn.execute(
        "UPDATE snippet SET copy_count = copy_count + 1, last_used_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    if rows == 0 {
        return Err(AppError::Other(format!("snippet {id} not found")));
    }
    Ok(())
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_snippet(
    db: State<Mutex<Connection>>,
    title: String,
    content: String,
    content_type: String,
    tags: Vec<TagInput>,
) -> Result<Snippet, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_create_snippet(&conn, &title, &content, &content_type, &tags)
}

#[tauri::command]
pub fn list_snippets(
    db: State<Mutex<Connection>>,
    tag_filter: Option<String>,
) -> Result<Vec<Snippet>, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_list_snippets(&conn, tag_filter.as_deref())
}

#[tauri::command]
pub fn get_snippet(db: State<Mutex<Connection>>, id: i64) -> Result<Snippet, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_get_snippet(&conn, id)
}

#[tauri::command]
pub fn update_snippet(
    db: State<Mutex<Connection>>,
    id: i64,
    title: String,
    content: String,
    content_type: String,
    tags: Vec<TagInput>,
) -> Result<Snippet, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_update_snippet(&conn, id, &title, &content, &content_type, &tags)
}

#[tauri::command]
pub fn delete_snippet(db: State<Mutex<Connection>>, id: i64) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_delete_snippet(&conn, id)
}

#[tauri::command]
pub fn search_snippets(
    db: State<Mutex<Connection>>,
    query: String,
) -> Result<Vec<Snippet>, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_search_snippets(&conn, &query)
}

#[tauri::command]
pub fn list_snippets_by_period(
    db: State<Mutex<Connection>>,
    period: String,
) -> Result<Vec<Snippet>, AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_list_snippets_by_period(&conn, &period)
}

#[tauri::command]
pub fn record_copy(db: State<Mutex<Connection>>, id: i64) -> Result<(), AppError> {
    let conn = db.lock().map_err(|e| AppError::Other(e.to_string()))?;
    db_record_copy(&conn, id)
}

// ── Tests ────────────────────────────────────────────────────────────────────

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

    fn tag(name: &str, source: &str) -> TagInput {
        TagInput { name: name.to_string(), source: source.to_string() }
    }

    #[test]
    fn create_snippet_returns_snippet_with_tags() {
        let conn = setup();
        let s = db_create_snippet(
            &conn, "My title", "fn main() {}", "code", &[tag("rust", "user")],
        )
        .unwrap();
        assert_eq!(s.title, "My title");
        assert_eq!(s.content_type, ContentType::Code);
        assert_eq!(s.tags.len(), 1);
        assert_eq!(s.tags[0].name, "rust");
        assert_eq!(s.tags[0].source, "user");
    }

    #[test]
    fn create_snippet_no_tags() {
        let conn = setup();
        let s = db_create_snippet(&conn, "Bare", "bare content", "text", &[]).unwrap();
        assert!(s.tags.is_empty());
    }

    #[test]
    fn list_snippets_ordered_newest_first() {
        let conn = setup();
        let s1 = db_create_snippet(&conn, "First", "a", "text", &[]).unwrap();
        let s2 = db_create_snippet(&conn, "Second", "b", "text", &[]).unwrap();
        let snippets = db_list_snippets(&conn, None).unwrap();
        assert_eq!(snippets.len(), 2);
        assert_eq!(snippets[0].id, s2.id, "newer snippet must come first");
        assert_eq!(snippets[1].id, s1.id);
    }

    #[test]
    fn list_snippets_filtered_by_tag() {
        let conn = setup();
        db_create_snippet(&conn, "Rust snippet", "code", "code", &[tag("rust", "user")]).unwrap();
        db_create_snippet(&conn, "Plain snippet", "text", "text", &[]).unwrap();
        let snippets = db_list_snippets(&conn, Some("rust")).unwrap();
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].title, "Rust snippet");
    }

    #[test]
    fn get_snippet_returns_with_tags() {
        let conn = setup();
        let created = db_create_snippet(
            &conn, "JS snippet", "const x=1", "code", &[tag("js", "system")],
        )
        .unwrap();
        let fetched = db_get_snippet(&conn, created.id).unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.tags[0].source, "system");
    }

    #[test]
    fn update_snippet_replaces_tags() {
        let conn = setup();
        let original = db_create_snippet(
            &conn, "Title", "content", "code", &[tag("rust", "user")],
        )
        .unwrap();
        let updated = db_update_snippet(
            &conn, original.id, "New title", "new", "text", &[tag("python", "user")],
        )
        .unwrap();
        assert_eq!(updated.title, "New title");
        assert_eq!(updated.content_type, ContentType::Text);
        assert_eq!(updated.tags.len(), 1);
        assert_eq!(updated.tags[0].name, "python");
    }

    #[test]
    fn delete_snippet_removes_row() {
        let conn = setup();
        let s = db_create_snippet(&conn, "To delete", "x", "text", &[]).unwrap();
        db_delete_snippet(&conn, s.id).unwrap();
        assert!(db_list_snippets(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn delete_snippet_removes_snippet_tag_rows() {
        let conn = setup();
        let s = db_create_snippet(&conn, "Tagged", "x", "text", &[tag("go", "user")]).unwrap();
        db_delete_snippet(&conn, s.id).unwrap();
        let orphans: i64 = conn
            .query_row(
                "SELECT count(*) FROM snippet_tag WHERE snippet_id=?1",
                [s.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(orphans, 0, "cascade must remove snippet_tag rows");
    }

    #[test]
    fn search_snippets_matches_title() {
        let conn = setup();
        db_create_snippet(&conn, "Rust async tips", "use tokio::runtime", "code", &[]).unwrap();
        db_create_snippet(&conn, "Python basics", "print('hello')", "code", &[]).unwrap();
        let results = db_search_snippets(&conn, "Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust async tips");
    }

    #[test]
    fn search_snippets_matches_content() {
        let conn = setup();
        db_create_snippet(&conn, "Untitled", "SELECT * FROM users", "code", &[]).unwrap();
        db_create_snippet(&conn, "Other", "print('hello')", "code", &[]).unwrap();
        let results = db_search_snippets(&conn, "SELECT").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "SELECT * FROM users");
    }

    #[test]
    fn search_snippets_prefix_matches_partial_word() {
        let conn = setup();
        db_create_snippet(&conn, "Website tips", "visit the website", "text", &[]).unwrap();
        db_create_snippet(&conn, "Other", "unrelated content", "text", &[]).unwrap();
        let results = db_search_snippets(&conn, "web").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Website tips");
    }

    #[test]
    fn search_snippets_empty_query_returns_empty() {
        let conn = setup();
        db_create_snippet(&conn, "Some snippet", "content here", "text", &[]).unwrap();
        let results = db_search_snippets(&conn, "").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn list_snippets_by_period_today() {
        let conn = setup();
        db_create_snippet(&conn, "Today snippet", "x", "text", &[]).unwrap();
        let results = db_list_snippets_by_period(&conn, "today").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Today snippet");
    }

    #[test]
    fn list_snippets_by_period_unknown_period_errors() {
        let conn = setup();
        let result = db_list_snippets_by_period(&conn, "bogus");
        assert!(result.is_err());
    }

    #[test]
    fn record_copy_increments_count() {
        let conn = setup();
        let s = db_create_snippet(&conn, "t", "c", "text", &[]).unwrap();
        db_record_copy(&conn, s.id).unwrap();
        db_record_copy(&conn, s.id).unwrap();
        let (count, last_used): (i64, Option<String>) = conn
            .query_row(
                "SELECT copy_count, last_used_at FROM snippet WHERE id = ?1",
                [s.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(count, 2);
        assert!(last_used.is_some());
    }

    #[test]
    fn record_copy_unknown_id_errors() {
        let conn = setup();
        assert!(db_record_copy(&conn, 9999).is_err());
    }

    #[test]
    fn load_snippets_for_ids_batches_tags() {
        let conn = setup();
        let s1 = db_create_snippet(&conn, "A", "a", "text", &[tag("go", "user")]).unwrap();
        let s2 = db_create_snippet(&conn, "B", "b", "code", &[tag("rust", "user"), tag("async", "user")]).unwrap();
        let result = load_snippets_for_ids(&conn, &[s1.id, s2.id]).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, s1.id);
        assert_eq!(result[0].tags.len(), 1);
        assert_eq!(result[1].id, s2.id);
        assert_eq!(result[1].tags.len(), 2);
    }

    #[test]
    fn load_snippets_for_ids_empty_returns_empty() {
        let conn = setup();
        let result = load_snippets_for_ids(&conn, &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn tag_filter_excludes_suppressed_snippets() {
        let conn = setup();
        let s = db_create_snippet(&conn, "Tagged", "x", "text", &[
            tag("rust", "suppressed"),
        ]).unwrap();
        // Suppressed tag should not match the filter
        let results = db_list_snippets(&conn, Some("rust")).unwrap();
        assert!(!results.iter().any(|r| r.id == s.id));
    }
}
