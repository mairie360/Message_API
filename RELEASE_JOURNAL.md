# Journal des releases — Message_API

<!-- release-journal-state {"dev": "273e6cd2170ee9df2aa7d157dc82408083d7bcf7", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-10-21 08:33 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `273e6cd` | 2026-10-21 08:33 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Changements de comportement

- Un push sur `main` ne lance plus que la release dev : `cicd.yml` s'arrête à `release-dev`. Staging et prod se promeuvent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ...`, `-f sha=<sha>` optionnel), avec l'approbation des environnements `Staging` et `Prod`. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Dans les stacks compose (dev, integration, security, performance), l'API se connecte avec le rôle Postgres `message_api` au lieu du superuser `postgres`, et Liquibase reçoit `-Dmessage_api_password`. Une requête hors des grants de ce rôle échoue désormais en `permission denied`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Corrections

- Promotion prod : `promote-prod` retaggait les images puis échouait à créer le tag et la release (HTTP 403 de l'API releases). Avec CICD v4.4.2, la création du tag et de la release réussit. (MAIR-612, [#200](https://github.com/mairie360/Message_API/pull/200))
- Promotion prod : avec CICD v4.4.0, la version n'était jamais publiée, le dry run semantic-release ne lisant pas son miroir. Corrigé en v4.4.1. (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198))

#### Technique

- Dépendances Rust : `mairie360_api_lib` 3.4.0 vers 4.2.0 ([#186](https://github.com/mairie360/Message_API/pull/186)) ; `tokio` 1.53.3 vers 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192), diff absent) ; `utoipa-swagger-ui` 10.0.1 vers 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194), diff absent) ; ignores `cargo audit` RUSTSEC-2026-0367, 0368 et 0369 ([#186](https://github.com/mairie360/Message_API/pull/186)) ; `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 selon la description ([#196](https://github.com/mairie360/Message_API/pull/196)).
- Images : toolchain `rust:1.99-bookworm` épinglé par digest dans `development.Dockerfile` ([#185](https://github.com/mairie360/Message_API/pull/185), [#201](https://github.com/mairie360/Message_API/pull/201)) ; `database` et `liquibase-migrations` en `dev-949384c` dans les compose et `TEST_DB_VERSION` ([#186](https://github.com/mairie360/Message_API/pull/186)).
- CI : CICD épinglé de v4.2.2 à v4.4.2 ([#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191), [#196](https://github.com/mairie360/Message_API/pull/196), [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200)) ; secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow pour le journal des releases ([#190](https://github.com/mairie360/Message_API/pull/190)) ; `CLAUDE.md` documente la promotion et les rôles DB ([#196](https://github.com/mairie360/Message_API/pull/196), [#186](https://github.com/mairie360/Message_API/pull/186)).

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections

- Promotion prod : avec CICD v4.4.0, la version n'était jamais publiée, le dry run semantic-release ne lisant pas son miroir. Corrigé en v4.4.1 ([#198](https://github.com/mairie360/Message_API/pull/198), MAIR-612).
- Promotion prod : avec CICD v4.4.1, `promote-prod` retaggait les images puis échouait à créer le tag et la release (HTTP 403). Corrigé en v4.4.2 ([#200](https://github.com/mairie360/Message_API/pull/200), MAIR-612).

#### Technique

- Image de dev : toolchain `rust:1.99-bookworm` épinglé au digest `5c1a173` dans `development.Dockerfile`, sans changement fonctionnel ([#201](https://github.com/mairie360/Message_API/pull/201)).
- CI : les workflows `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` passent de CICD v4.4.0 à v4.4.1 puis v4.4.2 ([#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200)).

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement

- Un push sur `main` ne lance plus que la release dev : `cicd.yml` s'arrête à `release-dev`. Staging et prod se promeuvent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ...`, `-f sha=<sha>` optionnel), avec l'approbation des environnements `Staging` et `Prod`. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Dans les stacks compose (dev, integration, security, performance), l'API se connecte avec le rôle Postgres `message_api` au lieu du superuser `postgres`, et Liquibase reçoit `-Dmessage_api_password`. Une requête hors des grants de ce rôle échoue désormais en `permission denied`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Technique

- Dépendances Rust : `mairie360_api_lib` 3.4.0 vers 4.2.0, ignores `cargo audit` RUSTSEC-2026-0367, 0368 et 0369 ([#186](https://github.com/mairie360/Message_API/pull/186)) ; `tokio` 1.53.3 vers 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192), diff absent) ; `utoipa-swagger-ui` 10.0.1 vers 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194), diff absent) ; `rustls-webpki` et `webpki-roots` selon la description ([#196](https://github.com/mairie360/Message_API/pull/196)).
- Images : `database` et `liquibase-migrations` en `dev-949384c` dans les compose et `TEST_DB_VERSION` ([#186](https://github.com/mairie360/Message_API/pull/186)) ; toolchain `rust:1.99-bookworm` épinglé au digest `e12566d` dans `development.Dockerfile` ([#185](https://github.com/mairie360/Message_API/pull/185)).
- CI : CICD épinglé de v4.2.2 à v4.4.1 ([#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191), [#196](https://github.com/mairie360/Message_API/pull/196), [#198](https://github.com/mairie360/Message_API/pull/198)) ; secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow pour le journal des releases ([#190](https://github.com/mairie360/Message_API/pull/190)).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 08:33 UTC | dev | — | `273e6cd` | [run](https://github.com/mairie360/Message_API/actions/runs/39568785077) |
| 2026-10-21 08:32 UTC | dev | — | `e215278` | [run](https://github.com/mairie360/Message_API/actions/runs/39568594235) |
| 2026-10-21 03:48 UTC | dev | — | `2dc9ac1` | [run](https://github.com/mairie360/Message_API/actions/runs/39539811319) |
| 2026-10-21 03:20 UTC | staging | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39537392816) |
| 2026-10-21 01:47 UTC | dev | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39527958201) |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
