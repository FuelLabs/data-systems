use std::{
    env,
    fs,
    ops::Range,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use fuel_streams::prelude::*;
use fuel_streams_core::prelude::*;
use futures::StreamExt;
use rand::Rng;
use streams_tests::{publish_blocks, server_setup};

const INTERVAL: Range<u64> = 10..15;

fn find_workspace_root() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok()?;

    loop {
        if current_dir.join("Cargo.toml").exists() {
            // Check if this is a workspace root
            let cargo_toml = current_dir.join("Cargo.toml");
            let content = fs::read_to_string(&cargo_toml).ok()?;
            if content.contains("[workspace]") {
                return Some(current_dir);
            }
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}

fn start_nats(makefile_path: &Path) {
    let status = Command::new("make")
        .arg("-f")
        .arg(makefile_path.to_str().unwrap())
        .arg("cluster_up")
        .status()
        .expect("Failed to start NATS");

    if status.success() {
        println!("NATS started successfully.");
    } else {
        println!("Failed to start NATS.");
    }
}

fn stop_nats(makefile_path: &Path) {
    let status = Command::new("make")
        .arg("-f")
        .arg(makefile_path.to_str().unwrap())
        .arg("cluster_up")
        .status()
        .expect("Failed to stop NATS");

    if status.success() {
        println!("NATS stopped successfully.");
    } else {
        println!("Failed to stop NATS.");
    }
}

#[tokio::main]
async fn main() -> BoxedResult<()> {
    let workspace_root =
        find_workspace_root().expect("Could not find the workspace root");
    let makefile_path = workspace_root.join("Makefile");
    env::set_current_dir(&workspace_root)
        .expect("Failed to change directory to workspace root");

    // ensure nats is connected and running
    let client_opts = NatsClientOpts::admin_opts()
        .with_rdn_namespace()
        .with_timeout(1);

    let s3_client_opts = S3ClientOpts::admin_opts();

    let is_connected = Client::with_opts(&client_opts, &s3_client_opts)
        .await
        .ok()
        .map(|c| c.nats_conn.is_connected())
        .unwrap_or_default();
    if !is_connected {
        println!("Starting nats ...");
        start_nats(&makefile_path);
    }

    // create a subscription
    let (_, _, client) = server_setup().await.unwrap();
    let stream = fuel_streams::Stream::<Block>::new(&client).await;
    let mut sub = stream.subscribe_raw().await.unwrap().enumerate();

    // publish all items in a separate thread
    let (items, publish_join_handle) =
        publish_blocks(stream.stream(), Some(Address::zeroed()), None).unwrap();

    // await publishing to finish
    publish_join_handle.await.unwrap();
    println!("All items pushed to nats !");

    let mut rng = rand::thread_rng();
    let mut action_interval =
        tokio::time::interval(Duration::from_secs(rng.gen_range(INTERVAL)));

    loop {
        tokio::select! {
            Some((index, bytes)) = sub.next() => {
                    println!("Valid subscription");
                    let decoded_msg = Block::decode_raw(bytes).unwrap();
                    let (subject, block) = items[index].to_owned();
                    let height = decoded_msg.payload.height;
                    assert_eq!(decoded_msg.subject, subject.parse());
                    assert_eq!(decoded_msg.payload, block);
                    assert_eq!(height, index as u32);
                    if index == 9 {
                        break;
                    }
            }
            _ = action_interval.tick() => {
                let client_opts = NatsClientOpts::admin_opts()
                .with_rdn_namespace()
                .with_timeout(1);

                let is_nats_connected = Client::with_opts(&client_opts, &s3_client_opts).await.ok().map(|c| c.nats_conn.is_connected()).unwrap_or_default();
                if is_nats_connected {
                    stop_nats(&makefile_path);
                } else {
                    start_nats(&makefile_path);
                }
                action_interval = tokio::time::interval(Duration::from_secs(rng.gen_range(INTERVAL)));
            }
        }
    }

    Ok(())
}
