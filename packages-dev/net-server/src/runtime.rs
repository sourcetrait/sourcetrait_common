use native_tls as tls;
use tokio::{self, sync::watch, sync::broadcast, sync::mpsc};
use crate::*;

pub trait Runtime {
    fn new<CFG: Config>(config: CFG) -> Self;
}

/*pub enum UniverseChannel {
    Send(u32, u8, model::ZoneToUniverseMessage),
    Receive(u32, u8, model::UniverseToZoneMessage)
}

pub struct ZoneRuntime {
    config: ZoneConfig,
    world: Option<model::World>,
    timeframe: Option<model::TimeFrame>,
    timeframe_channel_tx: watch::Sender<model::TimeFrame>,
    universe_send: (mpsc::Sender<model::ZoneToUniverseMessage>, mpsc::Receiver<model::ZoneToUniverseMessage>),
    universe_receive: (broadcast::Sender<model::UniverseToZoneMessage>, broadcast::Receiver<model::UniverseToZoneMessage>)
}

impl ZoneRuntime {
    pub fn new(config: ZoneConfig) -> Self {
        let timeframe_channel_tx = watch::channel(model::TimeFrame::new(0,0)).0;
        let universe_send = mpsc::channel(8);
        let universe_receive = broadcast::channel(8);

        Self {
            config: config,
            world: None,
            timeframe: None,
            timeframe_channel_tx,
            universe_send,
            universe_receive
        }
    }

    pub fn config(&self) -> &ZoneConfig {
        &self.config
    }

    pub fn ready(&self) -> bool {
        self.world.is_some()
    }

    pub fn timeframe(&self) -> Option<&model::TimeFrame> {
        self.timeframe.as_ref()
    }

    pub fn world(&self) -> Option<&model::World> {
        self.world.as_ref()
    }

    pub fn sync_world(&mut self, bytes: Vec<u8>) -> Result<&model::World, ()> {
        let world: model::World = bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
            .map_err(|_| ())?
            .0;
        self.world = Some(world);
        Ok(self.world.as_ref().unwrap())
    }

    pub fn sync(&mut self, sync: model::Sync) -> model::Result<()> {
        sync.sync(self.world.as_mut().unwrap())?;
        Ok(())
    }

    pub fn sync_timeframe(&mut self, timeframe: model::TimeFrame) {
        self.timeframe = Some(timeframe);
        let _ = self.timeframe_channel_tx.send(self.timeframe.as_ref().unwrap().clone());
    }

    pub fn subscribe_timeframe(&mut self) -> watch::Receiver<model::TimeFrame> {
        self.timeframe_channel_tx.subscribe()
    }

    pub fn subscribe_universe_receive(&mut self) -> broadcast::Receiver<model::UniverseToZoneMessage> {
        self.universe_receive.0.subscribe()
    }

    pub fn subscribe_universe_send(&mut self) -> mpsc::Sender<model::ZoneToUniverseMessage> {
        self.universe_send.0.clone()
    }
}

pub type ZoneRuntimeSync = std::sync::Arc<tokio::sync::Mutex<ZoneRuntime>>;*/
