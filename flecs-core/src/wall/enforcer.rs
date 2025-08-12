use crate::forge::iter::TryAnyExtension;
use crate::lore::AuthLoreRef;
use casbin::CoreApi;
use slog::KV;
use std::collections::{BTreeMap, HashSet};
use tracing::{debug, error, trace, warn};

struct TracingSerializer<'a>(&'a mut BTreeMap<String, String>);

pub struct Enforcer {
    enforcer: tokio::sync::RwLock<casbin::Enforcer>,
}

impl slog::Serializer for TracingSerializer<'_> {
    fn emit_arguments(&mut self, key: slog::Key, value: &std::fmt::Arguments<'_>) -> slog::Result {
        self.0.insert(key.to_string(), format!("{}", value));
        Ok(())
    }
}
pub struct SlogToTracing;

impl slog::Drain for SlogToTracing {
    type Ok = ();
    type Err = slog::Never;

    fn log(
        &self,
        record: &slog::Record<'_>,
        values: &slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        let msg = format!("{}", record.msg());
        let mut fields = BTreeMap::new();
        let mut serializer = TracingSerializer(&mut fields);

        // Serialize fields from both record and its values
        record.kv().serialize(record, &mut serializer).unwrap();
        values.serialize(record, &mut serializer).unwrap();

        match record.level() {
            slog::Level::Critical | slog::Level::Error => error!(%msg, ?fields),
            slog::Level::Warning => warn!(%msg, ?fields),
            slog::Level::Debug | slog::Level::Info => debug!(%msg, ?fields),
            slog::Level::Trace => trace!(%msg, ?fields),
        }

        Ok(())
    }
}

impl Enforcer {
    pub async fn verify_roles(
        &self,
        path: &str,
        roles: &HashSet<String>,
        method: &http::method::Method,
    ) -> casbin::Result<bool> {
        let method = method.as_str();
        let e = self.enforcer.read().await;
        std::iter::once("*")
            .chain(roles.iter().map(|r| r.as_str()))
            .try_any(|role| e.enforce((role, path, method)))
    }

    pub async fn new_with_lore(lore: AuthLoreRef) -> casbin::Result<Self> {
        let lore = lore.as_ref().as_ref();
        let casbin_model = casbin::DefaultModel::from_file(lore.casbin_model_path.clone()).await?;
        let casbin_policy = casbin::FileAdapter::new(lore.casbin_policy_path.clone());
        let mut enforcer = casbin::Enforcer::new(casbin_model, casbin_policy).await?;
        let drain = slog::Fuse::new(SlogToTracing);
        let log = slog::Logger::root(drain, slog::o!());
        enforcer.set_logger(Box::new(log));
        enforcer.enable_log(true);
        Ok(Self {
            enforcer: tokio::sync::RwLock::new(enforcer),
        })
    }
}
