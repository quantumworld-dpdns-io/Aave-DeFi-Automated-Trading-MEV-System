// Configuration management
pub mod chain {
    pub mod manager;
}

pub mod config {
    pub struct Config {
        pub chains: Vec<ChainConfig>,
        pub rpc_timeout: u64,
        pub max_gas_price_gwei: u64,
        pub min_profit_usd: f64,
    }

    impl Config {
        pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
            let chains = vec![
                ChainConfig {
                    name: "base".to_string(),
                    rpc_ws: std::env::var("RPC_WSS_URL_BASE")?,
                    rpc_http: std::env::var("RPC_HTTP_URL_BASE")?,
                    chain_id: 8453,
                },
                ChainConfig {
                    name: "polygon".to_string(),
                    rpc_ws: std::env::var("RPC_WSS_URL_POLYGON")?,
                    rpc_http: std::env::var("RPC_HTTP_URL_POLYGON")?,
                    chain_id: 137,
                },
            ];

            Ok(Self {
                chains,
                rpc_timeout: 5000,
                max_gas_price_gwei: 100,
                min_profit_usd: 10.0,
            })
        }
    }

    #[derive(Debug, Clone)]
    pub struct ChainConfig {
        pub name: String,
        pub rpc_ws: String,
        pub rpc_http: String,
        pub chain_id: u64,
    }
}

pub mod mempool {
    pub mod monitor;
}

pub mod simulator {
    pub mod engine;
    pub mod tracer;
}

pub mod api {
    pub mod grpc;
    pub mod http;
    pub mod server;
}

pub mod metrics {
    pub struct Metrics;
}
