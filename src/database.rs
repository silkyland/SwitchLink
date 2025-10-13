/// SQLite database for file management
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct GameFile {
    pub id: i64,
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub hash: Option<String>,
    pub added_at: i64,
    pub last_modified: i64,
    pub install_count: i32,
    pub last_installed: Option<i64>,
    pub favorite: bool,
    pub tags: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open database
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                size INTEGER NOT NULL,
                hash TEXT,
                added_at INTEGER NOT NULL,
                last_modified INTEGER NOT NULL,
                install_count INTEGER DEFAULT 0,
                last_installed INTEGER,
                favorite INTEGER DEFAULT 0,
                tags TEXT DEFAULT ''
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_filename ON files(filename)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_favorite ON files(favorite)",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Add a file to database
    pub fn add_file(&self, path: &Path) -> Result<i64> {
        let metadata = std::fs::metadata(path)?;
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let path_str = path.to_string_lossy().to_string();
        let size = metadata.len();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;

        let last_modified = metadata
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;

        // Insert or update
        self.conn.execute(
            "INSERT INTO files (filename, path, size, added_at, last_modified)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(path) DO UPDATE SET
                size = excluded.size,
                last_modified = excluded.last_modified",
            params![filename, path_str, size as i64, now, last_modified],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Add files from directory recursively
    pub fn add_directory(&self, dir: &Path, extensions: &[&str]) -> Result<usize> {
        let mut count = 0;

        fn scan_dir(
            db: &Database,
            dir: &Path,
            extensions: &[&str],
            count: &mut usize,
        ) -> Result<()> {
            if !dir.is_dir() {
                return Ok(());
            }

            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Recursive scan
                    scan_dir(db, &path, extensions, count)?;
                } else if path.is_file() {
                    // Check extension
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if extensions.iter().any(|&e| e == ext_str) {
                            db.add_file(&path)?;
                            *count += 1;
                        }
                    }
                }
            }

            Ok(())
        }

        scan_dir(self, dir, extensions, &mut count)?;
        Ok(count)
    }

    /// Get all files
    pub fn get_files(&self) -> Result<Vec<GameFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, path, size, hash, added_at, last_modified,
                    install_count, last_installed, favorite, tags
             FROM files
             ORDER BY added_at DESC",
        )?;

        let files = stmt
            .query_map([], |row| {
                Ok(GameFile {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    path: row.get(2)?,
                    size: row.get::<_, i64>(3)? as u64,
                    hash: row.get(4)?,
                    added_at: row.get(5)?,
                    last_modified: row.get(6)?,
                    install_count: row.get(7)?,
                    last_installed: row.get(8)?,
                    favorite: row.get::<_, i32>(9)? != 0,
                    tags: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// Get favorites
    pub fn get_favorites(&self) -> Result<Vec<GameFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, path, size, hash, added_at, last_modified,
                    install_count, last_installed, favorite, tags
             FROM files
             WHERE favorite = 1
             ORDER BY added_at DESC",
        )?;

        let files = stmt
            .query_map([], |row| {
                Ok(GameFile {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    path: row.get(2)?,
                    size: row.get::<_, i64>(3)? as u64,
                    hash: row.get(4)?,
                    added_at: row.get(5)?,
                    last_modified: row.get(6)?,
                    install_count: row.get(7)?,
                    last_installed: row.get(8)?,
                    favorite: row.get::<_, i32>(9)? != 0,
                    tags: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// Search files
    pub fn search(&self, query: &str) -> Result<Vec<GameFile>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, path, size, hash, added_at, last_modified,
                    install_count, last_installed, favorite, tags
             FROM files
             WHERE filename LIKE ?1 OR tags LIKE ?1
             ORDER BY favorite DESC, install_count DESC",
        )?;

        let files = stmt
            .query_map([pattern], |row| {
                Ok(GameFile {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    path: row.get(2)?,
                    size: row.get::<_, i64>(3)? as u64,
                    hash: row.get(4)?,
                    added_at: row.get(5)?,
                    last_modified: row.get(6)?,
                    install_count: row.get(7)?,
                    last_installed: row.get(8)?,
                    favorite: row.get::<_, i32>(9)? != 0,
                    tags: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// Remove file
    pub fn remove_file(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM files WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Toggle favorite
    pub fn toggle_favorite(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE files SET favorite = NOT favorite WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Add tag
    pub fn add_tag(&self, id: i64, tag: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE files SET tags = tags || ?1 || ',' WHERE id = ?2",
            params![tag, id],
        )?;
        Ok(())
    }

    /// Record installation
    pub fn record_install(&self, id: i64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "UPDATE files SET 
                install_count = install_count + 1,
                last_installed = ?1
             WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<(usize, u64, i32)> {
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*), SUM(size), SUM(install_count) FROM files")?;

        let (count, total_size, total_installs) = stmt.query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, i64>(1).unwrap_or(0) as u64,
                row.get::<_, i32>(2).unwrap_or(0),
            ))
        })?;

        Ok((count, total_size, total_installs))
    }

    /// Clean up missing files
    pub fn cleanup(&self) -> Result<usize> {
        let files = self.get_files()?;
        let mut removed = 0;

        for file in files {
            if !Path::new(&file.path).exists() {
                self.remove_file(file.id)?;
                removed += 1;
            }
        }

        Ok(removed)
    }
}
