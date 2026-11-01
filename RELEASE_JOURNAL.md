# Journal des releases — Message_API

<!-- release-journal-state {"dev": "7c85c0bdc0b478d180c5837049721e2124fd91fa", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-01 12:42 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `7c85c0b` | 2026-11-01 12:42 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Corrections
- Journaux de requêtes : avant, la query string et la valeur refusée par serde (une adresse e-mail, un numéro de téléphone) pouvaient apparaître dans le span racine des requêtes et dans l'évènement d'erreur. Maintenant le span ne garde que le chemin, l'erreur est décrite par son type, et le relais SSE ne journalise plus la valeur d'un évènement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))
- Image d'exécution : avant, la base distroless Debian 12 embarquait une libssl3 non corrigée (CVE-2026-84782). Maintenant l'image est `gcr.io/distroless/cc-debian13:nonroot` et le binaire est compilé sur Debian 13 (builder `rust:1.99-slim-trixie`). (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215))

#### Technique
- Dépendances Rust : zstd 0.14.2 → 0.14.3, zstd-safe 8.1.0 → 8.2.0, zstd-sys 2.1.1 → 2.1.2 et ctutils 0.4.2 → 0.4.7 pour corriger les alertes RustSec ; RUSTSEC-2026-0387 (crypto-bigint) est ignorée dans `.cargo/audit.toml` avec sa justification. (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213))
- Mises à jour Renovate : redis 1.7.1 → 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.0 → 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)), et digests de `rust:1.99-bookworm` dans `development.Dockerfile` ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)).
- Tests et contrôles ajoutés : test des images sur Debian 13 (`tests/dockerfile_test.rs`), matrice d'accès `access-matrix.yaml` vérifiée contre l'OpenAPI (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)), marqueur de données personnelles dans les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)), et checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)).
- Pins de workflows mairie360/CICD : v4.4.0 → v4.4.1 ([#198](https://github.com/mairie360/Message_API/pull/198)), v4.4.1 → v4.4.2 ([#200](https://github.com/mairie360/Message_API/pull/200)), puis v4.4.2 → v4.4.3 pour la release prod ([#203](https://github.com/mairie360/Message_API/pull/203)).

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Journaux de requêtes : avant, la query string et la valeur refusée par serde (une adresse e-mail, un numéro de téléphone) pouvaient apparaître dans le span racine des requêtes et dans l'évènement d'erreur. Maintenant le span ne garde que le chemin, l'erreur est décrite par son type, et le relais SSE ne journalise plus la valeur d'un évènement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))
- Image d'exécution : avant, la base distroless Debian 12 embarquait une libssl3 non corrigée (CVE-2026-84782). Maintenant l'image est `gcr.io/distroless/cc-debian13:nonroot` et le binaire est compilé sur Debian 13 (builder `rust:1.99-slim-trixie`). (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215))

#### Technique
- Dépendances Rust : zstd 0.14.2 → 0.14.3, zstd-safe 8.1.0 → 8.2.0, zstd-sys 2.1.1 → 2.1.2 et ctutils 0.4.2 → 0.4.7 pour corriger les alertes RustSec ; RUSTSEC-2026-0387 (crypto-bigint) est ignorée dans `.cargo/audit.toml` avec sa justification. (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213))
- Mises à jour Renovate : redis 1.7.1 → 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.0 → 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)), et digests de `rust:1.99-bookworm` dans `development.Dockerfile` ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207)).
- Tests et contrôles ajoutés : test des images sur Debian 13 (`tests/dockerfile_test.rs`), matrice d'accès `access-matrix.yaml` vérifiée contre l'OpenAPI (MAIR-288, [#161](https://github.com/mairie360/Message_API/pull/161)), marqueur de données personnelles dans les journaux (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154)), et checklist RGPD sur les PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)).
- Pins de workflows mairie360/CICD : v4.4.0 → v4.4.1 ([#198](https://github.com/mairie360/Message_API/pull/198)), v4.4.1 → v4.4.2 ([#200](https://github.com/mairie360/Message_API/pull/200)), puis v4.4.2 → v4.4.3 pour la release prod ([#203](https://github.com/mairie360/Message_API/pull/203)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
