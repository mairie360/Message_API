# Journal des releases — Message_API

<!-- release-journal-state {"dev": "db230538476eb3e088d1b67bb2f0afceef6ce0cf", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-04 02:13 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `db23053` | 2026-11-04 02:13 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Changements de comportement
- Les tokens HS256 ne sont plus acceptés : `JWT_SECRET` n'est plus lu et l'API exige `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API, au format PEM). Sans `JWT_SECRET`, tout token HS256 reçoit `401` ; un appelant (BFF ou outil de test) qui signe encore avec le secret partagé n'est plus accepté. Le déploiement doit fournir `JWT_PUBLIC_KEY`. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Journaux des requêtes : avant, la span enregistrait la query string (ex. `?limit=<valeur>`) et l'erreur en `Debug`, qui reprenait la valeur refusée (e-mail, téléphone). Maintenant, seul le chemin est journalisé et l'erreur est décrite par son type et son contexte. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))
- Relais SSE : un événement mal formé est journalisé sans la valeur citée par serde. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Images, dépendances et CI : digests Docker épinglés dans les fichiers compose et `development.Dockerfile` (rust 1.99 trixie), image d'exécution distroless Debian 13 (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)), mises à jour Renovate (dont [#225](https://github.com/mairie360/Message_API/pull/225), [#209](https://github.com/mairie360/Message_API/pull/209), [#208](https://github.com/mairie360/Message_API/pull/208), [#205](https://github.com/mairie360/Message_API/pull/205)), correctifs RustSec de cargo audit (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)), CICD porté en v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)), alignements CICD v4.4.1 à v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)), checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)).
- Tests : matrice d'accès `access-matrix.yaml` et tests de refus (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)), test du marqueur RGPD dans les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)), tests des tokens EdDSA et du rejet HS256 (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223)).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les tokens HS256 ne sont plus acceptés : `JWT_SECRET` n'est plus lu et l'API exige `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API, au format PEM). Sans `JWT_SECRET`, tout token HS256 reçoit `401` ; un appelant (BFF ou outil de test) qui signe encore avec le secret partagé n'est plus accepté. Le déploiement doit fournir `JWT_PUBLIC_KEY`. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Journaux des requêtes : avant, la span enregistrait la query string (ex. `?limit=<valeur>`) et l'erreur en `Debug`, qui reprenait la valeur refusée (e-mail, téléphone). Maintenant, seul le chemin est journalisé et l'erreur est décrite par son type et son contexte. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))
- Relais SSE : un événement mal formé est journalisé sans la valeur citée par serde. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Images, dépendances et CI : digests Docker épinglés dans les fichiers compose et `development.Dockerfile` (rust 1.99 trixie), image d'exécution distroless Debian 13 (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)), mises à jour Renovate (dont [#225](https://github.com/mairie360/Message_API/pull/225), [#209](https://github.com/mairie360/Message_API/pull/209), [#208](https://github.com/mairie360/Message_API/pull/208), [#205](https://github.com/mairie360/Message_API/pull/205)), correctifs RustSec de cargo audit (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)), CICD porté en v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)), alignements CICD v4.4.1 à v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)), checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)).
- Tests : matrice d'accès `access-matrix.yaml` et tests de refus (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)), test du marqueur RGPD dans les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)), tests des tokens EdDSA et du rejet HS256 (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
