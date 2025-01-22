use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task;
use crate::network::node::Node;
use crate::network::peer::Peer;
use crate::network::message::{NetworkMessage, BlockData, TransactionData};
use crate::wallet::Wallet;
use chrono::Utc;

#[tokio::main]
async fn main() {
    let mut cluster = Vec::new();
    let blockchain = Arc::new(RwLock::new(Vec::<BlockData>::new()));

    let node_count = 3;
    for i in 0..node_count {
        let address = format!("127.0.0.1:{}", 8000 + i).parse().unwrap();
        let (tx_out, rx_out) = mpsc::channel::<NetworkMessage>(100);
        let (tx_in, rx_in) = mpsc::channel::<NetworkMessage>(100);
        let node = Node::new(address, rx_in, tx_out.clone(), 1000);
        let mut wallet = Wallet::new();
        wallet.balance = 100;
        cluster.push((node, wallet, tx_in, tx_out));
    }

    for (i, (node, _, tx_in, _)) in cluster.iter().enumerate() {
        for (j, (other_node, _, _, tx_out)) in cluster.iter().enumerate() {
            if i != j {
                let peer = Peer::new(other_node.address, false, 0);
                node.add_peer(peer.clone());

                let tx_in_clone = tx_in.clone();
                let tx_out_clone = tx_out.clone();
                task::spawn(async move {
                    while let Some(msg) = tx_out_clone.recv().await {
                        tx_in_clone.send(msg).await.unwrap();
                    }
                });
            }
        }
    }

    if let Some((_, wallet, tx_in, _)) = cluster.get(0) {
        let recipient_address = cluster[1].1.addresses[0].clone();
        let tx = wallet.send_payment(&recipient_address, 50).unwrap();

        let msg = NetworkMessage::Transaction(tx);
        tx_in.send(msg).await.unwrap();
    }

    for (node, _, _, _) in &cluster {
        let blockchain_clone = blockchain.clone();
        let mut node_clone = node.clone();

        task::spawn(async move {
            node_clone.handle_incoming().await;

            let blocks = blockchain_clone.read().await;
            for block in blocks.iter() {
                println!("Block: {:#?}", block);
            }
        });
    }

    for (_, wallet, _, _) in &cluster {
        let blocks = blockchain.read().await.clone();
        wallet.sync_balance(&blocks);
        println!("Wallet: {} - Balance: {}", wallet.addresses[0], wallet.get_balance());
    }

    if let Some((node, _, tx_in, _)) = cluster.get(0) {
        let mut node_clone = node.clone();

        task::spawn(async move {
            node_clone.create_and_broadcast_block().await;
            let blocks = blockchain.read().await;
            println!("Updated Blockchain: {:#?}", *blocks);
        });
    }

    tokio::signal::ctrl_c().await.unwrap();
    println!("Simulation finished.");
}