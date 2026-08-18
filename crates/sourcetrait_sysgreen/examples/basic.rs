///! Defines a subsystem that will, upon request, either Add or Multiply a number
///! by a specified operand and an internally accumulated number. The internal
///! number is initialized by configuration.
use sourcetrait_sysgreen::{ self as green, prelude::* };
use sourcetrait_cereal_macro as cereal;
use sourcetrait_agnostic::{ self as agnostic, prelude::* };
use sourcetrait_tomlx::{ self as tomlx, prelude::* };
use std::{
    path::PathBuf,
};

#[tokio::main]
async fn main() {
    let paths = ExampleSysPaths::default();
    let config = ExampleSysConfig::default();
    let params = ExampleSysParams { op: Operation::Mul };
    let mut sys: green::SystemControl<ExampleSys> = green::SystemControl::start(
        paths,
        config,
        params,
    ).await.unwrap();
    
    let request = green::Packet::request(ToExampleSys::MathRequest(
        MathRequest {
            operand: 2.,
        }
    ));
    let expected_request = green::Packet {
        id: 1,
        nature: green::PacketNature::Request,
        msg: ToExampleSys::MathRequest(
            MathRequest {
                operand: 2.,
            }
        ),
    };
    assert_eq!(expected_request, request);
    sys.send_packet(request).await.unwrap();
    
    let response = sys.recv().await.unwrap();
    let expected_response = green::MsgFromSys::Packet(green::Packet {
        id: 2,
        nature: green::PacketNature::Response(1),
        msg: FromExampleSys::MathResponse(MathResponse {
            result: Ok(2.),
        }),
    });
    assert_eq!(expected_response, response);
    
    let request = green::Packet::request(ToExampleSys::MathRequest(
        MathRequest {
            operand: 3.,
        }
    ));
    let expected_request = green::Packet {
        id: 3,
        nature: green::PacketNature::Request,
        msg: ToExampleSys::MathRequest(
            MathRequest {
                operand: 3.,
            }
        ),
    };
    assert_eq!(expected_request, request);
    sys.send_packet(request).await.unwrap();
    
    let expected_response = green::MsgFromSys::Packet(green::Packet {
        id: 4,
        nature: green::PacketNature::Response(3),
        msg: FromExampleSys::MathResponse(MathResponse {
            result: Ok(6.),
        }),
    });
    let response = sys.recv().await.unwrap();
    assert_eq!(expected_response, response);
}

pub struct ExampleSys {
    inner: green::InnerSystem<Self>,
    op: Operation,
    num: f64,
}

// SYSTEM CHANNEL AND MESSAGES

#[cereal::derived(Copy, Eq, Data)]
pub enum Operation {
    Add,
    Mul,
}

#[cereal::derived(Copy, Eq, Data)]
pub enum MathFail {
    Error,
}

pub type MathResult = Result<f64, MathFail>;

#[cereal::derived(Copy, Data)]
pub struct MathRequest {
    operand: f64,
}

#[cereal::derived(Copy, Data)]
pub enum ToExampleSys {
    MathRequest(MathRequest),
}

#[cereal::derived(Copy, Data)]
pub struct MathResponse {
    result: MathResult,
}

#[cereal::derived(Data)]
pub enum FromExampleSys {
    MathResponse(MathResponse),
}

#[cereal::derived(Eq, Data)]
pub struct ExampleSysParams {
    op: Operation,
}

impl green::Params for ExampleSysParams {}

impl green::System for ExampleSys {
    const CHANNEL_SIZE: usize = 100;
    
    type Paths = ExampleSysPaths;
    type Params = ExampleSysParams;
    type Config = ExampleSysConfig;
    type ToSys = ToExampleSys;
    type FromSys = FromExampleSys;
    type Flow = green::StdFlow;

    fn inner(&self) -> &green::InnerSystem<Self> { &self.inner }
    fn inner_mut(&mut self) -> &mut green::InnerSystem<Self> { &mut self.inner }
    
    async fn init(inner: green::InnerSystem<Self>, params: Self::Params) -> green::GreenResult<Self> {
        Ok(Self {
            op: params.op,
            num: 1.,
            inner,
        })
    }
    
    async fn run(mut self) -> green::UnitResult {
        let result = loop {
            let result = tokio::select! {
                rx = self.inner.channel.rx.recv() => match rx {
                    Some(msg) => self.handle_channel_recv(msg).await,
                    None => Err(green::Failure),
                },
            };
            
            if let Err(e) = result {
                break Err(e);
            }
        };
        
        self.done(result).await
    }
    
    async fn on_channel_recv(&mut self, pkt: green::Packet<ToExampleSys>) -> green::FlowResult<Self::Flow> {
        let (id, nature, msg) = pkt.into_tuple();
        match (nature, msg) {
            (green::PacketNature::Request, ToExampleSys::MathRequest(req))
                => self.on_math_request(green::Packet::new(id, nature, req)).await,
            _ => todo!(),
        }
    }

    async fn on_stop(&mut self, _halt: bool) -> green::UnitResult {
        green::Succeed
    }

    async fn on_resume(&mut self) -> green::SysResult<bool> {
        Ok(true)
    }
}

impl ExampleSys {
    async fn on_math_request(&mut self, req: green::Packet<MathRequest>) -> green::FlowResult<green::StdFlow> {
        self.num = match self.op {
            Operation::Add => req.msg.operand + self.num,
            Operation::Mul => req.msg.operand * self.num,
        };
        
        self.send_channel_packet(req.respond(FromExampleSys::MathResponse(
            MathResponse {
                result: Ok(self.num),
            }
        ))).await?;
        
        Ok(green::Flow::Continue)
    }
}

// SYSTEM PATHS

#[derive(Clone)]
pub struct ExampleSysPaths(agnostic::AppPathRouter);

impl Default for ExampleSysPaths {
    fn default() -> Self {
        Self(agnostic::DefaultAppPaths::Default(
            "sourcetrait/sysgreen/examples/basic"
        ).into())
    }
}

impl HasAppPaths for ExampleSysPaths {
    fn app_paths(&self) -> &impl AppPaths {
        &self.0
    }
}

impl ExampleSysPaths {
    pub const CONFIG_TOML: &'static str = "config.toml";
    
    pub fn config_toml(&self) -> PathBuf {
        self.config_dir().join(Self::CONFIG_TOML)
    }
}

// SYSTEM CONFIG 

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExampleSysConfig {
}

impl ExampleSysConfig {
    pub fn read(paths: &ExampleSysPaths) -> green::GreenResult<Self> {
        let config_path = paths.config_toml(); 
        ExampleSysConfig::from_toml_file(&config_path)
            .map_err(|e| green::GreenError::into_io(e))
    }
}

impl green::Config for ExampleSysConfig {}
impl tomlx::FromToml for ExampleSysConfig {}


