// use std::{collections::HashSet, sync::Arc, time::Duration};

// use fuel_streams::prelude::*;
// use fuel_streams_core::prelude::{types, *};
// use futures::{
//     future::{try_join_all, BoxFuture},
//     FutureExt,
//     StreamExt,
//     TryStreamExt,
// };
// use rand::{distributions::Alphanumeric, Rng};
// use streams_tests::{publish_blocks, server_setup};
// use tokio::time::timeout;

// fn gen_random_string(size: usize) -> String {
//     rand::thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(size)
//         .map(char::from)
//         .collect()
// }

// #[tokio::test]
// async fn conn_streams_has_required_streams() -> BoxedResult<()> {
//     let (client, streams, _) = server_setup().await.unwrap();
//     let mut context_streams = client.jetstream.stream_names();

//     let mut names = HashSet::new();
//     while let Some(name) = context_streams.try_next().await? {
//         names.insert(name);
//     }
//     streams.blocks.assert_has_stream(&names).await;
//     streams.transactions.assert_has_stream(&names).await;

//     for name in names.iter() {
//         let empty = streams.blocks.is_empty(name).await;
//         assert!(empty, "stream must be empty after creation");
//     }
//     Ok(())
// }

// #[tokio::test]
// async fn fuel_streams_client_connection() -> BoxedResult<()> {
//     let nats_opts = NatsClientOpts::admin_opts();
//     let client = NatsClient::connect(&nats_opts).await?;
//     assert!(client.is_connected());
//     let s3_opts = Arc::new(S3ClientOpts::admin_opts());
//     let client = Client::with_opts(&nats_opts, &s3_opts).await?;
//     assert!(client.nats_conn.is_connected());
//     Ok(())
// }

// #[tokio::test]
// async fn multiple_client_connections() -> BoxedResult<()> {
//     let nats_opts = NatsClientOpts::admin_opts();
//     let s3_opts = Arc::new(S3ClientOpts::admin_opts());
//     let tasks: Vec<_> = (0..100)
//         .map(|_| {
//             let nats_opts = nats_opts.clone();
//             let s3_opts = s3_opts.clone();
//             async move {
//                 let client =
//                     Client::with_opts(&nats_opts, &s3_opts).await.unwrap();
//                 assert!(client.nats_conn.is_connected());
//                 Ok::<(), NatsError>(())
//             }
//         })
//         .collect();

//     assert!(try_join_all(tasks).await.is_ok());
//     Ok(())
// }

// #[tokio::test]
// async fn public_user_cannot_create_streams() -> BoxedResult<()> {
//     let network = FuelNetwork::Local;
//     let opts = NatsClientOpts::public_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);
//     let client = NatsClient::connect(&opts).await?;
//     let (random_stream_title, random_subject) =
//         (gen_random_string(6), gen_random_string(6));

//     assert!(client
//         .jetstream
//         .create_stream(types::NatsStreamConfig {
//             name: random_stream_title,
//             subjects: vec![random_subject],
//             ..Default::default()
//         })
//         .await
//         .is_err());

//     Ok(())
// }

// #[tokio::test]
// async fn public_user_cannot_create_stores() -> BoxedResult<()> {
//     let network = FuelNetwork::Local;
//     let opts = NatsClientOpts::public_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);

//     let random_bucket_title = gen_random_string(6);

//     let client = NatsClient::connect(&opts).await?;
//     assert!(client
//         .jetstream
//         .create_key_value(types::KvStoreConfig {
//             bucket: random_bucket_title,
//             ..Default::default()
//         })
//         .await
//         .is_err());

//     Ok(())
// }

// #[tokio::test]
// async fn public_user_cannot_delete_stores() -> BoxedResult<()> {
//     let network = FuelNetwork::Local;
//     let opts = NatsClientOpts::admin_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);

//     let random_bucket_title = gen_random_string(6);

//     let client = NatsClient::connect(&opts).await?;
//     client
//         .jetstream
//         .create_key_value(types::KvStoreConfig {
//             bucket: random_bucket_title.clone(),
//             ..Default::default()
//         })
//         .await?;

//     let opts = NatsClientOpts::public_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);
//     let client = NatsClient::connect(&opts).await?;

//     assert!(client
//         .jetstream
//         .delete_key_value(&random_bucket_title)
//         .await
//         .is_err());

//     Ok(())
// }

// #[tokio::test]
// async fn public_user_cannot_delete_stream() -> BoxedResult<()> {
//     let opts = NatsClientOpts::admin_opts()
//         .with_rdn_namespace()
//         .with_timeout(1);
//     let client = NatsClient::connect(&opts).await?;

//     let (random_stream_title, random_subject) =
//         (gen_random_string(6), gen_random_string(6));

//     client
//         .jetstream
//         .create_stream(types::NatsStreamConfig {
//             name: random_stream_title.clone(),
//             subjects: vec![random_subject],
//             ..Default::default()
//         })
//         .await?;

//     let network = FuelNetwork::Local;
//     let public_opts =
//         NatsClientOpts::public_opts().with_url(network.to_nats_url());
//     let public_client = NatsClient::connect(&public_opts).await?;

//     assert!(
//         public_client
//             .jetstream
//             .delete_stream(&random_stream_title)
//             .await
//             .is_err(),
//         "Stream must be deleted at this point"
//     );

//     Ok(())
// }

// #[tokio::test]
// async fn public_user_can_access_streams_after_created() {
//     let network = FuelNetwork::Local;
//     let admin_opts = NatsClientOpts::admin_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);

//     let public_opts = NatsClientOpts::public_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);

//     assert!(NatsClient::connect(&admin_opts).await.is_ok());
//     assert!(NatsClient::connect(&public_opts).await.is_ok());
// }

// #[tokio::test]
// async fn public_and_admin_user_can_access_streams_after_created(
// ) -> BoxedResult<()> {
//     let network = FuelNetwork::Local;
//     let admin_opts = NatsClientOpts::admin_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);
//     let s3_opts = Arc::new(S3ClientOpts::admin_opts());
//     let admin_tasks: Vec<BoxFuture<'_, Result<(), NatsError>>> = (0..100)
//         .map(|_| {
//             let opts: NatsClientOpts = admin_opts.clone();
//             let s3_opts = s3_opts.clone();
//             async move {
//                 let client = Client::with_opts(&opts, &s3_opts).await.unwrap();
//                 assert!(client.nats_conn.is_connected());
//                 Ok::<(), NatsError>(())
//             }
//             .boxed()
//         })
//         .collect();

//     let public_opts = NatsClientOpts::public_opts()
//         .with_url(network.to_nats_url())
//         .with_rdn_namespace()
//         .with_timeout(1);
//     let s3_public_opts =
//         Arc::new(S3ClientOpts::new(S3Env::Local, S3Role::Public));
//     let public_tasks: Vec<BoxFuture<'_, Result<(), NatsError>>> = (0..100)
//         .map(|_| {
//             let opts: NatsClientOpts = public_opts.clone();
//             let s3_opts = s3_public_opts.clone();
//             async move {
//                 let client = Client::with_opts(&opts, &s3_opts).await.unwrap();
//                 assert!(client.nats_conn.is_connected());
//                 Ok::<(), NatsError>(())
//             }
//             .boxed()
//         })
//         .collect();

//     // Combine both vectors into one
//     let mut all_tasks =
//         Vec::with_capacity(admin_tasks.len() + public_tasks.len());
//     all_tasks.extend(admin_tasks);
//     all_tasks.extend(public_tasks);

//     assert!(try_join_all(all_tasks).await.is_ok());
//     Ok(())
// }

// #[tokio::test]
// async fn admin_user_can_delete_stream() -> BoxedResult<()> {
//     let opts = NatsClientOpts::admin_opts()
//         .with_rdn_namespace()
//         .with_timeout(1);
//     let client = NatsClient::connect(&opts).await?;

//     let (random_stream_title, random_subject) =
//         (gen_random_string(6), gen_random_string(6));

//     client
//         .jetstream
//         .create_stream(types::NatsStreamConfig {
//             name: random_stream_title.clone(),
//             subjects: vec![random_subject],
//             ..Default::default()
//         })
//         .await?;

//     let status = client.jetstream.delete_stream(&random_stream_title).await?;
//     assert!(status.success, "Stream must be deleted at this point");

//     Ok(())
// }

// #[tokio::test]
// async fn admin_user_can_delete_stores() -> BoxedResult<()> {
//     let opts = NatsClientOpts::admin_opts()
//         .with_rdn_namespace()
//         .with_timeout(1);

//     let random_bucket_title = gen_random_string(6);

//     let client = NatsClient::connect(&opts).await?;
//     client
//         .jetstream
//         .create_key_value(types::KvStoreConfig {
//             bucket: random_bucket_title.clone(),
//             ..Default::default()
//         })
//         .await?;

//     assert!(client
//         .jetstream
//         .delete_key_value(&random_bucket_title)
//         .await
//         .is_ok());

//     Ok(())
// }

// #[tokio::test]
// async fn ensure_deduplication_when_publishing() -> BoxedResult<()> {
//     let (_, _, client) = server_setup().await.unwrap();
//     let stream = fuel_streams::Stream::<Block>::new(&client).await;
//     let producer = Some(Address::zeroed());
//     let const_block_height = 1001;
//     let items =
//         publish_blocks(stream.stream(), producer, Some(const_block_height))
//             .unwrap()
//             .0;

//     let mut sub = stream.subscribe_raw().await.unwrap().enumerate();
//     let timeout_duration = Duration::from_secs(1);

//     // ensure just one message was published
//     'l: loop {
//         match timeout(timeout_duration, sub.next()).await {
//             Ok(Some((idx, entry))) => {
//                 let decoded_msg = Block::decode_raw(entry).unwrap();
//                 let (subject, _block) = items[idx].to_owned();
//                 let height = decoded_msg.payload.height;
//                 assert_eq!(decoded_msg.subject, subject.parse());
//                 assert_eq!(height, const_block_height);
//                 assert!(idx < 1);
//             }
//             _ => {
//                 break 'l;
//             }
//         }
//     }

//     Ok(())
// }
