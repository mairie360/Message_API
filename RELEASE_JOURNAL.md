# Journal des releases — Message_API

<!-- release-journal-state {"dev": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-10-21 01:47 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `71dd8d1` | 2026-10-21 01:47 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Changements de comportement
- Un push sur `main` ne publie plus que dev : `cicd.yml` s'arrête à `release-dev`. Staging et prod se lancent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -R mairie360/Message_API [-f sha=SHA]`, sur `main` uniquement, avec l'approbation des environnements `Staging` / `Prod`). (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Les stacks compose (dev, integration, security, performance) connectent désormais l'API avec le rôle Postgres `message_api` au lieu du superuser ; une requête hors des grants de ce rôle échoue en `permission denied`. Les tests d'intégration des APIs font de même, les fixtures gardent le superuser. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Technique
- Dépendances : `mairie360_api_lib` 3.4.0 → 4.2.0 (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)), `tokio` 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192)), `utoipa-swagger-ui` 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194)), `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 ([#196](https://github.com/mairie360/Message_API/pull/196)), digest de `rust:1.99-bookworm` ([#185](https://github.com/mairie360/Message_API/pull/185)).
- CI : `mairie360/CICD` épinglé de v4.2.2 à v4.4.0 ([#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191), [#196](https://github.com/mairie360/Message_API/pull/196)), secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow réutilisable (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190)), exceptions `RUSTSEC-2026-0367/0368/0369` dans `.cargo/audit.toml` (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)), image Database `dev-949384c` (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)), regroupement Renovate des trois workflows ([#196](https://github.com/mairie360/Message_API/pull/196)).

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Un push sur `main` ne publie plus que dev : `cicd.yml` s'arrête à `release-dev`. Staging et prod se lancent à la main avec `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -R mairie360/Message_API [-f sha=SHA]`, sur `main` uniquement, avec l'approbation des environnements `Staging` / `Prod`). (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Les stacks compose (dev, integration, security, performance) connectent désormais l'API avec le rôle Postgres `message_api` au lieu du superuser ; une requête hors des grants de ce rôle échoue en `permission denied`. Les tests d'intégration des APIs font de même, les fixtures gardent le superuser. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

#### Technique
- Dépendances : `mairie360_api_lib` 3.4.0 → 4.2.0 (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)), `tokio` 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192)), `utoipa-swagger-ui` 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194)), `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 ([#196](https://github.com/mairie360/Message_API/pull/196)), digest de `rust:1.99-bookworm` ([#185](https://github.com/mairie360/Message_API/pull/185)).
- CI : `mairie360/CICD` épinglé de v4.2.2 à v4.4.0 ([#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191), [#196](https://github.com/mairie360/Message_API/pull/196)), secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` transmis au workflow réutilisable (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190)), exceptions `RUSTSEC-2026-0367/0368/0369` dans `.cargo/audit.toml` (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)), image Database `dev-949384c` (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186)), regroupement Renovate des trois workflows ([#196](https://github.com/mairie360/Message_API/pull/196)).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 03:20 UTC | staging | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39537392816) |
| 2026-10-21 01:47 UTC | dev | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39527958201) |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
