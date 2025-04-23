mod cellar;
pub mod enchantment;
mod flecs_rest;
pub mod forge;
pub mod fsm;
pub mod jeweler;
pub mod legacy;
pub mod lore;
/// [quest::Quest]s are our abstraction for long-running processes that need to be tracked and can
/// depend on each other. They can be scheduled via [quest::quest_master::QuestMaster].
/// [quest::Quest]s in general can return any type wrapped in a [Result], but
/// [quest::quest_master::QuestMaster] accepts only [quest::Quest]s that return [Result]<()>.
/// # Examples
/// ## Scheduling Quests
/// If we already have a function that takes a [quest::SyncQuest] and returns a [Result]<()>
/// scheduling it becomes quite easy, we just have to provide a name and the function.
/// ```
/// use flecs_core::enchantment::quest_master::QuestMaster;
/// use flecs_core::quest::Result;
/// use flecs_core::quest::SyncQuest;
///
/// async fn print_hello(quest: SyncQuest) -> Result<()> {
///     println!("Hello");
///     Ok(())
/// }
/// # tokio_test::block_on(
/// async {
///     let mut quest_master = QuestMaster::default();
///     let (_id, _quest) = quest_master
///         .lock()
///         .await
///         .schedule_quest("Print hello".to_string(), print_hello)
///         .await
///         .unwrap();
/// }
/// # )
/// ```
/// If the signature of the function we want to call does not fit we need to wrap it in a closure
/// that takes a [quest::SyncQuest] and returns [Result]<()>.
/// ```
/// use flecs_core::enchantment::quest_master::QuestMaster;
/// use flecs_core::quest::SyncQuest;
///
/// async fn print_2n(quest: SyncQuest, n: u64) -> u64 {
///     let result = 2 * n;
///     println!("{result}");
///     result
/// }
/// # tokio_test::block_on(
/// async {
///     let mut quest_master = QuestMaster::default();
///     let n = 50;
///     let (_id, _quest) = quest_master
///         .lock()
///         .await
///         .schedule_quest(format!("Print 2 * {n}"), move |quest| async move {
///             let _n = print_2n(quest, n).await;
///             Ok(())
///         })
///         .await
///         .unwrap();
/// }
/// # )
/// ```
/// It is also possible to directly schedule a closure without any function call.
/// ```
/// use flecs_core::enchantment::quest_master::QuestMaster;
///
/// # tokio_test::block_on(
/// async {
///     let mut quest_master = QuestMaster::default();
///     let n = 50;
///     let (_id, _quest) = quest_master
///         .lock()
///         .await
///         .schedule_quest(format!("Print 3 * {n}"), move |quest| async move {
///             println!("{}", 3 * n);
///             Ok(())
///         })
///         .await
///         .unwrap();
/// }
/// # )
/// ```
/// ## Quest with updating Progress
/// ```
/// use flecs_core::quest::SyncQuest;
/// use flecs_core::quest::{Progress, Result};
///
/// async fn print_to_10(quest: SyncQuest) -> Result<(), String> {
///     const N: u64 = 10;
///     quest.lock().await.progress = Some(Progress {
///         current: 0,
///         total: Some(N),
///     });
///     for i in 0..=N {
///         println!("{i}");
///         quest.lock().await.progress.as_mut().unwrap().current = i;
///     }
///     Ok(())
/// }
/// ```
/// ## Quest with updating State
/// ```
/// use anyhow::anyhow;
/// use flecs_core::quest::Result;
/// use flecs_core::quest::{State, SyncQuest};
///
/// async fn failing(quest: SyncQuest) -> Result<(), anyhow::Error> {
///     let result: Result<()> = Err(anyhow!("Some error"));
///     match result {
///         Ok(()) => Ok(()),
///         Err(e) => {
///             quest.lock().await.state = State::Failing;
///             // Do some cleanup
///             Err(e)
///         }
///     }
/// }
/// ```
/// ## Subquests
/// ### Independent subquests
/// ```
/// use anyhow::anyhow;
/// use flecs_core::quest::SyncQuest;
/// use flecs_core::quest::{Quest, Result};
/// use futures::join;
///
/// async fn f1(quest: SyncQuest) -> Result<u64, String> {
///     Ok(20)
/// }
///
/// async fn f2(quest: SyncQuest) -> Result<u64, String> {
///     Ok(27)
/// }
///
/// async fn compound_f(quest: SyncQuest) -> Result<u64, String> {
///     let (.., result1) = quest
///         .lock()
///         .await
///         .create_sub_quest("f1".to_string(), f1)
///         .await;
///     let (.., result2) = quest
///         .lock()
///         .await
///         .create_sub_quest("f2".to_string(), f2)
///         .await;
///     // if the order the subquests are executed in is relevant, do not use join.
///     // They have to be awaited in order instead, but be sure to await them all
///     // - especially if errors occur
///     let (result1, result2) = join!(result1, result2);
///     Ok(result1? + result2?)
/// }
/// # tokio_test::block_on(
/// async {
///     let quest = Quest::new_synced("Compound quest".to_string());
///     assert_eq!(compound_f(quest).await.unwrap(), 47)
/// }
/// # )
/// ```
/// ### Independent concurrent subquests
/// Compute heavy subquests can be spawned which means that they can progress concurrently.
/// See [tokio::spawn] for details.
/// ```
/// use flecs_core::quest::SyncQuest;
/// use flecs_core::quest::{Quest, Result};
/// use futures::join;
///
/// async fn f1(quest: SyncQuest) -> Result<u64, anyhow::Error> {
///     let mut n = 0;
///     for i in 1..=1000 {
///         n += i;
///     }
///     Ok(n)
/// }
///
/// async fn f2(quest: SyncQuest) -> Result<u64, anyhow::Error> {
///     let mut n = 0;
///     for i in 1..=200 {
///         n += i;
///     }
///     Ok(n)
/// }
///
/// async fn compound_f(quest: SyncQuest) -> Result<u64, anyhow::Error> {
///     let (.., result1) = quest
///         .lock()
///         .await
///         .spawn_sub_quest("f1".to_string(), f1)
///         .await;
///     let (.., result2) = quest
///         .lock()
///         .await
///         .spawn_sub_quest("f2".to_string(), f2)
///         .await;
///     // if the order the subquests are executed in is relevant, do not use join.
///     // They have to be awaited in order instead, but be sure to await them all
///     // - especially if errors occur
///     let (result1, result2) = join!(result1, result2);
///     Ok(result1?? + result2??)
/// }
/// # tokio_test::block_on(
/// async {
///     let quest = Quest::new_synced("Compute heavy compound quest".to_string());
///     assert_eq!(compound_f(quest).await.unwrap(), 520600)
/// }
/// # )
/// ```
/// ### Dependent subquests
/// #### Single subquest depends on one subquest
/// ```
/// use flecs_core::quest::SyncQuest;
/// use flecs_core::quest::{Quest, Result};
///
/// async fn independent_f(quest: SyncQuest) -> Result<u64, String> {
///     let mut n = 12345;
///     Ok(n % 100)
/// }
///
/// async fn dependent_f(quest: SyncQuest, n: u64) -> Result<u64, String> {
///     Ok(n % 10)
/// }
///
/// async fn compound_f(quest: SyncQuest) -> Result<u64, String> {
///     let (.., result1) = quest
///         .lock()
///         .await
///         .create_sub_quest("Independent quest".to_string(), independent_f)
///         .await;
///     let (.., result2) = quest
///         .lock()
///         .await
///         .create_sub_quest("Dependent quest".to_string(), |quest| async move {
///             dependent_f(quest, result1.await?).await
///         })
///         .await;
///     Ok(result2.await?)
/// }
/// # tokio_test::block_on(
/// async {
///     let quest = Quest::new_synced("Compound quest".to_string());
///     assert_eq!(compound_f(quest).await.unwrap(), 5)
/// }
/// # )
/// ```
/// #### Single subquest depends on multiple subquests
/// ```
/// use flecs_core::quest::SyncQuest;
/// use flecs_core::quest::{Quest, Result};
/// use futures::join;
///
/// async fn independent_f1(quest: SyncQuest) -> Result<u64, String> {
///     let mut n = 12345;
///     Ok(n % 100)
/// }
///
/// async fn independent_f2(quest: SyncQuest) -> Result<u64, String> {
///     let mut n = 12345;
///     Ok(n % 25)
/// }
///
/// async fn independent_f3(quest: SyncQuest) -> Result<u64, String> {
///     let mut n = 12345;
///     Ok(n % 2)
/// }
///
/// async fn dependent_f(quest: SyncQuest, a: u64, b: u64, c: u64) -> Result<u64, String> {
///     Ok((a + b + c) % 10)
/// }
///
/// async fn compound_f(quest: SyncQuest) -> Result<u64, String> {
///     let (.., result1) = quest
///         .lock()
///         .await
///         .create_sub_quest("Independent quest #1".to_string(), independent_f1)
///         .await;
///     let (.., result2) = quest
///         .lock()
///         .await
///         .create_sub_quest("Independent quest #2".to_string(), independent_f2)
///         .await;
///     let (.., result3) = quest
///         .lock()
///         .await
///         .create_sub_quest("Independent quest #3".to_string(), independent_f3)
///         .await;
///     let (.., result) = quest
///         .lock()
///         .await
///         .create_sub_quest("Dependent quest".to_string(), |quest| async move {
///             let (a, b, c) = join!(result1, result2, result3);
///             dependent_f(quest, a?, b?, c?).await
///         })
///         .await;
///     Ok(result.await?)
/// }
/// # tokio_test::block_on(
/// async {
///     let quest = Quest::new_synced("Compound quest".to_string());
///     assert_eq!(compound_f(quest).await.unwrap(), 6)
/// }
/// # )
/// ```
/// #### Multiple subquests depend on single subquest
/// In this example [tokio::sync::broadcast] is used for sending the value to all dependent quests,
/// but other channels or synchronization methods can be used as well.
/// ```
/// use flecs_core::quest::SyncQuest;
/// use flecs_core::quest::{Quest, Result};
/// use futures::join;
/// use tokio::sync::broadcast;
///
/// async fn dependent_f1(quest: SyncQuest, s: &str) -> Result<(), anyhow::Error> {
///     println!("Hello {s}");
///     Ok(())
/// }
///
/// async fn dependent_f2(quest: SyncQuest, s: &str) -> Result<(), anyhow::Error> {
///     println!("Guten Tag {s}");
///     Ok(())
/// }
///
/// async fn dependent_f3(quest: SyncQuest, s: &str) -> Result<(), anyhow::Error> {
///     println!("こんにちは{s}さん");
///     Ok(())
/// }
///
/// async fn independent_f(quest: SyncQuest) -> Result<String, anyhow::Error> {
///     Ok("Peter".to_string())
/// }
///
/// async fn compound_f(quest: SyncQuest) -> Result<(), anyhow::Error> {
///     let (send, mut sub1) = broadcast::channel(1);
///     let (mut sub2, mut sub3) = (send.subscribe(), send.subscribe());
///     let (.., result) = quest
///         .lock()
///         .await
///         .create_sub_quest("Independent quest".to_string(), |quest| async move {
///             send.send(independent_f(quest).await?)?;
///             Ok::<_, anyhow::Error>(())
///         })
///         .await;
///     let (.., result1) = quest
///         .lock()
///         .await
///         .create_sub_quest("Dependent quest #1".to_string(), |quest| async move {
///             dependent_f1(quest, &sub1.recv().await?).await
///         })
///         .await;
///     let (.., result2) = quest
///         .lock()
///         .await
///         .create_sub_quest("Dependent quest #2".to_string(), |quest| async move {
///             dependent_f2(quest, &sub2.recv().await?).await
///         })
///         .await;
///     let (.., result3) = quest
///         .lock()
///         .await
///         .create_sub_quest("Dependent quest #3".to_string(), |quest| async move {
///             dependent_f3(quest, &sub3.recv().await?).await
///         })
///         .await;
///     let (result, result1, result2, result3) = join!(result, result1, result2, result3);
///     let _ = (result?, result1?, result2?, result3?);
///     Ok(())
/// }
/// # tokio_test::block_on(
/// async {
///     let quest = Quest::new_synced("Compound quest".to_string());
///     compound_f(quest).await.unwrap();
/// }
/// # )
/// ```
/// ### Subsubquests
/// Subquests can have subquests themselves
/// ```
/// use flecs_core::quest::{Quest, Result};
/// use flecs_core::quest::SyncQuest;
///
/// async fn subsubsubquest(quest: SyncQuest) -> Result<u64, String> {
///     Ok(20)
/// }
///
/// async fn subsubquest(quest: SyncQuest) -> Result<u64, String> {
///     let result = quest
///         .lock()
///         .await
///         .create_sub_quest("SubSubSubQuest".to_string(), subsubsubquest)
///         .await.2;
///     let result = result.await?;
///     Ok(2* result)
/// }
///
/// async fn subquest(quest: SyncQuest) -> Result<u64, String> {
///     let result = quest
///         .lock()
///         .await
///         .create_sub_quest("SubSubQuest".to_string(), subsubquest)
///         .await.2;
///     let result = result.await?;
///     Ok(10 + result)
/// }
///
/// async fn quest(quest: SyncQuest) -> Result<u64, String> {
///     let result = quest.lock().await.create_sub_quest("SubQuest".to_string(), subquest).await.2;
///     let result = result.await?;
///     Ok(2000 - result)
/// }
/// # tokio_test::block_on(
/// async {
///     let q = Quest::new_synced("Compound quest".to_string());
///     assert_eq!(1950, quest(q).await.unwrap());
/// }
/// # )
pub mod quest;
pub mod relic;
pub mod sorcerer;
pub mod vault;
pub use anyhow::Error;
pub use anyhow::Result;
// TODO: Unify structs (App, Instance, Deployment, ...) with structs from Pouches and move them there

#[cfg(test)]
mod tests {
    use flecs_console_client::apis::configuration::Configuration;
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::sync::{Arc, Mutex};

    pub async fn create_test_server_and_config() -> (mockito::ServerGuard, Arc<Configuration>) {
        let server = mockito::Server::new_async().await;
        let config = Arc::new(Configuration {
            base_path: server.url(),
            ..Configuration::default()
        });
        (server, config)
    }
    const TEST_PATH: &str = "/tmp/flecs-tests/";

    fn validate_test_path_unique(test_path: PathBuf) {
        static TEST_PATHS: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();

        assert!(
            TEST_PATHS
                .get_or_init(Default::default)
                .lock()
                .unwrap()
                .insert(test_path.clone()),
            "Test path is not unique: {:?}",
            test_path
        )
    }

    pub fn prepare_test_path(module_path: &str, test_name: &str) -> PathBuf {
        let module_path = module_path.replace(':', "_");
        let path = Path::new(TEST_PATH).join(module_path).join(test_name);
        // Using the same directory in multiple tests creates unpredictable test outcomes (i.e. some tests fail sometimes)
        validate_test_path_unique(path.clone());
        println!("Preparing {:?}", path);
        let _ = fs::remove_dir_all(&path);
        assert!(!path.try_exists().unwrap());
        fs::create_dir_all(&path).unwrap();
        assert!(path.try_exists().unwrap());
        path
    }
}
