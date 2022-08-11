use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::KafkaPlugin;
use solana_client::rpc_client::RpcClient;
use solana_client::{pubsub_client::PubsubClient, rpc_response::RpcKeyedAccount};
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;

pub struct RpcObserver {
    rpc_client: RpcClient,
    kafka_plugin: Arc<RwLock<KafkaPlugin>>,
    owner_accounts: Vec<Pubkey>,
    slot: u64,
}

impl RpcObserver {
    pub fn new(rpc_client: RpcClient, kafka_plugin: Arc<RwLock<KafkaPlugin>>) -> Self {
        Self {
            rpc_client,
            kafka_plugin,
            owner_accounts: vec![],
            slot: 0,
        }
    }

    pub fn run(&mut self) {
        self.load_startup_state();
        self.setup_ws_connection();
    }

    pub fn load_startup_state(&mut self) {
        self.slot = self.rpc_client.get_slot().unwrap();
        self.owner_accounts.iter().for_each(|owner| {
            let accounts = self.rpc_client.get_program_accounts(owner).unwrap();
            accounts.iter().for_each(|raw_account| {
                self.kafka_plugin
                    .read()
                    .unwrap()
                    .update_account(raw_account, self.slot, true)
                    .unwrap();
            });
        })
    }

    pub fn setup_ws_connection(&self) {
        self.owner_accounts.iter().for_each(|owner| {
            let (_subscription, a) =
                PubsubClient::program_subscribe(self.rpc_client.url().as_str(), owner, None)
                    .unwrap();
            let kafka_plugin = self.kafka_plugin.clone();
            let slot = self.slot;

            let _a = thread::spawn(move || loop {
                let RpcKeyedAccount {
                    pubkey: pubkey_str,
                    account: ui_account,
                } = a.recv().unwrap().value;

                let address = Pubkey::from_str(&pubkey_str).unwrap();
                let account: Account = ui_account.decode().unwrap();

                let raw_account = (address, account);

                kafka_plugin
                    .read()
                    .unwrap()
                    .update_account(&raw_account, slot, true)
                    .unwrap();
            });
        });
    }
}
