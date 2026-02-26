use crate::lore::MargoLoreRef;
use crate::quest::SyncQuest;
use crate::relic;
use crate::sorcerer::Sorcerer;
use crate::sorcerer::appraiser::ManifestSource;
use crate::sorcerer::cleric::Cleric;
use crate::vault::Vault;
use crate::vault::pouch::Pouch;
use async_trait::async_trait;
use margo_types::application_deployment::{ApplicationDeployment, DeploymentProfile};
use margo_workload_management_api_client_rs::apis::configuration::Configuration;
use margo_workload_management_api_client_rs::apis::default_api::{
    api_v1_clients_client_id_bundles_digest_get, api_v1_clients_client_id_deployments_get,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

pub struct ClericImpl;

impl Sorcerer for ClericImpl {}

#[async_trait]
impl Cleric for ClericImpl {
    async fn onboarding(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        lore: MargoLoreRef,
    ) -> anyhow::Result<HashMap<String, Vec<ManifestSource>>> {
        // TODO: Use quest system
        let base_path = lore
            .as_ref()
            .as_ref()
            .url
            .to_string()
            .trim_end_matches('/')
            .to_string();
        let config = Configuration {
            base_path,
            ..Configuration::default()
        };
        // TODO: Complete margo onboarding
        let client_id = "flecs-core";
        let deployments = {
            let config = config.clone();
            quest
                .lock()
                .await
                .create_sub_quest("Query deployments", |_quest| async move {
                    api_v1_clients_client_id_deployments_get(&config, client_id, None, None).await
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
            let config = config.clone();
            quest
                .lock()
                .await
                .create_sub_quest(format!("Get bundle {digest}"), |_quest| {
                    get_bundle(digest, client_id.to_string(), config)
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
    client_id: String,
    config: Configuration,
) -> anyhow::Result<HashMap<String, ApplicationDeployment>> {
    let bundle_data =
        api_v1_clients_client_id_bundles_digest_get(&config, &client_id, &bundle_digest, None)
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
