extern crate electrs;

#[cfg(not(feature = "liquid"))]
#[macro_use]
extern crate log;

#[cfg(not(feature = "liquid"))]
fn main() {
    use electrs::{
        config::Config,
        daemon::Daemon,
        metrics::Metrics,
        new_index::{FetchFrom, Indexer, Store, start_fetcher},
        signal::Waiter,
    };
    use std::sync::Arc;

    let signal = Waiter::start();
    let config = Config::from_args();
    let store = Arc::new(Store::open(&config.db_path.join("newindex"), &config));

    let metrics = Metrics::new(config.monitoring_addr);
    metrics.start();

    let daemon = Arc::new(
        Daemon::new(
            &config.daemon_dir,
            &config.blocks_dir,
            config.daemon_rpc_addr,
            config.cookie_getter(),
            config.network_type,
            signal,
            &metrics,
        )
        .unwrap(),
    );
    let from = FetchFrom::BlkFiles;
    let mut indexer = Indexer::open(Arc::clone(&store), from, &config, &metrics);
    indexer.update(&daemon).unwrap();

    let to_index = indexer.get_all_indexed_headers();
    debug!(
        "Re-indexing history from {} blocks using {:?}",
        to_index.len(),
        from
    );
    start_fetcher(from, &daemon, to_index)
        .unwrap()
        .map(|blocks| indexer.index(&blocks));
    store.history_db().flush();
}

#[cfg(feature = "liquid")]
fn main() {}
