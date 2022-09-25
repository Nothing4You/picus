use agent::AgentConfig;
use env::*;
use hetzner::HetznerAgentProviderParams;
use reqwest::Error;
use tokio::time::sleep;

mod strategy;
use strategy::*;

mod agent;
mod env;
mod hetzner;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let wp_token = read_env_or_exit("PICUS_WOODPECKER_TOKEN");
    let wp_server = read_env_or_exit("PICUS_WOODPECKER_SERVER");
    let poll_interval =
        parse_duration::parse(&read_env_or_default("PICUS_POLL_INTERVAL", "10s")).unwrap();
    let shutdown_timer =
        parse_duration::parse(&read_env_or_default("PICUS_MAX_IDLE_TIME", "30m")).unwrap();

    let agent_config = AgentConfig::from_env();
    let hetzner_params = HetznerAgentProviderParams::from_env();
    let hetzner_agent_provider = hetzner::HetznerAgentProvider::new(hetzner_params, agent_config);
    let mut strategy = Strategy::new(Box::new(hetzner_agent_provider), shutdown_timer);

    let request_url = format!("{}/api/queue/info", wp_server);
    let client = reqwest::Client::new();

    loop {
        let response = client
            .get(&request_url)
            .bearer_auth(&wp_token)
            .send()
            .await?;

        let wp_queue_info: WpQueueInfo = response.json().await?;
        println!("{:?}", wp_queue_info);

        strategy.apply(&wp_queue_info).await;

        sleep(poll_interval).await;
    }
}
