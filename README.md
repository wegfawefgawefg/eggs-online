# eggs-online
eggs over ez

## archive
This repo was a small Rust multiplayer sandbox, mostly for experimenting with client/server netcode and simple rendering.

What I was making/playing around with:
- UDP client/server messaging in `tokio`
- player spawn/sync/position relay
- basic local movement input (WASD)
- simple raylib rendering (circles/text), not a full game loop with win/lose mechanics

Timeline:
- first commit: `047f174` on 2023-10-07 03:01:29 -0500 (`Initial commit`)
- latest commit: `15c70b0` on 2026-02-24 17:40:37 +0900 (`apparently fix?`)

## TODONE
- server can handle arbitrary connection numbers
- server is relay with message churn
- tx and rx
- handles disconnects with broadcast


## TODO
- keepalive on udp
- abstract server so game state and message responses can be provided as callbacks

## notes
- special case, a client never recieves the disconnect message from the server, so it never removes the player from the list
    - causing the client to memory leak client data or hold old data for a player that has left long ago
    - solution: timeout the client if it hasnt recieved a message in a while
    - try a 2 second keep alive heartbeat from either side

## bugs
- client never dc on server death
