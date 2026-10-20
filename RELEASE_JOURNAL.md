# Journal des releases — Message_API

<!-- release-journal-state {"dev": "75d7650bd013ecc9e1c15b4a4dc7823de7e26f6a", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "75d7650bd013ecc9e1c15b4a4dc7823de7e26f6a", "updated": {"dev": "2026-10-19 20:24 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `75d7650` | 2026-10-20 17:07 UTC |
| dev | — | `75d7650` | 2026-10-19 20:24 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Technique
- `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans changement du code de l'API. D'après la PR, le middleware JWT de la lib 4.2.0 lit `users.email`. L'API doit donc tourner sur une image Database qui accorde cette lecture au rôle `message_api` (`ghcr.io/mairie360/database:dev-949384c`, désormais épinglée dans les compose et `TEST_DB_VERSION`). Les tests et les stacks compose (dev, intégration, sécurité, performance) se connectent maintenant avec le rôle `message_api` au lieu du superuser, comme en staging et en prod. Liquibase reçoit `-Dmessage_api_password`. Un test de non-régression `tests/queries/db_role.rs` est ajouté. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))
- 5 changements d'outillage sans effet fonctionnel : workflow `mairie360/CICD` mis à jour de v4.2.1 à v4.2.2 puis v4.3.1, transmission des secrets du journal des releases (MAIR-571), `tokio` 1.54.0, digest de l'image `rust:1.99-bookworm` de développement. ([#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191), [#190](https://github.com/mairie360/Message_API/pull/190), [#192](https://github.com/mairie360/Message_API/pull/192), [#185](https://github.com/mairie360/Message_API/pull/185))

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

#### Technique
- `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans changement du code de l'API. D'après la PR, le middleware JWT de la lib 4.2.0 lit `users.email`. L'API doit donc tourner sur une image Database qui accorde cette lecture au rôle `message_api` (`ghcr.io/mairie360/database:dev-949384c`, désormais épinglée dans les compose et `TEST_DB_VERSION`). Les tests et les stacks compose (dev, intégration, sécurité, performance) se connectent maintenant avec le rôle `message_api` au lieu du superuser, comme en staging et en prod. Liquibase reçoit `-Dmessage_api_password`. Un test de non-régression `tests/queries/db_role.rs` est ajouté. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))
- 5 changements d'outillage sans effet fonctionnel : workflow `mairie360/CICD` mis à jour de v4.2.1 à v4.2.2 puis v4.3.1, transmission des secrets du journal des releases (MAIR-571), `tokio` 1.54.0, digest de l'image `rust:1.99-bookworm` de développement. ([#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191), [#190](https://github.com/mairie360/Message_API/pull/190), [#192](https://github.com/mairie360/Message_API/pull/192), [#185](https://github.com/mairie360/Message_API/pull/185))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
