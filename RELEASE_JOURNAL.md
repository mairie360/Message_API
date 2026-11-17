# Journal des releases — Message_API

<!-- release-journal-state {"dev": "1535d240e8fe648210777a1ccc456c5ebc923d06", "prod": "b71b66c8994b824cfed1712dc602194fc6e94a17", "prod_tag": "v2.2.0", "staging": "b71b66c8994b824cfed1712dc602194fc6e94a17", "updated": {"dev": "2026-11-17 15:03 UTC", "prod": "2026-11-12 23:53 UTC", "staging": "2026-11-12 18:52 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v2.2.0 | `b71b66c` | 2026-11-12 23:53 UTC |
| staging | — | `b71b66c` | 2026-11-12 18:52 UTC |
| dev | — | `1535d24` | 2026-11-17 15:03 UTC |

## Ajouté sur main depuis la dernière release prod (v2.2.0)

- feat: MAIR-998 Test stacks follow the moving upstream tag of the stage (#251) ([#251](https://github.com/mairie360/Message_API/pull/251))
- chore(deps): pin actions/checkout action to 3d3c42e (#254) ([#254](https://github.com/mairie360/Message_API/pull/254))
- ci: MAIR-1008 Nightly rehearsal of the promotion gates (#249) ([#249](https://github.com/mairie360/Message_API/pull/249))
- test: MAIR-1002 Speed up the APIs' CI tests (Rust cache, one database per test, optimized dependencies) ([#253](https://github.com/mairie360/Message_API/pull/253))
- test: MAIR-1002 Speed up the APIs' CI tests (Rust cache, one database per test, optimized dependencies) ([#252](https://github.com/mairie360/Message_API/pull/252))
- chore(deps): update rust crate mairie360_api_lib to v4.4.0 (#232) ([#232](https://github.com/mairie360/Message_API/pull/232))
- fix: MAIR-247 Translate the APIs' OpenAPI descriptions and messages into English (#243) ([#243](https://github.com/mairie360/Message_API/pull/243))
- chore(deps): MAIR-243 Bump quinn-udp to the patched 0.5.16 (RUSTSEC-2026-0538) (#245) ([#245](https://github.com/mairie360/Message_API/pull/245))
- docs: MAIR-245 Rewrite the APIs' README and API.md (#234) ([#234](https://github.com/mairie360/Message_API/pull/234))
- chore: MAIR-243 Align the test Database version on the 3.4.0 release (#240) ([#240](https://github.com/mairie360/Message_API/pull/240))

## Différence dev → staging (en dev, pas encore en staging)

- feat: MAIR-998 Test stacks follow the moving upstream tag of the stage (#251) ([#251](https://github.com/mairie360/Message_API/pull/251))
- chore(deps): pin actions/checkout action to 3d3c42e (#254) ([#254](https://github.com/mairie360/Message_API/pull/254))
- ci: MAIR-1008 Nightly rehearsal of the promotion gates (#249) ([#249](https://github.com/mairie360/Message_API/pull/249))
- test: MAIR-1002 Speed up the APIs' CI tests (Rust cache, one database per test, optimized dependencies) ([#253](https://github.com/mairie360/Message_API/pull/253))
- test: MAIR-1002 Speed up the APIs' CI tests (Rust cache, one database per test, optimized dependencies) ([#252](https://github.com/mairie360/Message_API/pull/252))
- chore(deps): update rust crate mairie360_api_lib to v4.4.0 (#232) ([#232](https://github.com/mairie360/Message_API/pull/232))
- fix: MAIR-247 Translate the APIs' OpenAPI descriptions and messages into English (#243) ([#243](https://github.com/mairie360/Message_API/pull/243))
- chore(deps): MAIR-243 Bump quinn-udp to the patched 0.5.16 (RUSTSEC-2026-0538) (#245) ([#245](https://github.com/mairie360/Message_API/pull/245))
- docs: MAIR-245 Rewrite the APIs' README and API.md (#234) ([#234](https://github.com/mairie360/Message_API/pull/234))
- chore: MAIR-243 Align the test Database version on the 3.4.0 release (#240) ([#240](https://github.com/mairie360/Message_API/pull/240))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

> Résumé Claude indisponible pour ce passage : listes brutes des commits.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-17 15:03 UTC | dev | — | `1535d24` | [run](https://github.com/mairie360/Message_API/actions/runs/44107638870) |
| 2026-11-17 11:53 UTC | dev | — | `3ffcbba` | [run](https://github.com/mairie360/Message_API/actions/runs/44075598680) |
| 2026-11-17 01:30 UTC | dev | — | `42f940f` | [run](https://github.com/mairie360/Message_API/actions/runs/43994232469) |
| 2026-11-17 00:09 UTC | dev | — | `11f20b1` | [run](https://github.com/mairie360/Message_API/actions/runs/43981561706) |
| 2026-11-16 23:09 UTC | dev | — | `bfe0f63` | [run](https://github.com/mairie360/Message_API/actions/runs/43974682926) |
| 2026-11-16 17:24 UTC | dev | — | `651e84c` | [run](https://github.com/mairie360/Message_API/actions/runs/43916545476) |
| 2026-11-16 16:50 UTC | dev | — | `5a57d86` | [run](https://github.com/mairie360/Message_API/actions/runs/43909754028) |
| 2026-11-16 16:36 UTC | dev | — | `f570b7e` | [run](https://github.com/mairie360/Message_API/actions/runs/43905386173) |
| 2026-11-12 23:53 UTC | prod | v2.2.0 | `b71b66c` | [run](https://github.com/mairie360/Message_API/actions/runs/43330556457) |
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
