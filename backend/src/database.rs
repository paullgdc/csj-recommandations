use crate::models::*;
use rusqlite::OptionalExtension;
use uuid::Uuid;

fn row_to_rec(row: &rusqlite::Row<'_>) -> rusqlite::Result<Recommandation> {
    Ok(Recommandation {
        id: row.get("id")?,
        name: row.get("name")?,
        created_by: row.get("created_by")?,
        media: row.get("media")?,
        link: row.get("link")?,
    })
}

fn row_to_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get("id")?,
        name: row.get("username")?,
    })
}

#[derive(Debug)]
pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users(
                id STRING PRIMARY KEY NOT NULL,
                username TEXT
            );",
            rusqlite::params![],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS recommandations(
                id BLOB PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                media INTEGER NOT NULL,
                created_by TEXT NOT NULL,
                link TEXT
            );",
            rusqlite::params![],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS upvotes(
                user_id STRING,
                reco_id BLOB,
                vote INTEGER NOT NULL,
                PRIMARY KEY(reco_id, user_id),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (reco_id) REFERENCES recommandations(id) ON DELETE CASCADE
            );",
            rusqlite::params![],
        )?;
        self.conn.pragma_update(None, "journal_mode", &"WAL")?;
        Ok(())
    }

    pub fn all_recommandations(&self) -> rusqlite::Result<Vec<Recommandation>> {
        self.conn
            .prepare("SELECT id, name, media, link, created_by FROM recommandations")?
            .query_map(rusqlite::NO_PARAMS, row_to_rec)?
            .map(|reco| Ok(reco?))
            .collect()
    }

    pub fn create_recommandation(&self, new_reco: &Recommandation) -> rusqlite::Result<()> {
        self.conn
            .prepare(
                "INSERT INTO recommandations(id, name, media, link, created_by)
                VALUES(?1, ?2, ?3, ?4, ?5)",
            )?
            .execute(rusqlite::params![
                &new_reco.id,
                &new_reco.name,
                &new_reco.media,
                &new_reco.link,
                &new_reco.created_by,
            ])?;
        Ok(())
    }

    pub fn delete_recommandation(&self, reco_id: Uuid) -> rusqlite::Result<()> {
        self.conn
            .prepare("DELETE FROM recommandations WHERE id=?1")?
            .execute(rusqlite::params![&reco_id])?;
        Ok(())
    }

    pub fn upvotes_by_recommandation_id(&self, reco_id: Uuid) -> rusqlite::Result<Vec<String>> {
        self.conn
            .prepare("SELECT user_id FROM upvotes WHERE reco_id=?1 AND vote=1")?
            .query_map(rusqlite::params![&reco_id], |row| row.get("user_id"))?
            .map(|data| Ok(data?))
            .collect()
    }

    pub fn upvote_by_id(&self, reco_id: Uuid, user_id: &str) -> rusqlite::Result<i32> {
        Ok(self
            .conn
            .prepare(
                "SELECT IFNULL((SELECT vote FROM upvotes WHERE user_id=?1 AND reco_id=?2), 0) vote",
            )?
            .query_row(rusqlite::params![&user_id, &reco_id], |row| row.get("vote"))?)
    }

    pub fn recommandation_by_id(&self, reco_id: Uuid) -> rusqlite::Result<Recommandation> {
        Ok(self
            .conn
            .prepare("SELECT id, name, media, link, created_by FROM recommandations WHERE id=?1")?
            .query_row(rusqlite::params![&reco_id], row_to_rec)?)
    }

    pub fn flip_upvote(&self, reco_id: Uuid, user_id: &str) -> rusqlite::Result<()> {
        self.conn
            .prepare(
                "INSERT OR REPLACE INTO upvotes(user_id, reco_id, vote)
                VALUES (
                    ?1, ?2,
                    (IFNULL(
                        (SELECT vote FROM upvotes WHERE user_id=?1 AND reco_id=?2),
                        0
                    )  + 1) % 2
                );
            ",
            )?
            .execute(rusqlite::params![&user_id, &reco_id])?;
        Ok(())
    }

    pub fn insert_user(&self, user: &User) -> rusqlite::Result<()> {
        self.conn
            .prepare("INSERT INTO users(id, username) VALUES(?1, ?2)")?
            .execute(rusqlite::params![&user.id, &user.name])?;
        Ok(())
    }

    pub fn user_by_id(&self, id: Uuid) -> rusqlite::Result<Option<User>> {
        self.conn
            .prepare("SELECT id, username FROM users WHERE id=?1")?
            .query_row(rusqlite::params![&id], row_to_user)
            .optional()
    }

    pub fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn new_id(&self) -> Uuid {
        Uuid::new_v4()
    }
}

#[derive(Debug)]
pub struct DatabaseManager {
    pub path: std::path::PathBuf,
}

impl r2d2::ManageConnection for DatabaseManager {
    type Connection = Database;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Database::open(&self.path)
    }

    fn is_valid(&self, db: &mut Self::Connection) -> Result<(), Self::Error> {
        db.conn.execute_batch("").map_err(Into::into)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub type DbPool = r2d2::Pool<DatabaseManager>;
