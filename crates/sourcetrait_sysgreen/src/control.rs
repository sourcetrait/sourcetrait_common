use crate::*;

/// Handles incoming messages
/// Returns None if handled, else Some(msg) if unhandled.
//pub type HandlerFn<SYS> = fn(msg: MsgFromSys<<SYS as System>::FromSys>)
//    -> Pin<Box<dyn Future<Output = Option<MsgFromSys<<SYS as System>::FromSys>>> + Send + 'static>>;

pub trait Handler<SYS: System>: 'static + Send {
    fn on_packet(&mut self, msg: Packet<SYS::FromSys>)
        -> impl Future<Output = Option<Packet<SYS::FromSys>>> + 'static + Send;
}

pub struct SystemControl<SYS: System>  {
    tx: tkio::mpsc::Sender<MsgToSys<<SYS as System>::ToSys>>,
    handle: tkio::task::JoinHandle<Result<Success, Failure>>,
    cancel: tkio::CancellationToken,
    requests: Arc<Mutex<FxHashMap<u64, tokio::sync::oneshot::Sender<MsgFromSys<<SYS as System>::FromSys>>>>>,
}

struct ControlTask<SYS: System> {
    rx: tkio::mpsc::Receiver<MsgFromSys<<SYS as System>::FromSys>>,
    requests: Arc<Mutex<FxHashMap<u64, tokio::sync::oneshot::Sender<MsgFromSys<<SYS as System>::FromSys>>>>>,
    cancel: tkio::CancellationToken,
}

impl<SYS: System> SystemControl<SYS> {
    pub async fn start<HNDL: Handler<SYS>>(
        paths: SYS::Paths,
        config: SYS::Config,
        params: SYS::Params,
        mut handler: HNDL,
    ) -> GreenResult<Self> {
        let channel = InnerSystem::<SYS>::start(
            paths,
            config,
            params,
        ).await?;

        let Channel { tx, rx, cancel, handle } = channel;
        let requests = Arc::new(Mutex::new(FxHashMap::default()));

        let mut task: ControlTask<SYS> = ControlTask {
            rx,
            cancel: cancel.clone(),
            requests: requests.clone(),
        };
        
        let this = Self {
            tx,
            cancel,
            handle,
            requests,
        };
        
        tokio::spawn(async move {
            let result: Result<(), Failure> = loop {
                let result = tokio::select! {
                    rx = task.rx.recv() => match rx {
                        Some(msg @ MsgFromSys::Packet(pkt)) => {
                            let remains = if let PacketNature::Response(reqid) = pkt.nature {
                                let remains = {
                                    let mut requests = task.requests.lock().expect("lock");
                                    if let Some(req) = requests.remove(&reqid) {
                                        req.send(msg)
                                            .unwrap(); //todo
                                        None
                                    } else {
                                        Some(msg)
                                    }
                                };
                                
                                remains
                            } else { None };
                            if let Some(msg) = remains {
                                match handler.on_packet(msg.take_packet().expect("packet")).await {
                                    Some(msg) => Ok(()),
                                    None => Ok(())
                                }
                            } else {
                                Ok(())
                            }
                        },
                        _ => Err(Failure),
                        None => Err(Failure),
                    },
                };
                
                if let Err(e) = result {
                    break Err(e);
                }
            };
        });
        
        //Ok(this)
        unreachable!()
    }
    
    pub async fn send_sys(&self, msg: MsgToSys<SYS::ToSys>) -> GreenResult<()> {
        self.tx.send(msg).await?;
        Ok(())
    }
    
    pub async fn send(&self, pkt: Packet<SYS::ToSys>) -> GreenResult<()> {
        self.tx.send(MsgToSys::Packet(pkt)).await?;
        Ok(())
    }
    
    pub async fn request<T: Request<SYS>>(&mut self, req: T) -> GreenResult<T::ResponseType>
    where
        <T as Request<SYS>>::ResponseType: From<MsgFromSys<<SYS as System>::FromSys>>
    {
        let packet = Packet::request(req.into());
        let reqid = packet.id;
        let msg = MsgToSys::Packet(packet);
        let (send, recv) = tokio::sync::oneshot::channel();
        self.requests
            .lock().expect("lock")
            .insert(reqid, send);
        self.tx.send(msg).await?;
        let response: T::ResponseType = recv.await
            .map_err(|_| GreenError::Fatal)?
            .into();

        Ok(response)
    }
}

pub trait Request<SYS: System>: Into<SYS::ToSys> {
    type ResponseType: From<SYS::FromSys>;
}
