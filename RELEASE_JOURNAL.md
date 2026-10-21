# Journal des releases — Message_API

<!-- release-journal-state {"dev": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod": "d35e7dfca2982a2c8efa501374346b6021b48246", "prod_tag": "v2.0.0", "staging": "75d7650bd013ecc9e1c15b4a4dc7823de7e26f6a", "updated": {"dev": "2026-10-21 01:47 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.0.0 | `d35e7df` | — |
| staging | — | `75d7650` | 2026-10-20 17:07 UTC |
| dev | — | `71dd8d1` | 2026-10-21 01:47 UTC |

## Ajouté sur main depuis la dernière release prod (v2.0.0)

#### Changements de comportement
- Les environnements de test du dépôt (compose dev, integration, security, performance) connectent désormais l'API avec le rôle Postgres `message_api` (mot de passe `message-api-password`) au lieu du superuser. Une requête hors des grants de ce rôle échoue donc en local et en CI. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))
- Le journal des releases est rédigé par Claude sur `main` : les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow CICD. (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190))
- Un push sur `main` ne déclenche plus que la release dev. Staging et prod passent par les nouveaux workflows manuels `promote-staging.yml` et `promote-prod.yml`. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
#### Technique
- Bumps de CI : `mairie360/CICD` passe de v4.2.1 à v4.2.2, v4.3.1 puis v4.4.0 (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196) ; [#187](https://github.com/mairie360/Message_API/pull/187), [#191](https://github.com/mairie360/Message_API/pull/191)).
- La description de [#196](https://github.com/mairie360/Message_API/pull/196) annonce aussi un relèvement de `rustls-webpki`, `webpki-roots` et `webpki-root-certs` pour débloquer `cargo audit` ; cette modification n'apparaît pas dans le diff fourni.
- `mairie360_api_lib` passe en 4.2.0, sans changement de source, et `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369 (zip 8 via utoipa-swagger-ui, build-time seulement). (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))
- Bumps de dépendances : `tokio` 1.53.3 → 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192)), `utoipa-swagger-ui` 10.0.1 → 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194)) et l'image Rust de `development.Dockerfile` repinnée par digest ([#185](https://github.com/mairie360/Message_API/pull/185)). Les diffs de [#192](https://github.com/mairie360/Message_API/pull/192) et [#194](https://github.com/mairie360/Message_API/pull/194) sont absents du fichier fourni.
- Tests : les tests passent par `get_smart_db()` et `AppState` en rôle `message_api`, les fixtures restent en superuser, et `tests/queries/db_role.rs` est ajouté. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Un push sur `main` ne déclenche plus que la release dev : `cicd.yml`, épinglé sur CICD v4.4.0, s'arrête à `release-dev` et ne laisse plus de run en attente sur `Staging` ou `Prod`. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- Deux nouveaux workflows manuels, `promote-staging.yml` et `promote-prod.yml`, sont lancés par `gh workflow run promote-staging.yml -R mairie360/Message_API [-f sha=SHA]` (et équivalent pour prod). Ils ne fonctionnent que sur `main`, avec un verrou par cible. Selon `CLAUDE.md`, staging refuse un commit sans tag `dev-<sha>` et sans run vert sur `main`, et prod refuse un commit sans tag `staging-<sha>`. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
#### Technique
- Renovate regroupe désormais les trois fichiers de workflow dans le même pin `cicd_version`, et `CLAUDE.md` documente la procédure de promotion. (MAIR-494, [#196](https://github.com/mairie360/Message_API/pull/196))
- La description de [#196](https://github.com/mairie360/Message_API/pull/196) annonce aussi un relèvement de `rustls-webpki` et `webpki-roots` pour débloquer `cargo audit` ; cette modification n'apparaît pas dans le diff fourni.
- Bump de `utoipa-swagger-ui` 10.0.1 → 10.0.2 ([#194](https://github.com/mairie360/Message_API/pull/194)). Le diff est absent du fichier fourni.

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Les stacks compose de dev, integration, security et performance connectent l'API avec le rôle `message_api` (mot de passe `message-api-password`) et non plus avec le superuser. Liquibase reçoit `-Dmessage_api_password`. Une requête hors des grants de ce rôle échoue donc en local et en CI. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))
- Le journal des releases est rédigé par Claude sur `main`, grâce au transfert des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` vers le workflow CICD. (MAIR-571, [#190](https://github.com/mairie360/Message_API/pull/190))
#### Technique
- Bumps de CI : `mairie360/CICD` passe de v4.2.2 à v4.3.1 ([#191](https://github.com/mairie360/Message_API/pull/191)), après un passage de v4.2.1 à v4.2.2 ([#187](https://github.com/mairie360/Message_API/pull/187)).
- Images de base de données et de migrations repinnées en `dev-949384c`, avec `TEST_DB_VERSION`. La base lit `users.email` pour le middleware JWT. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))
- Bumps de dépendances : `tokio` 1.53.3 → 1.54.0 ([#192](https://github.com/mairie360/Message_API/pull/192)) et l'image Rust de `development.Dockerfile` repinnée par digest ([#185](https://github.com/mairie360/Message_API/pull/185)). Le diff de [#192](https://github.com/mairie360/Message_API/pull/192) est absent du fichier fourni.
- `mairie360_api_lib` passe en 4.2.0 sans changement de source, et `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369. (MAIR-556, [#186](https://github.com/mairie360/Message_API/pull/186))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 01:47 UTC | dev | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39527958201) |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
