use crate::*;

pub struct SystemControl<SYS: System>  {
    channel: Channel<MsgToSys<<SYS as System>::ToSys>, MsgFromSys<<SYS as System>::FromSys>>,
}

impl<SYS: System> SystemControl<SYS> {
    pub async fn start(
        paths: SYS::Paths,
        config: SYS::Config,
        params: SYS::Params,
    ) -> GreenResult<Self> {
        let channel = InnerSystem::<SYS>::start(
            paths,
            config,
            params,
        ).await?;
        
        
        Ok(Self {
            channel,
        })
    }
    
    pub async fn send(&self, msg: MsgToSys<SYS::ToSys>) -> GreenResult<()> {
        self.channel.send(msg).await
    }
    
    pub async fn send_packet(&self, pkt: Packet<SYS::ToSys>) -> GreenResult<()> {
        self.channel.send(MsgToSys::Packet(pkt)).await
    }
    
    pub fn rx(&mut self) -> &mut tkio::mpsc::Receiver<MsgFromSys<SYS::FromSys>> {
        &mut self.channel.rx
    }
    
    pub async fn recv(&mut self) -> Option<MsgFromSys<SYS::FromSys>> {
        self.channel.rx.recv().await
    }
}