use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::{config, Config, KafkaPlugin};
use log::{debug, info};
use solana_client::rpc_client::RpcClient;
use solana_client::{pubsub_client::PubsubClient, rpc_response::RpcKeyedAccount};
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;

pub struct RpcObserver {
    rpc_client: RpcClient,
    kafka_plugin: Arc<RwLock<KafkaPlugin>>,
    programs: Vec<Pubkey>,
    slot: u64,
}

impl RpcObserver {
    pub fn new(
        rpc_client: RpcClient,
        kafka_plugin: Arc<RwLock<KafkaPlugin>>,
        config: &Config,
    ) -> Self {
        Self {
            rpc_client,
            kafka_plugin,
            programs: config
                .programs
                .iter()
                .map(|p| Pubkey::from_str(&p).unwrap())
                .collect(),
            slot: 0,
        }
    }

    pub fn run(&mut self) {
        self.load_startup_state();
        self.setup_ws_connection();
    }

    pub fn load_startup_state(&mut self) {
        self.slot = self.rpc_client.get_slot().unwrap();
        self.programs.iter().for_each(|owner| {
            info!("Loading accounts for program {:?}", owner);
            let accounts = self.rpc_client.get_program_accounts(owner).unwrap();
            info!("Publishing {} accounts", accounts.len());
            accounts.iter().for_each(|raw_account| {
                self.kafka_plugin
                    .read()
                    .unwrap()
                    .update_account(raw_account, self.slot, true)
                    .unwrap();
            });

            info!("Initial publish done");
        })
    }

    pub fn setup_ws_connection(&self) {
        // self.programs.iter().for_each(|owner| {
        //     info!("Subscribing to account changes for program {:?}", owner);
        //     let mut threads = vec![];
        //     let (_, receiver) =
        //         PubsubClient::program_subscribe("wss://api.devnet.solana.com", owner, None)
        //             .unwrap();
        //     let kafka_plugin = self.kafka_plugin.clone();
        //     let slot = self.slot;

        //     let t = thread::spawn(move || loop {
        //         let RpcKeyedAccount {
        //             pubkey: pubkey_str,
        //             account: ui_account,
        //         } = receiver.recv().unwrap().value;

        //         let address = Pubkey::from_str(&pubkey_str).unwrap();
        //         let account: Account = ui_account.decode().unwrap();

        //         let raw_account = (address, account);

        //         kafka_plugin
        //             .read()
        //             .unwrap()
        //             .update_account(&raw_account, slot, true)
        //             .unwrap();
        //     });

        //     threads.push(t);

        //     for thread in threads {
        //         thread.join().unwrap();
        //     }
        // });
    }
}
