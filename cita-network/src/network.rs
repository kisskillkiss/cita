use connection::Connection;
use libproto::router::{MsgType, RoutingKey, SubModules};
use libproto::snapshot::{Cmd, Resp, SnapshotResp};
use libproto::{Message, Response};
use std::convert::{Into, TryFrom, TryInto};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use Source;

/// Message forwarding, include p2p and local
pub struct NetWork {
    con: Arc<Connection>,
    tx_pub: Sender<(String, Vec<u8>)>,
    tx_sync: Sender<(Source, (String, Vec<u8>))>,
    tx_new_tx: Sender<(String, Vec<u8>)>,
    tx_consensus: Sender<(String, Vec<u8>)>,
}

impl NetWork {
    pub fn new(
        con: Arc<Connection>,
        tx_pub: Sender<(String, Vec<u8>)>,
        tx_sync: Sender<(Source, (String, Vec<u8>))>,
        tx_new_tx: Sender<(String, Vec<u8>)>,
        tx_consensus: Sender<(String, Vec<u8>)>,
    ) -> Self {
        NetWork {
            con,
            tx_pub,
            tx_sync,
            tx_new_tx,
            tx_consensus,
        }
    }

    pub fn receiver(&self, source: Source, payload: (String, Vec<u8>)) {
        let (key, data) = payload;
        let rtkey = RoutingKey::from(&key);
        trace!("Network receive Msg from {:?}/{}", source, key);
        if self.con.is_disconnect.load(Ordering::SeqCst)
            && rtkey.get_sub_module() != SubModules::Snapshot
        {
            return;
        }
        match source {
            // Come from MQ
            Source::LOCAL => match rtkey {
                routing_key!(Chain >> Status) => {
                    self.tx_sync.send((source, (key, data)));
                }
                routing_key!(Chain >> SyncResponse) => {
                    self.con.broadcast_rawbytes(
                        routing_key!(Synchronizer >> SyncResponse).into(),
                        &data,
                    );
                }
                routing_key!(Jsonrpc >> RequestNet) => {
                    self.reply_rpc(&data);
                }
                routing_key!(Snapshot >> SnapshotReq) => {
                    info!("set disconnect and response");
                    self.snapshot_req(&data);
                }
                _ => {
                    error!("Unexpected key {} from {:?}", key, source);
                }
            },
            // Come from Netserver
            Source::REMOTE => match rtkey {
                routing_key!(Synchronizer >> Status)
                | routing_key!(Synchronizer >> SyncResponse) => {
                    self.tx_sync.send((source, (key, data)));
                }
                routing_key!(Synchronizer >> SyncRequest) => {
                    self.tx_pub
                        .send((routing_key!(Net >> SyncRequest).into(), data));
                }
                routing_key!(Auth >> Request) => {
                    self.tx_new_tx
                        .send((routing_key!(Net >> Request).into(), data));
                }
                routing_key!(Consensus >> SignedProposal) => {
                    self.tx_consensus
                        .send((routing_key!(Net >> SignedProposal).into(), data));
                }
                routing_key!(Consensus >> RawBytes) => {
                    self.tx_consensus
                        .send((routing_key!(Net >> RawBytes).into(), data));
                }
                _ => {
                    error!("Unexpected key {} from {:?}", key, source);
                }
            },
        }
    }

    fn snapshot_req(&self, data: &[u8]) {
        let mut msg = Message::try_from(data).unwrap();
        let req = msg.take_snapshot_req().unwrap();
        let mut resp = SnapshotResp::new();
        let mut send = false;
        match req.cmd {
            Cmd::Snapshot => {
                info!("[snapshot] receive cmd: Snapshot");
            }
            Cmd::Begin => {
                info!("[snapshot] receive cmd: Begin");
                self.con.is_disconnect.store(true, Ordering::SeqCst);
                resp.set_resp(Resp::BeginAck);
                resp.set_flag(true);
                send = true;
            }
            Cmd::Restore => {
                info!("[snapshot] receive cmd: Restore");
            }
            Cmd::Clear => {
                info!("[snapshot] receive cmd: Clear");
                resp.set_resp(Resp::ClearAck);
                resp.set_flag(true);
                send = true;
            }
            Cmd::End => {
                info!("[snapshot] receive cmd: End");
                self.con.is_disconnect.store(false, Ordering::SeqCst);
                resp.set_resp(Resp::EndAck);
                resp.set_flag(true);
                send = true;
            }
        }

        if send {
            let msg: Message = resp.into();
            self.tx_pub
                .send((
                    routing_key!(Net >> SnapshotResp).into(),
                    (&msg).try_into().unwrap(),
                ))
                .unwrap();
        }
    }

    pub fn reply_rpc(&self, data: &[u8]) {
        let mut msg = Message::try_from(data).unwrap();
        let req_opt = msg.take_request();
        {
            if let Some(mut ts) = req_opt {
                let mut response = Response::new();
                response.set_request_id(ts.take_request_id());
                if ts.has_peercount() {
                    let peercount = self
                        .con
                        .peers_pair
                        .read()
                        .iter()
                        .filter(|x| x.2.is_some())
                        .count();
                    response.set_peercount(peercount as u32);
                    let ms: Message = response.into();
                    self.tx_pub
                        .send((routing_key!(Net >> Response).into(), ms.try_into().unwrap()))
                        .unwrap();
                }
            } else {
                warn!("receive unexpected rpc data");
            }
        }
    }
}
