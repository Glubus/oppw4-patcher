# Skill : Architecte Modulaire Rust

**Déclencheur :** À chaque fois que l'utilisateur demande de générer, refactoriser ou organiser du code Rust.

**Instructions de structuration :**
1. **Division stricte par fichiers :** Isole chaque composant logique (structs, enums, implémentations, traits) dépassant 50 lignes dans son propre fichier source `.rs`. INTERDICTION de faire des fichiers monolithiques (comme tout mettre dans `main.rs`).
2. **Arborescence explicite :** Avant de donner le code, affiche toujours l'arbre de l'architecture des fichiers (ex: `src/main.rs`, `src/api.rs`, `src/api/routes.rs`).
3. **Chemins de fichiers :** Commente la première ligne de chaque bloc de code généré avec son chemin relatif exact (ex: `// src/models/user.rs`).
4. **Liaison des modules :** Montre toujours le fichier parent (ex: `main.rs` ou `lib.rs`) où les modules sont déclarés via `pub mod nom_du_fichier;`.
5. **Syntaxe de module :** N'encapsule jamais le contenu d'un fichier `.rs` dans un bloc `mod nom { ... }`. Le nom du fichier définit déjà le module.
6. **Visibilité et Imports :** Utilise `pub(crate)` pour la visibilité interne et `use crate::...` pour les imports absolus depuis la racine.
