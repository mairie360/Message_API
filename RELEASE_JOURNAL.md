# Journal des releases — Message_API

<!-- release-journal-state {"dev": "4cd62043b5f79b794d1ed9b907b09b7c9db0b242", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-10-22 23:04 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `4cd6204` | 2026-10-22 23:04 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Technique
- Image de développement (`development.Dockerfile`) : la base `rust:1.99-bookworm` est épinglée au digest `48b7f43`, après les étapes `e12566d` → `5c1a173` puis `edaa2a8`. (Renovate, [#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207))
- Pins `mairie360/CICD` passés de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. Les versions intermédiaires corrigent la création du tag et de la release dans `promote-prod`. (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203))
- Mise à jour de tokio annoncée de 1.54.0 à 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)). Le diff n'est pas fourni, donc le changement de `Cargo.toml` ou `Cargo.lock` n'est pas vérifié.

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Image de développement (`development.Dockerfile`) : la base `rust:1.99-bookworm` est épinglée au digest `48b7f43`, après les étapes `e12566d` → `5c1a173` puis `edaa2a8`. (Renovate, [#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207))
- Pins `mairie360/CICD` passés de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. Les versions intermédiaires corrigent la création du tag et de la release dans `promote-prod`. (MAIR-612, [#198](https://github.com/mairie360/Message_API/pull/198), [#200](https://github.com/mairie360/Message_API/pull/200), [#203](https://github.com/mairie360/Message_API/pull/203))
- Mise à jour de tokio annoncée de 1.54.0 à 1.54.1 ([#205](https://github.com/mairie360/Message_API/pull/205)). Le diff n'est pas fourni, donc le changement de `Cargo.toml` ou `Cargo.lock` n'est pas vérifié.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
