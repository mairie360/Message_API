# Journal des releases — Message_API

<!-- release-journal-state {"dev": "0bd7c04f8ae023ea6cfb6234d7de125bcf73ab21", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-03 15:41 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `0bd7c04` | 2026-11-03 15:41 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Changements de comportement
- `JWT_SECRET` n'est plus lu par l'API : elle vérifie les jetons avec la clé publique Ed25519 de Core (`JWT_PUBLIC_KEY`, PEM). Sans `JWT_PUBLIC_KEY`, chaque jeton HS256 est refusé en 401, y compris ceux signés avec l'ancien secret partagé. Un appelant qui émet encore des HS256 doit passer à des jetons EdDSA signés par Core. Le diff est tronqué : ce point est tiré de CLAUDE.md et des compose. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Journaux : avant, la query string (par exemple `?limit=<valeur>`) et la valeur citée par serde dans le `Debug` de l'erreur (une adresse e-mail ou un téléphone refusé) étaient écrits dans les journaux. Maintenant, la span ne garde que le chemin, et l'erreur est décrite par son type et son contexte, sans la valeur reçue. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Dépendances Renovate : `actix-web` 4.17.0 → 4.18.1 et `redis` 1.8.0 → 1.8.1 (d'après la description, le diff est vide), `redis` 1.7.1 → 1.8.0 et `tokio` 1.54.0 → 1.54.1 (diffs vides dans l'export). `Cargo.lock` : zstd, zstd-safe, zstd-sys et ctutils mis à jour, et RUSTSEC-2026-0387 ignoré dans `.cargo/audit.toml` (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)).
- Image runtime distroless passée de Debian 12 à Debian 13 et builder `rust:1.99-slim-trixie`, `development.Dockerfile` en trixie, avec test `mair_741_every_image_runs_on_debian_13` (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)).
- Images des compose épinglées par digest (postgres, redis, curl, k6, nginx, node, newman, zap) et digest de `rust:1.99-bookworm` mis à jour trois fois dans `development.Dockerfile` (Renovate, [#217](https://github.com/mairie360/Message_API/pull/217), [#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)).
- Workflows CI : CICD passé de v4.4.0 à v4.4.1, v4.4.2 puis v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)), puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` et un test `mair_771_cicd_version.rs` (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)). Nouveau workflow de checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)).
- Tests et outillage : matrice d'accès `access-matrix.yaml` et test `access_matrix.rs`, avec la dépendance dev `yaml_serde` (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)) ; stack et scénario de test du marqueur RGPD dans les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)) ; tests et scripts de test d'intégration, ZAP, k6 et newman adaptés à la clé Ed25519 (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223)).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- `JWT_SECRET` n'est plus lu par l'API : elle vérifie les jetons avec la clé publique Ed25519 de Core (`JWT_PUBLIC_KEY`, PEM). Sans `JWT_PUBLIC_KEY`, chaque jeton HS256 est refusé en 401, y compris ceux signés avec l'ancien secret partagé. Un appelant qui émet encore des HS256 doit passer à des jetons EdDSA signés par Core. Le diff est tronqué : ce point est tiré de CLAUDE.md et des compose. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Journaux : avant, la query string (par exemple `?limit=<valeur>`) et la valeur citée par serde dans le `Debug` de l'erreur (une adresse e-mail ou un téléphone refusé) étaient écrits dans les journaux. Maintenant, la span ne garde que le chemin, et l'erreur est décrite par son type et son contexte, sans la valeur reçue. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Dépendances Renovate : `actix-web` 4.17.0 → 4.18.1 et `redis` 1.8.0 → 1.8.1 (d'après la description, le diff est vide), `redis` 1.7.1 → 1.8.0 et `tokio` 1.54.0 → 1.54.1 (diffs vides dans l'export). `Cargo.lock` : zstd, zstd-safe, zstd-sys et ctutils mis à jour, et RUSTSEC-2026-0387 ignoré dans `.cargo/audit.toml` (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)).
- Image runtime distroless passée de Debian 12 à Debian 13 et builder `rust:1.99-slim-trixie`, `development.Dockerfile` en trixie, avec test `mair_741_every_image_runs_on_debian_13` (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)).
- Images des compose épinglées par digest (postgres, redis, curl, k6, nginx, node, newman, zap) et digest de `rust:1.99-bookworm` mis à jour trois fois dans `development.Dockerfile` (Renovate, [#217](https://github.com/mairie360/Message_API/pull/217), [#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)).
- Workflows CI : CICD passé de v4.4.0 à v4.4.1, v4.4.2 puis v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)), puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` et un test `mair_771_cicd_version.rs` (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)). Nouveau workflow de checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)).
- Tests et outillage : matrice d'accès `access-matrix.yaml` et test `access_matrix.rs`, avec la dépendance dev `yaml_serde` (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)) ; stack et scénario de test du marqueur RGPD dans les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)) ; tests et scripts de test d'intégration, ZAP, k6 et newman adaptés à la clé Ed25519 (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-03 15:41 UTC | dev | — | `0bd7c04` | [run](https://github.com/mairie360/Message_API/actions/runs/41664122078) |
| 2026-11-03 09:41 UTC | dev | — | `525bc8c` | [run](https://github.com/mairie360/Message_API/actions/runs/41612021996) |
| 2026-11-03 01:35 UTC | dev | — | `3597cbf` | [run](https://github.com/mairie360/Message_API/actions/runs/41559223424) |
| 2026-11-02 23:00 UTC | dev | — | `85087ed` | [run](https://github.com/mairie360/Message_API/actions/runs/41539993387) |
| 2026-11-01 12:42 UTC | dev | — | `7c85c0b` | [run](https://github.com/mairie360/Message_API/actions/runs/41306772708) |
| 2026-10-22 23:04 UTC | dev | — | `4cd6204` | [run](https://github.com/mairie360/Message_API/actions/runs/39868457608) |
| 2026-10-22 03:40 UTC | dev | — | `df749ae` | [run](https://github.com/mairie360/Message_API/actions/runs/39717315791) |
| 2026-10-21 22:20 UTC | dev | — | `0505cab` | [run](https://github.com/mairie360/Message_API/actions/runs/39683997595) |
| 2026-10-21 09:11 UTC | dev | — | `d74a645` | [run](https://github.com/mairie360/Message_API/actions/runs/39573914704) |
| 2026-10-21 09:10 UTC | prod | v2.1.0 | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39573967470) |
| 2026-10-21 08:33 UTC | dev | — | `273e6cd` | [run](https://github.com/mairie360/Message_API/actions/runs/39568785077) |
| 2026-10-21 08:32 UTC | dev | — | `e215278` | [run](https://github.com/mairie360/Message_API/actions/runs/39568594235) |
| 2026-10-21 03:48 UTC | dev | — | `2dc9ac1` | [run](https://github.com/mairie360/Message_API/actions/runs/39539811319) |
| 2026-10-21 03:20 UTC | staging | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39537392816) |
| 2026-10-21 01:47 UTC | dev | — | `71dd8d1` | [run](https://github.com/mairie360/Message_API/actions/runs/39527958201) |
| 2026-10-20 17:07 UTC | staging | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
| 2026-10-19 20:24 UTC | dev | — | `75d7650` | [run](https://github.com/mairie360/Message_API/actions/runs/39316653706) |
<!-- release-journal-history:end -->
