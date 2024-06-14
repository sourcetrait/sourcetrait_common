use std::sync::Arc;
use futures_util::FutureExt;
use tokio;
use tokio_native_tls as tls;
use asmov_common_util::sync::num_reserve;
use crate::*;

pub type RuntimeSync<R> = std::sync::Arc<tokio::sync::Mutex<R>>;
pub type Listener = (tls::TlsAcceptor, tokio::net::TcpListener);

pub async fn run<CFG: Config, RUN: Runtime>(config: Option<CFG>) -> std::process::ExitCode {
    const DEFAULT_DURATION: tokio::time::Duration = tokio::time::Duration::from_secs(30);

    let config = match config {
        Some(c) => c,
        None => match CFG::load().await {
            Ok(c) => c,
            Err(_) => return std::process::ExitCode::FAILURE
        }
    };

    let client_listening = match prepare_client_listening(&config).await {
        Ok(c) => c,
        Err(_) => return std::process::ExitCode::FAILURE
    };

    let runtime: RuntimeSync<RUN> = std::sync::Arc::new(tokio::sync::Mutex::new(RUN::new(config)));

    //let _host_connector_task = tokio::spawn(host_connector_task(Arc::clone(&runtime)));
    let _server_listener_task = tokio::spawn(server_listener_task(host_listening, Arc::clone(&runtime)));
    let _client_listener_task = tokio::spawn(client_listener_task(client_listening, Arc::clone(&runtime)));

    //let _world_connector_task = tokio::spawn(world_connector_task(Arc::clone(&runtime)));
    //let _universe_connector_task = tokio::spawn(universe_connector_task(Arc::clone(&runtime)));
    let _client_listener_task = tokio::spawn(client_listener_task(client_listening, Arc::clone(&runtime)));

    let sleep = tokio::time::sleep(DEFAULT_DURATION);
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            () = &mut sleep => {
                sleep.as_mut().reset(tokio::time::Instant::now() + DEFAULT_DURATION);
            }
        }
    }
}

async fn connector_task<R: Runtime>(runtime: RuntimeSync<R>) {
    let world_host = {
        let runtime_lock = runtime.lock().await;
        runtime_lock.config().world.host.clone()
    };

    let world_server_url = format!("wss://{}", world_host);
    let mut next_world_connection_num: ConnectionID = 1;
    let mut reconnect_attempts: i32 = -1;

    loop {
        if reconnect_attempts > -1 {
            let wait = std::cmp::min(MAX_RECONNECT_WAIT, 15 + 3 * reconnect_attempts as u64);
            log!("Reconnecting in {wait} seconds ...");
            tokio::time::sleep(tokio::time::Duration::from_secs(15 + wait)).await;
            reconnect_attempts += 1;
        } else {
            reconnect_attempts = 0;
        }

        let tls_connector = build_tls_connector();
        let result = tokio_tungstenite::connect_async_tls_with_config(
            world_server_url.clone(),
            None,
            false,
            Some(tls_connector)
        ).await;

        let world_websocket_stream = match result {
            Ok((stream, _)) => stream,
            Err(e) => {
                log_error!("Unable to connect to world server at {world_server_url} :> {e}");
                continue
            }
        };

        let world_server_who = Who::World(next_world_connection_num, format!("{}", world_host));
        next_world_connection_num += 1;
        log!("Established connection with {world_server_who}.");

        let conn = Connection::new(world_server_who, Stream::Outgoing(world_websocket_stream));
        let conn = match negotiate_world_session(conn).await {
            Err(e) => {
                log_error!("{e}");
                continue;
            },
            Ok(conn) => conn
        };

        reconnect_attempts = 0; // reset after a successful handshake

        match world_stream_task(conn, Arc::clone(&runtime)).await {
            Err(e) => {
                log_error!("{e}");
            },
            Ok(who) => {
                log!("Session finished with {who}");
            }
        }
    }
}

async fn universe_connector_task(runtime: ZoneRuntimeSync) {
    let universe_host = {
        let runtime_lock = runtime.lock().await;
        runtime_lock.config().universe.host.clone()
    };

    let mut next_universe_connection_num: ConnectionID = 1;
    let universe_server_url = format!("wss://{}", universe_host);
    let mut reconnect_attempts: i32 = -1;

    loop {
        if reconnect_attempts > -1 {
            let wait = std::cmp::min(MAX_RECONNECT_WAIT, 15 + 3 * reconnect_attempts as u64);
            log!("Reconnecting in {wait} seconds ...");
            tokio::time::sleep(tokio::time::Duration::from_secs(15 + wait)).await;
            reconnect_attempts += 1;
        } else {
            reconnect_attempts = 0;
        }

        let tls_connector = build_tls_connector();
        let result = tokio_tungstenite::connect_async_tls_with_config(
            universe_server_url.clone(),
            None,
            false,
            Some(tls_connector)
        ).await;

        let universe_websocket_stream = match result {
            Ok((stream, _)) => stream,
            Err(e) => {
                log_error!("Unable to connect to universe server at {universe_server_url} :> {e}");
                continue
            }
        };

        let universe_server_who = Who::Universe(next_universe_connection_num, format!("{}", universe_host));
        next_universe_connection_num += 1;
        log!("Established connection with {universe_server_who}.");

        let conn = Connection::new(universe_server_who, Stream::Outgoing(universe_websocket_stream));
        let conn = match negotiate_universe_session(conn).await {
            Err(e) => {
                log_error!("{e}");
                continue;
            },
            Ok(conn) => conn
        };

        reconnect_attempts = 0; // reset after a successful handshake

        match universe_stream_task(conn, Arc::clone(&runtime)).await {
            Err(e) => {
                log_error!("{e}");
            },
            Ok(who) => {
                log!("Session finished with {who}");
            }
        }
    }
}

async fn bind_client_listener(config: &ZoneConfig) -> LoggedResult<tokio::net::TcpListener> {
    tokio::net::TcpListener::bind(&config.clients.listen).await
        .and_then(|listener| {
            log!("Listening for client connections on {}.", config.clients.listen);
            Ok(listener)
        })
        .map_err(|e| {
            log_error!("Unable to bind to address {}. :> {e}", config.clients.listen);
            ()
        })
}


async fn prepare_client_listener(config: impl &Config) -> LoggedResult<Listenier> {
    let tls_acceptor = build_tls_acceptor(config)?;
    let client_websocket_listener = bind_client_listener(config).await?;
    Ok((tls_acceptor, client_websocket_listener))
}

async fn client_listener_task(listening: ClientListening, runtime: ZoneRuntimeSync) {
    let mut next_client_connection_num: ConnectionID = 1;
    let mut client_websocket_stream_tasks = Vec::new();
    let (tls_acceptor, client_websocket_listener) = listening;

    loop {
        let (tcp_stream, addr) = match client_websocket_listener.accept().await {
            Ok(r) => r,
            Err(e) => {
                log_error!("Unable to accept client connection :> {e}");
                break;
            }
        };

        let acceptor = tls_acceptor.clone();
        let tls_stream = match acceptor.accept(tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                log_error!("Unable to accept TLS connection from client ({addr}). :> {e}");
                continue
            }
        };

        let websocket_stream = match tokio_tungstenite::accept_async(tls_stream).await {
            Ok(s) => s,
            Err(e) => {
                log_error!("Unable to accept TLS websocket connection from client ({addr}). :> {e}");
                continue
            }
        };

        let client_who = Who::Client(next_client_connection_num, format!("{}:{}", addr.ip(), addr.port()));
        next_client_connection_num += 1;
        log!("Established connection with {client_who}.");

        let conn = Connection::new(client_who.clone(), Stream::Incoming(websocket_stream));
        let runtime_clone = Arc::clone(&runtime);
        let task = tokio::spawn(async move {
            let (conn, interface_uid) = match negotiate_client_session(conn, Arc::clone(&runtime_clone)).await {
                Err(e) => {
                    log_error!("{e}");
                    return
                },
                Ok((conn, interface_uid)) => {
                    log!("Negotiated session with {}", conn.who());
                    (conn, interface_uid)
                }
            };

            match client_stream_task(conn, runtime_clone).await {
                Ok(who) => {
                    log!("Session finished with {who}");
                    return
                },
                Err(e) => {
                    log_error!("{e}");
                    return
                }
            }
        });

        client_websocket_stream_tasks.push((client_who, task));
    }
}

async fn negotiate_session_with_host<P: Protocol>(mut conn: Connection<P>) -> ConnectionResult<P> {
    let protocol_header = ProtocolHeader::current::<P>(ProtocolIdentity::Client);

    // protocol verification: 1. the connector sends its protocol header
    conn.send(protocol_header.clone()).await?;

    // protocol verification: 2. server sends the expected corresponding protocol header or Protocol::Unsupported
    let their_protocol_header: ProtocolHeader = conn.receive().await?;
    if !protocol_header.compatible(&their_protocol_header) {
        // either the protocol is Unsupported or the version is wrong
        return Err(conn.error_payload("incompatible protocol").await);
    }

    // send a connection request
    let msg = ClientSessionMessage::Connect;
    conn.send(msg).await?;

    // receive a connection response
    let msg: HostSessionMessage = conn.receive().await?;
    match msg {
        HostSessionMessage::Connected => {
            log!("Connection negotiated with {}.", conn.who());
            Ok(conn)
        },
        HostSessionMessage::ConnectRejected => {
            log_error!("Connection negotiation rejected by {}.", conn.who());
            conn.halt().await;
            Err(NetworkError::Rejected{who: conn.who().clone().to_string()})
        },
        _ => Err(conn.error_payload("HostSessionMessage::[Connected, ConnectRejected]").await)
    }
}

async fn world_stream_task(mut conn: Connection, runtime: ZoneRuntimeSync) -> StreamResult {
    loop {
        let msg: WorldToZoneMessage = conn.receive().await?;
        match msg {
            WorldToZoneMessage::WorldBytes(timeframe, bytes) => {
                let frame = timeframe.frame();
                {
                    let mut runtime_lock = runtime.lock().await;
                    runtime_lock.sync_world(bytes).unwrap(); //todo: Don't Panic
                    runtime_lock.sync_timeframe(timeframe);
                }

                log!("Synchronized world at frame {frame}.");
            },
            WorldToZoneMessage::Sync(sync) => {
                {
                    let mut runtime_lock = runtime.lock().await;
                    runtime_lock.sync(sync).unwrap();
                }
                log!("Sync");
            },
            WorldToZoneMessage::TimeFrame(newtimeframe) => {
                let timeframe = newtimeframe.timeframe;
                let frame = timeframe.frame();
                {
                    let mut runtime_lock = runtime.lock().await;
                    runtime_lock.sync_timeframe(timeframe);
                };

                log!("Frame: {frame}");
            },
            WorldToZoneMessage::Disconnect => {
                log!("Disconnected from {}", conn.who());
                conn.halt().await;
                return Ok(conn.who().clone());
            },
            _ => {
                return Err(elsenet::NetworkError::UnexpectedResponse{
                    who: conn.who().clone(), expected: "appropriate WorldToZone".to_string()})
            }
        }
    }
}

async fn negotiate_universe_session(mut conn: Connection) -> ConnectionResult {
    // protocol verification: 1. the connector sends its protocol header
    let msg = ProtocolHeader::current(Protocol::ZoneToUniverse);
    conn.send(msg).await?;

    // protocol verification: 2. server sends the expected corresponding protocol header or Protocol::Unsupported
    let their_protocol_header: ProtocolHeader = conn.receive().await?;
    if !their_protocol_header.compatible(Protocol::UniverseToZone) {
        // either the protocol is Unsupported or the version is wrong
        return Err(conn.error_payload("compatible protocol").await);
    }

    // send a connection request
    let msg = ZoneToUniverseMessage::Connect;
    conn.send(msg).await?;

    // receive a connection response
    let msg: UniverseToZoneMessage = conn.receive().await?;
    match msg {
        UniverseToZoneMessage::Connected => {
            log!("Connection negotiated with {}.", conn.who());
            Ok(conn)
        },
        UniverseToZoneMessage::ConnectRejected => {
            log_error!("Connection negotiation rejected by {}.", conn.who());
            conn.halt().await;
            Err(NetworkError::Rejected{who: conn.who().clone()})
        },
        _ => Err(conn.error_payload("UniverseToZoneMessage::[Connected, ConnectRejected]").await)
    }
}

async fn universe_stream_task(mut conn: Connection, runtime: ZoneRuntimeSync) -> StreamResult {
    loop {
        let msg: UniverseToZoneMessage = conn.receive().await?;
        match msg {
            UniverseToZoneMessage::Disconnect => {
                log!("Disconnected from {}", conn.who());
                conn.halt().await;
                return Ok(conn.who().clone());
            },
            _ => {
                return Err(elsenet::NetworkError::UnexpectedResponse{
                    who: conn.who().clone(), expected: "appropriate UniverseToZone".to_string()})
            }
        }
    }
}

/// Returns Some(AuthenticatedMsg) if the client is authorized, None if the client is rejected
async fn attempt_client_auth(
    conn: &mut Connection,
    universe_send: &mut mpsc::Sender<ZoneToUniverseMessage>,
    universe_receive: &mut broadcast::Receiver<UniverseToZoneMessage>,
) -> Result<Option<model::UID>, NetworkError> {
    let msg: ClientToZoneMessage = conn.receive().await?;
    match msg {
        ClientToZoneMessage::AuthRequest(auth_request) => {
            let msg_num = conn.next_msg_num();
            let msg = ZoneToUniverseMessage::AuthRequest(conn.who().connection_id(), msg_num, auth_request);
            universe_send.send(msg).await.unwrap(); //todo: don't panic

            loop {
                //todo: timeout
                let msg = universe_receive.recv().await.unwrap(); //todo: don't panic
                match msg {
                    UniverseToZoneMessage::Authenticated(conn_id, msg_id, authenticated_msg) => {
                        if conn_id != conn.who().connection_id() || msg_id != msg_num {
                            continue
                        }

                        let interface_uid = authenticated_msg.interface_uid;
                        let msg = ZoneToClientMessage::Authorized(authenticated_msg);
                        conn.send(msg).await?;
                        return Ok(Some(interface_uid))
                    },
                    UniverseToZoneMessage::AuthRejected(conn_id, msg_id) => {
                        if conn_id != conn.who().connection_id() || msg_id != msg_num {
                            continue
                        }

                        let msg = ZoneToClientMessage::AuthRejected;
                        conn.send(msg).await?;
                        return Ok(None)
                    },
                    _ => {
                        return Err(NetworkError::UnexpectedResponse {
                            who: conn.who().clone(),
                            expected: "UniverseToZoneMessage::[Authenticated | AuthRejected | AuthChallenge]".to_string()
                        })
                    }
                }
            }
        },
        _ => {
                return Err(NetworkError::UnexpectedResponse {
                    who: conn.who().clone(),
                    expected: "ClientToZoneMessage::[AuthRequest | AuthRegister]".to_string()
                })

        }
    }
}

async fn negotiate_client_session(
    mut conn: Connection,
    runtime: ZoneRuntimeSync
) -> Result<(Connection, model::UID), NetworkError> {
    elsenet::negotiate_protocol(&mut conn, true, Protocol::ZoneToClient, Protocol::ClientToZone).await?;

    let (mut universe_send, mut universe_receive) = {
        let mut runtime_lock = runtime.lock().await;
        (runtime_lock.subscribe_universe_send(), runtime_lock.subscribe_universe_receive())
    };

    const MAX_CLIENT_AUTH_ATTEMPTS: usize = 3;

    for _auth_attempt in 0..MAX_CLIENT_AUTH_ATTEMPTS {
        match attempt_client_auth(&mut conn, &mut universe_send, &mut universe_receive).await? {
            Some(interface_uid) => return Ok((conn, interface_uid)),
            None => continue,
        }
    }

    // max auth attempts reached
    Err(NetworkError::Rejected{who: conn.who().clone()})
}

async fn client_stream_task(mut conn: Connection, runtime: ZoneRuntimeSync) -> StreamResult {

    // init session
    let session;
    {
        let timeframe = {
            let runtime_lock = runtime.lock().await;
            session = ClientSession::todo_from_universe_server(runtime_lock.world().unwrap()).unwrap(); //todo: don't panic
            runtime_lock.timeframe().unwrap().clone()
        };

        let bytes = bincode::serde::encode_to_vec(&session.interface_view(), bincode::config::standard()).unwrap();
        let msg = ZoneToClientMessage::InitInterfaceView(timeframe, bytes);
        conn.send(msg).await?;
    }

    let mut timeframe_subscriber = {
        let mut runtime_lock = runtime.lock().await;
        runtime_lock.subscribe_timeframe()
    };

    loop {
        tokio::select! {
            result = conn.receive::<ClientToZoneMessage>() => {
                let msg = result?;

                match msg {
                    ClientToZoneMessage::Disconnect => {
                        log!("Disconnection from {}", conn.who());
                        conn.halt().await;
                        return Ok(conn.who().clone());
                    },
                    _ => {
                        return Err(elsenet::NetworkError::UnexpectedResponse{
                            who: conn.who().clone(), expected: "appropriate WorldToZone".to_string()})
                    }
                }
            }

            _result = timeframe_subscriber.changed() => {
                let timeframe: model::TimeFrame = timeframe_subscriber.borrow_and_update().clone();
                let msg = ZoneToClientMessage::TimeFrame(NewTimeFrameMsg{timeframe});
                conn.send(msg).await?;
            }
        }
    }
}
