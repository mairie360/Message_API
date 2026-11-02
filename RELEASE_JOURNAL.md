# Journal des releases — Message_API

<!-- release-journal-state {"dev": "85087ed501106541cdf46da89798b53163b1c7ad", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-02 23:00 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `85087ed` | 2026-11-02 23:00 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Corrections
- Les journaux de requêtes n'enregistrent plus la query string. Avant, une valeur passée dans `?limit=<nom>` ou une valeur refusée (e-mail, téléphone) apparaissait dans les logs JSON. Maintenant, seul le chemin est journalisé et l'erreur est décrite par son type, sans la valeur reçue. Le relais SSE ne journalise plus la valeur d'un événement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- CI et promotion : CICD passé en v5.2.2 avec la clé `ANTHROPIC_WORKSPACE_ID` transmise (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)), checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)), et montées de version CICD de v4.4.0 à v4.4.3 pour la promotion (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)).
- Image : runtime en distroless Debian 13 et builder en trixie, sans changement d'API (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)). Corrections des alertes RustSec de zstd et ctutils, avec une exception documentée pour crypto-bigint (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)).
- Tests et documentation : matrice d'accès `access-matrix.yaml` vérifiée contre l'OpenAPI (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)) et script de marqueur RGPD pour les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)).
- Dépendances : redis 1.7.1 → 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.0 → 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)), digests de l'image `rust:1.99-bookworm` de développement ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)).

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Les journaux de requêtes n'enregistrent plus la query string. Avant, une valeur passée dans `?limit=<nom>` ou une valeur refusée (e-mail, téléphone) apparaissait dans les logs JSON. Maintenant, seul le chemin est journalisé et l'erreur est décrite par son type, sans la valeur reçue. Le relais SSE ne journalise plus la valeur d'un événement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- CI et promotion : CICD passé en v5.2.2 avec la clé `ANTHROPIC_WORKSPACE_ID` transmise (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)), checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)), et montées de version CICD de v4.4.0 à v4.4.3 pour la promotion (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)).
- Image : runtime en distroless Debian 13 et builder en trixie, sans changement d'API (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)). Corrections des alertes RustSec de zstd et ctutils, avec une exception documentée pour crypto-bigint (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)).
- Tests et documentation : matrice d'accès `access-matrix.yaml` vérifiée contre l'OpenAPI (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)) et script de marqueur RGPD pour les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)).
- Dépendances : redis 1.7.1 → 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.0 → 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)), digests de l'image `rust:1.99-bookworm` de développement ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
