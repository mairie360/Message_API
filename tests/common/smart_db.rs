use mairie360_api_lib::database::db_interface::Database;
use mairie360_api_lib::redis::redis_interface::Redis;
use mairie360_api_lib::smart_db::SmartDatabase;

/// Construit un `SmartDatabase` branché sur le Postgres de test.
///
/// Redis n'est pas requis : aucune `QueryView` de la couche `chats` ne déclare
/// de `cache_key`, donc le cache-aside n'est jamais sollicité. Si un Redis est
/// tout de même joignable, une connexion en échec est silencieusement ignorée
/// par la lib et l'appel retombe sur PostgreSQL.
pub async fn get_smart_db(db_url: &str) -> SmartDatabase {
    let db = Database::new(db_url).await;
    let redis = Redis::new("redis://127.0.0.1:6379");
    SmartDatabase::new(db, redis)
}
