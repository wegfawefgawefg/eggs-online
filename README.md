# eggs-online
eggs over ez

## TODONE
- server can handle arbitrary connection numbers
- server is relay with message churn
- tx and rx
- handles disconnects with broadcast


## TODO
- keepalive
- abstract server so game state and message responses can be provided as callbacks

## notes
- special case, a client never recieves the disconnect message from the server, so it never removes the player from the list
    - causing the client to memory leak client data or hold old data for a player that has left long ago
    - solution: timeout the client if it hasnt recieved a message in a while
    - try a 2 second keep alive heartbeat from either side

## bugs
- client never dc on server death