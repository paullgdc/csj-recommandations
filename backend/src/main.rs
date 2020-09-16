use anyhow::{Context, Result};
use num_traits::cast::ToPrimitive;
use tokio;
use uuid::Uuid;
use warp::Filter;

use std::env;
use std::path::PathBuf;

mod errors;
pub(crate) use errors::*;

mod models;
pub(crate) use models::*;

mod auth;
use auth::*;

mod database;
use database::*;

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

#[derive(Clone, Debug)]
pub struct Ctx(DbPool, sessions::Session);

impl juniper::Context for Ctx {}

struct Query;

#[juniper::graphql_object(Context = Ctx)]
impl Query {
    fn apiVersion() -> &'static str {
        "1.0"
    }

    fn recommandations(ctx: &Ctx) -> ApiResult<Vec<Recommandation>> {
        Ok(ctx.0.get()?.all_recommandations()?)
    }

    fn me(ctx: &Ctx, id: juniper::ID) -> ApiResult<User> {
        Ok(ctx
            .0
            .get()?
            .user_by_id(id.parse().map_err(|_| ApiError::InvalidId)?)
            .transpose()
            .ok_or(ApiError::NoUserFound)??)
    }
}

struct Mutation;

#[juniper::graphql_object(Context = Ctx)]
impl Mutation {
    fn create_recommandation(ctx: &Ctx, new: NewRecommandation) -> ApiResult<Recommandation> {
        let db = ctx.0.get()?;
        let new_todo = Recommandation {
            id: db.new_id(),
            name: new.name,
            link: new.link,
            created_by: "paul".to_string(), // TODO: use user id passed by authentificationx
            media: new.media.to_u8().unwrap(),
        };
        db.create_recommandation(&new_todo)?;
        Ok(new_todo)
    }

    fn delete_recommandation(ctx: &Ctx, reco_id: juniper::ID) -> ApiResult<Recommandation> {
        let db = ctx.0.get()?;
        let id = reco_id.parse().map_err(|_| ApiError::InvalidId)?;
        let reco = db.recommandation_by_id(id)?;
        if reco.created_by != "paul" {
            return Err(ApiError::UnauthorizedOperation);
        }
        db.delete_recommandation(id)?;
        Ok(reco)
    }

    fn flip_recommandation_vote(
        ctx: &Ctx,
        user_id: juniper::ID,
        reco_id: juniper::ID,
    ) -> ApiResult<Recommandation> {
        let db = ctx.0.get()?;
        let reco_id: Uuid = reco_id.parse().map_err(|_| ApiError::InvalidId)?;
        db.flip_upvote(reco_id, &user_id)?;
        Ok(db.recommandation_by_id(reco_id)?)
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

    let session_store = sessions::SessionsStore::new();

    let state = warp::any()
        .and(auth_middleware(session_store.clone()))
        .map(move |session| Ctx(pool.clone(), session));
    let graphql_filter = juniper_warp::make_graphql_filter_sync(schema(), state.boxed());

    println!("starting the server at http://{}", config.address);
    warp::serve(
        warp::path("graphql")
            .and(graphql_filter.recover(|_err| async move {
                Ok::<_, warp::Rejection>(warp::reply::with_status(
                    warp::reply(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }))
            .or(warp::path("auth")
                .and(warp::post())
                .and(auth_endpoint(session_store))
                .recover(|_err| async move {
                    Ok::<_, warp::Rejection>(warp::reply::with_status(
                        warp::reply(),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }))
            .or(warp::get()
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
