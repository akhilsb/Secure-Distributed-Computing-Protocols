use std::sync::Arc;

use super::ProtMsg;
use crate::context::Context;

use crypto::hash::verf_mac;
use types::{WrapperMsg};

impl Context {
    pub fn check_proposal(&self, wrapper_msg: Arc<WrapperMsg<ProtMsg>>) -> bool {
        let byte_val =
            bincode::serialize(&wrapper_msg.protmsg).expect("Failed to serialize object");

        let sec_key = match self.sec_key_map.get(&wrapper_msg.sender) {
            Some(val) => val,
            None => {
                panic!("Secret key not available, this shouldn't happen")
            }
        };

        if !verf_mac(&byte_val, sec_key.as_slice(), &wrapper_msg.mac) {
            log::warn!("MAC Verification failed.");
            return false;
        }
        true
    }

    pub(crate) async fn process_msg(&mut self, wrapper_msg: WrapperMsg<ProtMsg>) {
        log::debug!("Received protocol msg: {:?}", wrapper_msg);
        let msg = Arc::new(wrapper_msg.clone());

        if self.check_proposal(msg) {
            match wrapper_msg.protmsg {
                ProtMsg::Echo(main_msg, instance_id) => {
                    log::info!(
                        "Received Echo for instance id {} from node {:?}",
                        instance_id,
                        main_msg.origin
                    );
                    self.handle_echo(main_msg, instance_id).await;
                }
                ProtMsg::Ready(main_msg, instance_id) => {
                    log::info!(
                        "Received Ready for instance id {} from node {:?}",
                        instance_id,
                        main_msg.origin
                    );
                    self.handle_ready(main_msg, instance_id).await;
                }
                ProtMsg::Init(main_msg, instance_id) => {
                    log::info!(
                        "Received Init for instance id {} from node {:?}",
                        instance_id,
                        main_msg.origin
                    );
                    self.handle_init(main_msg, instance_id).await;
                }
            }
        } else {
            log::warn!(
                "MAC Verification failed for message {:?}",
                wrapper_msg.protmsg
            );
        }
    }

    // Invoke this function once you terminate the protocol
    pub async fn terminate(&mut self, instance_id: usize, data: Vec<u8>) {
        let inst_id = instance_id % self.threshold;
        let party = instance_id/self.threshold;
        log::info!("Terminated {}th RBC initiated by instance {}, sending message back to the channel",inst_id, party);

        let status = self.out_rbc.send((inst_id,party,data)).await;
        if status.is_err(){
            log::error!("Error sending message back to the channel: {}",status.unwrap_err());
        }
    }
}
