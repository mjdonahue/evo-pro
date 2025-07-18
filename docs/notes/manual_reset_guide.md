# Manual Database Reset Guide

## Current Issue
Your SQLite schema has syntax errors that prevent automatic migration. Here's how to reset it manually:

## Option 1: Quick Reset (Recommended)
1. **Stop your Tauri application** if it's running
2. **Delete existing database files:**
   ```bash
   rm -f ~/Library/Application\ Support/app.evo-design.com/data.db*
   ```
3. **Run your Tauri application** - it will recreate the database automatically using the built-in migration logic that handles version mismatches

## Option 2: Manual Schema Fix
If you want to fix the schema file, the main issues are:
- Line 1237: Extra spaces in settings table
- Missing owner_id column in documents table  
- Trailing commas before closing parentheses
- Non-existent column references in indexes

## Option 3: Use SQLx CLI
If you have sqlx-cli installed:
```bash
cd src-tauri
sqlx database drop --database-url "sqlite:///Users/michael/Library/Application Support/app.evo-design.com/data.db"
sqlx database create --database-url "sqlite:///Users/michael/Library/Application Support/app.evo-design.com/data.db"
sqlx migrate run --database-url "sqlite:///Users/michael/Library/Application Support/app.evo-design.com/data.db"
```

## Verification
After reset, verify the database was created:
```bash
ls -la ~/Library/Application\ Support/app.evo-design.com/
```

## Next Steps
1. Try Option 1 first (simplest)
2. If that doesn't work, we can fix the schema file systematically
3. The Tauri application has built-in logic to handle migration version mismatches by recreating the database

## Database Location
- **macOS**: `~/Library/Application Support/app.evo-design.com/data.db`
- **Linux**: `~/.local/share/app.evo-design.com/data.db`
- **Windows**: `%APPDATA%\app.evo-design.com\data.db`
