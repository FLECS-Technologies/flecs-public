use crate::lore::{MargoLore, MargoLoreRef};
use crate::quest::SyncQuest;
use crate::sorcerer::Sorcerer;
use crate::sorcerer::appraiser::ManifestSource;
use crate::sorcerer::cleric::{Cleric, Client};
use crate::vault::Vault;
use crate::vault::pouch::Pouch;
use crate::{quest, relic};
use async_trait::async_trait;
use base64::Engine;
use futures_util::future::join_all;
use http::Extensions;
use margo_workload_management_api_client_rs::apis::configuration::Configuration;
use margo_workload_management_api_client_rs::apis::default_api::{
    api_v1_clients_client_id_bundles_digest_get, api_v1_clients_client_id_capabilities_post,
    api_v1_clients_client_id_deployments_deployment_id_digest_get,
    api_v1_clients_client_id_deployments_get, api_v1_onboarding_certificate_get,
    api_v1_onboarding_post,
};
use margo_workload_management_api_client_rs::models;
use margo_workload_management_api_client_rs::models::{
    ApiV1OnboardingPostRequest, DeploymentManifestRef,
};
use models::app_deployment_manifest::AppDeploymentManifest as ApplicationDeployment;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

pub struct ClericImpl;

impl Sorcerer for ClericImpl {}

struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        debug!("{req:?}");
        let res = next.run(req, extensions).await;
        debug!("{res:?}");
        res
    }
}

pub trait GetApplicationDeploymentId {
    fn get_deployment_id(&self) -> String;
}

impl GetApplicationDeploymentId for ApplicationDeployment {
    fn get_deployment_id(&self) -> String {
        if let Some(id) = &self.metadata.id {
            return id.clone();
        }
        if let Some(annotations) = &self.metadata.annotations {
            if let Some(id) = annotations.get("id") {
                return id.clone();
            }
        }
        self.metadata.name.clone()
    }
}

#[async_trait]
impl Cleric for ClericImpl {
    async fn onboarding(&self, quest: SyncQuest, lore: MargoLoreRef) -> anyhow::Result<Client> {
        // TODO: Use quest system
        let lore = lore.as_ref().as_ref();
        let base_path = lore.url.to_string().trim_end_matches('/').to_string();
        let config = Configuration {
            base_path,
            client: ClientBuilder::new(reqwest::Client::new())
                .with(LoggingMiddleware)
                .build(),
            ..Configuration::default()
        };
        let cert_and_identity = {
            let base_path = lore.base_path.clone();
            quest
                .lock()
                .await
                .create_sub_quest(
                    "Construct identity from certificate and private key",
                    |_quest| async move {
                        let cert = tokio::fs::read_to_string(base_path.join("cert.pem")).await?;
                        debug!("Certificate file read");
                        let key = tokio::fs::read_to_string(base_path.join("key.pem")).await?;
                        debug!("Key file read");
                        let identity =
                            reqwest::Identity::from_pkcs8_pem(cert.as_bytes(), key.as_bytes())?;
                        Result::<_, anyhow::Error>::Ok((cert, identity))
                    },
                )
                .await
                .2
        };
        let remote_certificate = {
            let config = config.clone();
            quest
                .lock()
                .await
                .create_sub_quest(
                    format!("Receive certificate for {}", config.base_path),
                    |_quest| async move {
                        let certificate = api_v1_onboarding_certificate_get(&config)
                            .await?
                            .certificate
                            .unwrap();
                        debug!(
                            "Received certificate {certificate} from {}",
                            config.base_path
                        );
                        let certificate = base64::prelude::BASE64_STANDARD.decode(certificate)?;
                        debug!("Decoded base64 of {}", config.base_path);
                        reqwest::Certificate::from_der(&certificate).map_err(anyhow::Error::from)
                    },
                )
                .await
                .2
        };
        let remote_certificate = remote_certificate.await?;
        let (cert, identity) = cert_and_identity.await?;
        let client = reqwest::Client::builder()
            .tls_certs_merge([remote_certificate])
            .identity(identity)
            .build()?;
        let client = ClientBuilder::new(client).with(LoggingMiddleware).build();
        let config = Configuration { client, ..config };
        debug!("Constructed client for {}", config.base_path);

        let client_id = {
            let config = config.clone();
            quest
                .lock()
                .await
                .create_sub_quest(
                    format!("Register device at {}", config.base_path),
                    |_quest| async move {
                        api_v1_onboarding_post(
                            &config,
                            ApiV1OnboardingPostRequest {
                                api_version: MargoLore::API_VERSION.to_string(),
                                certificate: cert,
                                ..ApiV1OnboardingPostRequest::default()
                            },
                        )
                        .await?
                        .client_id
                        .ok_or_else(|| anyhow::anyhow!("Received no client id"))
                    },
                )
                .await
                .2
        };
        let client_id = client_id.await?;
        debug!(
            "Onboarding complete, received client_id {client_id} for {}",
            config.base_path
        );
        let client = Client {
            id: client_id,
            config,
        };
        if let Err(e) = api_v1_clients_client_id_capabilities_post(
            &client.config,
            &client.id,
            models::DeviceCapabilitiesManifest {
                api_version: MargoLore::API_VERSION.to_string(),
                kind: Default::default(),
                properties: Box::new(models::DeviceCapabilitiesManifestProperties {
                    id: client.id.clone(),
                    vendor: "Some Vendor".to_string(),
                    model_number: "1234".to_string(),
                    serial_number: "5678".to_string(),
                    roles: vec![
                        models::device_capabilities_manifest_properties::Roles::StandaloneDevice,
                    ],
                    resources: Box::new(models::DeviceCapabilitiesManifestPropertiesResources {
                        cpu: Box::new(models::DeviceCapabilitiesManifestPropertiesResourcesCpu {
                            cores: 8.0,
                            architecture: None,
                        }),
                        memory: "16GB".to_string(),
                        storage: "128GB".to_string(),
                        peripherals: vec![],
                        interfaces: vec![],
                    }),
                }),
            },
        )
        .await
        {
            warn!(
                "Failed to post capabilities to {}: {e}",
                client.config.base_path
            );
        }
        Ok(client)
    }

    async fn receive_bundle(
        &self,
        quest: SyncQuest,
        vault: Arc<Vault>,
        client: Client,
    ) -> anyhow::Result<HashMap<String, HashMap<String, ManifestSource>>> {
        let deployments = {
            let client = client.clone();
            quest
                .lock()
                .await
                .create_sub_quest("Query deployments", |_quest| async move {
                    api_v1_clients_client_id_deployments_get(&client.config, &client.id, None, None)
                        .await
                })
                .await
                .2
        };
        let deployments = deployments.await?;
        let digest = deployments
            .bundle
            .map(|bundle| {
                bundle
                    .digest
                    .ok_or_else(|| anyhow::anyhow!("No bundle digest present"))
            })
            .transpose()?;
        let bundle_application_deployments = {
            quest
                .lock()
                .await
                .spawn_sub_quest("Get bundle", |quest| {
                    get_bundle(quest, digest, client.clone())
                })
                .await
                .2
        };
        let additional_application_deployments = {
            quest
                .lock()
                .await
                .create_sub_quest("Get additional application deployments", |quest| {
                    get_application_deployments(quest, deployments.deployments, client.clone())
                })
                .await
                .2
        };
        let bundle_application_deployments = bundle_application_deployments.await??;
        let mut application_deployments = bundle_application_deployments;
        application_deployments.extend(additional_application_deployments.await?);
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

async fn get_application_deployments(
    quest: SyncQuest,
    deployment_manifests: Vec<DeploymentManifestRef>,
    client: Client,
) -> anyhow::Result<HashMap<String, ApplicationDeployment>> {
    let mut sub_quests = Vec::new();
    for deployment_manifest in deployment_manifests {
        let sub_quest = quest
            .lock()
            .await
            .spawn_sub_quest(
                format!(
                    "Get application deployment {}",
                    deployment_manifest.deployment_id
                ),
                |_quest| get_application_deployment(deployment_manifest, client.clone()),
            )
            .await
            .2;
        sub_quests.push(sub_quest);
    }
    let application_deployments = join_all(sub_quests).await;
    let total = application_deployments.len();
    let succeeded = application_deployments
        .iter()
        .filter(|result| matches!(result, Ok(Ok(_))))
        .count();
    anyhow::ensure!(
        total == succeeded,
        "Failed to get {} application deployments out of {total}",
        total - succeeded
    );
    Ok(application_deployments
        .into_iter()
        .filter_map(|application_deployment| {
            let application_deployment = application_deployment.ok()?.ok()?;
            Some((
                application_deployment.get_deployment_id(),
                application_deployment,
            ))
        })
        .collect())
}

async fn get_application_deployment(
    deployment_manifest: DeploymentManifestRef,
    client: Client,
) -> anyhow::Result<ApplicationDeployment> {
    let deployment = api_v1_clients_client_id_deployments_deployment_id_digest_get(
        &client.config,
        &client.id,
        &deployment_manifest.deployment_id,
        &deployment_manifest.digest,
        None,
        None,
    )
    .await?;
    let deployment: ApplicationDeployment = serde_norway::from_str(&deployment)?;
    Ok(deployment)
}

async fn get_bundle(
    quest: SyncQuest,
    digest: Option<String>,
    client: Client,
) -> anyhow::Result<HashMap<String, ApplicationDeployment>> {
    // No bundle = no deployments = done
    let Some(digest) = digest else {
        debug!("Received no bundle from {}", client.config.base_path);
        let mut quest = quest.lock().await;
        quest.state = quest::State::Skipped;
        quest.detail = Some("No bundle present".to_string());
        return Ok(HashMap::new());
    };
    let bundle_data =
        api_v1_clients_client_id_bundles_digest_get(&client.config, &client.id, &digest, None)
            .await?;
    let application_deployments: HashMap<String, ApplicationDeployment> =
        relic::async_flecstract::decompress_in_memory(bundle_data.bytes_stream())
            .await?
            .values()
            .map(|bytes| {
                serde_norway::from_slice::<ApplicationDeployment>(bytes).map(
                    |application_deployment| {
                        (
                            application_deployment.get_deployment_id(),
                            application_deployment,
                        )
                    },
                )
            })
            .collect::<Result<HashMap<String, ApplicationDeployment>, _>>()?;
    debug!(
        "Received {} application deployments in bundle {digest}",
        application_deployments.len()
    );
    Ok(application_deployments)
}

fn extract_compose_components_package_locations(
    application_deployment: &ApplicationDeployment,
) -> Option<HashMap<String, ManifestSource>> {
    match &application_deployment.spec.deployment_profile.r#type {
        models::app_deployment_profile::Type::Compose =>
            Some(
                application_deployment.spec.deployment_profile.components
                    .iter()
                    .filter_map(
                        |component| match component {
                            models::AppDeploymentProfileComponentsInner::ComposeApplicationDeploymentProfileComponent(compose) => {
                                match compose.properties.package_location.parse() {
                                    Ok(url) => Some((compose.name.clone(), ManifestSource::Url(url))),
                                    Err(e) => {
                                        warn!("Ignoring invalid package location {} of component {}: {e}", compose.properties.package_location, compose.name);
                                        None
                                    }
                                }
                            }
                            models::AppDeploymentProfileComponentsInner::HelmApplicationDeploymentProfileComponent(helm) => {
                                warn!("Unsupported component inside compose application deployment: {}", helm.name);
                                None
                            }
                        }

                    )
                    .collect()

            ),
        models::app_deployment_profile::Type::HelmV3 => {
            warn!("HelmV3 deployments are unsupported");
            None
        }
    }
}
