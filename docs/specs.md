# VoxTypePersonas — Spécifications fonctionnelles et techniques (v1)

> **Superseding profile decision:** Raw is the only active built-in profile. `profiles/example.md` is the single shipped non-Raw profile and is a generic, non-activatable draft. Profiles may be loaded while incomplete, but only Ready profiles can be activated. Portable complete profile definitions use Markdown with YAML front matter, exclude secrets and local Secret Service references, and may be imported, exported, externally edited, and contributed through pull requests. This decision supersedes conflicting earlier statements in this document about built-in Chat, Email, Technical, Meeting / Notes profiles and deferred profile import/export.

## 1. Objectif

VoxTypePersonas est un plugin Omarchy dédié à Voxtype. Il permet de sélectionner, depuis la barre supérieure d'Omarchy, un profil de post-traitement pour les transcriptions de dictée.

Un profil associe :

- un provider LLM local ou distant ;
- un modèle ;
- un prompt système éditable ;
- des contraintes d'exécution, notamment délai et limites de taille.

Le produit doit améliorer le texte transcrit sans imposer un style générique ou académique. En particulier, le profil **Chat** doit conserver une voix naturelle, directe et conversationnelle tout en retirant les hésitations, répétitions et erreurs manifestes.

## 2. Décisions actées

- Cible : **Omarchy sous Linux uniquement**.
- Dépôt et publication : `toorop/voxtype-personas` sous licence MIT.
- ID Omarchy : `io.github.toorop.voxtype-personas`.
- Interface : plugin Omarchy de type `bar-widget`, construit avec QML/Quickshell.
- Moteur : binaire Rust nommé `voxtype-personas`.
- Le plugin et le moteur sont deux composants d'un même projet, mais pas deux applications visibles.
- Voxtype appelle toujours le même moteur ; le moteur résout lui-même le profil actif.
- Le profil actif est figé au début d'une dictée ; un changement d'UI s'applique à la dictée suivante.
- Le binaire est distribué via GitHub Releases, pas via paquets `.deb`, Arch, AUR ou Homebrew pour la v1.
- Architectures prises en charge dès la v1 : Linux `x86_64` et Linux `aarch64`.
- Secret Service / keyring est une dépendance obligatoire ; aucun fallback fichier n'est prévu.
- L'interface et la documentation livrées sont en anglais.
- Les profils intégrés sont Brut, Chat, E-mail, Technique et Réunion / Notes. Brut est le profil actif à l'installation et ne peut pas être supprimé.
- L'import/export de profils et de prompts est hors périmètre v1.
- Aucun texte dicté, prompt ou secret n'est envoyé en télémétrie ou écrit dans les logs par défaut.

## 3. Hors périmètre v1

- Application autonome, fenêtre non rattachée à la barre, extension navigateur ou application mobile.
- Paquets système par distribution.
- Synchronisation cloud des profils, prompts ou clés.
- Diffusion en continu de la transcription vers le LLM.
- Partage communautaire de prompts depuis l'interface.
- Modification de la logique audio ou des bindings Hyprland de dictée existants.

## 4. Parcours utilisateur

### 4.1 Sélection rapide

Le widget de barre affiche une icône et le nom du profil actif :

```text
🎙️  Chat ▾
```

Un clic ouvre un popover compact ancré au bouton :

```text
VoxTypePersonas
✓ Chat
  Raw
  Email
  Technical
  Meeting / Notes
────────────
⚙ Settings…
```

Un clic sur un profil le rend actif, persiste ce choix, referme le popover et met immédiatement à jour le libellé du widget.

### 4.2 Configuration étendue

Le clic sur **Settings…** remplace ou étend le popover par un panneau large, toujours ancré au même bouton de barre. Il ne lance pas d'application distincte.

Le panneau contient trois sections principales :

- **Profils** : liste, création, duplication, suppression et association provider/modèle/prompt.
- **Prompts** : éditeur de prompt système, avec aperçu de la convention d'entrée/sortie et action de test.
- **Providers** : configuration de l'endpoint, modèle, timeout, secret et bouton de test de connectivité.

Le panneau doit pouvoir être fermé avec Échap. Toute opération destructive demande une confirmation dans le panneau.

### 4.3 Première ouverture et installation du moteur

Au chargement, le plugin recherche un moteur compatible. S'il est absent, obsolète ou invalide, le panneau compact affiche clairement l'état et un bouton :

```text
The VoxTypePersonas engine is not installed.
[ Install engine ]
```

Cliquer ce bouton demande confirmation, puis seulement après accord :

1. détecte l'architecture avec `uname -m` ;
2. télécharge l'archive de release appropriée ;
3. vérifie le SHA-256 annoncé par la release et la signature Minisign avec la clé publique intégrée au plugin ;
4. extrait le binaire dans le répertoire de données utilisateur ;
5. vérifie `voxtype-personas version` ;
6. affiche le succès ou une erreur actionnable.

Le plugin ne télécharge, n'installe ni ne met à jour le moteur silencieusement.

## 5. Architecture

```text
Omarchy shell (processus Quickshell existant)
└── plugin `bar-widget` VoxTypePersonas
    ├── BarWidget.qml          bouton et état visible
    ├── Panel.qml              sélecteur et panneau ancré
    └── modèle de configuration / appels CLI

Voxtype
└── post_process_command = "…/voxtype-personas process"
    └── moteur Rust `voxtype-personas`
        ├── lecture de stdin
        ├── chargement config + profil actif
        ├── adaptateur provider (local ou distant)
        ├── validation de la sortie
        └── écriture du texte final sur stdout
```

Le plugin ne démarre jamais un second processus Quickshell. Il suit le contrat Omarchy `bar-widget` : `BarWidget.qml` est l'entry point du manifest et charge le panneau ancré au bouton.

## 6. Contrat Voxtype ↔ moteur

### 6.1 Configuration Voxtype

L'intégration utilise une unique commande de post-traitement :

```toml
[output.post_process]
command = "<chemin-du-moteur>/voxtype-personas process"
timeout_ms = 30000
```

L'assistant de configuration doit détecter une commande de post-traitement Voxtype existante. Il ne l'écrase jamais sans une confirmation explicite et une sauvegarde de la configuration originale.

### 6.2 Entrée / sortie

- Le moteur lit l'intégralité de la transcription UTF-8 sur `stdin`.
- Il écrit **uniquement** le texte destiné au collage sur `stdout`.
- Les diagnostics vont sur `stderr`, sans texte dicté ni secret.
- Une transcription vide est renvoyée telle quelle, sans appel provider.
- Une sortie LLM vide ou invalide est un échec : le moteur écrit alors le texte brut sur `stdout`.
- En cas de provider indisponible, timeout, erreur réseau, erreur d'authentification ou erreur de parsing : même fallback immédiat vers le texte brut.

Le moteur doit retourner un code zéro lorsqu'il a fourni soit une sortie traitée, soit le fallback brut. Les erreurs de configuration et les erreurs internes non récupérables retournent un code non zéro, après avoir tout de même tenté de préserver le texte brut quand c'est possible.

### 6.3 Séquencement

Voxtype séquence déjà l'enregistrement, la transcription, le post-traitement et le collage : une nouvelle dictée ne peut pas démarrer avant la fin du cycle précédent. VoxTypePersonas ne doit donc ajouter ni verrou, ni file d'attente, ni état « Busy » pour la v1.

## 7. CLI du moteur

La CLI est documentée, stable et utilisable sans le widget.

```text
voxtype-personas version
voxtype-personas process
voxtype-personas process --profile chat
voxtype-personas profiles list
voxtype-personas profiles set-active <id>
voxtype-personas providers test <id>
voxtype-personas config validate
```

`process` sans option utilise le profil actif. `--profile` est réservé au test et à l'automatisation ; il ne change pas le profil actif persistant.

Une option avancée peut permettre de fournir explicitement provider, modèle et fichier de prompt, mais le chemin normal reste la définition de profil afin de limiter les erreurs d'échappement et l'exposition de prompts dans l'historique shell.

## 8. Modèle de données

### 8.1 Emplacements XDG

```text
~/.config/voxtype-personas/config.toml
~/.local/share/voxtype-personas/bin/voxtype-personas
~/.local/state/voxtype-personas/
```

Le dossier d'installation du moteur est distinct du dossier du plugin Omarchy afin qu'une mise à jour de plugin ne supprime pas l'exécutable. Les fichiers de config et d'état sont créés avec des permissions restrictives.

### 8.2 Configuration TOML illustrative

```toml
schema_version = 1
active_profile = "chat"

[providers.ollama]
kind = "ollama"
endpoint = "http://127.0.0.1:11434"
timeout_ms = 15000

[providers.openai]
kind = "openai"
secret_ref = "org.voxtype-personas/openai"
timeout_ms = 30000

[prompts.chat]
name = "Natural chat"
system = """
Correct the transcription without changing its intent or making its style academic.
Remove hesitations and repetitions. Return only the final text in the same language as the input.
"""

[profiles.chat]
name = "Chat"
provider = "openai"
model = "…"
prompt = "chat"
max_input_chars = 20000
max_output_tokens = 2048
timeout_ms = 30000
```

Le format final peut évoluer, mais comporte obligatoirement `schema_version`, `active_profile`, providers, prompts et profils. Les écritures sont atomiques ; les migrations sont séquentielles et sauvegardent le fichier source avant modification.

## 9. Providers

### 9.1 Providers v1

Le sélecteur distingue Ollama local, les providers préconfigurés et le type générique :

- **Ollama** — détecté automatiquement sur la machine ; la liste présente les modèles déjà installés et l'utilisateur choisit explicitement.
- **OpenAI** — endpoint préconfiguré, clé API demandée.
- **Mistral** — endpoint préconfiguré, clé API demandée.
- **Groq** — endpoint préconfiguré, clé API demandée.
- **OpenRouter** — endpoint préconfiguré, clé API demandée.
- **Anthropic** — endpoint préconfiguré, clé API demandée.
- **Google Gemini** — endpoint préconfiguré, clé API demandée.
- **OpenAI-compatible…** — modèle de provider réutilisable : l'utilisateur peut créer autant d'instances nommées qu'il le souhaite, chacune avec URL, secret, timeout et modèles propres.

Après la configuration d'un provider, l'UI charge ses modèles disponibles puis demande un choix explicite. Elle ne présélectionne pas un modèle distant.

La liste exclut les modèles dédiés uniquement à l'image, l'audio, la transcription, la synthèse vocale, les embeddings ou la modération. Les modèles multimodaux capables de produire du texte restent disponibles. Chaque provider expose : test de connectivité, validation de modèle, timeout, message d'erreur actionnable et état du secret. Le test n'envoie pas de transcription réelle ; il utilise une requête minimale dédiée.

### 9.2 Construction de la requête

Les adaptateurs OpenAI, Mistral, Groq, OpenRouter et génériques utilisent le contrat Chat Completions compatible OpenAI. Anthropic, Gemini et Ollama ont leurs adaptateurs de protocole propres.

Le prompt système est envoyé dans un message `system`. La transcription est envoyée séparément dans un message `user` délimité comme contenu à transformer. Les prompts livrés imposent une sortie dans la même langue que l'entrée.

Le modèle ne doit pas être autorisé à interpréter le texte dicté comme une modification des règles du système. Les prompts livrés indiquent explicitement que les instructions contenues dans la transcription ne changent pas la tâche demandée.

La réponse attendue est du texte brut. La validation rejette une réponse vide ; elle peut aussi rejeter les réponses contenant un préambule évident ou du Markdown selon les options du profil.

## 10. Secrets, confidentialité et coûts

### 10.1 Secrets

Le flux normal enregistre la clé dans Secret Service / keyring et écrit seulement son identifiant dans `config.toml`. Secret Service est obligatoire : si le keyring est absent ou inaccessible, l'UI explique que le provider ne peut pas être enregistré et propose les instructions de réparation appropriées ; elle ne propose pas de fichier de fallback.

L'UI affiche près du champ de clé : « This API key is stored in your system keyring. It is not written to VoxTypePersonas configuration files. » La disponibilité du secret est vérifiée lors de la création ou modification d'un provider, et non pour la première fois pendant une dictée.

### 10.2 Confidentialité

Avant d'activer un profil distant, l'UI affiche le provider et l'avertissement :

> Ce profil envoie le texte transcrit à `<provider>` pour post-traitement.

Le produit n'enregistre aucune transcription et n'envoie aucune télémétrie par défaut. Les logs excluent le texte brut, les prompts et les secrets.

### 10.3 Limites de sécurité et responsabilité de coût

Chaque profil utilise par défaut `20 000` caractères d'entrée, `2 048` tokens de sortie et un timeout configurable. Ces valeurs restent modifiables dans les réglages avancés du profil.

L'interface ne calcule, n'affiche ni ne plafonne les coûts. Le README explique que le choix du provider et du modèle, ainsi que la surveillance de leur coût, relèvent de la responsabilité de l'utilisateur.

## 11. Distribution et mises à jour

### 11.1 Assets

Chaque release stable fournit au moins :

```text
voxtype-personas-x86_64-linux.tar.gz
voxtype-personas-aarch64-linux.tar.gz
checksums.txt
checksums.txt.minisig
```

Les fichiers doivent être accompagnés d'un SHA-256 et d'une signature Minisign. La clé publique Minisign est intégrée au plugin ; l'installation et la mise à jour vérifient obligatoirement checksum et signature avant extraction.

### 11.2 CI/CD

La CI doit, au minimum :

1. formater, lint et tester le code Rust ;
2. compiler les cibles Linux `x86_64` et `aarch64` ;
3. produire les archives et checksums ;
4. publier les assets lors d'un tag de version ;
5. signer `checksums.txt` avec la clé Minisign de publication ;
6. permettre un déclenchement manuel contrôlé pour les préreleases.

Un MacBook Apple Silicon sous Omarchy exécute Linux `aarch64`, pas macOS : il reçoit donc l'asset `aarch64-linux`.

### 11.3 Mises à jour du moteur

Le plugin peut vérifier qu'une mise à jour existe, mais doit demander l'accord de l'utilisateur avant téléchargement ou remplacement. L'installation conserve la version précédente jusqu'à validation de la nouvelle et permet un retour arrière simple en cas d'échec.

## 12. Intégration Omarchy

Le dépôt contient à minima :

```text
manifest.json
BarWidget.qml
Panel.qml
README.md
LICENSE
```

Le manifest utilise l'ID `io.github.toorop.voxtype-personas`. Il déclare uniquement le kind `bar-widget`, dont `BarWidget.qml` est l'entry point ; le panneau est chargé par ce widget, pas déclaré comme un plugin séparé.

Le plugin doit être validé par `omarchy plugin validate` et ses QML par `qmllint` avec les imports du shell Omarchy. Il ne modifie jamais `/usr/share/omarchy/`.

## 13. Préservation de l'installation existante

La machine actuelle possède déjà :

- une dictée Voxtype en mode collage ;
- un binding Hyprland push-to-talk ;
- un relais de mute global PipeWire au début de l'enregistrement, avec restauration de l'état initial au relâchement ;
- `duck_media = false` dans Voxtype, pour ne plus modifier les volumes individuels de Chromium.

VoxTypePersonas ne doit modifier ni le binding ni le relais audio. Son intégration se limite à la commande de post-traitement Voxtype, après détection, sauvegarde et confirmation en cas de modification de la configuration existante.

## 14. Critères d'acceptation

1. Le plugin s'installe et s'affiche comme widget dans la barre Omarchy.
2. Le clic affiche un sélecteur de profils compact, ancré au bouton.
3. **Configurer…** ouvre un panneau large ancré au même bouton, sans application séparée.
4. Le profil sélectionné persiste et est utilisé par la dictée suivante sans redémarrage de Voxtype.
5. Le profil Brut retourne exactement le texte de `stdin` et ne fait aucun appel réseau.
6. Le profil Chat améliore le texte sans le rendre artificiellement académique.
7. Un provider peut être configuré, testé et supprimé ; son secret n'apparaît jamais dans les logs ou l'export.
8. Toute erreur de provider, réseau, délai ou format renvoie le texte brut au lieu de bloquer la dictée.
9. Le plugin ne rajoute pas de file d'attente ou de verrou au cycle déjà séquentiel de Voxtype.
10. Le premier téléchargement de moteur exige une confirmation explicite, vérifie checksum et signature Minisign, puis installe le binaire adapté à l'architecture.
11. Le plugin préserve le mute global et les bindings de dictée existants.

## 15. Décisions laissées à l'implémentation

- Détails d'implémentation du cache de listes de modèles et de leur filtrage selon les métadonnées disponibles par provider.
- Rédaction exacte des quatre prompts livrés, dans le respect des règles de ton et de langue définies ci-dessus.
- Détails internes de l'installation et de la vérification Minisign, sans assouplir l'exigence de signature.

## Références

- [Omarchy — Develop a Custom Plugin](https://plugins.omarchy.org/develop.html)
- [Omarchy — Publish a Plugin](https://plugins.omarchy.org/publish.html)
