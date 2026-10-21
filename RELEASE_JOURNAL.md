# Journal des releases — Message_API

<!-- release-journal-state {"dev": "2dc9ac1279e261ce35e4db7c927977a26e42fc20", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-10-21 03:48 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `2dc9ac1` | 2026-10-21 03:48 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Changements de comportement
- Un push sur `main` ne déclenche plus que la release dev (`cicd.yml` s'arrête à `release-dev`). Staging et prod se promeuvent à la main : `gh workflow run promote-staging.yml -R mairie360/Message_API`, puis `promote-prod.yml`, avec `-f sha=<sha>` pour un commit donné. Staging refuse un commit sans `dev-<sha>` ni run main vert, et prod refuse un commit sans `staging-<sha>` (d'après CLAUDE.md). (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Dans les stacks compose (dev, integration, security, performance), l'API se connecte avec le rôle Postgres `message_api` au lieu du superuser : une requête hors des grants de ce rôle échoue en `permission denied`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Corrections
- Avant : les runs `main` échouaient sur `cargo audit` (RUSTSEC-2026-0381/0382/0383). Maintenant : `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 débloquent la CI (d'après la description de la PR). (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Avant : `promote-prod` en v4.4.0 ne publiait pas la release, le dry run semantic-release ne lisant pas son miroir. Maintenant : pin en v4.4.1 qui corrige ce point. (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198))

#### Technique
- Pin `mairie360/CICD` successivement en v4.2.2 ([#187](https://github.com/mairie360/Message_API/pull/187)), v4.3.1 ([#191](https://github.com/mairie360/Message_API/pull/191)), v4.4.0 ([#196](https://github.com/mairie360/Message_API/pull/196)) puis v4.4.1 ([#198](https://github.com/mairie360/Message_API/pull/198)) dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` ; Renovate regroupe les trois fichiers.
- Secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow réutilisable pour le journal des releases. (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190))
- Dépendances : `mairie360_api_lib` 3.4.0 vers 4.2.0 sans changement de source ([#186](https://github.com/mairie360/Message_API/pull/186)), `utoipa-swagger-ui` 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194), diff absent), `tokio` 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192), diff absent), digest de l'image `rust:1.99-bookworm` ([#185](https://github.com/mairie360/Message_API/pull/185)), image Database `dev-949384c`.
- Tests : `tests/queries/db_role.rs` ajouté et fixtures sur `get_fixture_db()`. `cargo audit` ignore RUSTSEC-2026-0367/0368/0369 (zip, build-dependency de utoipa-swagger-ui).

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Pin `mairie360/CICD` en v4.4.1 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. Corrige le `promote-prod` en v4.4.0 qui ne publiait pas la release (dry run semantic-release). Aucun changement applicatif. (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198))

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Un push sur `main` ne déclenche plus que la release dev (`cicd.yml` s'arrête à `release-dev`). Staging et prod se promeuvent à la main : `gh workflow run promote-staging.yml -R mairie360/Message_API`, puis `promote-prod.yml`, avec `-f sha=<sha>` pour un commit donné. Staging refuse un commit sans `dev-<sha>` ni run main vert, et prod refuse un commit sans `staging-<sha>` (d'après CLAUDE.md). (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Dans les stacks compose (dev, integration, security, performance), l'API se connecte avec le rôle Postgres `message_api` au lieu du superuser : une requête hors des grants de ce rôle échoue en `permission denied`. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Corrections
- Avant : les runs `main` échouaient sur `cargo audit` (RUSTSEC-2026-0381/0382/0383). Maintenant : `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 débloquent la CI (d'après la description de la PR). (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))

#### Technique
- Pin `mairie360/CICD` successivement en v4.2.2 ([#187](https://github.com/mairie360/Message_API/pull/187)), v4.3.1 ([#191](https://github.com/mairie360/Message_API/pull/191)) puis v4.4.0 ([#196](https://github.com/mairie360/Message_API/pull/196)) dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`.
- Secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow réutilisable pour le journal des releases. (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190))
- Dépendances : `mairie360_api_lib` 3.4.0 vers 4.2.0 sans changement de source ([#186](https://github.com/mairie360/Message_API/pull/186)), `utoipa-swagger-ui` 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194), diff absent), `tokio` 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192), diff absent), digest de l'image `rust:1.99-bookworm` ([#185](https://github.com/mairie360/Message_API/pull/185)), image Database `dev-949384c`.
- Tests : `tests/queries/db_role.rs` ajouté et fixtures sur `get_fixture_db()`. `cargo audit` ignore RUSTSEC-2026-0367/0368/0369 (zip, build-dependency de utoipa-swagger-ui).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 03:48 UTC | dev | — | `2dc9ac1` | [run](https://github.com/mairie360/Message_API/actions/runs/39539811319) |
| 2026-10-21 03:20 UTC | staging | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39537392816) |
| 2026-10-21 01:47 UTC | dev | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39527958201) |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
