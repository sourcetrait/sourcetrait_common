use crate::*;

#[cereal::derived(Copy, Eq)]
pub struct Success;

#[cereal::derived(Copy, Eq)]
pub struct Failure;

#[cereal::derived(Copy, Eq)]
#[serde(bound(
    serialize = "T: cereal::DataCopyEq",
    deserialize = "T: cereal::DataCopyEq"
))]
pub enum Flow<T> {
    Continue,
    /// Affects control flow of [System::run]
    Control(Control),
    /// Custom types that affect control flow of [System::run]
    Flow(T),
}

#[cereal::derived(Copy, Eq, Data)]
pub enum GreenFlow {
    Normal,
    /// Affects control flow of [System::run]
    Control(Control),
}

#[cereal::derived(Copy, Eq, Data)]
pub struct StdFlow;

pub type UnitResult = Result<Success, Failure>;
pub type SysResult<T> = Result<T, Failure>;
pub type FlowResult<T> = Result<Flow<T>, Failure>;
pub type GreenFlowResult = Result<GreenFlow, Failure>;

#[allow(non_upper_case_globals)]
pub const Fail: UnitResult = Err(Failure);
#[allow(non_upper_case_globals)]
pub const Succeed: UnitResult = Ok(Success);

pub trait ExitTrait {
    fn exit_code(self) -> ExitCode;
}

#[derive(Debug, snafu::Snafu)]
pub enum GreenError {
    #[snafu(display("Channel closed"))]
    ChannelClosed,
    
    #[snafu(display("IO error: {source}"))]
    IO { source: io::Error },

    #[snafu(display("Unable to join task"))]
    TaskJoin,
    
    #[snafu(display("Task failed"))]
    TaskFail,
    
    #[snafu(display("Unable to lock"))]
    Lock,
    
    #[snafu(display("Sub-system failed"))]
    Fatal,

    #[snafu(display("Unexpected response"))]
    ResponseType,
}

pub type GreenResult<T> = Result<T, GreenError>;

impl GreenError {
    #[inline]
    #[track_caller]
    pub const fn channel_closed() -> Self {
        Self::ChannelClosed
    }
    
    #[inline]
    #[track_caller]
    pub const fn err_channel_closed<T>() -> GreenResult<T> {
        Err(Self::channel_closed())
    }
    
    #[inline]
    #[track_caller]
    pub const fn io(source: io::Error) -> Self {
        Self::IO { source }
    }
    
    #[inline]
    #[track_caller]
    pub fn err_into_io<T, E: Into<io::Error>>(source: E) -> GreenResult<T> {
        Err(Self::into_io(source))
    }
    
    #[inline]
    #[track_caller]
    pub fn into_io<E: Into<io::Error>>(source: E) -> Self {
        Self::IO { source: source.into() }
    }
}

impl ExitTrait for UnitResult {
    #[inline]
    fn exit_code(self) -> ExitCode {
        match self {
            Ok(_) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        }
    }
}

impl From<io::Error> for GreenError {
    fn from(source: io::Error) -> Self {
        Self::io(source)
    }
}

impl From<tokio::task::JoinError> for GreenError {
    fn from(_: tokio::task::JoinError) -> Self {
        Self::TaskJoin
    }
}

impl<T> From<tkio::mpsc::error::SendError<T>> for GreenError {
    fn from(_: tkio::mpsc::error::SendError<T>) -> Self {
        Self::ChannelClosed
    }
}

impl From<GreenError> for Failure { fn from(_: GreenError) -> Self { Self } }

impl From<GreenError> for ExitCode {
    fn from(_: GreenError) -> Self {
        Self::FAILURE
    }
}

impl<T> From<Success> for Flow<T> {
    fn from(_v: Success) -> Self {
        Self::Continue
    }
}
