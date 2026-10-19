# Journal des releases — Message_API

<!-- release-journal-state {"dev": "75d7650bd013ecc9e1c15b4a4dc7823de7e26f6a", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "d35e7dfca2982a2c8efa501374346b6021b48246", "updated": {"dev": "2026-10-19 20:24 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `d35e7df` | — |
| dev | — | `75d7650` | 2026-10-19 20:24 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Changements de comportement
- L'API embarque `mairie360_api_lib` 4.2.0 (au lieu de 3.4.0), dont le middleware JWT lit par défaut `users.email`. Elle doit donc être déployée sur une base où le rôle `message_api` a ce droit : image Database `dev-949384c` (MAIR-556). La PR indique qu'aucun changement de code n'a été nécessaire. Les stacks docker-compose (dev, intégration, sécurité, performance) connectent désormais l'API avec le rôle Postgres `message_api` / `message-api-password` au lieu du superuser, comme en staging et en prod. Une requête non couverte par les grants y échoue donc avec `permission denied`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Technique
- Tests et sécurité (MAIR-556) :
  - Les tests d'intégration s'exécutent avec le rôle `message_api`. Les fixtures gardent le superuser via `get_fixture_db()`.
  - Ajout du test de non-régression `tests/queries/db_role.rs`.
  - Les avis `RUSTSEC-2026-0367/0368/0369` sont ignorés (zip, dépendance de build de `utoipa-swagger-ui`).
  - ([#186](https://github.com/mairie360/Message_API/pull/186))
- CI et dépendances :
  - Workflow `mairie360/CICD` mis à jour de v4.2.1 à v4.3.1.
  - Transmission des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour le journal des releases.
  - `tokio` passe en 1.54.0.
  - Nouveau digest de l'image `rust:1.99-bookworm` du `development.Dockerfile`.
  - (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190), [#191](https://github.com/mairie360/Message_API/pull/191), [#187](https://github.com/mairie360/Message_API/pull/187), [#192](https://github.com/mairie360/Message_API/pull/192), [#185](https://github.com/mairie360/Message_API/pull/185))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- L'API embarque `mairie360_api_lib` 4.2.0 (au lieu de 3.4.0), dont le middleware JWT lit par défaut `users.email`. Elle doit donc être déployée sur une base où le rôle `message_api` a ce droit : image Database `dev-949384c` (MAIR-556). La PR indique qu'aucun changement de code n'a été nécessaire. Les stacks docker-compose (dev, intégration, sécurité, performance) connectent désormais l'API avec le rôle Postgres `message_api` / `message-api-password` au lieu du superuser, comme en staging et en prod. Une requête non couverte par les grants y échoue donc avec `permission denied`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Technique
- Tests et sécurité (MAIR-556) :
  - Les tests d'intégration s'exécutent avec le rôle `message_api`. Les fixtures gardent le superuser via `get_fixture_db()`.
  - Ajout du test de non-régression `tests/queries/db_role.rs`.
  - Les avis `RUSTSEC-2026-0367/0368/0369` sont ignorés (zip, dépendance de build de `utoipa-swagger-ui`).
  - ([#186](https://github.com/mairie360/Message_API/pull/186))
- CI et dépendances :
  - Workflow `mairie360/CICD` mis à jour de v4.2.1 à v4.3.1.
  - Transmission des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour le journal des releases.
  - `tokio` passe en 1.54.0.
  - Nouveau digest de l'image `rust:1.99-bookworm` du `development.Dockerfile`.
  - (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190), [#191](https://github.com/mairie360/Message_API/pull/191), [#187](https://github.com/mairie360/Message_API/pull/187), [#192](https://github.com/mairie360/Message_API/pull/192), [#185](https://github.com/mairie360/Message_API/pull/185))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
