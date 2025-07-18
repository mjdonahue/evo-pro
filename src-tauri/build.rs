use color_eyre::{Result, eyre::eyre};
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};
use std::{
    env::{self, temp_dir},
    io::ErrorKind,
    path::PathBuf,
    str::FromStr,
};
use tauri_specta::ts;

/// Build file for migration scripts to ensure that the compile-time
/// queries are compatible with the latest migration scripts
/// Generate TypeScript interfaces from Rust types annotated with specta::Type
fn generate_typescript_interfaces() -> Result<()> {
    println!("cargo:rerun-if-changed=src/entities-new");

    // Create the output directory if it doesn't exist
    std::fs::create_dir_all("../src/bindings/generated")?;

    // Export types from entities-new
    ts::export(
        specta::collect_types![
            crate::entities_new::conversations::Conversation,
            crate::entities_new::conversations::ConversationType,
            crate::entities_new::conversations::ConversationStatus,
            crate::entities_new::conversations::ConversationFilter,
            crate::entities_new::messages::Message,
            crate::entities_new::messages::MessageStatus,
            crate::entities_new::messages::MessageType,
            crate::entities_new::messages::ContentType,
            crate::entities_new::messages::SenderType,
            crate::entities_new::messages::MessageFilter,
            // Add more types as needed
        ],
        "../src/bindings/generated/index.ts",
    )?;

    Ok(())
}

async fn run_migrations() -> Result<()> {
    // Attempt to load .env file. This is fine if it doesn't exist.
    dotenvy::dotenv().ok();

    // Get DATABASE_URL from environment, then .env, then fallback
    let db_file = match env::var("DATABASE_URL") {
        Ok(k) => k,
        Err(_) => {
            let default_db_path = temp_dir().join("evo-core-build.db");
            let mut db_url = default_db_path.display().to_string();
            println!("cargo:warning=DATABASE_URL not found in .env, using default: {db_url}");
            // Calling `display` on a Windows path will produce a string with backslashes
            // as path separators, which is not valid for a SQLite URL. So we need to replace them with forward slashes
            // We also need to add a leading slash to the path since Windows drive letters do not
            // have a leading slash
            if cfg!(windows) {
                db_url = format!("/{}", db_url.replace('\\', "/"));
            }
            format!("sqlite://{db_url}")
        }
    };

    println!("cargo:rustc-env=DATABASE_URL={db_file}"); // Make it available to the crate

    let db_path = PathBuf::from(if let Some(path) = db_file.strip_prefix("file:") {
        path
    } else if let Some(path) = db_file.strip_prefix("sqlite:") {
        path
    } else {
        // If no prefix, assume it's a raw path, which sqlx treats as sqlite:
        db_file.as_str()
    });

    match std::fs::remove_file(&db_path) {
        // Only return an error if it doesn't talk about the file not existing
        // since this likely means that this is the first time the database is being created
        Err(e) if e.kind() != ErrorKind::NotFound => {
            return Err(eyre!(e));
        }
        _ => {}
    }

    let pool: Pool<Sqlite> = Pool::connect_lazy_with(
        SqliteConnectOptions::from_str(&db_file)?
            .foreign_keys(false) // Disable during migration due to table creation order
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            // Only use NORMAL if WAL mode is enabled
            // as it provides extra performance benefits
            // at the cost of durability
            .synchronous(SqliteSynchronous::Normal),
    );
    if cfg!(debug_assertions) {
        sqlx::migrate!("./seeding").run(&pool).await?;
    } else {
        sqlx::migrate!("./migrations").run(&pool).await?;
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Rebuild if any of these files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/napi.rs");
    println!("cargo:rerun-if-changed=src/uniffi.rs");
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=src/entities-new");

    // Run database migrations
    tokio::try_join!(run_migrations())?;

    // Generate TypeScript interfaces
    generate_typescript_interfaces()?;

    Ok(())
}
