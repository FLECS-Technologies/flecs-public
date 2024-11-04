use flecs_core::quest::quest_master::QuestMaster;
use flecs_core::quest::Quest;
use std::time::Duration;
use tokio::time::sleep;

pub mod save_princess {
    use flecs_core::quest::{Progress, Result, State, SyncQuest};
    use std::fmt::{Display, Formatter};
    use tokio::time::{sleep, Duration};

    enum Enemy {
        Rat,
        Boar,
        Wolf,
        Vampire,
        Dragon,
    }

    impl Display for Enemy {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match &self {
                    Enemy::Rat => "Rat",
                    Enemy::Boar => "Boar",
                    Enemy::Wolf => "Wolf",
                    Enemy::Vampire => "Vampire",
                    Enemy::Dragon => "Dragon",
                }
            )
        }
    }

    enum Experience {
        Amateur,
        Novice,
        Adapt,
        Experienced,
        Professional,
        Master,
    }

    impl Display for Experience {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match &self {
                    Experience::Amateur => "Amateur",
                    Experience::Novice => "Novice",
                    Experience::Adapt => "Adapt",
                    Experience::Experienced => "Experienced",
                    Experience::Professional => "Professional",
                    Experience::Master => "Master",
                }
            )
        }
    }

    #[derive(Clone, Debug)]
    pub enum Ruby {
        Raw,
        Cut,
        FinelyCut,
        Polished,
        FinelyPolished,
        Dust,
    }

    #[derive(Debug)]
    enum Sword {
        IronOre(u16),
        Iron(u16),
        Forged,
        Enchanted(Ruby),
    }

    pub enum Kidnapper {
        Hotzenplotz,
        Bowser,
        Strahd,
        TheJoker,
    }

    impl Display for Kidnapper {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match &self {
                    Kidnapper::Hotzenplotz => "Hotzenplotz",
                    Kidnapper::Bowser => "Bowser",
                    Kidnapper::Strahd => "Strahd",
                    Kidnapper::TheJoker => "TheJoker",
                }
            )
        }
    }
    #[derive(Debug)]
    pub enum Princess {
        Peach,
        Fiona,
    }

    pub async fn save_princess(quest: SyncQuest, name: &str, cats: u16) -> Result<Princess> {
        let (.., suspect) = quest
            .lock()
            .await
            .create_sub_quest(
                format!("Investigating kidnapping of {name}"),
                |quest| async move { investigate_kidnapping(quest, cats).await },
            )
            .await;

        let (.., results) = quest
            .lock()
            .await
            .create_sub_quest("Getting strong".to_string(), |quest| async move {
                get_strong(quest).await
            })
            .await;

        let (suspect, results) = futures::join!(suspect, results);

        let suspect = suspect?;
        let (experience, sword) = results?;

        let (.., princess) = quest
            .lock()
            .await
            .create_sub_quest(format!("Fighting {suspect}"), |quest| async move {
                fight_kidnapper(quest, suspect, experience, sword).await
            })
            .await;
        princess.await
    }

    async fn fight_kidnapper(
        quest: SyncQuest,
        kidnapper: Kidnapper,
        experience: Experience,
        sword: Sword,
    ) -> Result<Princess> {
        match kidnapper {
            Kidnapper::Bowser => {}
            x => anyhow::bail!(
                "You are fighting {x}, but the kidnapper is {}",
                Kidnapper::Bowser
            ),
        }

        match experience {
            Experience::Master => {}
            x => anyhow::bail!(
                "Your experience level is {x}, but you need at least {} to fight {}",
                Experience::Master,
                Kidnapper::Bowser
            ),
        }

        match sword {
            Sword::Enchanted(Ruby::FinelyPolished) => {}
            x => anyhow::bail!(
                "Your sword is too weak, you have {x:?}, but you need {:?}",
                Sword::Enchanted(Ruby::FinelyPolished)
            ),
        }

        for i in 0..10 {
            quest.lock().await.detail = Some(if i % 2 == 0 { " 🐢" } else { "🐢 " }.to_string());
            sleep(Duration::from_millis(500)).await;
        }
        quest.lock().await.detail = Some("🎆".to_string());
        quest.lock().await.state = State::Success;
        Ok(Princess::Fiona)
    }

    async fn get_strong(quest: SyncQuest) -> Result<(Experience, Sword)> {
        let (.., sword) = quest
            .lock()
            .await
            .spawn_sub_quest("Craft mighty sword".to_string(), craft_mighty_sword)
            .await;
        let (.., experience) = quest
            .lock()
            .await
            .spawn_sub_quest("Train fighting".to_string(), train_fighting)
            .await;

        match (sword.await, experience.await) {
            (Ok(Ok(sword)), Ok(Ok(experience))) => {
                quest.lock().await.state = State::Success;
                Ok((experience, sword))
            }
            (Ok(Ok(_)), Ok(Err(e))) | (Ok(Err(e)), Ok(_)) => {
                quest.lock().await.fail_with_error(&e);
                Err(e)
            }
            (Err(e), _) | (_, Err(e)) => {
                let error = anyhow::anyhow!(e);
                quest.lock().await.fail_with_error(&error);
                Err(error)
            }
        }
    }

    async fn craft_mighty_sword(quest: SyncQuest) -> Result<Sword> {
        let (.., ruby) = quest
            .lock()
            .await
            .create_sub_quest("Creating ruby".to_string(), create_ruby)
            .await;

        let (.., ore) = quest
            .lock()
            .await
            .create_sub_quest("Gather 100 iron ore".to_string(), |quest| async move {
                gather_iron_ore(quest, 100).await
            })
            .await;

        let (.., iron) = quest
            .lock()
            .await
            .create_sub_quest("Smelt iron ore".to_string(), |quest| async move {
                smelt_iron_ore(quest, ore.await?).await
            })
            .await;

        let (.., sword) = quest
            .lock()
            .await
            .create_sub_quest("Forge sword".to_string(), |quest| async move {
                forge_sword(quest, iron.await?).await
            })
            .await;

        let (.., sword) = quest
            .lock()
            .await
            .create_sub_quest("Enchant sword".to_string(), |quest| async move {
                let (sword, ruby) = futures::join!(sword, ruby);
                enchant_sword(quest, sword?, ruby?).await
            })
            .await;

        sword.await
    }

    async fn create_ruby(quest: SyncQuest) -> Result<Ruby> {
        let (.., raw_ruby) = quest
            .lock()
            .await
            .create_sub_quest("Find ruby".to_string(), find_ruby)
            .await;
        let (.., processed_ruby) = quest
            .lock()
            .await
            .create_sub_quest("Process ruby".to_string(), |quest| async move {
                process_ruby(quest, raw_ruby.await?).await
            })
            .await;
        processed_ruby.await
    }

    async fn find_ruby(_quest: SyncQuest) -> Result<Ruby> {
        sleep(Duration::from_secs(2)).await;
        Ok(Ruby::Raw)
    }

    async fn process_ruby(quest: SyncQuest, ruby: Ruby) -> Result<Ruby> {
        let mut ruby = ruby;
        loop {
            match ruby {
                Ruby::Raw => {
                    quest.lock().await.detail = Some("Cutting raw ruby".to_string());
                    sleep(Duration::from_secs(2)).await;
                    ruby = Ruby::Cut;
                }
                Ruby::Cut => {
                    quest.lock().await.detail = Some("Cutting ruby".to_string());
                    sleep(Duration::from_secs(2)).await;
                    ruby = Ruby::FinelyCut;
                }
                Ruby::FinelyCut => {
                    quest.lock().await.detail = Some("Polishing ruby".to_string());
                    sleep(Duration::from_secs(2)).await;
                    ruby = Ruby::Polished;
                }
                Ruby::Polished => {
                    quest.lock().await.detail = Some("Fine polishing ruby".to_string());
                    sleep(Duration::from_secs(2)).await;
                    ruby = Ruby::FinelyPolished;
                }
                Ruby::FinelyPolished => {
                    quest.lock().await.detail = Some("Ruby Ready".to_string());
                    break;
                }
                Ruby::Dust => {
                    anyhow::bail!("Received only dust")
                }
            }
        }
        Ok(ruby)
    }

    async fn forge_sword(quest: SyncQuest, iron: Sword) -> Result<Sword> {
        let amount = match iron {
            Sword::Iron(amount) => amount,
            x => {
                let e = anyhow::anyhow!("Expected Iron, received {x:?}");
                quest.lock().await.fail_with_error(&e);
                anyhow::bail!(e)
            }
        };

        if amount < 100 {
            anyhow::bail!("Expected at least 100 IronOre, received {amount}");
        }
        sleep(Duration::from_secs(4)).await;
        Ok(Sword::Forged)
    }

    async fn enchant_sword(_quest: SyncQuest, sword: Sword, ruby: Ruby) -> Result<Sword> {
        match (sword, ruby.clone()) {
            (Sword::Forged, Ruby::FinelyPolished) => {}
            x => {
                anyhow::bail!("Expected (Sword::Forged, Ruby::FinelyPolished), received {x:?}")
            }
        };

        sleep(Duration::from_secs(1)).await;
        Ok(Sword::Enchanted(ruby))
    }

    async fn train_fighting(quest: SyncQuest) -> Result<Experience> {
        const RAT_COUNT: u16 = 1000;
        const BOAR_COUNT: u16 = 500;
        const WOLF_COUNT: u16 = 200;
        const VAMPIRE_COUNT: u16 = 50;
        const DRAGON_COUNT: u16 = 25;
        let mut experience = Experience::Amateur;
        let mut quests = Vec::new();
        let (.., rat_quest) = quest
            .lock()
            .await
            .spawn_sub_quest(format!("Fighting {RAT_COUNT} rats"), |quest| async move {
                fight(quest, Enemy::Rat, RAT_COUNT).await
            })
            .await;
        quests.push(rat_quest);
        let (.., boar_quest) = quest
            .lock()
            .await
            .spawn_sub_quest(format!("Fighting {BOAR_COUNT} boars"), |quest| async move {
                fight(quest, Enemy::Boar, BOAR_COUNT).await
            })
            .await;
        quests.push(boar_quest);
        let (.., wolf_quest) = quest
            .lock()
            .await
            .spawn_sub_quest(format!("Fighting {WOLF_COUNT} wolfs"), |quest| async move {
                fight(quest, Enemy::Wolf, WOLF_COUNT).await
            })
            .await;
        quests.push(wolf_quest);
        let (.., vampire_quest) = quest
            .lock()
            .await
            .spawn_sub_quest(
                format!("Fighting {VAMPIRE_COUNT} vampires"),
                |quest| async move { fight(quest, Enemy::Vampire, VAMPIRE_COUNT).await },
            )
            .await;
        quests.push(vampire_quest);
        let (.., dragon_quest) = quest
            .lock()
            .await
            .spawn_sub_quest(
                format!("Fighting {DRAGON_COUNT} dragons"),
                |quest| async move { fight(quest, Enemy::Dragon, DRAGON_COUNT).await },
            )
            .await;
        quests.push(dragon_quest);

        quest.lock().await.detail = Some(experience.to_string());

        for _ in futures::future::join_all(quests).await {
            match experience {
                Experience::Amateur => experience = Experience::Novice,
                Experience::Novice => experience = Experience::Adapt,
                Experience::Adapt => experience = Experience::Experienced,
                Experience::Experienced => experience = Experience::Professional,
                Experience::Professional => experience = Experience::Master,
                Experience::Master => {}
            }
            quest.lock().await.detail = Some(experience.to_string());
        }

        Ok(experience)
    }

    async fn fight(quest: SyncQuest, enemy: Enemy, number: u16) -> Result<()> {
        let difficulty = match enemy {
            Enemy::Rat => 2,
            Enemy::Boar => 10,
            Enemy::Wolf => 100,
            Enemy::Vampire => 200,
            Enemy::Dragon => 400,
        };

        for ore in 1..=number {
            sleep(Duration::from_millis(difficulty)).await;
            quest.lock().await.progress = Some(Progress {
                total: number as u64,
                current: ore as u64,
            })
        }

        Ok(())
    }

    async fn gather_iron_ore(quest: SyncQuest, amount: u16) -> Result<Sword> {
        let mut total_ore = Sword::IronOre(0);
        for ore in 1..=amount {
            if ore > 90 {
                let e = anyhow::anyhow!("Too much ore");
                quest.lock().await.fail_with_error(&e);
                return Err(e);
            }
            sleep(Duration::from_millis(100)).await;
            total_ore = Sword::IronOre(ore);
            quest.lock().await.progress = Some(Progress {
                total: amount as u64,
                current: ore as u64,
            })
        }
        Ok(total_ore)
    }

    async fn smelt_iron_ore(quest: SyncQuest, ore: Sword) -> Result<Sword> {
        let amount = match ore {
            Sword::IronOre(amount) => amount,
            x => {
                anyhow::bail!("Expected IronOre, received {x:?}")
            }
        };
        let mut total_iron = Sword::Iron(0);
        for iron in 1..=amount {
            sleep(Duration::from_millis(50)).await;
            total_iron = Sword::Iron(iron);
            quest.lock().await.progress = Some(Progress {
                total: amount as u64,
                current: iron as u64,
            })
        }
        Ok(total_iron)
    }

    async fn investigate_kidnapping(quest: SyncQuest, cats: u16) -> Result<Kidnapper> {
        sleep(Duration::from_secs(2)).await;
        let (.., examine_results) = quest
            .lock()
            .await
            .create_sub_quest("Examine crime scene".to_string(), examine_crime_scene)
            .await;
        let (tx_examine_results1, rx_examine_results) = tokio::sync::oneshot::channel::<String>();
        let (.., witness_results) = quest
            .lock()
            .await
            .create_sub_quest("Questioning witnesses".to_string(), |quest| async move {
                question_witnesses(quest, rx_examine_results.await?, cats).await
            })
            .await;
        let (tx_examine_results2, rx_examine_results) = tokio::sync::oneshot::channel::<String>();
        let (.., conclusion) = quest
            .lock()
            .await
            .create_sub_quest("Drawing conclusion".to_string(), |quest| async move {
                draw_conclusion(quest, rx_examine_results.await?, witness_results.await?).await
            })
            .await;
        match examine_results.await {
            Err(e) => {
                drop(tx_examine_results1);
                drop(tx_examine_results2);
                let _ = conclusion.await;
                Err(e)
            }
            Ok(examine_results) => {
                let _ = tx_examine_results1.send(examine_results.clone());
                let _ = tx_examine_results2.send(examine_results);
                sleep(Duration::from_secs(2)).await;
                conclusion.await
            }
        }
    }

    async fn draw_conclusion(
        quest: SyncQuest,
        crime_scene_infos: String,
        witness_reports: Vec<String>,
    ) -> Result<Kidnapper> {
        // study crime scene for some time
        quest.lock().await.detail = Some("Studying crime scene infos".to_string());
        let _infos = crime_scene_infos;
        sleep(Duration::from_secs(5)).await;

        // study witness reports for some time
        quest.lock().await.detail = Some("Studying witness reports".to_string());
        let _infos = witness_reports;
        sleep(Duration::from_secs(2)).await;

        quest.lock().await.detail = Some("Thinking".to_string());

        sleep(Duration::from_secs(4)).await;
        quest.lock().await.detail = None;
        Ok(Kidnapper::Bowser)
    }

    async fn examine_crime_scene(quest: SyncQuest) -> Result<String> {
        quest.lock().await.detail = Some("Looking around...".to_string());
        // use some time
        sleep(Duration::from_secs(5)).await;
        quest.lock().await.detail = None;
        Ok("Sketches and notes".to_string())
    }

    async fn question_witnesses(
        quest: SyncQuest,
        crime_scene_infos: String,
        cats: u16,
    ) -> Result<Vec<String>> {
        let mut results = Vec::new();
        let dog_crime_scene_infos = crime_scene_infos.clone();
        let (.., dog_answer) = quest
            .lock()
            .await
            .create_sub_quest("Questioning the dog".to_string(), |quest| async move {
                question_dog(quest, dog_crime_scene_infos).await
            })
            .await;
        results.push(dog_answer);
        for cat in 1..cats {
            let cat_crime_scene_infos = crime_scene_infos.clone();
            let (.., cat_answer) = quest
                .lock()
                .await
                .create_sub_quest(format!("Questioning cat #{cat}"), |quest| async move {
                    question_cat(quest, cat_crime_scene_infos, cat).await
                })
                .await;
            results.push(cat_answer);
        }
        let results = futures::future::join_all(results).await;
        let mut answers = Vec::with_capacity(results.len());
        for result in results {
            answers.push(result?)
        }
        Ok(answers)
    }

    async fn question_dog(_quest: SyncQuest, _crime_scene_infos: String) -> Result<String> {
        // use some time
        sleep(Duration::from_secs(5)).await;
        Ok("Wuff".to_string())
    }

    async fn question_cat(
        _quest: SyncQuest,
        _crime_scene_infos: String,
        number: u16,
    ) -> Result<String> {
        // use some time
        sleep(Duration::from_secs((number % 5) as u64)).await;
        Ok(match number % 3 {
            0 => "Hiss",
            1 => "Scratch",
            _ => "Miou",
        }
        .to_string())
    }
}

#[tokio::main]
async fn main() {
    const PRINCESS: &str = "Fiona";
    let mut master = QuestMaster::default();
    let (_, quest) = master
        .schedule_quest(format!("Saving princes {PRINCESS}"), |quest| async {
            let princess = save_princess::save_princess(quest, PRINCESS, 3).await;

            println!("Result {:?}", princess);
            Ok(())
        })
        .await
        .unwrap();

    while !quest.lock().await.state.is_finished() {
        println!("{}", Quest::fmt(quest.clone()).await);
        sleep(Duration::from_millis(500)).await;
    }
    println!("{}", Quest::fmt(quest.clone()).await);
}
