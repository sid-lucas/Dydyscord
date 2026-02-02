# Dydyscord

### redis :

Installation :
`sudo apt install redis-server`
Lancer :
`sudo systemctl enable --now redis-server`
Verifier :
`sudo systemctl status redis-server`
`redis-cli ping    # Doit répondre : PONG`

TODO :
- Implement redis for keeping in login_id for OPAQUE login
- Gérer proprement les erreurs HTTP