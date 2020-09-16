use crate::errors::*;
use crate::Ctx;
use num_traits::cast::FromPrimitive;
use uuid::Uuid;

macro_rules! pub_fields {
    {
        $(#[$macro:meta])*
        struct $name:ident {
            $($field:ident: $t:ty,)*
        }
    } => {
        $(#[$macro])*
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

#[derive(
    Clone, Copy, Debug, juniper::GraphQLEnum, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
pub enum Media {
    Manga = 1,
    Anime = 2,
    Other = 3,
}

pub_fields! {
    #[derive(Clone, Debug)]
    struct Recommandation {
        id: Uuid,
        name: String,
        media: u8,
        created_by: String,
        link: Option<String>,
    }
}

#[juniper::graphql_object(Context = Ctx)]
impl Recommandation {
    fn id(&self) -> juniper::ID {
        juniper::ID::new(self.id.to_hyphenated_ref().to_string())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn upvotes(&self, ctx: &Ctx) -> ApiResult<Vec<String>> {
        Ok(ctx.0.get()?.upvotes_by_recommandation_id(self.id)?)
    }

    fn upvote_count(&self, ctx: &Ctx) -> ApiResult<i32> {
        Ok(ctx.0.get()?.upvotes_by_recommandation_id(self.id)?.len() as i32)
    }

    fn is_upvoted_by(&self, ctx: &Ctx, user_id: juniper::ID) -> ApiResult<bool> {
        Ok(ctx.0.get()?.upvote_by_id(self.id, &user_id)? == 1)
    }

    fn media(&self) -> ApiResult<Media> {
        Media::from_u8(self.media).ok_or(ApiError::UnrecognizedMediaValue)
    }
    fn link(&self) -> &Option<String> {
        &self.link
    }
    fn created_by(&self) -> &str {
        &self.created_by
    }
}

pub_fields! {
    #[derive(juniper::GraphQLInputObject)]
    struct NewRecommandation {
        name: String,
        link: Option<String>,
        media: Media,
    }
}

pub_fields! {
    struct User {
        id: Uuid,
        name: String,
    }
}

#[juniper::graphql_object(Context = Ctx)]
impl User {
    fn id(&self) -> juniper::ID {
        juniper::ID::new(self.id.to_hyphenated_ref().to_string())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

