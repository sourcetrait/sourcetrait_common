use crate::*;

pub struct Channel<TX, RX> {
    pub tx: mpsc::Sender<TX>,
    pub rx: mpsc::Receiver<RX>,
    pub handle: task::JoinHandle<UnitResult>,
}

pub struct InternalChannel<TX, RX> {
    pub tx: mpsc::Sender<TX>,
    pub rx: mpsc::Receiver<RX>,
}

impl<TX,RX> InternalChannel<TX,RX> {
    pub fn new_pair(size: usize) -> (Self, (mpsc::Sender<RX>, mpsc::Receiver<TX>)) {
        let (left_tx, right_rx) = mpsc::channel(size);
        let (right_tx, left_rx) = mpsc::channel(size);
        
        ( Self { tx: left_tx, rx: left_rx }, (right_tx, right_rx) )
    }
    
    pub async fn send(&self, msg: TX) -> GreenResult<()> {
        self.tx.send(msg).await?;
        Ok(())
    }
}

impl<TX,RX> Channel<TX, RX> {
    pub fn new(tuple: (mpsc::Sender<TX>, mpsc::Receiver<RX>), handle: task::JoinHandle<UnitResult>) -> Self {
        Self { tx: tuple.0, rx: tuple.1, handle }
    }
    
    pub async fn send(&self, msg: TX) -> GreenResult<()> {
        self.tx.send(msg).await?;
        Ok(())
    }
    
    pub async fn join(self) -> GreenResult<()> {
        match self.handle.await? {
            Ok(_) => Ok(()),
            Err(_) => Err(GreenError::TaskFail)
        }
    }
}

