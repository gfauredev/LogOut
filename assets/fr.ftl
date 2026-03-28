# LogOut – Traductions françaises

## Toute l'appli
app-title = 💪 LogOut
app-subtitle = Éteins ton ordinateur, consigne tes entraînements

## Page d'accueil
no-sessions = Aucune séance passée
start-first-workout = Appuie sur + pour démarrer ta première séance
start-new-workout = Nouvelle séance
session-repeat-title = Démarrer une nouvelle séance basée sur celle-ci
session-delete-title = Supprimer la séance
session-show-more = +{ $count } autres
session-delete-confirm = Supprimer cette séance ?
session-delete-confirm-btn = 🗑️ Supprimer
cancel-btn = ❌ Annuler

## Page des exercices
browse-exercises = { $count } exercices disponibles
search-placeholder = Rechercher des exercices par noms ou attributs
add-exercise = Ajouter un exercice personnalisé
filter-add = Activer le filtre
filter-remove = Supprimer le filtre

## Fiche exercice
exercise-edit = Modifier
exercise-clone = Dupliquer puis modifier

## Séance active – recherche
session-search-placeholder = Rechercher un exercice...
session-add-exercise-title = Ajouter un exercice personnalisé
session-filter-remove = Supprimer le filtre
session-filter-add = Ajouter le filtre
pending-more = Plus en attente ({ $count })

## Séance active – en-tête
session-title = ⏱️ Séance active
session-timer-title = Cliquer pour définir la durée du repos
session-pause-btn = Mettre en pause
session-resume-btn = Reprendre la séance
session-cancel-btn = Annuler la séance
session-finish-btn = Terminer la séance

## Séance active – durée de repos
rest-duration-aria = Définir la durée du repos
rest-duration-label = Durée du repos

## Séance active – exercices complétés
completed-exercises-title = Exercices complétés

## Formulaire d'exercice
exercise-complete-title = Valider l'exercice
time-placeholder = mm:ss
weight-placeholder = kg
distance-placeholder = km
reps-placeholder = rép.

## Journal d'exercice complété
log-replay-title = Faire une autre série
log-edit-title = Modifier cet exercice
log-delete-title = Supprimer cet exercice

## Pages ajout / modification d'exercice
add-exercise-page-title = Ajouter un exercice
edit-exercise-page-title = Modifier l'exercice
exercise-not-found = Exercice introuvable
exercise-save = Enregistrer l'exercice
exercise-save-changes = Enregistrer les modifications
cancel-title = Annuler

## Champs du formulaire d'exercice
form-name-label = Nom de l'exercice *
form-category-label = Catégorie *
form-force-label = Type de force
form-equipment-label = Équipement
form-muscles-primary-label = Muscles principaux
form-muscles-secondary-label = Muscles secondaires
form-instructions-label = Instructions
form-images-label = Images
form-name-placeholder = Pompes
form-muscle-select-default = Sélectionner un muscle...
form-instruction-placeholder = Ajouter une étape d'instruction...
form-image-url-placeholder = https://example.com/image.jpg
form-none-option = Aucun
form-image-upload-title = Téléverser une image locale
form-local-image-placeholder = /chemin/vers/image.jpg
form-local-image-title = Chemin vers un fichier image local (sera copié dans le stockage de l'application)
form-save-aria = Enregistrer

## Page Plus
more-title = ⚙️ Plus
more-export-section = 📤 Exporter
more-export-exercises-btn = 💾 Exercices personnalisés
more-export-sessions-btn = 💾 Séances
more-import-section = 📥 Importer
more-import-exercises-btn = 📂 Exercices personnalisés
more-import-sessions-btn = 📂 Séances
more-about-section = LogOut
more-db-url-section = ⚙️ URL de la base de données d'exercices
more-db-url-desc = Remplace la source de données d'exercices. Enregistre pour forcer un nouveau téléchargement au prochain rechargement.
more-db-url-save-aria = Enregistrer
more-oss-section = Open Source & Licences
more-built-with-section = Construit avec
more-replace-confirm = Remplacer l'exercice personnalisé { $name } ?
more-replace-btn = 💾 Remplacer

## Messages toast (préfixe statique ; le détail technique est ajouté à l'exécution)
toast-export-failed = ⚠️ Échec de l'export
toast-export-sessions-failed = ⚠️ Échec de l'export des séances
toast-sessions-invalid = ⚠️ JSON de séances invalide
toast-exercises-invalid = ⚠️ JSON d'exercices invalide

## Étiquettes de date relative
date-today = Aujourd'hui
date-yesterday = Hier
date-days-ago = Il y a { $count } jour{ $count ->
    [one] {""}
   *[other] s
}

## Toast de félicitations et notifications
congratulations = 🎉 Beau travail ! Séance terminée !
notif-permission-blocked = ⚠️ Notifications bloquées
notif-permission-enable = ⚠️ Appuie ici pour activer les notifications
notif-duration-title = Durée atteinte
notif-duration-body = Durée cible de l'exercice atteinte !
notif-rest-title = Repos terminé
notif-rest-body = C'est l'heure de ta prochaine série !

## Page Statistiques
analytics-title = 📊 Statistiques
analytics-subtitle = Suis ta progression dans le temps
analytics-pairs-label = Paires métrique–exercice (⩽ 8)
analytics-empty = Sélectionnez des exercices pour voir les statistiques
analytics-metric-weight = Poids (kg)
analytics-metric-reps = Répétitions
analytics-metric-distance = Distance
analytics-metric-duration = Durée
analytics-select-exercise = -- Sélectionner un exercice --
analytics-remove-series = Supprimer cette série
