use anyhow::{anyhow, Result};
use clap::{load_yaml, App};
use config::Node;

use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
use tokio::sync::{mpsc::{channel}, oneshot};
use std::{net::{SocketAddr, SocketAddrV4}};

#[tokio::main]
async fn main() -> Result<()> {
    log::error!("{}", std::env::current_dir().unwrap().display());
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();
    //println!("{:?}",m);
    let conf_str = m
        .value_of("config")
        .expect("unable to convert config file into a string");
    let vss_type = m
        .value_of("protocol")
        .expect("Unable to detect protocol to run");
    
    let _syncer_file = m
        .value_of("syncer")
        .expect("Unable to parse syncer ip file");
    let _batches = m
        .value_of("batches")
        .expect("Unable to parse number of batches")
        .parse::<usize>().unwrap();
    let _per_batch = m
        .value_of("per")
        .expect("Unable to parse per batch")
        .parse::<usize>().unwrap();
    let _lin_quad = m
        .value_of("lin")
        .expect("Unable to parse per lin_quad")
        .parse::<bool>().unwrap();
    let _opt_pess = m
        .value_of("opt")
        .expect("Unable to parse per lin_quad")
        .parse::<bool>().unwrap();
    let _ibft = m
        .value_of("ibft")
        .expect("Unable to parse per ibft")
        .parse::<bool>().unwrap();

    // let broadcast_msgs_file = m
    //     .value_of("bfile")
    //     .expect("Unable to parse broadcast messages file");
    // let byz_flag = m.value_of("byz").expect("Unable to parse Byzantine flag");
    // let node_normal: bool = match byz_flag {
    //     "true" => true,
    //     "false" => false,
    //     _ => {
    //         panic!("Byz flag invalid value");
    //     }
    // };
    let conf_file = std::path::Path::new(conf_str);
    let str = String::from(conf_str);
    let mut config = match conf_file
        .extension()
        .expect("Unable to get file extension")
        .to_str()
        .expect("Failed to convert the extension into ascii string")
    {
        "json" => Node::from_json(str),
        "dat" => Node::from_bin(str),
        "toml" => Node::from_toml(str),
        "yaml" => Node::from_yaml(str),
        _ => panic!("Invalid config file extension"),
    };

    simple_logger::SimpleLogger::new()
        .with_utc_timestamps()
        .init()
        .unwrap();
    log::set_max_level(log::LevelFilter::Info);
    config.validate().expect("The decoded config is not valid");
    if let Some(f) = m.value_of("ip") {
        let f_str = f.to_string();
        log::info!("Logging the file f {}", f_str);
        config.update_config(util::io::file_to_ips(f.to_string()));
    }
    let config = config;
    // Start the Reliable Broadcast protocol
    let exit_tx;
    match vss_type {
        "ctrbc" => {
            log::info!("Cachin Tessaro RBC protocol");
            let exit_tx_1;
            let _status;

            (exit_tx_1, _status) = spawn(config).await;
            exit_tx = exit_tx_1.unwrap();
        }
        _ => {
            log::error!(
                "Matching Distributed Computing protocol not provided {}, canceling execution",
                vss_type
            );
            return Ok(());
        }
    }
    //let exit_tx = pedavss_cc::node::Context::spawn(config).unwrap();
    // Implement a waiting strategy
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    signals.forever().next();
    log::error!("Received termination signal");
    exit_tx
        .send(())
        .map_err(|_| anyhow!("Server already shut down"))?;
    log::error!("Shutting down server");
    Ok(())
}

pub fn to_socket_address(ip_str: &str, port: u16) -> SocketAddr {
    let addr = SocketAddrV4::new(ip_str.parse().unwrap(), port);
    addr.into()
}

pub async fn spawn(config: Node)-> (anyhow::Result<oneshot::Sender<()>>, Vec<Result<oneshot::Sender<()>>>){
    // ctrbc_req_send_channel: Request sending channel, request receiving channel. The sending channel can be used to issue message requests to the RBC module. 
    // ctrbc_req_recv_channel: Request receiving channel - passed as an argument. The RBC module listens to this channel. 
    let (ctrbc_req_send_channel, ctrbc_req_recv_channel) = channel(10000);
    
    // ctrbc_out_send_channel: Output sending channel - passed as an argument. The RBC module sends outputs on this channel. 
    // ctrbc_out_recv_channel: Output receiving channel. We poll this channel to get outputs from RBC module.
    let (ctrbc_out_send_channel, mut ctrbc_out_recv_channel) = channel(10000);

    let mut statuses = Vec::new();

    let _rbc_serv_status = ctrbc::Context::spawn(
        config,
        ctrbc_req_recv_channel, 
        ctrbc_out_send_channel, 
        false
    );

    statuses.push(_rbc_serv_status);
    
    let _resp = ctrbc_req_send_channel.send(Vec::new()).await.unwrap();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = ctrbc_out_recv_channel.recv() => {
                    // Execute handling logic for the received message from the channel
                    log::debug!("Received message from CTRBC channel {:?}", msg);
                    // self.process_ctrbc_event(ctrbc_msg.1, ctrbc_msg.0, ctrbc_msg.2).await;
                }
            }
        }
    });
    let (exit_tx, _exit_rx) = oneshot::channel();
    (Ok(exit_tx), vec![])
}
