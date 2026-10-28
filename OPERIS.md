# Fork Operis de Ruffle

Ce dépôt est le fork d'[Operis](https://github.com/Operis-Fr) de [ruffle-rs/ruffle](https://github.com/ruffle-rs/ruffle).
Il sert à exécuter dans un navigateur, sans Flash Player, les applications Flex historiques d'Operis
(Droits de Cités en Flex 2/3, Oxalis en Flex 4.1/Spark), lancées par [navigatis-web](https://github.com/Operis-Fr/navigatis-web).

Responsable : Damien Leroux ([@DLROperis](https://github.com/DLROperis)), avec [l'équipe Numerobis](https://github.com/orgs/Operis-Fr/teams/numerobis).

## Branches

| Branche | Rôle |
| --- | --- |
| `master` | Copie d'upstream (`ruffle-rs/ruffle`), sans modification Operis. |
| `operis/main` | **Branche d'intégration et branche par défaut** : upstream + correctifs Operis. C'est d'elle qu'est construit le Ruffle utilisé par navigatis-web. Protégée : modifications par PR uniquement. |
| `operis/compat-stubs` | Stub `adobe.utils.ProductManager` (sans lui, DdC s'arrêtait après la connexion). |
| `operis/fte-line-breaking` | Moteur de texte FTE pour les libellés Spark : coupure des lignes à la largeur, ligne de base, métriques, atomes, `getFontMetrics`, `getBaselinePosition`, correctif de `Label.truncateText` (Flex 4.1). |
| `operis/tlf-editing` | Édition TLF pour les champs de saisie Spark : événements clavier, `textInput` et presse-papiers vers l'objet qui a le focus, frontières d'atomes et de mots, menu contextuel presse-papiers. |

## Contribuer

- Une branche par sujet, créée depuis `operis/main`, nommée `operis/main__<sujet>` ; PR vers `operis/main`.
- **Toujours cibler le fork** : sur un fork, `gh pr create` propose par défaut le dépôt d'origine. Utiliser `gh pr create --repo Operis-Fr/ruffle --base operis/main`.
- Les correctifs génériques (sans rien de propre à Operis) ont vocation à être proposés à upstream, après relecture interne et accord, pour limiter la divergence du fork.
- Messages de commit en anglais, dans le style d'upstream (`avm2: …`, `core: …`, `web: …`).

## Se recaler sur upstream

```sh
git remote add upstream https://github.com/ruffle-rs/ruffle.git   # une fois
git fetch upstream
git switch master && git merge --ff-only upstream/master && git push origin master
git switch -c operis/main__maj-upstream origin/operis/main && git merge master
# résoudre les conflits, lancer les tests, puis PR vers operis/main
```

## Compiler et tester

Prérequis : Rust (cible `wasm32-unknown-unknown`), `wasm-bindgen-cli` à la version indiquée dans `web/README.md`, Node et npm ; `wasm-opt` (binaryen) recommandé pour optimiser le module.

```sh
cd web && npm install && npm run build          # paquet web/packages/selfhosted/dist
cargo test -p tests --test tests                # tests de régression (SWF)
```

Les nouveaux tests SWF demandent une sortie de référence produite par le vrai Flash Player ; à défaut, les correctifs sont couverts par des tests unitaires Rust.

## Suivi

L'état d'avancement, les défauts connus et le reste à faire sont suivis dans [navigatis-web/TODO.md](https://github.com/Operis-Fr/navigatis-web/blob/main/TODO.md).
