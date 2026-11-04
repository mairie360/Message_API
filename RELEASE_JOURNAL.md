# Journal des releases — Message_API

<!-- release-journal-state {"dev": "9cc1a61396c42a729e2313f844b9db84d138f22a", "prod": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "prod_tag": "v2.1.0", "staging": "71dd8d10cc668813b5012bb27c9aefc962ba30d9", "updated": {"dev": "2026-11-04 18:29 UTC", "prod": "2026-10-21 09:10 UTC", "staging": "2026-10-21 03:20 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.1.0 | `71dd8d1` | 2026-10-21 09:10 UTC |
| staging | — | `71dd8d1` | 2026-10-21 03:20 UTC |
| dev | — | `9cc1a61` | 2026-11-04 18:29 UTC |

## Ajouté sur main depuis la dernière release prod (v2.1.0)

#### Changements de comportement
- Les jetons de session HS256 sont refusés (401) : l'API ne lit plus `JWT_SECRET` et vérifie les jetons avec la clé publique Ed25519 de Core_API (`JWT_PUBLIC_KEY`). Avant : un jeton HS256 signé avec le secret partagé était accepté. Maintenant : il est refusé dès que `JWT_SECRET` est absent (tant qu'il est défini, la lib accepte encore HS256, en transition). Tout appelant qui signait ses propres jetons (BFF, scripts) doit utiliser des jetons EdDSA émis par Core_API. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Les logs de requête n'exposent plus les valeurs envoyées. Avant : la query string (par exemple `?limit=<nom>`) et le `Debug` des erreurs de désérialisation, qui cite la valeur refusée (e-mail, téléphone), étaient journalisés. Maintenant : seuls le chemin et la description du type d'erreur sont écrits, et un événement SSE malformé ne cite plus sa valeur. Les erreurs de base de données sont décrites par SQLSTATE, contrainte, table et colonne. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- CI/CD : CICD passe de v4.4.3 à v5.2.2 ([#219](https://github.com/mairie360/Message_API/pull/219)) puis v5.3.1 ([#221](https://github.com/mairie360/Message_API/pull/221)). Ajout de la simulation d'usage RGPD avant prod (`gdpr-simulation.yaml`, `docker-compose-gdpr-simulation.yml`, `gdpr_simulation_test.sh`), de la checklist RGPD des PR ([#157](https://github.com/mairie360/Message_API/pull/157)) et d'une garde de version des workflows. Sans effet sur l'API en service.
- Tests : `test_secrets.sh` génère une paire Ed25519 par exécution, et les stacks de test signent leurs jetons avec elle. Matrice d'accès `access-matrix.yaml` et test de couverture ([#161](https://github.com/mairie360/Message_API/pull/161)), sans changement d'exécution.
- Image d'exécution distroless Debian 13 et builder trixie, pour corriger CVE-2026-84782 ([#215](https://github.com/mairie360/Message_API/pull/215)). Corrections RustSec de zstd et ctutils ([#213](https://github.com/mairie360/Message_API/pull/213)).
- Dépendances : `mairie360_api_lib` en 3.0.1 puis 4.3.0, mises à jour Renovate ([#205](https://github.com/mairie360/Message_API/pull/205), [#208](https://github.com/mairie360/Message_API/pull/208), [#209](https://github.com/mairie360/Message_API/pull/209), dont le diff n'est pas visible ici), digests Docker épinglés ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207), [#217](https://github.com/mairie360/Message_API/pull/217), [#225](https://github.com/mairie360/Message_API/pull/225)).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les jetons de session HS256 sont refusés (401) : l'API ne lit plus `JWT_SECRET` et vérifie les jetons avec la clé publique Ed25519 de Core_API (`JWT_PUBLIC_KEY`). Avant : un jeton HS256 signé avec le secret partagé était accepté. Maintenant : il est refusé dès que `JWT_SECRET` est absent (tant qu'il est défini, la lib accepte encore HS256, en transition). Tout appelant qui signait ses propres jetons (BFF, scripts) doit utiliser des jetons EdDSA émis par Core_API. (MAIR-680, [#223](https://github.com/mairie360/Message_API/pull/223))

#### Corrections
- Les logs de requête n'exposent plus les valeurs envoyées. Avant : la query string (par exemple `?limit=<nom>`) et le `Debug` des erreurs de désérialisation, qui cite la valeur refusée (e-mail, téléphone), étaient journalisés. Maintenant : seuls le chemin et la description du type d'erreur sont écrits, et un événement SSE malformé ne cite plus sa valeur. Les erreurs de base de données sont décrites par SQLSTATE, contrainte, table et colonne. (MAIR-290, [#154](https://github.com/mairie360/Message_API/pull/154))

#### Technique
- CI/CD : CICD passe de v4.4.3 à v5.2.2 ([#219](https://github.com/mairie360/Message_API/pull/219)) puis v5.3.1 ([#221](https://github.com/mairie360/Message_API/pull/221)). Ajout de la simulation d'usage RGPD avant prod (`gdpr-simulation.yaml`, `docker-compose-gdpr-simulation.yml`, `gdpr_simulation_test.sh`), de la checklist RGPD des PR ([#157](https://github.com/mairie360/Message_API/pull/157)) et d'une garde de version des workflows. Sans effet sur l'API en service.
- Tests : `test_secrets.sh` génère une paire Ed25519 par exécution, et les stacks de test signent leurs jetons avec elle. Matrice d'accès `access-matrix.yaml` et test de couverture ([#161](https://github.com/mairie360/Message_API/pull/161)), sans changement d'exécution.
- Image d'exécution distroless Debian 13 et builder trixie, pour corriger CVE-2026-84782 ([#215](https://github.com/mairie360/Message_API/pull/215)). Corrections RustSec de zstd et ctutils ([#213](https://github.com/mairie360/Message_API/pull/213)).
- Dépendances : `mairie360_api_lib` en 3.0.1 puis 4.3.0, mises à jour Renovate ([#205](https://github.com/mairie360/Message_API/pull/205), [#208](https://github.com/mairie360/Message_API/pull/208), [#209](https://github.com/mairie360/Message_API/pull/209), dont le diff n'est pas visible ici), digests Docker épinglés ([#201](https://github.com/mairie360/Message_API/pull/201), [#206](https://github.com/mairie360/Message_API/pull/206), [#207](https://github.com/mairie360/Message_API/pull/207), [#217](https://github.com/mairie360/Message_API/pull/217), [#225](https://github.com/mairie360/Message_API/pull/225)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
