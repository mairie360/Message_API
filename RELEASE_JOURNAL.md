# Journal des releases — Message_API

<!-- release-journal-state {"dev": "525bc8c7e7d7c02588813b82e2fa7ba449508e60", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-03 09:41 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `525bc8c` | 2026-11-03 09:41 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Changements de comportement

- Les jetons HS256 ne sont plus acceptés une fois `JWT_SECRET` retiré : l'API vérifie les jetons de session avec la clé publique Ed25519 de Core (`JWT_PUBLIC_KEY`, PEM) et, sans `JWT_SECRET`, tout jeton HS256 reçoit `401`. Pendant la transition, la lib accepte encore HS256 tant que `JWT_SECRET` est défini. Les jetons doivent être signés en `EdDSA` par Core, avec `iss` et `aud` attendus ; un appelant qui signe encore avec l'ancien secret partagé est refusé. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))
- Pour les déploiements : dans les fichiers compose de test, `JWT_SECRET` est remplacé par `JWT_PUBLIC_KEY` (obligatoire). La stack dev prend par défaut la clé publique de test de la lib, avec `JWT_ALLOW_WEAK_SECRET` (dev uniquement). Le diff de cette PR est tronqué : le détail des fichiers non vus n'est pas confirmé.

#### Corrections

- Les journaux de requêtes ne contiennent plus les valeurs saisies. Avant : la query string (`?limit=...`) et le `Debug` des erreurs serde, qui cite la valeur refusée, étaient écrits dans les logs. Maintenant : seul le chemin est tracé, l'erreur est décrite par son type, et le relais SSE ne journalise plus la valeur d'un événement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique

- Digests Docker épinglés dans les fichiers compose (Renovate, [#217](https://github.com/mairie360/Message_API/pull/217)) ; image runtime de l'API passée en distroless Debian 13 avec builder et image de dev en trixie (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)) ; images `rust:1.99` de dev repassées par digest (#201, #206, #207).
- Dépendances Rust : zstd, ctutils et une exception `RUSTSEC-2026-0387` dans `cargo audit` (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)), redis 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)).
- CI : CICD monté de v4.4.0 à v5.2.2 (MAIR-612, #198, #200, #203 ; MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)), contrôle RGPD des PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)). Matrice d'accès MAIR-288 ([#161](https://github.com/mairie360/Message_API/pull/161)) : fichier et tests uniquement, sans effet à l'exécution.

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement

- Les jetons HS256 ne sont plus acceptés une fois `JWT_SECRET` retiré : l'API vérifie les jetons de session avec la clé publique Ed25519 de Core (`JWT_PUBLIC_KEY`, PEM) et, sans `JWT_SECRET`, tout jeton HS256 reçoit `401`. Pendant la transition, la lib accepte encore HS256 tant que `JWT_SECRET` est défini. Les jetons doivent être signés en `EdDSA` par Core, avec `iss` et `aud` attendus ; un appelant qui signe encore avec l'ancien secret partagé est refusé. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))
- Pour les déploiements : dans les fichiers compose de test, `JWT_SECRET` est remplacé par `JWT_PUBLIC_KEY` (obligatoire). La stack dev prend par défaut la clé publique de test de la lib, avec `JWT_ALLOW_WEAK_SECRET` (dev uniquement). Le diff de cette PR est tronqué : le détail des fichiers non vus n'est pas confirmé.

#### Corrections

- Les journaux de requêtes ne contiennent plus les valeurs saisies. Avant : la query string (`?limit=...`) et le `Debug` des erreurs serde, qui cite la valeur refusée, étaient écrits dans les logs. Maintenant : seul le chemin est tracé, l'erreur est décrite par son type, et le relais SSE ne journalise plus la valeur d'un événement mal formé. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique

- Digests Docker épinglés dans les fichiers compose (Renovate, [#217](https://github.com/mairie360/Message_API/pull/217)) ; image runtime de l'API passée en distroless Debian 13 avec builder et image de dev en trixie (MAIR-741, [#215](https://github.com/mairie360/Message_API/pull/215)) ; images `rust:1.99` de dev repassées par digest (#201, #206, #207).
- Dépendances Rust : zstd, ctutils et une exception `RUSTSEC-2026-0387` dans `cargo audit` (MAIR-740, [#213](https://github.com/mairie360/Message_API/pull/213)), redis 1.8.0 ([#208](https://github.com/mairie360/Message_API/pull/208)), tokio 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)).
- CI : CICD monté de v4.4.0 à v5.2.2 (MAIR-612, #198, #200, #203 ; MAIR-771, [#219](https://github.com/mairie360/Message_API/pull/219)), contrôle RGPD des PR (MAIR-295, [#157](https://github.com/mairie360/Message_API/pull/157)). Matrice d'accès MAIR-288 ([#161](https://github.com/mairie360/Message_API/pull/161)) : fichier et tests uniquement, sans effet à l'exécution.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
