# Journal des releases — Message_API

<!-- release-journal-state {"dev": "b71b66c8994b824cfed1712dc602194fc6e94a17", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "b71b66c8994b824cfed1712dc602194fc6e94a17", "updated": {"dev": "2026-11-12 12:58 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-11-12 18:52 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `b71b66c` | 2026-11-12 18:52 UTC |
| dev | — | `b71b66c` | 2026-11-12 12:58 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

- fix: MAIR-262 Integration tests - API functional tests, BFF business tests and regression tests (#236) ([#236](https://github.com/mairie360/Message_API/pull/236))
- chore(deps): update gcr.io/distroless/cc-debian13:nonroot docker digest to cc66d22 (#231) ([#231](https://github.com/mairie360/Message_API/pull/231))
- fix(deps): MAIR-740 APIs: fix the cargo audit RustSec alerts (actix-http, libsqlite3-sys) (#235) ([#235](https://github.com/mairie360/Message_API/pull/235))
- chore(deps): update rust:1.99-trixie docker digest to 8a4e90c (#230) ([#230](https://github.com/mairie360/Message_API/pull/230))
- chore(deps): update rust:1.99-trixie docker digest to bc2c4c2 (#229) ([#229](https://github.com/mairie360/Message_API/pull/229))
- chore(deps): pin node.js to c86ecea (#226) ([#226](https://github.com/mairie360/Message_API/pull/226))
- feat: MAIR-497 GDPR - Usage simulation and AI review of logs and database before prod ([#221](https://github.com/mairie360/Message_API/pull/221))
- chore(deps): update rust:1.99-trixie docker digest to 4c4c8ec (#225) ([#225](https://github.com/mairie360/Message_API/pull/225))
- chore(deps): update rust dependencies (#209) ([#209](https://github.com/mairie360/Message_API/pull/209))
- chore(deps): pin dependencies (#217) ([#217](https://github.com/mairie360/Message_API/pull/217))
- fix: MAIR-680 [Audit 23/10] Toutes les APIs et BFFs : remplacer le secret JWT HS256 partagé par 12 services (A1) ([#223](https://github.com/mairie360/Message_API/pull/223))
- ci: MAIR-771 APIs, BFFs, Database : monter CICD en v5.2.2 pour activer les contrôles RGPD (#219) ([#219](https://github.com/mairie360/Message_API/pull/219))
- fix: MAIR-741 APIs : passer l'image d'exécution en distroless Debian 13 (CVE-2026-84782) (#215) ([#215](https://github.com/mairie360/Message_API/pull/215))
- fix(deps): MAIR-740 APIs : corriger les alertes RustSec de cargo audit (crypto-bigint, ctutils, zstd) (#213) ([#213](https://github.com/mairie360/Message_API/pull/213))
- ci: MAIR-295 Ajouter une checklist RGPD aux PR et des rappels d'échéances automatiques (#157) ([#157](https://github.com/mairie360/Message_API/pull/157))
- feat: MAIR-288 Définir la matrice d'accès des APIs et la vérifier depuis l'OpenAPI (#161) ([#161](https://github.com/mairie360/Message_API/pull/161))
- fix: MAIR-290 Détecter les données personnelles dans les journaux avec un test du marqueur (#154) ([#154](https://github.com/mairie360/Message_API/pull/154))
- chore(deps): update rust crate redis to v1.8.0 (#208) ([#208](https://github.com/mairie360/Message_API/pull/208))
- chore(deps): update rust:1.99-bookworm docker digest to 48b7f43 (#207) ([#207](https://github.com/mairie360/Message_API/pull/207))
- chore(deps): update rust:1.99-bookworm docker digest to edaa2a8 (#206) ([#206](https://github.com/mairie360/Message_API/pull/206))
- chore(deps): update rust crate tokio to v1.54.1 (#205) ([#205](https://github.com/mairie360/Message_API/pull/205))
- ci: MAIR-612 Release prod: align internal versions and deploy (#203) ([#203](https://github.com/mairie360/Message_API/pull/203))
- chore(deps): update rust:1.99-bookworm docker digest to 5c1a173 (#201) ([#201](https://github.com/mairie360/Message_API/pull/201))
- ci: MAIR-612 Release prod: align internal versions and deploy (#200) ([#200](https://github.com/mairie360/Message_API/pull/200))
- ci: MAIR-612 Release prod: align internal versions and deploy (#198) ([#198](https://github.com/mairie360/Message_API/pull/198))

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

- fix: MAIR-262 Integration tests - API functional tests, BFF business tests and regression tests (#236) ([#236](https://github.com/mairie360/Message_API/pull/236))
- chore(deps): update gcr.io/distroless/cc-debian13:nonroot docker digest to cc66d22 (#231) ([#231](https://github.com/mairie360/Message_API/pull/231))
- fix(deps): MAIR-740 APIs: fix the cargo audit RustSec alerts (actix-http, libsqlite3-sys) (#235) ([#235](https://github.com/mairie360/Message_API/pull/235))
- chore(deps): update rust:1.99-trixie docker digest to 8a4e90c (#230) ([#230](https://github.com/mairie360/Message_API/pull/230))
- chore(deps): update rust:1.99-trixie docker digest to bc2c4c2 (#229) ([#229](https://github.com/mairie360/Message_API/pull/229))
- chore(deps): pin node.js to c86ecea (#226) ([#226](https://github.com/mairie360/Message_API/pull/226))
- feat: MAIR-497 GDPR - Usage simulation and AI review of logs and database before prod ([#221](https://github.com/mairie360/Message_API/pull/221))
- chore(deps): update rust:1.99-trixie docker digest to 4c4c8ec (#225) ([#225](https://github.com/mairie360/Message_API/pull/225))
- chore(deps): update rust dependencies (#209) ([#209](https://github.com/mairie360/Message_API/pull/209))
- chore(deps): pin dependencies (#217) ([#217](https://github.com/mairie360/Message_API/pull/217))
- fix: MAIR-680 [Audit 23/10] Toutes les APIs et BFFs : remplacer le secret JWT HS256 partagé par 12 services (A1) ([#223](https://github.com/mairie360/Message_API/pull/223))
- ci: MAIR-771 APIs, BFFs, Database : monter CICD en v5.2.2 pour activer les contrôles RGPD (#219) ([#219](https://github.com/mairie360/Message_API/pull/219))
- fix: MAIR-741 APIs : passer l'image d'exécution en distroless Debian 13 (CVE-2026-84782) (#215) ([#215](https://github.com/mairie360/Message_API/pull/215))
- fix(deps): MAIR-740 APIs : corriger les alertes RustSec de cargo audit (crypto-bigint, ctutils, zstd) (#213) ([#213](https://github.com/mairie360/Message_API/pull/213))
- ci: MAIR-295 Ajouter une checklist RGPD aux PR et des rappels d'échéances automatiques (#157) ([#157](https://github.com/mairie360/Message_API/pull/157))
- feat: MAIR-288 Définir la matrice d'accès des APIs et la vérifier depuis l'OpenAPI (#161) ([#161](https://github.com/mairie360/Message_API/pull/161))
- fix: MAIR-290 Détecter les données personnelles dans les journaux avec un test du marqueur (#154) ([#154](https://github.com/mairie360/Message_API/pull/154))
- chore(deps): update rust crate redis to v1.8.0 (#208) ([#208](https://github.com/mairie360/Message_API/pull/208))
- chore(deps): update rust:1.99-bookworm docker digest to 48b7f43 (#207) ([#207](https://github.com/mairie360/Message_API/pull/207))
- chore(deps): update rust:1.99-bookworm docker digest to edaa2a8 (#206) ([#206](https://github.com/mairie360/Message_API/pull/206))
- chore(deps): update rust crate tokio to v1.54.1 (#205) ([#205](https://github.com/mairie360/Message_API/pull/205))
- ci: MAIR-612 Release prod: align internal versions and deploy (#203) ([#203](https://github.com/mairie360/Message_API/pull/203))
- chore(deps): update rust:1.99-bookworm docker digest to 5c1a173 (#201) ([#201](https://github.com/mairie360/Message_API/pull/201))
- ci: MAIR-612 Release prod: align internal versions and deploy (#200) ([#200](https://github.com/mairie360/Message_API/pull/200))
- ci: MAIR-612 Release prod: align internal versions and deploy (#198) ([#198](https://github.com/mairie360/Message_API/pull/198))

> Résumé Claude indisponible pour ce passage : listes brutes des commits.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-12 18:52 UTC | staging | — | `b71b66c` | [run](https://github.com/mairie360/Message_API/actions/runs/43282717597) |
| 2026-11-12 12:58 UTC | dev | — | `b71b66c` | [run](https://github.com/mairie360/Message_API/actions/runs/43220703802) |
| 2026-11-12 11:52 UTC | dev | — | `b2d2f9e` | [run](https://github.com/mairie360/Message_API/actions/runs/43211628549) |
| 2026-11-06 05:18 UTC | dev | — | `d810a63` | [run](https://github.com/mairie360/Message_API/actions/runs/42166863876) |
| 2026-11-05 02:04 UTC | dev | — | `4b06970` | [run](https://github.com/mairie360/Message_API/actions/runs/41951797525) |
| 2026-11-04 18:29 UTC | dev | — | `9cc1a61` | [run](https://github.com/mairie360/Message_API/actions/runs/41883197579) |
| 2026-11-04 02:13 UTC | dev | — | `db23053` | [run](https://github.com/mairie360/Message_API/actions/runs/41758287247) |
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
