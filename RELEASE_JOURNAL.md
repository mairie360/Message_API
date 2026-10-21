# Journal des releases — Message_API

<!-- release-journal-state {"dev": "0505cab5a6e63b819c62bbffdf0e2b4cc702987c", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-10-21 22:20 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `0505cab` | 2026-10-21 22:20 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Technique
- Les workflows `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` passent de CICD v4.4.0 à v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)). Ce sont des changements de pipeline, sans effet sur le service lui-même.
- L'image de développement `development.Dockerfile` est épinglée au digest `rust:1.99-bookworm` `5c1a173` ([#201](https://github.com/mairie360/Message_API/pull/201)). Le Dockerfile de production n'est pas modifié.
- Mise à jour de la dépendance `tokio` de 1.54.0 à 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)). Le diff fourni ne contient pas le `Cargo.lock`, donc le changement de code effectif n'a pas pu être vérifié.

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Les workflows `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` passent de CICD v4.4.0 à v4.4.3 (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203)). Ce sont des changements de pipeline, sans effet sur le service lui-même.
- L'image de développement `development.Dockerfile` est épinglée au digest `rust:1.99-bookworm` `5c1a173` ([#201](https://github.com/mairie360/Message_API/pull/201)). Le Dockerfile de production n'est pas modifié.
- Mise à jour de la dépendance `tokio` de 1.54.0 à 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)). Le diff fourni ne contient pas le `Cargo.lock`, donc le changement de code effectif n'a pas pu être vérifié.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
