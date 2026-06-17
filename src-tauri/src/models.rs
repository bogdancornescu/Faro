use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Code,
    Cli,
    Text,
    Url,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Code => write!(f, "code"),
            ContentType::Cli => write!(f, "cli"),
            ContentType::Text => write!(f, "text"),
            ContentType::Url => write!(f, "url"),
        }
    }
}

impl std::str::FromStr for ContentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(ContentType::Code),
            "cli" => Ok(ContentType::Cli),
            "text" => Ok(ContentType::Text),
            "url" => Ok(ContentType::Url),
            other => Err(format!("unknown content_type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub copy_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<Tag>,
}

/// A tag, optionally carrying its `source` from the `snippet_tag` join table.
/// `source` is always populated when a tag is loaded through a snippet;
/// it is an empty string when listing all tags without snippet context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagInput {
    pub name: String,
    pub source: String,
}
