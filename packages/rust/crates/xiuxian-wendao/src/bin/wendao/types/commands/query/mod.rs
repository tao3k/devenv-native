mod command;
mod graphql;
mod rest;
mod sql;

pub(crate) use self::command::QueryCommand;
#[cfg(test)]
pub(crate) use self::command::query;
pub(crate) use self::graphql::GraphqlQueryArgs;
pub(crate) use self::rest::RestQueryArgs;
pub(crate) use self::sql::SqlQueryArgs;

#[cfg(test)]
#[path = "../../../../../../tests/unit/bin/wendao/types/commands/query.rs"]
mod tests;
