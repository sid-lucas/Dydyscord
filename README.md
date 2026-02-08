# Dydyscord

TODO :
- Faire les modules de manière idiomatic (enlever les mod.rs)
- Ajouter une étape de validation mail pour l'inscription -> permet privacy-first, n'expose pas le fait que l'utilisateur existe deja.
- Login timing attack : Si le login fail, continuer de faire les mêmes opérations dans le backend avec un DUMMY_USER et DUMMY_PASSWORD

NOTE :
- si la db n'existe pas ou que la db_key n'existe pas dans la keychain, alors c'est considéré comme un nouveau device et on re-init tout.
- par contre si tout est ok, mais quand on retrieve les elements OpenMLS, il en manque un, plusieurs ou tous -> il faut agir en conséquences... (considéré comme un nouveau device? que faire? cas spécial...)