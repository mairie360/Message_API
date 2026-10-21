# Journal des releases — Message_API

<!-- release-journal-state {"dev": "e215278e80c7c5f1e09e5dd1554f7493ddbf29b4", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-10-21 08:32 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `e215278` | 2026-10-21 08:32 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Nouveautés

- Staging et prod se promeuvent désormais à la main avec les workflows `promote-staging.yml` et `promote-prod.yml`, avec un `sha` optionnel et un verrou par cible. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))

#### Changements de comportement

- Un push sur `main` ne libère plus que dev : `cicd.yml` s'arrête à `release-dev`, et staging ou prod ne partent plus automatiquement. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Les stacks compose (dev, integration, security, performance) connectent l'API avec le rôle Postgres `message_api` au lieu du superuser, comme en staging et prod. Une requête hors des grants de ce rôle échoue donc en local et en CI. Liquibase reçoit `-Dmessage_api_password`, et l'image Database passe à `dev-949384c`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Corrections

- Avant, `promote-prod` ne libérait jamais la version, car le dry run semantic-release ne lisait pas son miroir (CICD v4.4.0). Maintenant la version est libérée (CICD v4.4.1). (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198))
- Avant, `promote-prod` retaguait les images puis échouait à créer le tag et la release (HTTP 403). Maintenant la release est créée (CICD v4.4.2). (MAIR-612, [#200](https://github.com/mairie360/Message_API/pull/200))

#### Technique

- Le journal de release (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190)) reçoit ses secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` dans `cicd.yml`, et les pins CICD passent de v4.2.1 à v4.4.2 (MAIR-571, [#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191)).
- Mises à jour de dépendances : `utoipa-swagger-ui` 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194), diff non fourni), `tokio` 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192), diff non fourni), `mairie360_api_lib` 4.2.0 ([#186](https://github.com/mairie360/Message_API/pull/186)), `rustls-webpki` et `webpki-root-certs` pour `cargo audit` ([#196](https://github.com/mairie360/Message_API/pull/196)), et digest de l'image `rust:1.99-bookworm` ([#185](https://github.com/mairie360/Message_API/pull/185)).
- Documentation et tests : `CLAUDE.md` décrit la promotion et le rôle `message_api` dans les tests, et `tests/queries/db_role.rs` est ajouté (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)).

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections

- Avant, `promote-prod` ne libérait jamais la version, car le dry run semantic-release ne lisait pas son miroir. Maintenant la version est libérée (CICD v4.4.1). (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198))
- Avant, `promote-prod` retaguait les images puis échouait à créer le tag et la release (HTTP 403). Maintenant la release est créée (CICD v4.4.2). (MAIR-612, [#200](https://github.com/mairie360/Message_API/pull/200))

#### Technique

- Pins `mairie360/CICD` passés de v4.4.0 à v4.4.2 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200))

## Différence staging → prod (en staging, pas encore en prod)

#### Nouveautés

- Staging et prod se promeuvent désormais à la main avec les workflows `promote-staging.yml` et `promote-prod.yml`, avec un `sha` optionnel et un verrou par cible. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))

#### Changements de comportement

- Un push sur `main` ne libère plus que dev : `cicd.yml` s'arrête à `release-dev`, et staging ou prod ne partent plus automatiquement. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Les stacks compose (dev, integration, security, performance) connectent l'API avec le rôle Postgres `message_api` au lieu du superuser. Une requête hors des grants de ce rôle échoue donc en local et en CI. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Technique

- Le journal de release (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190)) reçoit ses secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` dans `cicd.yml`, et les pins CICD passent à v4.2.2 puis v4.3.1 (MAIR-571, [#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191)).
- Mises à jour de dépendances : `tokio` 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192), diff non fourni), `utoipa-swagger-ui` 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194), diff non fourni), `mairie360_api_lib` 4.2.0 ([#186](https://github.com/mairie360/Message_API/pull/186)), `rustls-webpki` et `webpki-root-certs` ([#196](https://github.com/mairie360/Message_API/pull/196)), et digest de l'image `rust:1.99-bookworm` ([#185](https://github.com/mairie360/Message_API/pull/185)).
- Configuration : `renovate.json` couvre les trois workflows CICD, et `.cargo/audit.toml` ignore trois advisories zip (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196); MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 08:32 UTC | dev | — | `e215278` | [run](https://github.com/mairie360/Message_API/actions/runs/39568594235) |
| 2026-10-21 03:48 UTC | dev | — | `2dc9ac1` | [run](https://github.com/mairie360/Message_API/actions/runs/39539811319) |
| 2026-10-21 03:20 UTC | staging | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39537392816) |
| 2026-10-21 01:47 UTC | dev | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39527958201) |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
