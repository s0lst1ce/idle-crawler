# Network schema

![](https://raw.githubusercontent.com/s0lst1ce/idle-crawler/master/network_schema.png)

*Here the PLAYERs represent all game objects needed by player X*

**Currently the image preview doesn't work because the repo is private. See `network_schema` in the root of the repo.**

## Server

Responsible for running the game, it is the core component of the game's network. Although the code for it is unique, there essentially exists two versions of this.

The host server is the heart of the network. It establishes a master-slave relationship with the other servers (also called client servers). It holds all game objects and may overrule other servers' decisions. This is done to prevent malicious servers from cheating and also as a simple failsafe for desyncs. In case of an erroneous request from a slave server the host will issue a `Response::Error` which the client server may choose to interpret as a desync. If so the data may be requested again for the server to correct itself.

If the host server is connected to the client servers of all currently playing users, local servers only know of the server. This has multiple benefits: first the local servers don't need to send duplicated events to all active servers which improves performance and decreases the chance of desync (especially with servers with slow connections). Next the host can assert the correctness of the request before propagating it, keeping the game state of most clean (although this could be avoided otherwise since the local servers only possess a fraction of the game objects). Finally, keeping the local server ignorant of each other is a security measure: the users only need to trust one server. If they do they don't need to worry about other users' intentions.

The previous point makes something else very simple: the reduction of objects the local servers need to keep track of. Since they only exist to server as a fast interface to the game for the client, which can only track one player, they also only need these objects. Hence the slave servers won't be able to interact with players they hold no right for, also saving performance along the way :wink:.

## Client

More than a simple human-readable interface between the server and the user, the client is responsible for a substantial amount of logic. Carrying out the player's intent usually means providing useful commands to the player like building iron mines anywhere and converting this to `Action::Build` events, choosing the position for the buildings and optimizing the number of constructions based on available resources and slots. The opposite is also true and maps of tiles may need to be abstracted as world maps...

Since the client is not responsible for running the game, it needs to make many calls to a server in order to always use up-to-date data. Thus the need for a close-by server for quick transfer of responses.

## Response

The packet of this network. Between components of the network, everything is responses. `Response` is used as the only way to communicate between clients and servers, with others and themselves. Over the network they are carried as `JSON`. The use of responses allows the network to carry intent across its components at great speeds while the responses are made to be as small as possible.
