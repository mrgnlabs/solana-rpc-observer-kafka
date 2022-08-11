// Copyright 2022 Blockdaemon Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    env,
    sync::{Arc, RwLock},
};

use log::info;

mod config;
mod event;
mod filter;
mod plugin;
mod publisher;
mod rpc;

pub use {
    config::{Config, Producer},
    event::*,
    filter::Filter,
    plugin::KafkaPlugin,
    publisher::Publisher,
};

fn main() {
    let mut kp = KafkaPlugin::new();

    let args: Vec<String> = env::args().collect();
    let file_path = &args.get(1).expect("<config path> must be set");
    info!("Loading Kafka plugin from config_file {:?}", file_path);
    let config = Config::read_from(file_path).unwrap();
    kp.init(&config);

    let mut rpc = rpc::RpcObserver::new(
        solana_client::rpc_client::RpcClient::new("https://devnet.genesysgo.net"),
        Arc::new(RwLock::new(kp)),
        &config,
    );

    rpc.run();
}
