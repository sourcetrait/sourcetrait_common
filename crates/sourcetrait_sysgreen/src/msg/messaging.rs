use crate::*;

pub type MsgID = u64;

#[cereal::derived(Copy, Eq)]
pub enum FromGreenSys {
    ControlResponse(Packet<ControlResponse>),
    StatusChange(Packet<StatusChange>),
    StatusResponse(Packet<StatusResponse>),
}

#[cereal::derived]
pub enum MsgToSys<T> {
    Envelope, //todo
    Packet(Packet<T>),
    Green(ToGreenSys),
}

#[cereal::derived]
pub enum MsgFromSys<T> {
    Envelope, // todo
    Packet(Packet<T>),
    Green(FromGreenSys),
}

#[cereal::derived]
pub struct Packet<T> {
    pub request_id: MsgID,
    pub msg: T,
}

impl<T: Eq> Eq for Packet<T> {}
impl<T: Copy> Copy for Packet<T> {}

impl<T> Packet<T> {
    pub const SIMPLEX_ID: MsgID = 0;
    
    pub const fn new(request_id: MsgID, msg: T) -> Self {
        Self {
            request_id,
            msg,
        }
    }
    
    pub const fn simplex(msg: T) -> Self {
        Self {
            request_id: Self::SIMPLEX_ID,
            msg,
        }
    }
    
    pub fn duplex(msg: T) -> Self {
        Self {
            request_id: Self::next_id(),
            msg,
        }
    }
    
    pub fn respond<U>(self, msg: U) -> Packet<U> {
        Packet {
            request_id: self.request_id,
            msg,
        }
    }
    
    pub const fn request_id(&self) -> MsgID {
        self.request_id
    }
    
    pub const fn msg(&self) -> &T {
        &self.msg
    }
    
    pub const fn is_simplex(&self) -> bool {
        self.request_id == Self::SIMPLEX_ID
    }
    
    pub fn into_tuple(self) -> (MsgID, T) {
        let Self { request_id, msg } = self;
        ( request_id, msg )
    }
    
    fn next_id() -> MsgID {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

pub trait PacketData: Sized {
    fn into_packet(self, request_id: MsgID) -> Packet<Self> {
        Packet {
            request_id,
            msg: self
        }
    }
}

#[cereal::derived(Eq, Data)]
pub enum ToGreenSys {
    ControlRequest(Packet<ControlRequest>),
    StatusRequest(Packet<StatusRequest>),
}
