use flecs_core::quest::Quest;
use std::time::Duration;
use tokio::time::sleep;

pub mod save_princess {
    use super::*;
    use flecs_core::quest::{AwaitResult, Progress, QuestResult, State, SyncAwaitResult};
    use std::fmt::{Display, Formatter};
    use tokio::task::JoinSet;
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

    pub fn save_princess(name: &str, cats: u16) -> QuestResult<Princess> {
        let quest = Quest::new_synced(format!("Saving princess {name}"));
        let closure_quest = quest.clone();
        let name = name.to_string();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            let (investigate_quest, suspect) = investigate_kidnapping(&name, cats);

            quest.lock().await.sub_quests.push(investigate_quest);
            let (get_strong_quest, results) = get_strong();

            quest.lock().await.sub_quests.push(get_strong_quest);
            // TODO: Error handling
            let suspect = suspect.await.unwrap();
            let (experience, sword) = results.await.unwrap();

            let (fight_quest, princess) = fight_kidnapper(suspect, experience, sword);
            quest.lock().await.sub_quests.push(fight_quest);
            let princess = princess.await.unwrap();
            quest.lock().await.state = State::Success;
            princess
        });
        (quest, handle)
    }

    fn fight_kidnapper(
        kidnapper: Kidnapper,
        experience: Experience,
        sword: Sword,
    ) -> QuestResult<Princess> {
        let quest = Quest::new_synced(format!("Fighting {kidnapper}"));
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            match kidnapper {
                Kidnapper::Bowser => {}
                x => panic!(
                    "You are fighting {x}, but the kidnapper is {}",
                    Kidnapper::Bowser
                ),
            }

            match experience {
                Experience::Master => {}
                x => panic!(
                    "Your experience level is {x}, but you need at least {} to fight {}",
                    Experience::Master,
                    Kidnapper::Bowser
                ),
            }

            match sword {
                Sword::Enchanted(Ruby::FinelyPolished) => {}
                x => panic!(
                    "Your sword is too weak, you have {x:?}, but you need {:?}",
                    Sword::Enchanted(Ruby::FinelyPolished)
                ),
            }

            for i in 0..10 {
                quest.lock().await.detail =
                    Some(if i % 2 == 0 { " 🐢" } else { "🐢 " }.to_string());
                sleep(Duration::from_millis(500)).await;
            }
            quest.lock().await.detail = Some("🎆".to_string());
            quest.lock().await.state = State::Success;
            Princess::Fiona
        });
        (quest, handle)
    }

    fn get_strong() -> QuestResult<(Experience, Sword)> {
        let quest = Quest::new_synced("Getting strong".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            let (sword_quest, sword) = craft_mighty_sword();
            let (train_quest, experience) = train_fighting();

            quest.lock().await.sub_quests.push(sword_quest);
            quest.lock().await.sub_quests.push(train_quest);
            // TODO: Error handling
            let sword = sword.await.unwrap();
            let experience = experience.await.unwrap();

            quest.lock().await.state = State::Success;
            (experience, sword)
        });
        (quest, handle)
    }

    fn craft_mighty_sword() -> QuestResult<Sword> {
        let quest = Quest::new_synced("Craft mighty sword".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            let (ruby_quest, ruby) = create_ruby();
            quest.lock().await.sub_quests.push(ruby_quest);
            let ruby = AwaitResult::new_synced_join_handle(ruby);

            let (ore_quest, ore) = gather_iron_ore(100);
            quest.lock().await.sub_quests.push(ore_quest);
            let ore = AwaitResult::new_synced_join_handle(ore);

            let (iron_quest, iron) = smelt_iron_ore(ore);
            quest.lock().await.sub_quests.push(iron_quest);
            let iron = AwaitResult::new_synced_join_handle(iron);

            let (forge_quest, sword) = forge_sword(iron);
            quest.lock().await.sub_quests.push(forge_quest);
            let sword = AwaitResult::new_synced_join_handle(sword);

            let (enchant_quest, sword) = enchant_sword(sword, ruby);
            quest.lock().await.sub_quests.push(enchant_quest);

            let sword = sword.await.unwrap();
            quest.lock().await.state = State::Success;
            sword
        });
        (quest, handle)
    }

    fn create_ruby() -> QuestResult<Ruby> {
        let quest = Quest::new_synced("Create ruby".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            let (find_quest, ruby) = find_ruby();
            quest.lock().await.sub_quests.push(find_quest);
            let ruby = AwaitResult::new_synced_join_handle(ruby);

            let (process_quest, ruby) = process_ruby(ruby);
            quest.lock().await.sub_quests.push(process_quest);

            let ruby = ruby.await.unwrap();
            quest.lock().await.state = State::Success;
            ruby
        });
        (quest, handle)
    }

    fn find_ruby() -> QuestResult<Ruby> {
        let quest = Quest::new_synced("Find ruby in mine".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            sleep(Duration::from_secs(2)).await;

            let ruby = Ruby::Raw;
            quest.lock().await.state = State::Success;
            ruby
        });
        (quest, handle)
    }

    fn process_ruby(ruby: SyncAwaitResult<Ruby>) -> QuestResult<Ruby> {
        let quest = Quest::new_synced("Process ruby".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            let mut ruby = ruby.lock().await.try_access_value().await.unwrap().clone();
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
                        panic!("Received only dust")
                    }
                }
            }

            quest.lock().await.state = State::Success;
            ruby
        });
        (quest, handle)
    }

    fn forge_sword(iron: SyncAwaitResult<Sword>) -> QuestResult<Sword> {
        let quest = Quest::new_synced("Forge sword".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            let amount = match iron.lock().await.try_access_value().await.unwrap() {
                Sword::Iron(amount) => *amount,
                x => {
                    panic!("Expected Iron, received {x:?}");
                }
            };

            if amount < 100 {
                panic!("Expected at least 100 IronOre, received {amount}");
            }
            sleep(Duration::from_secs(4)).await;
            quest.lock().await.state = State::Success;
            Sword::Forged
        });
        (quest, handle)
    }

    fn enchant_sword(
        sword: SyncAwaitResult<Sword>,
        ruby: SyncAwaitResult<Ruby>,
    ) -> QuestResult<Sword> {
        let quest = Quest::new_synced("Forge sword".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            match sword.lock().await.try_access_value().await.unwrap() {
                Sword::Forged => {}
                x => {
                    panic!("Expected forged sword, received {x:?}");
                }
            };

            let ruby = ruby.lock().await.try_access_value().await.unwrap().clone();

            sleep(Duration::from_secs(1)).await;
            quest.lock().await.state = State::Success;
            Sword::Enchanted(ruby)
        });
        (quest, handle)
    }

    fn train_fighting() -> QuestResult<Experience> {
        let quest = Quest::new_synced("Train fighting".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            let mut experience = Experience::Amateur;
            let mut results = JoinSet::new();

            let (rat_quest, handle) = fight(Enemy::Rat, 1000);
            quest.lock().await.sub_quests.push(rat_quest);
            results.spawn(handle);

            let (boar_quest, handle) = fight(Enemy::Boar, 500);
            quest.lock().await.sub_quests.push(boar_quest);
            results.spawn(handle);

            let (wolf_quest, handle) = fight(Enemy::Wolf, 200);
            quest.lock().await.sub_quests.push(wolf_quest);
            results.spawn(handle);

            let (vampire_quest, handle) = fight(Enemy::Vampire, 50);
            quest.lock().await.sub_quests.push(vampire_quest);
            results.spawn(handle);

            let (dragon_quest, handle) = fight(Enemy::Dragon, 25);
            quest.lock().await.sub_quests.push(dragon_quest);
            results.spawn(handle);

            quest.lock().await.detail = Some(experience.to_string());

            while let Some(Ok(_)) = results.join_next().await {
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

            quest.lock().await.state = State::Success;
            experience
        });
        (quest, handle)
    }

    fn fight(enemy: Enemy, number: u16) -> QuestResult<()> {
        let quest = Quest::new_synced(format!("Fighting {enemy}"));
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

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

            quest.lock().await.state = State::Success;
        });
        (quest, handle)
    }

    fn gather_iron_ore(amount: u16) -> QuestResult<Sword> {
        let quest = Quest::new_synced(format!("Gather {amount} iron ore"));
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;

            let mut total_ore = Sword::IronOre(0);
            for ore in 1..=amount {
                sleep(Duration::from_millis(100)).await;
                total_ore = Sword::IronOre(ore);
                quest.lock().await.progress = Some(Progress {
                    total: amount as u64,
                    current: ore as u64,
                })
            }
            quest.lock().await.state = State::Success;
            total_ore
        });
        (quest, handle)
    }

    fn smelt_iron_ore(ore: SyncAwaitResult<Sword>) -> QuestResult<Sword> {
        let quest = Quest::new_synced("Smelt iron ore".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            let amount = match ore.lock().await.try_access_value().await.unwrap() {
                Sword::IronOre(amount) => *amount,
                x => panic!("Expected IronOre, received {x:?}"),
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
            quest.lock().await.state = State::Success;
            total_iron
        });
        (quest, handle)
    }

    fn investigate_kidnapping(name: &str, cats: u16) -> QuestResult<Kidnapper> {
        let quest = Quest::new_synced(format!("Investigate kidnapping of princess {name}"));
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;

            sleep(Duration::from_secs(2)).await;
            quest.lock().await.state = State::Ongoing;
            let (examine_quest, results) = examine_crime_scene();
            quest.lock().await.sub_quests.push(examine_quest);

            let results = AwaitResult::new_synced_join_handle(results);
            let (question_quest, witness_results) = question_witnesses(results.clone(), cats);
            let witness_results = AwaitResult::new_synced_join_handle(witness_results);
            quest.lock().await.sub_quests.push(question_quest);
            let (draw_conclusion_quest, suspect) = draw_conclusion(results, witness_results);
            quest.lock().await.sub_quests.push(draw_conclusion_quest);
            // // TODO: Error handling
            sleep(Duration::from_secs(2)).await;
            let kidnapper = suspect.await.unwrap();
            quest.lock().await.state = State::Success;
            kidnapper
        });
        (quest, handle)
    }

    fn draw_conclusion(
        crime_scene_infos: SyncAwaitResult<String>,
        witness_reports: SyncAwaitResult<Vec<String>>,
    ) -> QuestResult<Kidnapper> {
        let quest = Quest::new_synced("Draw conclusion".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            // study crime scene for some time
            quest.lock().await.detail = Some("Studying crime scene infos".to_string());
            let _infos = crime_scene_infos
                .lock()
                .await
                .try_access_value()
                .await
                .unwrap();
            sleep(Duration::from_secs(5)).await;

            // study witness reports for some time
            quest.lock().await.detail = Some("Studying witness reports".to_string());
            let _infos = witness_reports
                .lock()
                .await
                .try_access_value()
                .await
                .unwrap();
            sleep(Duration::from_secs(2)).await;

            quest.lock().await.detail = Some("Thinking".to_string());

            sleep(Duration::from_secs(4)).await;
            quest.lock().await.detail = None;
            quest.lock().await.state = State::Success;
            Kidnapper::Bowser
        });
        (quest, handle)
    }

    fn examine_crime_scene() -> QuestResult<String> {
        let quest = Quest::new_synced("Examine crime scene".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            // use some time
            sleep(Duration::from_secs(5)).await;
            quest.lock().await.state = State::Success;
            "Sketches and notes".to_string()
        });
        (quest, handle)
    }

    fn question_witnesses(
        crime_scene_infos: SyncAwaitResult<String>,
        cats: u16,
    ) -> QuestResult<Vec<String>> {
        let quest = Quest::new_synced("Question Witnesses".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;

            quest.lock().await.state = State::Ongoing;
            let mut results = JoinSet::new();
            let (dog_quest, answer) = question_dog(crime_scene_infos.clone());
            results.spawn(answer);
            quest.lock().await.sub_quests.push(dog_quest);
            for cat in 1..cats {
                let (cat_quest, answer) = question_cat(crime_scene_infos.clone(), cat);
                results.spawn(answer);
                quest.lock().await.sub_quests.push(cat_quest);
            }
            let mut answers = Vec::new();
            while let Some(answer) = results.join_next().await {
                // TODO: Error handling
                answers.push(answer.unwrap().unwrap())
            }
            quest.lock().await.state = State::Success;
            answers
        });
        (quest, handle)
    }

    fn question_dog(crime_scene_infos: SyncAwaitResult<String>) -> QuestResult<String> {
        let quest = Quest::new_synced("Question dog".to_string());
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            // TODO: ErrorHandling
            let _ = crime_scene_infos.lock().await.try_access_value().await;
            // use some time
            sleep(Duration::from_secs(5)).await;
            quest.lock().await.state = State::Success;
            "Wuff".to_string()
        });
        (quest, handle)
    }

    fn question_cat(
        crime_scene_infos: SyncAwaitResult<String>,
        number: u16,
    ) -> QuestResult<String> {
        let quest = Quest::new_synced(format!("Question cat #{number}"));
        let closure_quest = quest.clone();
        let handle = tokio::spawn(async move {
            let quest = closure_quest;
            quest.lock().await.state = State::Ongoing;
            // TODO: ErrorHandling
            let _ = crime_scene_infos.lock().await.try_access_value().await;
            // use some time
            sleep(Duration::from_secs((number % 5) as u64)).await;
            quest.lock().await.state = State::Success;
            match number % 3 {
                0 => "Hiss",
                1 => "Scratch",
                _ => "Miou",
            }
            .to_string()
        });
        (quest, handle)
    }
}

#[tokio::main]
async fn main() {
    let result = save_princess::save_princess("Fiona", 3);
    while !result.0.lock().await.state.is_finished() {
        println!("{}", Quest::fmt(result.0.clone()).await);
        sleep(Duration::from_millis(500)).await;
    }
    println!("{}", Quest::fmt(result.0.clone()).await);
    println!("Result {:?}", result.1.await);
}
