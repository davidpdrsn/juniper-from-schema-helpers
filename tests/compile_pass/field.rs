use juniper::ID;
use juniper_from_schema::graphql_schema;
use juniper_from_schema_helpers::field;

graphql_schema! {
    schema {
        query: Query
    }

    type Query {
        foo: String!
        bar: Int
        user: User!
        optionUser: User
        users: [User!]!
        cursor: Cursor!
        cursors: [Cursor!]!
    }

    type User {
        id: ID!
    }

    scalar Cursor
}

pub struct Context;

impl juniper::Context for Context {}

pub struct Query {
    foo: String,
    other: Other,
    user: User,
    option_user: Option<User>,
    users: Vec<User>,
    cursor: Cursor,
    cursors: Vec<Cursor>,
}

impl QueryFields for Query {
    field!(foo -> String);
    field!(other.bar -> Option<i32>);
    field!(user -> User);
    field!(option_user -> Option<User>);
    field!(users -> Vec<User>);
    field!(cursor -> Cursor as scalar);
    field!(cursors -> Vec<Cursor> as scalar);
}

pub struct Other {
    bar: Option<i32>,
}

pub struct User {
    id: ID,
}

impl UserFields for User {
    field!(id -> ID);
}

fn main() {}
