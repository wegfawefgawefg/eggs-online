# eggs-online
eggs 

## bugs
theres an issue with the second connected client not updating serverside
also the recieved id is extremely large for some reason

## notes
- special case, a client never recieves the disconnect message from the server, so it never removes the player from the list
    - causing the client to memory leak client data or hold old data for a player that has left long ago
    - solution: timeout the client if it hasnt recieved a message in a while