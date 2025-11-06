//use crate::*;

pub trait Arkyv: Sized +
    rkyv::Archive<Archived: rkyv::Deserialize<Self, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>>
    + for<'a> rkyv::Serialize<rkyv::api::high::HighSerializer<rkyv::util::AlignedVec, rkyv::ser::allocator::ArenaHandle<'a>, rkyv::rancor::Error>>
{}

impl<T> Arkyv for T
where
    T: rkyv::Archive
        + for<'a> rkyv::Serialize<rkyv::api::high::HighSerializer<rkyv::util::AlignedVec, rkyv::ser::allocator::ArenaHandle<'a>, rkyv::rancor::Error>>,
    T::Archived: rkyv::Deserialize<T, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
{}