use crate::lore::MargoLoreRef;
use crate::quest::SyncQuest;
use crate::relic;
use crate::sorcerer::Sorcerer;
use crate::sorcerer::appraiser::ManifestSource;
use crate::sorcerer::cleric::{Cleric, Client};
use crate::vault::Vault;
use crate::vault::pouch::Pouch;
use async_trait::async_trait;
use margo_types::application_deployment::{ApplicationDeployment, DeploymentProfile};
use margo_workload_management_api_client_rs::apis::configuration::Configuration;
use margo_workload_management_api_client_rs::apis::default_api::{api_v1_clients_client_id_bundles_digest_get, api_v1_clients_client_id_capabilities_post, api_v1_clients_client_id_deployments_get, api_v1_onboarding_certificate_get, api_v1_onboarding_post};
use margo_workload_management_api_client_rs::models::ApiV1OnboardingPostRequest;
use reqwest_middleware::ClientBuilder;
use std::collections::HashMap;
use std::sync::Arc;
use base64::Engine;
use margo_workload_management_api_client_rs::models;
use tracing::{debug, warn};

pub struct ClericImpl;

impl Sorcerer for ClericImpl {}

#[async_trait]
impl Cleric for ClericImpl {
    async fn onboarding(&self, quest: SyncQuest, lore: MargoLoreRef) -> anyhow::Result<Client> {
        // TODO: Use quest system
        let lore = lore.as_ref().as_ref();
        let base_path = lore.url.to_string().trim_end_matches('/').to_string();
        let config = Configuration {
            base_path,
            ..Configuration::default()
        };
        let cert = tokio::fs::read_to_string(lore.base_path.join("cert.pem")).await?;
        debug!("Certificate file read");
        let key = tokio::fs::read_to_string(lore.base_path.join("key.pem")).await?;
        debug!("Key file read");
        let identity = reqwest::Identity::from_pkcs8_pem(cert.as_bytes(), key.as_bytes())?;
        debug!("Identity constructed from certificate and key file");
        let certificate = api_v1_onboarding_certificate_get(&config)
            .await?
            .certificate
            .unwrap();
        debug!("Received certificate {certificate} from {}", config.base_path);
        let certificate = base64::prelude::BASE64_STANDARD.decode(certificate)?;
        debug!("Decoded base64 of {}", config.base_path);
        let certificate = reqwest::Certificate::from_der(&certificate)?;
        debug!("Constructed certificate of {}", config.base_path);
        let client = reqwest::Client::builder()
            .tls_certs_merge([certificate])
            .identity(identity)
            .build()?;
        let client = ClientBuilder::new(client).build();
        let config = Configuration {
            client,
            ..config
        };
        debug!("Constructed client for {}", config.base_path);
        let client_id = api_v1_onboarding_post(&config, ApiV1OnboardingPostRequest {
            public_certificate: Some(cert),
        })
            .await?
            .client_id
            .unwrap();
        debug!("Onboarding complete, received client_id {client_id} for {}", config.base_path);
        let client = Client {
            id: client_id,
            config,
        };
        if let Err(e) = api_v1_clients_client_id_capabilities_post(&client.config, &client.id, models::DeviceCapabilitiesManifest {
            api_version: "device.margo.org/v1alpha1".to_string(),
            kind: Default::default(),
            properties: Box::new(models::DeviceCapabilitiesManifestProperties::default()),
        }).await {
            warn!("Failed to post capabilities to {}: {e}", client.config.base_path);
        }
        Ok(client)
    }

    async fn receive_bundle(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        client: Arc<Client>,
    ) -> anyhow::Result<HashMap<String, Vec<ManifestSource>>> {
        let deployments = {
            let client = client.clone();
            quest
                .lock()
                .await
                .create_sub_quest("Query deployments", |_quest| async move {
                    api_v1_clients_client_id_deployments_get(&client.config, &client.id, None, None).await
                })
                .await
                .2
        };
        let deployments = deployments.await?;
        // No bundle = no deployments = done
        let Some(bundle) = deployments.bundle else {
            return Ok(HashMap::new());
        };
        let digest = bundle
            .digest
            .ok_or_else(|| anyhow::anyhow!("No bundle digest present"))?;
        let application_deployments = {
            quest
                .lock()
                .await
                .create_sub_quest(format!("Get bundle {digest}"), move |_quest| {
                    get_bundle(digest, client)
                })
                .await
                .2
        };
        let application_deployments = application_deployments.await?;
        let manifest_sources = application_deployments
            .iter()
            .filter_map(|(id, application_deployment)| {
                extract_compose_components_package_locations(application_deployment)
                    .map(|package_locations| (id.clone(), package_locations))
            })
            .collect();
        {
            let mut grab = vault
                .reservation()
                .reserve_application_deployment_pouch_mut()
                .grab()
                .await;
            let application_deployment_pouch =
                grab.application_deployment_pouch_mut.as_mut().unwrap();
            application_deployment_pouch
                .gems_mut()
                .extend(application_deployments);
        }
        Ok(manifest_sources)
    }
}

async fn get_bundle(
    bundle_digest: String,
    client: Arc<Client>,
) -> anyhow::Result<HashMap<String, ApplicationDeployment>> {
    let bundle_data =
        api_v1_clients_client_id_bundles_digest_get(&client.config, &client.id, &bundle_digest, None)
            .await?;
    let application_deployments: HashMap<String, ApplicationDeployment> =
        relic::async_flecstract::decompress_in_memory(bundle_data.bytes_stream())
            .await?
            .values()
            .map(|bytes| {
                serde_norway::from_slice::<ApplicationDeployment>(bytes).map(
                    |application_deployment| {
                        (
                            application_deployment.metadata.annotations.id.clone(),
                            application_deployment,
                        )
                    },
                )
            })
            .collect::<Result<HashMap<String, ApplicationDeployment>, _>>()?;
    debug!(
        "Received {} application deployments in bundle {bundle_digest}",
        application_deployments.len()
    );
    Ok(application_deployments)
}

fn extract_compose_components_package_locations(
    application_deployment: &ApplicationDeployment,
) -> Option<Vec<ManifestSource>> {
    match &application_deployment.spec.deployment_profile {
        DeploymentProfile::Compose { components } => Some(
            components
                .iter()
                .filter_map(
                    |component| match component.properties.package_location.parse() {
                        Err(e) => {
                            warn!(
                                "Invalid package location '{}': {e}",
                                component.properties.package_location
                            );
                            None
                        }
                        Ok(url) => Some(ManifestSource::Url(url)),
                    },
                )
                .collect(),
        ),
        DeploymentProfile::HelmV3 { .. } => {
            warn!("HelmV3 deployments are unsupported");
            None
        }
    }
}
