use anyhow::Result;
use juniper;
use rusqlite::OptionalExtension;
use tokio;
use uuid::Uuid;
use warp::Filter;

use std::env;
use std::path::PathBuf;

struct Config {
    static_dir: PathBuf,
    database: PathBuf,
    address: std::net::SocketAddr,
}

fn config() -> Result<Config> {
    Ok(Config {
        static_dir: env::var_os("STATIC_DIR")
            .map(|e| e.into())
            .unwrap_or("public".to_owned().into()),
        database: env::var_os("DATABASE_PATH")
            .map(|e| e.into())
            .unwrap_or("database/test.sqlite".to_owned().into()),
        address: env::var_os("ADRESS")
            .and_then(|addr| addr.to_str().map(|s| s.parse()))
            .unwrap_or(Ok(([127, 0, 0, 1], 8080).into()))?,
    })
}

#[derive(Debug)]
struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS recommandations(
                id BLOB PRIMARY KEY NOT NULL,
                data BLOB
            );
        ",
            rusqlite::params![],
        )?;
        self.conn.pragma_update(None, "journal_mode", &"WAL")?;
        Ok(())
    }

    fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        Ok(Self { conn })
    }

    fn new_id(&self) -> Result<uuid::Uuid> {
        Ok(uuid::Uuid::new_v4())
    }

    fn put_recommandation(&self, rec: &Recommandation) -> Result<()> {
        self.conn
            .prepare("INSERT OR REPLACE INTO recommandations(id, data) VALUES (?1, ?2)")
            .unwrap()
            .execute(rusqlite::params![&rec.id, &serde_json::to_vec(rec)?])?;
        Ok(())
    }

    fn list_recommandations<'a>(&'a self) -> Result<Vec<Recommandation>> {
        self.conn
            .prepare("SELECT data FROM recommandations")?
            .query_map(rusqlite::NO_PARAMS, |row| row.get::<usize, Vec<u8>>(0))?
            .map(|data| Ok(serde_json::from_slice(&data?)?))
            .collect()
    }

    fn modify_recommandation(
        &mut self,
        id: Uuid,
        op: impl FnOnce(&mut Option<Recommandation>) -> Result<()>,
    ) -> Result<Option<Recommandation>> {
        let tx = self.conn.transaction()?;
        let mut rec = tx
            .prepare("SELECT data FROM recommandations WHERE id=?1")?
            .query_row(rusqlite::params![&id], |row| row.get::<usize, Vec<u8>>(0))
            .optional()?
            .map(|d| serde_json::from_slice::<Recommandation>(&d))
            .map_or(Ok(None), |r| r.map(Some))?;
        op(&mut rec)?;
        if let Some(ref rec) = &rec {
            if rec.id != id {
                panic!("Trying to modify object id durin mutation");
            }
            tx.prepare("INSERT OR REPLACE INTO recommandations(id, data) VALUES (?1, ?2)")
                .unwrap()
                .execute(rusqlite::params![&rec.id, &serde_json::to_vec(rec)?])?;
        }
        tx.commit()?;
        Ok(rec)
    }
}

#[derive(Debug)]
struct DatabaseManager {
    path: std::path::PathBuf,
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

type DbPool = r2d2::Pool<DatabaseManager>;

#[derive(Clone, Debug)]
struct Ctx(DbPool);

impl juniper::Context for Ctx {}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Recommandation {
    id: Uuid,
    name: String,
    upvotes: Vec<String>,
}

#[juniper::graphql_object]
impl Recommandation {
    fn id(&self) -> juniper::ID {
        juniper::ID::new(self.id.to_string())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn upvote_count(&self) -> i32 {
        self.upvotes.len() as i32
    }
}

struct Query;

#[juniper::graphql_object(Context = Ctx)]
impl Query {
    fn apiVersion() -> &'static str {
        "1.0"
    }

    fn recommandations(ctx: &Ctx) -> Option<Vec<Recommandation>> {
        ctx.0.get().ok()?.list_recommandations().ok()
    }
}

struct Mutation;

#[juniper::graphql_object(Context = Ctx)]
impl Mutation {
    fn create_recommandation(ctx: &Ctx, name: String) -> Option<Recommandation> {
        let db = ctx.0.get().ok()?;
        let new_todo = Recommandation {
            id: db.new_id().ok()?,
            name,
            upvotes: Vec::new(),
        };
        db.put_recommandation(&new_todo).ok()?;
        Some(new_todo)
    }

    fn upvote_recommandation(ctx: &Ctx, user: String, id: juniper::ID) -> Option<Recommandation> {
        ctx.0
            .get()
            .ok()?
            .modify_recommandation(id.parse().ok()?, move |reco| {
                let reco = match reco.as_mut() {
                    Some(reco) => reco,
                    None => return Ok(()),
                };
                if let None = reco.upvotes.iter().find(|up| *up == &user) {
                    reco.upvotes.push(user);
                }
                Ok(())
            })
            .ok()
            .flatten()
    }
}

type Schema = juniper::RootNode<'static, Query, Mutation, juniper::EmptySubscription<Ctx>>;

fn schema() -> Schema {
    Schema::new(Query, Mutation, juniper::EmptySubscription::new())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = config()?;

    let manager = DatabaseManager {
        path: config.database,
    };
    let pool = DbPool::new(manager)?;
    pool.get()?.init()?;

    let ctx = Ctx(pool);

    let state = warp::any().map(move || ctx.clone());
    let graphql_filter = juniper_warp::make_graphql_filter_sync(schema(), state.boxed());

    println!("starting the server");
    warp::serve(
        warp::path("graphql").and(graphql_filter).or(warp::get()
            .and(warp::path("graphiql").and(juniper_warp::graphiql_filter("/graphql")))
            .or(warp::fs::dir(config.static_dir.clone()))
            .or(warp::fs::file({
                let mut dir = config.static_dir.clone();
                dir.push("index.html");
                dir
            }))),
    )
    .run(config.address)
    .await;
    Err(anyhow::anyhow!("The server shouldn't stop"))
}
