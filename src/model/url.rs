#![allow(dead_code)]

use sqlx::{sqlite::SqliteRow, Row};

use super::{generate_short, generate_ulid, sha256_hash};

#[derive(Debug, Clone, Default)]
pub(crate) struct UrlModel {
    internal_id: Option<i64>,
    id: String,
    hash: String,
    expires: i64,
    raw: String,
    short: String,
}

impl UrlModel {
    pub(crate) async fn save(&self, db: &sqlx::SqlitePool) -> Option<Self> {
        let result = Self::find_one_by_hash(&self.hash, db).await;
        if result.is_some() {
            return result;
        }

        let mut short = generate_short();
        while Self::find_one_by_short(&short, db).await.is_some() {
            short = generate_short();
        }

        let sql = "INSERT INTO url (id, hash, expires, raw, short) VALUES (?, ?, ?, ?, ?)";
        if let Err(e) = sqlx::query(sql)
            .bind(&self.id)
            .bind(&self.hash)
            .bind(self.expires)
            .bind(&self.raw)
            .bind(&short)
            .execute(db)
            .await
        {
            eprintln!("Could not insert new URL record: {:?}", e);
        }

        Self::find_one_by("id", &self.id, db).await
    }

    pub(crate) fn new(raw: &str, expires: i64) -> Self {
        Self {
            internal_id: None,
            id: generate_ulid(),
            hash: sha256_hash(raw),
            raw: raw.to_string(),
            expires,
            short: "".into(),
        }
    }

    pub(crate) fn id(&self) -> String {
        self.id.clone()
    }

    pub(crate) fn hash(&self) -> String {
        self.hash.clone()
    }

    pub(crate) fn raw(&self) -> String {
        self.raw.clone()
    }

    pub(crate) fn expires(&self) -> i64 {
        self.expires
    }

    pub(crate) fn short(&self) -> String {
        self.short.clone()
    }

    pub(crate) async fn find_one_by_short(short: &str, db: &sqlx::SqlitePool) -> Option<Self> {
        Self::find_one_by("short", short, db).await
    }

    pub(crate) async fn find_one_by_hash(hash: &str, db: &sqlx::SqlitePool) -> Option<Self> {
        Self::find_one_by("hash", hash, db).await
    }

    pub(crate) async fn find_one_by(
        field: &str,
        value: impl ToString,
        db: &sqlx::SqlitePool,
    ) -> Option<Self> {
        let sql = format!("SELECT * FROM url WHERE {:?} = ? LIMIT 1", field);
        sqlx::query(&sql)
            .bind(value.to_string())
            .try_map(|r: SqliteRow| Ok(map_result(r)))
            .fetch_one(db)
            .await
            .ok()
    }

    pub(crate) async fn prune(db: &sqlx::SqlitePool) {
        let ts = chrono::Utc::now().timestamp_millis();
        let sql = "DELETE FROM url WHERE expires > 0 AND expires < ?";
        if let Err(e) = sqlx::query(sql).bind(ts).execute(db).await {
            eprintln!("could not prune expired URLs: {}", e);
        }
    }
}

fn map_result(r: SqliteRow) -> UrlModel {
    UrlModel {
        internal_id: r.get("internal_id"),
        id: r.get("id"),
        hash: r.get("hash"),
        expires: r.get("expires"),
        raw: r.get("raw"),
        short: r.get("short"),
    }
}
