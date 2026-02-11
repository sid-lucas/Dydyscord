# Dydyscord

TODO :
- Appeler la route serveur pour valider l'auth et passer d'un JWT Auth à un JWT Refresh, à faire dans storage/database.rs::init_device_storage()
- Créer la route sur le serveur qui transforme notre JWT Auth en JWT Refresh

- Ajouter une étape de validation mail pour l'inscription -> permet privacy-first, n'expose pas le fait que l'utilisateur existe deja.
- Login timing attack : Si le login fail, continuer de faire les mêmes opérations dans le backend avec un DUMMY_USER et DUMMY_PASSWORD

NOTE :
- Si login ok et que c'est pas un new device, et qu'on essaie de retrieve les elements OpenMLS -> s'il en manque un, plusieurs ou tous : Il faut agir en conséquences... (considéré comme un nouveau device? que faire? cas spécial...)