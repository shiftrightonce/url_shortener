use clap::{Parser, Subcommand};
use model::{api::ApiModel, url::UrlModel};
use std::{env, path::PathBuf, str::FromStr, sync::Arc, time::Duration};

mod database;
mod http;
mod model;

#[tokio::main]
async fn main() {
    let config = setup_config();
    busybody::helpers::register_service(config.clone());

    let db = setup_database(&config).await;
    let app_state = Arc::new(AppState {
        db_pool: db,
        config,
    });
    busybody::helpers::register_type(app_state.clone());

    let args = Args::parse();
    match args.command {
        Commands::Token { domain } => {
            let domain = if let Some(d) = domain {
                d
            } else {
                busybody::helpers::get_service::<Config>()
                    .unwrap()
                    .default_domain()
            };
            let pool = app_state.db_pool();
            if let Some(api) = ApiModel::new(&domain).save(&pool).await {
                println!("Token: {}", api.token());
            } else {
                println!("Could not create token for domain : {}", domain);
            }
        }
        Commands::Prune => {
            let pool = app_state.db_pool();
            UrlModel::prune(&pool).await;
        }
        Commands::Serve => {
            let app = http::setup_routers(app_state.clone());

            let listener = tokio::net::TcpListener::bind(app_state.config().listen_address())
                .await
                .unwrap();

            println!("URL Shortener");
            println!("listening on {}", listener.local_addr().unwrap());
            axum::serve(listener, app).await.unwrap();
        }
    }
}

fn setup_config() -> Config {
    let path = PathBuf::new().join("./");
    _ = dotenvy::from_filename(path.join(".env.default"));
    _ = dotenvy::from_filename_override(path.join(".env.prod"));
    _ = dotenvy::from_filename_override(path.join(".env.stage"));
    _ = dotenvy::from_filename_override(path.join(".env"));
    _ = dotenvy::from_filename_override(path.join(".env.dev"));

    Config {
        default_domain: env::var("DEFAULT_DOMAIN")
            .unwrap_or_else(|_| "http://127.0.0.1".to_string()),
        listen_address: env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
        data_dir: env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()),
    }
}

async fn setup_database(config: &Config) -> sqlx::SqlitePool {
    let full_path = format!("{}/database.db", config.data_dir());
    _ = std::fs::create_dir(config.data_dir());

    let options = sqlx::sqlite::SqliteConnectOptions::from_str(&full_path)
        .unwrap()
        .foreign_keys(true)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();

    database::setup::run(&pool).await;

    pool
}

pub(crate) struct AppState {
    db_pool: sqlx::SqlitePool,
    config: Config,
}

impl AppState {
    pub(crate) fn db_pool(&self) -> sqlx::SqlitePool {
        self.db_pool.clone()
    }

    pub(crate) fn config(&self) -> &Config {
        &self.config
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    default_domain: String,
    listen_address: String,
    data_dir: String,
}

impl Config {
    pub(crate) fn default_domain(&self) -> String {
        self.default_domain.clone()
    }

    pub(crate) fn listen_address(&self) -> String {
        self.listen_address.clone()
    }

    pub(crate) fn data_dir(&self) -> String {
        self.data_dir.clone()
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Prune,
    Serve,
    Token {
        #[arg(short, long)]
        domain: Option<String>,
    },
}
