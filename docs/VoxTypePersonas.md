# VoxTypePersonas — notes de cadrage

## Intention

Créer un projet exclusivement destiné à Omarchy qui permet de choisir simplement le « persona » de post-traitement d'une dictée Voxtype. L'objectif n'est pas de rendre toute dictée plus formelle : chaque contexte doit conserver le ton voulu, notamment un style conversationnel, naturel et peu académique.

Le livrable idéal est un plugin Omarchy installable via son système de plugins. Il doit offrir une présence dans la barre supérieure d'Omarchy pour afficher et sélectionner le profil actif.

Nom retenu : **VoxTypePersonas**.

La spécification v1 fait autorité : [specs.md](specs.md). Les décisions finalisées sont Rust, dépôt `toorop/voxtype-personas`, licence MIT, plugin Omarchy `io.github.toorop.voxtype-personas`, keyring obligatoire et moteur distribué en releases Linux `x86_64` et `aarch64` signées Minisign.

## Problème résolu

Voxtype produit une première transcription locale rapide avec Whisper. Le modèle local peut déjà supprimer une partie des bafouillages sur des phrases assez longues, mais il manque parfois de contexte et commet des erreurs de transcription.

Un second modèle, plus capable et hébergé à distance via API ou en local, peut améliorer ce texte avant son insertion : corriger les erreurs, retirer hésitations et répétitions, respecter du vocabulaire, et adapter la rédaction au contexte. Il ne doit cependant pas uniformiser la voix de l'utilisateur ni transformer une conversation en prose académique.

## Fonctionnement Voxtype à exploiter

Voxtype doit appeler une seule commande de post-traitement stable : le moteur neutre `voxtype-personas`. La sélection du profil ne dépend donc pas des profils internes de Voxtype et ne nécessite pas de recharger son démon après un changement dans l'interface.

La commande reçoit la transcription brute sur `stdin` et doit écrire uniquement le texte final sur `stdout` :

```text
Voxtype (Whisper local)
  → stdin de `voxtype-personas process`
  → lecture du profil actif et de ses réglages
  → appel éventuel du LLM local ou distant, avec prompt système propre au profil
  → stdout du script
  → collage par Voxtype dans l'application active
```

En cas d'échec, d'absence de réseau ou de dépassement du délai, le moteur doit renvoyer la transcription brute afin que la dictée reste utilisable.

Le moteur devra envoyer :

- le texte brut dans un message `user` ;
- les règles permanentes du profil dans un message `system` ;
- sur `stdout`, uniquement la réponse finale sans préambule, guillemets, Markdown ou explication.

Les informations sensibles (clé API) ne doivent pas être affichées par l'interface ou consignées dans les logs.

## Modèle de configuration

Un **provider** définit le moyen d'appeler un modèle : Ollama local, API OpenAI, Mistral, Groq, OpenRouter, Anthropic, Gemini ou endpoint compatible OpenAI. Un type générique doit pouvoir être instancié plusieurs fois. Il contient notamment l'endpoint, les modèles disponibles, le délai et une référence vers le secret du keyring.

Un **prompt** contient le prompt système éditable qui définit le traitement souhaité : ton conversationnel, corrections à appliquer, vocabulaire à respecter, niveau de reformulation, etc.

Un **profil** est l'association d'un provider, d'un modèle et d'un prompt, avec ses paramètres éventuels. Le profil actif est stocké localement et appliqué à la dictée suivante.

La CLI neutre doit aussi permettre les tests et l'automatisation explicites :

```text
voxtype-personas process --profile chat
voxtype-personas process --provider openai --model <modele> --prompt-file <prompt>
```

L'usage courant reste sans options : `voxtype-personas process` résout le profil actif. Le profil **Brut** fait un simple passage de `stdin` vers `stdout`, sans appel réseau.

## Profils envisagés

- **Brut** — aucun post-traitement ; transcription locale, instantanée et privée.
- **Chat** — retire hésitations et répétitions, corrige les contresens évidents, conserve un ton cool, direct et conversationnel ; ne sur-formalise pas.
- **E-mail** — rend le texte clair et correctement structuré, tout en restant fidèle à l'intention et au niveau de formalité dicté.
- **Technique** — préserve strictement les noms de produits, termes techniques, code, commandes et jargon ; corrige seulement ce qui est manifestement erroné.
- **Réunion / notes** — formulation plus complète et lisible ; à préciser selon le besoin réel.

Chaque profil doit être défini par un prompt système éditable, et idéalement par son propre fournisseur, modèle et délai. Les prompts livrés demandent une sortie dans la même langue que l'entrée.

## UX Omarchy recherchée

La barre supérieure doit afficher le profil actif, par exemple :

```text
🎙️  Brut ▾
```

Le menu permet de sélectionner immédiatement : Brut, Chat, E-mail, Technique, Réunion, puis d'éventuels profils personnels.

Le changement de profil doit être visible, persister entre les sessions et être pris en compte par la dictée suivante sans redémarrage manuel de Voxtype si possible.

Le clic sur le bouton ouvre d'abord ce menu compact. Une entrée **Configurer…** ouvre ensuite une grande fenêtre toujours ancrée au même bouton de barre, et non une application séparée. Cette vue étendue permet de gérer les profils, prompts et providers ; l'éditeur de prompt doit disposer d'une vraie zone de texte.

Le widget et son panneau doivent être un unique plugin Omarchy de type `bar-widget` : le widget QML charge lui-même le panneau attaché à son bouton.

## Architecture à explorer

```text
Plugin Omarchy
├── composant barre / menu Quickshell
├── panneau de configuration ancré au widget
├── stockage local du profil actif
├── gestion des profils, prompts et providers
└── moteur neutre `voxtype-personas`
    ├── stdin → texte final sur stdout
    ├── appels LLM locaux et distants
    └── fallback vers la transcription brute
```

Le post-traitement Voxtype appellera le moteur neutre. Le moteur lit le profil actif à l'exécution ; le binding Hyprland de dictée existant n'a donc pas à changer pour sélectionner les profils.

Le projet doit s'intégrer aux conventions Omarchy plutôt que modifier des fichiers sous `/usr/share/omarchy/`. Les personnalisations et le plugin doivent vivre dans les emplacements utilisateur prévus par Omarchy.

## Moteur binaire et distribution

Le plugin QML et le moteur binaire sont deux composants distincts du même projet : l'interface appelle le moteur, sans exposer une application séparée à l'utilisateur. Le moteur est écrit en Rust.

À la première ouverture, le plugin vérifie la présence et la version du moteur. S'il est absent, il affiche une demande explicite d'installation. Après accord, il télécharge l'asset correspondant depuis la GitHub Release officielle, vérifie son checksum — idéalement aussi sa signature — puis l'installe dans un emplacement de données utilisateur tel que `~/.local/share/voxtype-personas/bin/`.

Il ne faut pas de paquet par distribution (`.deb`, paquet Arch, etc.) : un binaire Linux par architecture suffit. Les releases initiales doivent au minimum proposer :

- `x86_64-linux` — PC classiques et MacBooks Intel ;
- `aarch64-linux` — MacBooks Apple Silicon exécutant Omarchy sous Linux.

Le plugin détecte l'architecture avec `uname -m` et télécharge l'asset adapté. La CI doit produire ces binaires et les publier dans une GitHub Release lors d'un tag de version ou d'un lancement manuel contrôlé.

## Secrets et comportement à l'exécution

Les clés API sont stockées dans le keyring/Secret Service obligatoire ; la configuration du provider ne conserve qu'une référence au secret. Aucun fallback fichier n'est prévu.

Dans le modèle de menace d'une machine personnelle avec disque chiffré, le keyring automatiquement déverrouillé n'apporte pas nécessairement une forte protection supplémentaire pendant une session ouverte. Il évite néanmoins les fuites accidentelles via fichiers de configuration, sauvegardes, logs ou Git.

L'accès au secret est vérifié pendant la configuration du provider, jamais pour la première fois au moment d'une dictée. Si un secret est inaccessible, si le provider échoue ou si le réseau manque, le moteur doit revenir immédiatement au texte brut.

## Robustesse opérationnelle

- Le profil doit être figé au démarrage de chaque dictée : un changement de profil pendant la transcription ne s'applique qu'à la dictée suivante.
- Voxtype séquence déjà les dictées, la transcription, le post-traitement et le collage ; le projet n'ajoute ni file d'attente ni verrou.
- L'interface de configuration doit proposer un bouton de test de provider, qui vérifie endpoint, secret, modèle et latence sans utiliser une dictée réelle.
- Chaque profil fixe un délai et des limites de sécurité larges. L'interface ne calcule ni ne limite les coûts : cette responsabilité appartient à l'utilisateur et est documentée dans le README.
- Un profil distant doit indiquer explicitement quel provider reçoit le texte transcrit. Aucun texte dicté ne doit être enregistré ou envoyé en télémétrie par défaut.
- Une réponse vide, manifestement invalide ou non conforme au format attendu (commentaire, Markdown, etc.) doit être traitée comme un échec et déclencher le fallback vers le texte brut.
- Le texte dicté doit rester du contenu `user`, clairement séparé du prompt système ; il ne doit pas pouvoir modifier les règles du profil par injection d'instructions.
- La configuration doit avoir une version, être écrite atomiquement et pouvoir être migrée lors d'une évolution de format.
- Une mise à jour du moteur doit être proposée et confirmée, jamais appliquée silencieusement. Prévoir un retour simple à la version précédente si la mise à jour échoue.
- L'import/export de profils et prompts est reporté après la v1.

## État actuel de l'installation

Voxtype est configuré avec Whisper local et une dictée pilotée par un binding Hyprland. La sortie utilise le collage.

Pendant une dictée, la sortie audio globale PipeWire est maintenant mutée par un relais local, puis son état initial est restauré au relâchement. Le mécanisme `duck_media` de Voxtype est désactivé : il modifiait les volumes par application et pouvait laisser Chromium muet. Cette partie ne relève pas directement de VoxTypePersonas, mais le projet devra éviter de la casser.

## Décisions de conception à prendre lors des specs

- Format et emplacement précis des définitions de profils, prompts, providers et du profil actif.
- Fournisseur(s) d'API et modèle par défaut ; possibilité de profils entièrement locaux.
- Format de release, mécanisme de mise à jour et vérification des signatures/checksums du moteur.
- Choix du langage d'implémentation du moteur : Go ou Rust.
- Comportement précis en cas d'erreur, de timeout, de réseau indisponible ou de réponse vide.
- Sélection de profil pendant une dictée : à interdire, différer à la prochaine dictée, ou prendre effet immédiatement.
- Édition des prompts et profils : fichiers, UI Omarchy, ou les deux.
- Indication d'activité et d'erreur dans la barre, sans divulguer le texte dicté.
- Installation, mise à jour et désinstallation via le système de plugins Omarchy.
- Compatibilité avec la configuration existante de Voxtype et sauvegarde/restauration des modifications éventuelles.
- Politique de concurrence entre plusieurs dictées et expérience d'annulation.
- Plafonds de coût et de volume pour les providers distants.

## Critères de réussite initiaux

1. Installer le plugin Omarchy et voir le profil actif dans la barre.
2. Changer de profil en deux actions au plus.
3. La dictée suivante utilise le bon script et le bon prompt système.
4. Le profil Brut ne réalise aucun appel réseau.
5. Un échec du LLM ne bloque jamais l'insertion de la transcription Whisper.
6. Le profil Chat améliore les erreurs sans rendre le texte artificiellement académique.

## Références

- [Documentation Omarchy — Develop a Custom Plugin](https://plugins.omarchy.org/develop.html) — référence principale pour le contrat du plugin, le widget de barre et son panneau ancré.
