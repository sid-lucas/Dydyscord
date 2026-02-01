# Dydyscord

TODO :
- Faire un ServerSetup une fois et persistent à jamais (dans la DB)
- Implémenter le login_lookup (HMAC(pepper, username)) côté serveur
    - l'utiliser comme credential_identifier dans ServerRegistration::start
- Faire un module network dédié (pour le client qui fait les requêtes, et le server qui les handle)
- Création d'une entrée (user_id (UUIDv4), login_lookup, opaque_record) à la fin du register_finish()
- Gérer proprement les erreurs HTTP