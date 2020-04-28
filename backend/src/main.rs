use anyhow::Result;
use juniper::FieldResult;
use num_traits::cast::{FromPrimitive, ToPrimitive};
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
                media TEXT NOT NULL,
                link TEXT,
                FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
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
            )
            ",
            rusqlite::params![],
        )?;
        self.conn.pragma_update(None, "journal_mode", &"WAL")?;
        Ok(())
    }

    fn all_recommandations(&self) -> Result<Vec<Recommandation>> {
        self.conn
            .prepare("SELECT id, name, media, link FROM recommandations")?
            .query_map(rusqlite::NO_PARAMS, |row| 
                Ok(Recommandation {
                    id: row.get(0)?,
                    name: row.get(0)?,
                    media: row.get(0)?,
                    link: row.get(0)?,
                })
            )?
            .map(|reco| Ok(reco?))
            .collect()
    }

    fn upvotes_by_recommandation_id(&self, reco_id: Uuid) -> Result<Vec<i32>> {
        self.conn
            .prepare("SELECT user_id FROM upvotes WHERE reco_id=?1")?
            .query_map(rusqlite::params![&reco_id], |row| row.get(0))?
            .map(|data| Ok(data?))
            .collect()
    }

    fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        let conn = rusqlite::Connection::open(path)?;
        Ok(Self { conn })
    }

    fn new_id(&self) -> Uuid {
        Uuid::new_v4()
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

#[derive(
    Clone, Copy, Debug, juniper::GraphQLEnum, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
enum Media {
    Manga = 1,
    Anime = 2,
    Other = 3,
}

#[derive(Clone, Debug)]
struct Recommandation {
    id: Uuid,
    name: String,
    media: u8,
    link: Option<String>,
}

struct User {
    id: i64,
    username: String,
}

#[juniper::graphql_object(Context = Ctx)]
impl Recommandation {
    fn id(&self) -> juniper::ID {
        juniper::ID::new(self.id.to_string())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn upvotes(&self, ctx: &Ctx) -> FieldResult<Vec<i32>> {
        Ok(ctx.0.get()?.upvotes_by_recommandation_id(self.id)?)
    }

    fn upvote_count(&self) -> i32 {
        unimplemented!()
    }

    fn is_upvoted_by(&self, user: String) -> bool {
        unimplemented!()
    }

    fn media(&self) -> FieldResult<Media> {
        Media::from_u8(self.media)
            .ok_or(anyhow::anyhow!("Cannot get media variant from value").into())
    }
    fn link(&self) -> &Option<String> {
        &self.link
    }
}

#[derive(juniper::GraphQLInputObject)]
struct NewRecommandation {
    name: String,
    link: Option<String>,
    media: Media,
}

struct Query;

#[juniper::graphql_object(Context = Ctx)]
impl Query {
    fn apiVersion() -> &'static str {
        "1.0"
    }

    fn recommandations(ctx: &Ctx) -> FieldResult<Vec<Recommandation>> {
        Ok(ctx.0.get()?.all_recommandations()?)
    }
}

struct Mutation;

#[juniper::graphql_object(Context = Ctx)]
impl Mutation {
    fn create_recommandation(ctx: &Ctx, new: NewRecommandation) -> Option<Recommandation> {
        let db = ctx.0.get().ok()?;
        let new_todo = Recommandation {
            id: db.new_id(),
            name: new.name,
            link: new.link,
            media: new.media.to_u8().unwrap(),
        };
        // db.put_recommandation(&new_todo).ok()?;
        Some(new_todo)
    }

    fn flip_recommandation_vote(
        ctx: &Ctx,
        user: String,
        id: juniper::ID,
    ) -> Option<Recommandation> {
        todo!()
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

    println!("starting the server at http://{}", config.address);
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
