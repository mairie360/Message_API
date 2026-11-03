# Journal des releases — Message_API

<!-- release-journal-state {"dev": "3597cbf284493105e2e9294ff9efbd97992b0ec1", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-03 01:35 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `3597cbf` | 2026-11-03 01:35 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Changements de comportement
- L'API ne lit plus `JWT_SECRET` : elle vérifie les jetons avec la clé publique Ed25519 de Core, fournie par `JWT_PUBLIC_KEY` (PEM, plusieurs blocs pendant une rotation). Sans `JWT_SECRET`, tout jeton HS256 est refusé en `401`. Tant que `JWT_SECRET` reste défini, la lib accepte encore les HS256 signés avec lui (transition) ; un déploiement le retire dès que `JWT_PUBLIC_KEY` est en place. Les appelants doivent présenter des jetons EdDSA émis par Core. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Journaux de requêtes : avant, la span racine de chaque requête contenait la query string (ex. `?limit=<nom>`), et l'erreur serde citait la valeur refusée (e-mail, téléphone). Maintenant, la span ne garde que le chemin, l'erreur est décrite par son type et son contexte, et un événement SSE mal formé ne journalise plus la valeur citée. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Stacks de test : clés Ed25519 générées à chaque run, jetons signés comme Core, newman et k6 reçoivent leurs jetons par variables ou fichier ; `mairie360_api_lib` passe en 4.3.0. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))
- Image d'exécution distroless Debian 13 et builder `trixie` pour corriger CVE-2026-84782, avec un test qui vérifie la version Debian de chaque image. (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215))
- Workflows CI et promotion : mairie360/CICD passe en v5.2.2 (contrôles RGPD), avec un test de cohérence des versions. (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219))
- Audit RustSec : zstd, zstd-safe, zstd-sys et ctutils mis à jour ; RUSTSEC-2026-0387 (crypto-bigint) ignoré avec sa justification. (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213))
- Checklist RGPD sur les pull requests (`gdpr-pr.yml`) et matrice d'accès `access-matrix.yaml` avec son test, sans effet sur l'API. (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157) ; MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161))
- Test du marqueur RGPD (`gdpr_marker_test.sh`, `gdpr-marker.yaml`) qui cherche les valeurs de test dans les journaux. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))
- Dépendances : redis 1.7.1 → 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.0 → 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)), et digests de l'image de développement ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)), remplacés ensuite par trixie.
- Pins mairie360/CICD v4.4.1 → v4.4.3 pour corriger la promotion en prod (création du tag et de la release). ([#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- L'API ne lit plus `JWT_SECRET` : elle vérifie les jetons avec la clé publique Ed25519 de Core, fournie par `JWT_PUBLIC_KEY` (PEM, plusieurs blocs pendant une rotation). Sans `JWT_SECRET`, tout jeton HS256 est refusé en `401`. Tant que `JWT_SECRET` reste défini, la lib accepte encore les HS256 signés avec lui (transition) ; un déploiement le retire dès que `JWT_PUBLIC_KEY` est en place. Les appelants doivent présenter des jetons EdDSA émis par Core. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Journaux de requêtes : avant, la span racine de chaque requête contenait la query string (ex. `?limit=<nom>`), et l'erreur serde citait la valeur refusée (e-mail, téléphone). Maintenant, la span ne garde que le chemin, l'erreur est décrite par son type et son contexte, et un événement SSE mal formé ne journalise plus la valeur citée. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- Stacks de test : clés Ed25519 générées à chaque run, jetons signés comme Core, newman et k6 reçoivent leurs jetons par variables ou fichier ; `mairie360_api_lib` passe en 4.3.0. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))
- Image d'exécution distroless Debian 13 et builder `trixie` pour corriger CVE-2026-84782, avec un test qui vérifie la version Debian de chaque image. (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215))
- Workflows CI et promotion : mairie360/CICD passe en v5.2.2 (contrôles RGPD), avec un test de cohérence des versions. (MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219))
- Audit RustSec : zstd, zstd-safe, zstd-sys et ctutils mis à jour ; RUSTSEC-2026-0387 (crypto-bigint) ignoré avec sa justification. (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213))
- Checklist RGPD sur les pull requests (`gdpr-pr.yml`) et matrice d'accès `access-matrix.yaml` avec son test, sans effet sur l'API. (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157) ; MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161))
- Test du marqueur RGPD (`gdpr_marker_test.sh`, `gdpr-marker.yaml`) qui cherche les valeurs de test dans les journaux. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))
- Dépendances : redis 1.7.1 → 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.0 → 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)), et digests de l'image de développement ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)), remplacés ensuite par trixie.
- Pins mairie360/CICD v4.4.1 → v4.4.3 pour corriger la promotion en prod (création du tag et de la release). ([#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
