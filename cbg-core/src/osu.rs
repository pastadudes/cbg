//! # osu
//! osu stuff is in here\
//! embask in the glorious ~400 lines of code i've written
//!
//! you
use moka::future::Cache;
use reqwest::Client as Reqwest;
use rosu_v2::prelude::*;
use serde::{Deserialize, Serialize};
use std::{num::ParseIntError, sync::Arc};
use thiserror::Error;
use tokio::time::Duration;

#[derive(Debug, Error)]
pub enum Error {
    #[error("osu!api error! {0}")]
    OsuApi(#[from] rosu_v2::error::OsuError),

    #[error("http error! {0}")]
    Http(#[from] reqwest::Error),

    #[error("env var error! {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("parse int error! {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("not found! {0}")]
    NotFound(String),
}

pub type OsuResult<T> = Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuUser {
    pub id: u32,
    pub username: String,
    pub country_code: String,

    pub pp: Option<f32>,
    pub global_rank: Option<u32>,
    pub country_rank: Option<u32>,
    pub accuracy: Option<f32>,
    pub play_count: Option<u32>,
    pub level: Option<f32>,

    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuBeatmap {
    pub id: u32,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub game_mode: String,
    pub version: String,

    pub stars: f32,
    pub bpm: f32,
    pub ar: f32,
    pub cs: f32,
    pub hp: f32,
    pub od: f32,

    pub max_combo: u32,
    pub play_count: u32,
    pub is_scoreable: bool,
    pub hit_objects: u32,

    #[serde(skip)]
    pub background_image: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsuScore {
    pub id: u64,
    pub score: u32,
    pub max_combo: u32,
    pub perfect: bool,
    pub mods: String,
    pub pp: Option<f32>,
    pub rank: String,
    pub accuracy: f32,

    pub user: OsuUser,
    pub beatmap_id: Option<u32>,
    pub beatmap: Option<OsuBeatmap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapScores {
    pub beatmap: OsuBeatmap,
    pub scores: Vec<OsuScore>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UserIdentifier {
    Id(u32),
    Username(String),
}

impl From<&User> for OsuUser {
    fn from(user: &User) -> Self {
        let stats = user.statistics.as_ref();
        Self {
            id: user.user_id,
            username: user.username.clone().into_string(),
            country_code: user.country_code.clone().into_string(),
            pp: stats.map(|s| s.pp),
            global_rank: stats.and_then(|s| s.global_rank),
            country_rank: stats.and_then(|s| s.country_rank),
            accuracy: stats.map(|s| s.accuracy),
            play_count: stats.map(|s| s.playcount),
            level: stats.map(|s| s.level.current as f32),
            avatar_url: user.avatar_url.clone(),
        }
    }
}

impl From<&UserExtended> for OsuUser {
    fn from(user: &UserExtended) -> Self {
        let stats = user.statistics.as_ref();
        Self {
            id: user.user_id,
            username: user.username.clone().into_string(),
            country_code: user.country_code.clone().into_string(),
            pp: stats.map(|s| s.pp),
            global_rank: stats.and_then(|s| s.global_rank),
            country_rank: stats.and_then(|s| s.country_rank),
            accuracy: stats.map(|s| s.accuracy),
            play_count: stats.map(|s| s.playcount),
            level: stats.map(|s| s.level.current as f32),
            avatar_url: user.avatar_url.clone(),
        }
    }
}

impl OsuBeatmap {
    fn from_beatmap(beatmap: &BeatmapExtended) -> Self {
        let (artist, title, creator) = if let Some(set) = &beatmap.mapset {
            (
                set.artist.clone(),
                set.title.clone(),
                set.creator_name.clone().to_string(),
            )
        } else {
            ("???".into(), "???".into(), "???".into())
        };

        Self {
            id: beatmap.map_id,
            artist,
            title,
            creator,
            version: beatmap.version.clone(),
            game_mode: beatmap.mode.to_string(),
            stars: beatmap.stars,
            bpm: beatmap.bpm,
            ar: beatmap.ar,
            cs: beatmap.cs,
            hp: beatmap.hp,
            od: beatmap.od,
            max_combo: beatmap.max_combo.unwrap_or(0),
            play_count: beatmap.playcount,
            is_scoreable: beatmap.is_scoreable,
            hit_objects: beatmap.count_objects(),
            background_image: None,
        }
    }
}

pub struct OsuClient {
    osu: Arc<Osu>,
    http: Reqwest,
    beatmap_cache: Cache<u32, OsuBeatmap>,
    user_cache: Cache<UserIdentifier, OsuUser>,
}

impl OsuClient {
    pub async fn new(client_id: u64, client_secret: String, user_agent: String) -> OsuResult<Self> {
        let osu = Osu::new(client_id, client_secret).await?;
        let http = Reqwest::builder().user_agent(user_agent).build()?;

        Ok(Self {
            osu: Arc::new(osu),
            http,
            beatmap_cache: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(Duration::from_secs(60 * 60))
                .build(),

            user_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(60 * 60))
                .build(),
        })
    }

    pub async fn from_env() -> OsuResult<Self> {
        let client_id = std::env::var("OSU_CLIENT_ID")?.parse()?;
        let secret = std::env::var("OSU_CLIENT_SECRET")?;
        let user_agent = std::env::var("USER_AGENT")?;
        Self::new(client_id, secret, user_agent).await
    }

    pub async fn get_user(&self, id: UserIdentifier) -> OsuResult<OsuUser> {
        if let Some(cached) = self.user_cache.get(&id).await {
            return Ok(cached);
        }

        let user = match &id {
            UserIdentifier::Id(uid) => self.osu.user(*uid).await?,
            UserIdentifier::Username(name) => self.osu.user(name).await?,
        };

        let osu_user: OsuUser = (&user).into();
        self.user_cache.insert(id, osu_user.clone()).await;

        Ok(osu_user)
    }

    pub async fn get_beatmap(&self, beatmap_id: u32) -> OsuResult<OsuBeatmap> {
        if let Some(beatmap) = self.beatmap_cache.get(&beatmap_id).await {
            return Ok(beatmap);
        }

        let beatmap = self.osu.beatmap().map_id(beatmap_id).await?;
        let mut converted = OsuBeatmap::from_beatmap(&beatmap);

        if let Ok(resp) = self
            .http
            .get(format!(
                "https://catboy.best/preview/background/{beatmap_id}"
            ))
            .send()
            .await?
            .error_for_status()
        {
            converted.background_image = Some(resp.bytes().await?.to_vec());
        }

        self.beatmap_cache
            .insert(beatmap_id, converted.clone())
            .await;

        Ok(converted)
    }

    pub async fn get_beatmap_scores(&self, beatmap_id: u32) -> OsuResult<BeatmapScores> {
        let beatmap = self.get_beatmap(beatmap_id).await?;
        let scores = self.osu.beatmap_scores(beatmap_id).await?;

        let mut converted_scores = Vec::new();

        for score in scores.scores {
            let user_id = score.user.as_ref().map(|u| u.user_id);
            let user = match user_id {
                Some(id) => self.get_user(UserIdentifier::Id(id)).await?,
                None => continue, // skip if no user
            };

            converted_scores.push(OsuScore {
                id: score.id,
                score: score.score,
                max_combo: score.max_combo,
                perfect: score.is_perfect_combo,
                mods: score.mods.to_string(),
                pp: score.pp,
                rank: score.grade.to_string(),
                accuracy: score.accuracy,
                user,
                beatmap_id: Some(beatmap.id),
                beatmap: None,
            });
        }

        Ok(BeatmapScores {
            beatmap,
            scores: converted_scores,
        })
    }

    #[cfg_attr(not(feature = "discord"), allow(dead_code))]
    pub async fn get_user_scores(
        &self,
        user: UserIdentifier,
        kind: ScoreType,
        limit: Option<usize>,
    ) -> OsuResult<Vec<OsuScore>> {
        let uid = match &user {
            UserIdentifier::Id(id) => *id,
            UserIdentifier::Username(name) => {
                self.get_user(UserIdentifier::Username(name.clone()))
                    .await?
                    .id
            }
        };

        let scores = match kind {
            ScoreType::Best => self.osu.user_scores(uid).best().await?,
            ScoreType::Recent => self.osu.user_scores(uid).recent().await?,
            ScoreType::Firsts => self.osu.user_scores(uid).firsts().await?,
        };

        let take = limit.unwrap_or(10);
        let mut converted = Vec::new();

        for score in scores.into_iter().take(take) {
            let user = self.get_user(UserIdentifier::Id(uid)).await?;
            converted.push(OsuScore {
                id: score.id,
                score: score.score,
                max_combo: score.max_combo,
                perfect: score.is_perfect_combo,
                mods: score.mods.to_string(),
                pp: score.pp,
                rank: score.grade.to_string(),
                accuracy: score.accuracy,
                user,
                beatmap_id: Some(score.map_id),
                beatmap: None,
            });
        }

        Ok(converted)
    }
}

impl From<u32> for UserIdentifier {
    fn from(id: u32) -> Self {
        Self::Id(id)
    }
}
impl From<&str> for UserIdentifier {
    fn from(name: &str) -> Self {
        Self::Username(name.to_owned())
    }
}
impl From<String> for UserIdentifier {
    fn from(name: String) -> Self {
        Self::Username(name)
    }
}

#[cfg(feature = "discord")]
#[derive(Debug, Clone, poise::ChoiceParameter)]
pub enum ScoreType {
    #[name = "best scores"]
    Best,
    #[name = "recent scores"]
    Recent,
    #[name = "firsts"]
    Firsts,
}

#[cfg(not(feature = "discord"))]
#[derive(Debug, Clone)]
pub enum ScoreType {
    Best,
    Recent,
    Firsts,
}
