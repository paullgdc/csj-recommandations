use std::str;
use uuid::Uuid;
use warp::Filter;

pub mod sessions {
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use std::sync::RwLock;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct SessionsStore(Arc<RwLock<BTreeMap<Uuid, Session>>>);

    impl SessionsStore {
        pub fn new() -> Self {
            Self(Arc::new(RwLock::new(BTreeMap::new())))
        }

        pub fn get_session(&self, key: &Uuid) -> Option<Session> {
            self.0
                .read()
                .expect("reading session: lock has been poisonned")
                .get(key)
                .cloned()
        }

        pub fn store_session(&mut self, session: Session) -> Uuid {
            let uuid = Uuid::new_v4();
            self.0
                .write()
                .expect("writing session: lock has been poisonned")
                .insert(uuid, session);
            uuid
        }
    }

    #[derive(Debug, Clone)]
    pub struct Session {
        pub user_id: Uuid,
        pub permission: String,
    }
}

#[derive(Debug)]
struct AuthError();

impl warp::reject::Reject for AuthError {}

pub fn auth_middleware(
    store: sessions::SessionsStore,
) -> impl warp::Filter<Extract = (sessions::Session,), Error = warp::Rejection> + Clone {
    warp::header::<Uuid>("Authorization").and_then(move |header| {
        let store = store.clone();
        async move {
            if let Some(session) = (&store).get_session(&header) {
                return Ok(session);
            }
            return Err(warp::reject::custom(AuthError()));
        }
    })
}

pub fn auth_endpoint(
    store: sessions::SessionsStore,
) -> impl warp::Filter<Extract = (warp::http::Response<Vec<u8>>,), Error = warp::Rejection> + Clone
{
    warp::body::content_length_limit(1024)
        .and(warp::body::bytes())
        .and_then(move |uuid: warp::hyper::body::Bytes| {
            let mut store = store.clone();
            async move {
                let uuid =
                    Uuid::parse_str(str::from_utf8(&uuid).unwrap()).map_err(|_| warp::reject())?;
                let session = sessions::Session {
                    user_id : uuid,
                    permission: "Default".to_owned(),
                };

                let id = store.store_session(session);
                let s: _ = warp::http::Response::builder()
                    .body(id.to_hyphenated().to_string().into_bytes())
                    .unwrap();
                Ok::<_, warp::Rejection>(s)
            }
        })
}
