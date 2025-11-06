mod srvclib { pub mod shared; }
use srvclib::shared::*;
use sourcetrait_sock as sock;

#[tokio::main]
async fn main() {
    let outbound_config = sock::OutboundTlsConfig {
        hostname: "127.0.0.1".to_string(),
        port: 4032,
        user_hash_id: 34,
        cert_file: None,
    };
    
    
    let outbound_tls = sock::OutboundTls::connect(&SERVICE, &ServiceDef::LANGUAGE_INDEX, outbound_config).await.unwrap();
    
    let mut tally = 8;
    let _: SetTallyResponse = outbound_tls.query(SetTallyRequest { tally }).await.unwrap();
    dbg!(tally);
    
    for operand in 1..255 {
        let request = AddRequest { operand };
        let response: AddResponse = outbound_tls.query(request).await.unwrap();
        tally += operand;
        assert_eq!(tally, response.tally);
        dbg!(tally);
    }
}

