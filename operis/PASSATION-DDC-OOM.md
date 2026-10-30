# Passation : DdC plante Ruffle après la connexion (mode conteneur)

> **Résolu (30/10/2026)** : correctif dans la PR Operis-Fr/ruffle #6 (fusionnée), vérifié par une connexion complète
> à DdC Form en mode conteneur. La dichotomie a montré que la base upstream du fork (`36bd357f3`) plante déjà : ce
> n'est pas une régression d'un commit Operis.

Note de passation conservée pour mémoire. Le diagnostic temporaire qui a donné la pile AS (`op_add` et `Array.join`,
jamais fusionné) est reproduit en [annexe](#annexe--diagnostic-temporaire), à réappliquer en cas d'OOM semblable.

Le Ruffle de l'enquête était `e1d2fcfca`, c'est-à-dire `operis/main` + `operis/main__localconnection-js` (avec
`navigateToUrlHandler`), celui que sert le serveur de développement de navigatis-web.

## Symptôme

Droits de Cités (DdC Form, `DDCINSTR.swf`) ouvert dans le conteneur de navigatis-web :

1. l'écran de connexion s'affiche normalement ;
2. après la saisie de l'identifiant et du mot de passe, DdC appelle `deconnexion`, `connexion`, `NomInstance`,
   puis `GetEcranGeneral` (réponse SOAP d'environ 890 Ko) ;
3. environ une seconde plus tard, la mémoire du navigateur augmente de 2 Go, puis
   `RuntimeError: unreachable` et `Unable to lock Ruffle core` en boucle. L'application est figée.

La pile wasm (session précédente) :

```
std::alloc::rust_oom
alloc::raw_vec::handle_error
ruffle_common::avm_string::AvmString::concat
ruffle_core::avm2::activation::Activation::run_actions
ruffle_core::avm2::function::exec
ruffle_core::avm2::value::Value::call_method_with_args
```

Il s'agit donc d'une **concaténation de chaînes AS3 (opcode `add`) qui dépasse la mémoire du wasm**. Ce n'est pas une
fuite progressive : rien ne bouge avant la connexion.

## Ce qui est établi (essais headless du 30/10/2026, sans capture du poste)

| Cas | Résultat |
| --- | --- |
| Conteneur qui répond à `DDC_ModeAccesDonnees` (DdC en mode conteneur : `Fonctions=2`, `Donnees=1` dans `ddc.xml`) | Plantage après `GetEcranGeneral`, à chaque essai. |
| Conteneur qui **ne** répond **pas** à `DDC_ModeAccesDonnees` (DdC reste en mode web) | Connexion complète (24 appels SOAP), pas de plantage, mémoire stable (~1 Go). |

La poignée de main `DDC_ModeAccesDonnees` a été ajoutée par la PR navigatis-web #17 (DdC et Word) : c'est elle qui
a fait passer DdC en mode conteneur et qui a révélé le problème. Aucun appel `ExternalInterface` (`[EI]`) n'est
journalisé entre `DDC_ModeAccesDonnees` et le plantage.

## Résultat du diagnostic (30/10/2026, 09h)

Pile AS obtenue avec le diagnostic (annexe) (seuil mémoire wasm de 1,5 Go dans `op_add`) :

```
[diag] op_add sur 259215 + 200 caractères, mémoire 1500250112 octets, pile AS :
	at appia.modele::PreparerEcranBD$/grilleToChaineValeursChamps()
	at appia.bd::GestionContexteApplicatifBD/_initialiser()
	at appia.bd::GestionContexteApplicatifBD/_result()
	at appia.bd::ApAppelBD/result()
	at appia.services::ApService/_result()
	at appia.services::ApService/result()
```

Ce n'est donc **pas une chaîne infinie** : `valeurs` ne fait que 259 215 caractères quand la mémoire atteint 1,5 Go.
La mémoire est remplie par les **copies intermédiaires** de `valeurs += …`, que Ruffle ne libère pas tant que la
fonction AS n'est pas terminée (le ramasse-miettes de gc-arena ne passe qu'entre deux exécutions de script).

Cause probable dans Ruffle : `AvmString::concat` sait ajouter une chaîne « sur place » (sans recopie) seulement si les
deux chaînes ont la même largeur (`AvmStringRepr::try_append_inline`, `core/common/src/avm_string/repr.rs` :
« we could also in-place append a byte string to a wide string, but it was skipped for now »). Dès que `valeurs`
contient un caractère hors Latin-1 (chaîne 16 bits), chaque ajout d'un morceau 8 bits (`tbVal.join(ð)`, `ñ`)
recopie toute la chaîne avec une capacité doublée : environ 1 Mo par ajout, deux ajouts par ligne de la grille des
paramètres, des milliers de lignes, d'où plusieurs Go avant la fin de la fonction. Flash Player ne recopie pas.

Correctif (PR #6) : ajout en place d'une chaîne 8 bits à la fin d'une chaîne 16 bits (élargissement des octets dans
le tampon), avec deux tests unitaires dans `core/common/src/avm_string/avm_string.rs`. Vérifié par une connexion
complète à DdC Form en mode conteneur : écran principal, une trentaine d'appels SOAP, mémoire stable vers 1 Go.

Dichotomie : la base upstream du fork (`36bd357f3`, sans commit Operis) plante de la même façon en mode conteneur et
fonctionne en mode web. Aucune PR du fork n'est en cause ; le mode conteneur n'était simplement jamais utilisé avant
navigatis-web #17.

## Piste initiale côté DdC

En mode conteneur uniquement, après le chargement du contexte applicatif
(`libDDC/appia/bd/GestionContexteApplicatifBD.as`, vers les lignes 461 et 483), DdC sérialise la grille des
paramètres avant de l'envoyer au conteneur (`APPEL_PARAMETRES`) :

```actionscript
monApp.appelFonctionExterne([AccesBD.APPEL_PARAMETRES,
    PreparerEcranBD.grilleToChaineValeursChamps(_magrilleParam)], null);
```

`PreparerEcranBD.grilleToChaineValeursChamps` (`libDonnees/appia/modele/PreparerEcranBD.as`, ligne 968) :

- range chaque valeur dans `tbVal[curChamp.numeroOrdre]` (tableau creux) ;
- fait `lg2 = tbVal.length` **dans** la boucle des lignes, ce qui change la borne de la boucle des champs pour les
  lignes suivantes ;
- concatène `valeurs += tbVal.join(SEP_NIV3)` puis `SEP_NIV2`.

Si `numeroOrdre` vaut une très grande valeur dans Ruffle (lecture ou coercition `int` différente de Flash Player),
`tbVal.length` devient énorme et la concaténation explose. À vérifier : c'est une hypothèse, pas un fait.

## Diagnostic utilisé

- `op_add` : si la mémoire wasm dépasse 1,5 Go, ou si la somme des longueurs des opérandes chaînes dépasse
  16 millions de caractères, la pile AS est écrite dans la console (`[diag] op_add sur … caractères, mémoire …,
  pile AS :`) et une erreur AS 1506 est levée au lieu de saturer la mémoire.
- `Array.join` : au-delà d'un million d'éléments, la pile AS est écrite (`[diag] Array.join sur … éléments`).

Compiler (dans `web/`) : `npm run build --workspace=ruffle-core && npm run build --workspace=ruffle-selfhosted`, puis
servir `web/packages/selfhosted/dist` comme version supplémentaire de navigatis-web
(`NAVIGATIS_RUFFLE_VERSIONS=diag=<dist>`, puis `?ruffle=diag` sur le lanceur).

## Reproduire

1. Serveur de développement de navigatis-web (branche `main__navigatis-full`), voir son
   `documentation/developpement.md`.
2. Ouvrir le favori « DdC Form » dans le conteneur, se connecter (identifiants : demander à Damien, ne jamais les
   écrire dans un fichier ni un commit). Si DdC signale une session existante, répondre « Oui ».
3. Observer la console : la panne survient dans la seconde qui suit la réponse à `GetEcranGeneral`.

Script headless utilisé (Playwright, console et mémoire, garde-fou à 6 Go) : à reprendre de navigatis-web
`dev/scenarios/`. Comparer avec un conteneur qui ne répond pas à `DDC_ModeAccesDonnees` (renvoyer `null`).

## Suites

1. Fait : fonction AS trouvée (`PreparerEcranBD.grilleToChaineValeursChamps`), différence avec Flash Player
   corrigée dans Ruffle (PR #6), avec des tests unitaires Rust plutôt qu'un test SWF.
2. Sans objet : aucun contournement n'a été nécessaire dans navigatis-web.
3. Tout Ruffle servi à navigatis-web doit contenir la PR #6, sinon DdC replante après la connexion en mode conteneur.
4. Le correctif est générique et peut être proposé à upstream, en suivant la politique LLM d'upstream (OPERIS.md).

## Annexe : diagnostic temporaire

À appliquer avec `git apply` depuis la racine du dépôt (contexte d'`operis/main` au 30/10/2026), puis compiler le paquet
`selfhosted` (voir « Diagnostic utilisé »). Ne jamais le fusionner.

```diff
diff --git a/core/src/avm2/activation.rs b/core/src/avm2/activation.rs
index a994b7660..ebce40b6f 100644
--- a/core/src/avm2/activation.rs
+++ b/core/src/avm2/activation.rs
@@ -2033,6 +2033,24 @@ impl<'a, 'gc> Activation<'a, 'gc> {
         let value2 = self.pop_stack();
         let value1 = self.pop_stack();
 
+        // DIAGNOSTIC (ne pas fusionner) : opérande démesuré → pile AS dans la console, erreur AS
+        let taille = |v: &Value<'gc>| if let Value::String(s) = v { s.len() } else { 0 };
+        #[cfg(target_arch = "wasm32")]
+        let memoire = core::arch::wasm32::memory_size(0) * 65536;
+        #[cfg(not(target_arch = "wasm32"))]
+        let memoire = 0usize;
+        if memoire > 1_500_000_000 || taille(&value1) + taille(&value2) > 16_000_000 {
+            let pile = self.avm2().capture_call_stack();
+            tracing::error!(
+                "[diag] op_add sur {} + {} caractères, mémoire {} octets, pile AS :{}",
+                taille(&value1),
+                taille(&value2),
+                memoire,
+                pile
+            );
+            return Err(make_error_1506(self));
+        }
+
         let sum_value = match (value1, value2) {
             (Value::Integer(n1), Value::Integer(n2)) => {
                 if let Some(res) = n1.checked_add(n2) {
diff --git a/core/src/avm2/globals/array.rs b/core/src/avm2/globals/array.rs
index 4c2fff986..8a8682383 100644
--- a/core/src/avm2/globals/array.rs
+++ b/core/src/avm2/globals/array.rs
@@ -141,6 +141,11 @@ pub fn join<'gc>(
         // The array may be modified by user code while joining, so only
         // borrow it to read the length and each element.
         let length = array.storage().length();
+        // DIAGNOSTIC (ne pas fusionner)
+        if length > 1_000_000 {
+            let pile = activation.avm2().capture_call_stack();
+            tracing::error!("[diag] Array.join sur {} éléments, pile AS :{}", length, pile);
+        }
         let mut accum = Vec::with_capacity(length);
 
         for i in 0..length {
```
