# Outils de diagnostic Operis

Outils jetables écrits pendant le travail sur FTE/TLF ([OPERIS.md](../OPERIS.md)). Ils ne font pas partie du workspace Cargo de Ruffle ni des tests de régression.

## `outils/abcdump`

Lecteur minimal du bytecode AVM2 (ABC) d'un SWF, fondé sur le crate `swf` du dépôt : liste les classes, méthodes et instructions. Il a servi à vérifier ce que fait réellement `spark.components.Label.truncateText` dans `spark_4.1.0.16076.swf`, chargé par Oxalis, et qui diffère des sources Flex publiées.

```sh
cd operis/outils/abcdump && cargo run --release -- <fichier.swf>
```

## `essais/`

Tests AS3 jetables de la coupure des lignes FTE (`fte-coupure-lignes`, `fte-largeur-insuffisante`). Ils sont compilés par le compilateur ASC intégré au harnais de tests (`[[compilers]]` dans `test.toml`).

**Le fichier `output.txt` n'est PAS une référence Flash Player** : c'est la sortie de Ruffle au moment de l'essai. Ces dossiers ne doivent donc pas être déplacés tels quels dans `tests/tests/swfs/` : un vrai test de régression demande une sortie produite par le Flash Player (voir `CONTRIBUTING.md` d'upstream).

Pour en lancer un ponctuellement : le copier temporairement dans `tests/tests/swfs/avm2/<nom>/`, puis `cargo test -p tests --test tests -- avm2/<nom>`.

## `PASSATION-DDC-OOM.md`

Enquête et correctif du plantage de Droits de Cités après la connexion en mode conteneur (OOM dans
`AvmString::concat`, PR #6), avec en annexe le diagnostic temporaire qui donne la pile AS d'une concaténation
démesurée.
