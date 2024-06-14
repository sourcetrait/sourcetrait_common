TODO for asmov-common-net-server
=======================================================================================================================

NOW
-----------------------------------------------------------------------------------------------------------------------
- [connection classes](#connection-classes)

### Connection classes 
there needs to be a configuration concept of an inbound connection class that includes:
- authorization requirements: cert, auth, both
- trust: internal (server to server), external (server to server or client app)
- permissions: rwx on ?
- capabilities: server, app
- direction: upstream, downstream, peer

each listener configuration (ip/port) lists which connection classes it supports

flow:
- server listens on configured addresses and ports

DONE
-----------------------------------------------------------------------------------------------------------------------
- copied else server code over and fixed most lib errors
