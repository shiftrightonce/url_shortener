pub(crate) async fn run(db: &sqlx::SqlitePool) {
    let sql = r#"
  CREATE TABLE IF NOT EXISTS "api" (
	"internal_id"	INTEGER,
	"id"	TEXT UNIQUE NOT NULL,
	"token"	TEXT UNIQUE NOT NULL,
	"domain" TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);

CREATE TABLE IF NOT EXISTS  "url"  (
	"internal_id"	INTEGER,
	"id"	TEXT UNIQUE NOT NULL,
	"hash"	TEXT UNIQUE NOT NULL,
	"short"	TEXT UNIQUE NOT NULL,
	"expires"	INTEGER,
	"raw"	TEXT NOT NULL,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);
  "#;

    if let Err(e) = sqlx::query(sql).execute(db).await {
        println!("could not create tables: {:?}", e)
    }
}
