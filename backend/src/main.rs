use anyhow::{Context, Result};
use async_trait::async_trait;
use dataloader::cached::Loader;
use dataloader::BatchFn;
use juniper::FieldResult;
use num_traits::cast::{FromPrimitive, ToPrimitive};
use tokio;
use uuid::Uuid;
use warp::Filter;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use error::*;

mod error {
    use juniper::graphql_value;
    use std::fmt;

    macro_rules! from_error {
        ($variant:ident, $error:ty) => {
            impl From<$error> for ApiError {
                fn from(error: $error) -> Self {
                    eprintln!("Error: {}", error);
                    ApiError::$variant
                }
            }
        };
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ApiError {
        Database,
    }

    impl fmt::Display for ApiError {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            use ApiError::*;
            match self {
                Database => write!(fmt, "Internal database error"),
            }
        }
    }

    from_error!(Database, r2d2::Error);
    from_error!(Database, rusqlite::Error);

    impl juniper::IntoFieldError for ApiError {
        fn into_field_error(self) -> juniper::FieldError {
            use ApiError::*;
            match self {
                Database => juniper::FieldError::new(
                    self,
                    graphql_value!({
                        "type": "DATABASE"
                    }),
                ),
            }
        }
    }

    pub trait IntoApiResult<T> {
        fn into_api(self) -> ApiResult<T>;
    }

    pub type ApiResult<T> = std::result::Result<T, ApiError>;
    impl<T, E> IntoApiResult<T> for std::result::Result<T, E>
    where
        E: Into<ApiError>,
    {
        fn into_api(self) -> ApiResult<T> {
            self.map_err(|e| e.into())
        }
    }
}

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

fn row_to_rec(row: &rusqlite::Row<'_>) -> rusqlite::Result<Recommandation> {
    Ok(Recommandation {
        id: row.get("id")?,
        name: row.get("name")?,
        media: row.get("media")?,
        link: row.get("link")?,
    })
}

fn result_to_batch<K, U, V, S, T>(
    result: std::result::Result<K, V>,
    map: &mut HashMap<S, Result<U, T>>,
    keys: &[S],
) -> Option<K>
where
    S: std::cmp::Eq + std::cmp::Ord + std::hash::Hash + Clone,
    T: From<V> + Clone,
{
    let error = match result {
        Ok(val) => return Some(val),
        Err(e) => T::from(e),
    };
    for key in keys {
        map.insert(key.clone(), Err(error.clone()));
    }
    None
}

fn query_in(base: &'static str, nb_parameters: usize) -> String {
    let mut query = String::with_capacity(base.len() + 2 * nb_parameters);
    query.push_str(base);
    for _ in 0..(nb_parameters - 1) {
        query.push_str(&"?,");
    }
    query.push_str(&"?);");
    query
}

pub struct UpvotesByRecoBatcb(DbPool);

#[async_trait]
impl BatchFn<Uuid, ApiResult<Vec<String>>> for UpvotesByRecoBatcb {
    async fn load(&self, keys: &[Uuid]) -> HashMap<Uuid, ApiResult<Vec<String>>> {
        let mut map = HashMap::new();
        let db = match result_to_batch(self.0.get().into_api(), &mut map, keys) {
            Some(db) => db,
            None => return map,
        };

        let query = query_in(
            "SELECT user_id, reco_id FROM upvotes WHERE vote=1 AND reco_id IN (",
            keys.len(),
        );
        let op = (|| {
            db.conn
                .prepare(&query)?
                .query_map(
                    &keys
                        .into_iter()
                        .map(|k| -> &dyn rusqlite::ToSql { k })
                        .collect::<Vec<_>>(),
                    |r| {
                        let u_id: String = r.get("user_id")?;
                        let r_id: Uuid = r.get("reco_id")?;
                        map.entry(r_id)
                            .or_insert(Ok(Vec::new()))
                            .as_mut()
                            .map(|v| v.push(u_id))
                            .unwrap();
                        Ok(())
                    },
                )?
                .collect::<rusqlite::Result<()>>()
        })();
        result_to_batch(op.into_api(), &mut map, keys);
        map
    }
}

type UpvotesLoader = Loader<Uuid, ApiResult<Vec<String>>, UpvotesByRecoBatcb>;

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
                media INTEGER NOT NULL,
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

    fn all_recommandations(&self) -> Result<Vec<Recommandation>> {
        self.conn
            .prepare("SELECT id, name, media, link FROM recommandations")?
            .query_map(rusqlite::NO_PARAMS, row_to_rec)?
            .map(|reco| Ok(reco?))
            .collect()
    }

    fn create_recommandation(&self, new_reco: &Recommandation) -> Result<()> {
        self.conn
            .prepare(
                "INSERT INTO recommandations(id, name, media, link)
                VALUES(?1, ?2, ?3, ?4)",
            )?
            .execute(rusqlite::params![
                &new_reco.id,
                &new_reco.name,
                &new_reco.media,
                &new_reco.link
            ])?;
        Ok(())
    }

    fn upvotes_by_recommandation_id(&self, reco_id: Uuid) -> Result<Vec<String>> {
        self.conn
            .prepare("SELECT user_id FROM upvotes WHERE reco_id=?1 AND vote=1")?
            .query_map(rusqlite::params![&reco_id], |row| row.get("user_id"))?
            .map(|data| Ok(data?))
            .collect()
    }

    fn upvote_by_id(&self, reco_id: Uuid, user_id: &str) -> Result<i32> {
        Ok(self
            .conn
            .prepare(
                "SELECT IFNULL((SELECT vote FROM upvotes WHERE user_id=?1 AND reco_id=?2), 0) vote",
            )?
            .query_row(rusqlite::params![&user_id, &reco_id], |row| row.get("vote"))?)
    }

    fn recommandation_by_id(&self, reco_id: Uuid) -> Result<Recommandation> {
        Ok(self
            .conn
            .prepare("SELECT id, name, media, link FROM recommandations WHERE id=?1")?
            .query_row(rusqlite::params![&reco_id], row_to_rec)?)
    }

    fn flip_upvote(&self, reco_id: Uuid, user_id: &str) -> Result<()> {
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

struct Ctx(DbPool, UpvotesLoader);

impl Ctx {
    fn from_db(db: &DbPool) -> Self {
        Self(
            db.clone(),
            UpvotesLoader::new(UpvotesByRecoBatcb(db.clone())),
        )
    }
}

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

#[juniper::graphql_object(Context = Ctx)]
impl Recommandation {
    async fn id(&self) -> juniper::ID {
        juniper::ID::new(self.id.to_string())
    }

    async fn name(&self) -> &str {
        &self.name
    }

    async fn upvotes(&self, ctx: &Ctx) -> FieldResult<Vec<String>> {
        Ok(ctx.1.load(self.id).await?)
    }

    async fn upvote_count(&self, ctx: &Ctx) -> FieldResult<i32> {
        Ok(ctx.0.get()?.upvotes_by_recommandation_id(self.id)?.len() as i32)
    }

    async fn is_upvoted_by(&self, ctx: &Ctx, user_id: juniper::ID) -> FieldResult<bool> {
        Ok(ctx.0.get()?.upvote_by_id(self.id, &user_id)? == 1)
    }

    async fn media(&self) -> FieldResult<Media> {
        Media::from_u8(self.media)
            .ok_or(anyhow::anyhow!("Cannot get media variant from value").into())
    }
    async fn link(&self) -> &Option<String> {
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
    async fn create_recommandation(ctx: &Ctx, new: NewRecommandation) -> FieldResult<Recommandation> {
        let db = ctx.0.get()?;
        let new_todo = Recommandation {
            id: db.new_id(),
            name: new.name,
            link: new.link,
            media: new.media.to_u8().unwrap(),
        };
        db.create_recommandation(&new_todo)?;
        Ok(new_todo)
    }

    async fn flip_recommandation_vote(
        ctx: &Ctx,
        user_id: juniper::ID,
        reco_id: juniper::ID,
    ) -> FieldResult<Recommandation> {
        println!("flipped reco");
        let db = ctx.0.get()?;
        let reco_id: Uuid = reco_id.parse()?;
        db.flip_upvote(reco_id, &user_id)?;
        println!("flipped reco");
        Ok(dbg!(db.recommandation_by_id(reco_id)?))
    }
}

type Schema = juniper::RootNode<'static, Query, Mutation, juniper::EmptySubscription<Ctx>>;

fn schema() -> Schema {
    Schema::new(Query, Mutation, juniper::EmptySubscription::new())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = config().context("Wasn't able to read the config")?;

    let manager = DatabaseManager {
        path: config.database,
    };
    let pool = DbPool::new(manager)?;
    pool.get()?
        .init()
        .context("Wasn't able to initialize the database")?;


    let state = warp::any().map(move || Ctx::from_db(&pool));
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

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
