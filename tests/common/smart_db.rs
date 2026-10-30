use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;
use mairie360_api_lib::test_setup::queries_setup::{get_shared_db, get_shared_db_as};

/// Postgres role Message connects with in every deployment (Database `api_roles.sql`, MAIR-114).
const MESSAGE_DB_ROLE: &str = "message_api";

/// URL of the shared test database as [`MESSAGE_DB_ROLE`] (MAIR-556): the application code under
/// test (`AppState`, query views, SSE listener) goes through it, so a query the role is not
/// granted fails here with `permission denied` instead of in staging. The fixtures keep the
/// superuser URL of `get_shared_db()` (see [`get_fixture_db`]).
pub async fn message_db_url() -> String {
    get_shared_db_as(MESSAGE_DB_ROLE).await
}

/// `SmartDatabase` on the shared test database, connected as [`MESSAGE_DB_ROLE`]: what the
/// query views under test run on.
pub async fn get_smart_db() -> SmartDatabase {
    smart_db_at(&message_db_url().await).await
}

/// `SmartDatabase` on the shared test database, connected as the superuser. For the fixtures
/// only (users, ACLs, moderation log reads...), which touch tables outside the grants of
/// `message_api`: never run the API's own queries through it.
pub async fn get_fixture_db() -> SmartDatabase {
    let (_container, superuser_url) = get_shared_db().await;
    smart_db_at(superuser_url).await
}

/// `SmartDatabase` on `db_url`.
///
/// Redis is not required: no `QueryView` of the `chats` layer declares a `cache_key`, so the
/// cache-aside is never used. If a Redis is reachable anyway, a failed connection is silently
/// ignored by the lib and the call falls back to PostgreSQL.
pub async fn smart_db_at(db_url: &str) -> SmartDatabase {
    let db = Database::new(db_url).await;
    let redis = Redis::new("redis://127.0.0.1:6379");
    SmartDatabase::new(db, redis)
}
