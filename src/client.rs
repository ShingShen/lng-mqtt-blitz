use anyhow::Result;
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::config::LngAppConfig;
use crate::stats::LngMetrics;

pub async fn spawn_lng_client(
    id: usize,
    config: Arc<LngAppConfig>,
    metrics: Arc<LngMetrics>,
) -> Result<()> {
    let client_id = format!("lng-blitz-{}", id);
    let mut mqtt_options = MqttOptions::new(&client_id, &config.target_host, config.port);
    mqtt_options.set_keep_alive(Duration::from_secs(60));

    if let (Some(u), Some(p)) = (&config.username, &config.password) {
        mqtt_options.set_credentials(u, p);
    }

    if config.use_tls {
        let mut root_cert_store = rustls::RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs()? {
            root_cert_store.add(cert)?;
        }

        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        mqtt_options.set_transport(Transport::tls_with_config(tls_config.into()));
    }

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    metrics.active_conns.fetch_add(1, Ordering::Relaxed);

    let metrics_p = Arc::clone(&metrics);
    let config_p = Arc::clone(&config);
    let client_p = client.clone();

    // Publisher task
    tokio::spawn(async move {
        let topic = format!("lng/blitz/{}", id);

        // prepare deterministic pieces outside the hot loop
        let id_str = id.to_string();
        let base_payload = config_p.payload_template.replace("{{id}}", &id_str);

        let mut rng = StdRng::from_entropy();

        loop {
            let payload = base_payload.replace("{{random}}", &rng.gen_range(10..100).to_string());

            if let Err(_) = client_p.publish(&topic, QoS::AtLeastOnce, false, payload).await {
                metrics_p.errors.fetch_add(1, Ordering::Relaxed);
                break;
            }
            metrics_p.sent.fetch_add(1, Ordering::Relaxed);
            sleep(Duration::from_millis(config_p.interval_ms)).await;
        }
    });

    // EventLoop task
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {
                    metrics.recv.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    metrics.errors.fetch_add(1, Ordering::Relaxed);
                    metrics.active_conns.fetch_sub(1, Ordering::Relaxed);
                    // Standard practice to just log error if needed
                    // eprintln!("Client {} error: {:?}", id, e);
                    let _ = e;
                    break;
                }
            }
        }
    });

    Ok(())
}
