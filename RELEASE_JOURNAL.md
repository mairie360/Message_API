# Journal des releases — Message_API

<!-- release-journal-state {"dev": "4b069704734cafc70dd3a7c9a4f3f79749beb396", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-05 02:04 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `4b06970` | 2026-11-05 02:04 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Changements de comportement
- Les jetons de session sont vérifiés avec la clé publique Ed25519 de Core, lue dans `JWT_PUBLIC_KEY` ; `JWT_SECRET` n'est plus lu. Un jeton HS256, quel que soit son secret, reçoit `401` : un appelant qui signe encore avec un secret partagé doit obtenir un jeton émis par Core. Le déploiement doit fournir `JWT_PUBLIC_KEY` (PEM) à la place de `JWT_SECRET`. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Les journaux de requête ne contiennent plus les valeurs envoyées. Avant, le span racine enregistrait la query string (`http.target`) et l'erreur de désérialisation telle que `Debug` la formatait, avec la valeur refusée. Maintenant, seul le chemin est enregistré et l'erreur est décrite par son type. Le relais SSE ne journalise plus la valeur d'un événement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Garde-fous et tests : simulation d'usage RGPD avant prod (MAIR-497, [#221](https://github.com/mairie360/Message_API/pull/221)), matrice d'accès `access-matrix.yaml` vérifiée contre l'OpenAPI (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)), checklist RGPD des PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)) et test de version CICD (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)).
- Image d'exécution passée en distroless Debian 13, builder et image de dev sur trixie (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)) ; alertes RustSec de `cargo audit` corrigées, RUSTSEC-2026-0387 ignorée avec justification (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)).
- Images de test et de dev épinglées par digest (Renovate, [#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207), [#217](https://github.com/mairie360/Message_API/pull/217), [#225](https://github.com/mairie360/Message_API/pull/225), [#226](https://github.com/mairie360/Message_API/pull/226)).
- Mises à jour de dépendances Rust (actix-web 4.18.1, redis 1.8.0 puis 1.8.1, tokio 1.54.1 : [#209](https://github.com/mairie360/Message_API/pull/209), [#208](https://github.com/mairie360/Message_API/pull/208), [#205](https://github.com/mairie360/Message_API/pull/205)) ; le diff de ces PR n'est pas visible dans le journal.
- Workflows CICD montés de v4.4.0 à v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)), puis v5.2.2 ([#219](https://github.com/mairie360/Message_API/pull/219)) et v5.3.1 ([#221](https://github.com/mairie360/Message_API/pull/221), [#223](https://github.com/mairie360/Message_API/pull/223)).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les jetons de session sont vérifiés avec la clé publique Ed25519 de Core, lue dans `JWT_PUBLIC_KEY` ; `JWT_SECRET` n'est plus lu. Un jeton HS256, quel que soit son secret, reçoit `401` : un appelant qui signe encore avec un secret partagé doit obtenir un jeton émis par Core. Le déploiement doit fournir `JWT_PUBLIC_KEY` (PEM) à la place de `JWT_SECRET`. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Les journaux de requête ne contiennent plus les valeurs envoyées. Avant, le span racine enregistrait la query string (`http.target`) et l'erreur de désérialisation telle que `Debug` la formatait, avec la valeur refusée. Maintenant, seul le chemin est enregistré et l'erreur est décrite par son type. Le relais SSE ne journalise plus la valeur d'un événement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Garde-fous et tests : simulation d'usage RGPD avant prod (MAIR-497, [#221](https://github.com/mairie360/Message_API/pull/221)), matrice d'accès `access-matrix.yaml` vérifiée contre l'OpenAPI (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)), checklist RGPD des PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)) et test de version CICD (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)).
- Image d'exécution passée en distroless Debian 13, builder et image de dev sur trixie (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)) ; alertes RustSec de `cargo audit` corrigées, RUSTSEC-2026-0387 ignorée avec justification (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)).
- Images de test et de dev épinglées par digest (Renovate, [#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207), [#217](https://github.com/mairie360/Message_API/pull/217), [#225](https://github.com/mairie360/Message_API/pull/225), [#226](https://github.com/mairie360/Message_API/pull/226)).
- Mises à jour de dépendances Rust (actix-web 4.18.1, redis 1.8.0 puis 1.8.1, tokio 1.54.1 : [#209](https://github.com/mairie360/Message_API/pull/209), [#208](https://github.com/mairie360/Message_API/pull/208), [#205](https://github.com/mairie360/Message_API/pull/205)) ; le diff de ces PR n'est pas visible dans le journal.
- Workflows CICD montés de v4.4.0 à v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)), puis v5.2.2 ([#219](https://github.com/mairie360/Message_API/pull/219)) et v5.3.1 ([#221](https://github.com/mairie360/Message_API/pull/221), [#223](https://github.com/mairie360/Message_API/pull/223)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
