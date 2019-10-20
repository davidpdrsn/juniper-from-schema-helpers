use juniper::{Executor, FieldResult};
use juniper_eager_loading::{prelude::*, EagerLoading, HasMany, HasOne};
use juniper_from_schema::graphql_schema;
use juniper_from_schema_helpers::loaded_association;
use std::error::Error;

graphql_schema! {
    schema {
        query: Query
    }

    type Query {
        allUsers: [User!]! @juniper(ownership: "owned")
    }

    type User {
        id: Int!
        country: Country!
    }

    type Country {
        id: Int!
        users: [User!]!
    }
}

mod models {
    use juniper_eager_loading::LoadFrom;
    use std::error::Error;

    #[derive(Clone)]
    pub struct User {
        pub id: i32,
        pub country_id: i32,
    }

    #[derive(Clone)]
    pub struct Country {
        pub id: i32,
    }

    impl LoadFrom<i32> for Country {
        type Error = Box<dyn Error>;
        type Connection = super::DbConnection;

        fn load(_: &[i32], _: &Self::Connection) -> Result<Vec<Self>, Self::Error> {
            unimplemented!()
        }
    }

    impl LoadFrom<i32> for User {
        type Error = Box<dyn Error>;
        type Connection = super::DbConnection;

        fn load(_: &[i32], _: &Self::Connection) -> Result<Vec<Self>, Self::Error> {
            unimplemented!()
        }
    }

    impl LoadFrom<Country> for User {
        type Error = Box<dyn Error>;
        type Connection = super::DbConnection;

        fn load(_: &[Country], _: &Self::Connection) -> Result<Vec<Self>, Self::Error> {
            unimplemented!()
        }
    }
}

pub struct DbConnection;

impl DbConnection {
    fn load_all_users(&self) -> Vec<models::User> {
        unimplemented!()
    }
}

pub struct Context {
    db: DbConnection,
}

impl juniper::Context for Context {}

#[derive(Clone, EagerLoading)]
#[eager_loading(connection = "DbConnection", error = "Box<dyn Error>")]
pub struct User {
    user: models::User,

    #[has_one(default)]
    country: HasOne<Country>,
}

#[derive(Clone, EagerLoading)]
#[eager_loading(connection = "DbConnection", error = "Box<dyn Error>")]
pub struct Country {
    country: models::Country,

    #[has_many(root_model_field = "user")]
    users: HasMany<User>,
}

pub struct Query;

impl QueryFields for Query {
    fn field_all_users(
        &self,
        _: &Executor<'_, Context>,
        _: &QueryTrail<'_, User, Walked>,
    ) -> FieldResult<Vec<User>> {
        unimplemented!()
    }
}

impl UserFields for User {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<&i32> {
        Ok(&self.user.id)
    }

    loaded_association!(country -> Country);
}

impl CountryFields for Country {
    fn field_id(&self, _: &Executor<'_, Context>) -> FieldResult<&i32> {
        Ok(&self.country.id)
    }

    loaded_association!(users -> Vec<User>);
}

fn main() {}
