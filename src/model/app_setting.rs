use sqlx::{sqlite::SqliteRow, Row};

#[derive(Debug, Clone)]
pub(crate) struct AppSetting {
    internal_id: Option<i64>,
    name: String,
    value: String,
}

impl AppSetting {
    pub fn new<T: ToString>(name: &str, value: T) -> Self {
        Self {
            internal_id: None,
            name: name.to_string(),
            value: value.to_string(),
        }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }

    pub(crate) async fn save(&self, db: &sqlx::SqlitePool) -> Option<Self> {
        if self.internal_id.is_some() {
            let sql = "UPDATE app_seting SET value = ? WHERE internal_id = ?";
            if let Err(e) = sqlx::query(sql).bind(&self.value).execute(db).await {
                eprintln!("Could not update APP SETTING record: {:?}", e);
            }
        } else {
            if let Some(mut result) = Self::find_by_name(db, &self.name).await {
                return Self::update(db, &result.name, &result.value).await;
            } else {
                let sql = "INSERT INTO app_setting (name, value) VALUES ( ?, ?)";
                if let Err(e) = sqlx::query(sql)
                    .bind(&self.name)
                    .bind(&self.value)
                    .execute(db)
                    .await
                {
                    eprintln!("Could not insert new APP SETTING record: {:?}", e);
                }
            }
        }
        Self::find_by_name(db, &self.name).await
    }

    pub async fn find_by_name(db: &sqlx::SqlitePool, name: &str) -> Option<Self> {
        let sql = "SELECT * FROM app_setting WHERE name = ? LIMIT 1";
        sqlx::query(&sql)
            .bind(name.to_string())
            .try_map(|r: SqliteRow| Ok(map_result(r)))
            .fetch_one(db)
            .await
            .ok()
    }

    pub async fn get_app_version(db: &sqlx::SqlitePool) -> String {
        if let Some(result) = Self::find_by_name(db, "app_setting").await {
            result.value()
        } else {
            "0.1.0".to_string()
        }
    }

    pub async fn set_app_version(db: &sqlx::SqlitePool, version: &str) -> Option<AppSetting> {
        Self::new("app_version", version).save(db).await
    }

    async fn update<T: ToString>(db: &sqlx::SqlitePool, name: &str, value: T) -> Option<Self> {
        let sql = "UPDATE app_seting SET value = ? WHERE name = ?";
        if let Err(e) = sqlx::query(sql)
            .bind(name)
            .bind(value.to_string())
            .execute(db)
            .await
        {
            eprintln!("Could not update APP SETTING record: {:?}", e);
        }

        Self::find_by_name(db, name).await
    }
}

fn map_result(r: SqliteRow) -> AppSetting {
    AppSetting {
        internal_id: r.get("internal_id"),
        name: r.get("name"),
        value: r.get("value"),
    }
}
