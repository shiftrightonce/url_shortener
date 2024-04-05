use sqlx::{sqlite::SqliteRow, Row};

use super::generate_ulid;

#[derive(Debug, Clone, Default)]
pub(crate) struct ApiModel {
    internal_id: Option<i64>,
    id: String,
    token: String,
    domain: String,
}

impl ApiModel {
    pub(crate) fn token(&self) -> String {
        format!("{}|{}", &self.id, &self.token)
    }

    pub(crate) fn domain(&self) -> String {
        self.domain.clone()
    }

    pub(crate) async fn save(&self, db: &sqlx::SqlitePool) -> Option<Self> {
        if self.internal_id.is_some() {
            let sql = "UPDATE api SET token = ? WHERE internal_id = ?";
            if let Err(e) = sqlx::query(sql).bind(&self.token).execute(db).await {
                eprintln!("Could not update API record: {:?}", e);
            }
            Self::find_by_internal_id(self.internal_id.unwrap(), db).await
        } else {
            let sql = "INSERT INTO api (id, token, domain) VALUES (?, ?, ?)";
            if let Err(e) = sqlx::query(sql)
                .bind(&self.id)
                .bind(&self.token)
                .bind(&self.domain)
                .execute(db)
                .await
            {
                eprintln!("Could not insert new API record: {:?}", e);
            }
            Self::find_by_id(&self.id, db).await
        }
    }

    pub(crate) async fn find_by_internal_id(id: i64, db: &sqlx::SqlitePool) -> Option<Self> {
        Self::find_one_by("internal_id", id, db).await
    }
    pub(crate) async fn find_by_id(id: &str, db: &sqlx::SqlitePool) -> Option<Self> {
        Self::find_one_by("id", id, db).await
    }

    pub(crate) async fn find_by_request_token(rt: &str, db: &sqlx::SqlitePool) -> Option<Self> {
        let mut split = rt.split('|');
        let id = split.next().unwrap_or_default();
        let token = split.next().unwrap_or_default();

        if id.is_empty() || token.is_empty() {
            return None;
        }

        let sql = "SELECT * FROM api WHERE id = ? AND token = ? LIMIT 1;";

        sqlx::query(sql)
            .bind(id)
            .bind(token)
            .try_map(|r: SqliteRow| Ok(map_result(r)))
            .fetch_one(db)
            .await
            .ok()
    }

    pub(crate) async fn find_one_by(
        field: &str,
        value: impl ToString,
        db: &sqlx::SqlitePool,
    ) -> Option<Self> {
        let sql = format!("SELECT * FROM api WHERE {:?} = ? LIMIT 1", field);
        sqlx::query(&sql)
            .bind(value.to_string())
            .try_map(|r: SqliteRow| Ok(map_result(r)))
            .fetch_one(db)
            .await
            .ok()
    }

    pub(crate) fn new(domain: &str) -> Self {
        let id = generate_ulid();
        let subject = format!("{:?}|{:?}", &id, generate_ulid());
        let token = super::sha256_hash(&subject);
        Self {
            id,
            token,
            domain: domain.to_string(),
            ..Default::default()
        }
    }
}

fn map_result(r: SqliteRow) -> ApiModel {
    ApiModel {
        internal_id: r.get("internal_id"),
        id: r.get("id"),
        token: r.get("token"),
        domain: r.get("domain"),
    }
}
