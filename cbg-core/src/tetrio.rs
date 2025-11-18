use crate::Error;
use tetrio_api::models::users::summaries::tetra_league::LeagueSummary;
use tetrio_api::models::users::user_rank::UserRank;
use tetrio_api::models::users::user_role::UserRole;
use tetrio_api::{
    http::clients::reqwest_client::InMemoryReqwestClient,
    models::{packet::Packet, users::user_info::UserInfo},
};
use tokio::time::Instant;

/// Abstraction over `tetrio_api` used for fetching user info.
#[derive(Debug)]
pub struct TetrioUser {
    /// nah im not explaining ts
    pub username: String,
    /// tetrio user id
    pub id: String,
    /// tetrio user xp\
    /// it's NOT truncated so it will be very long
    pub xp: f64,
    /// user role (`String` is temp until i stop being lazy and make an enum)
    pub role: UserRole,
    /// league data
    pub league: Option<LeagueSummary>,
    /// tetrio rank (example: X+)
    pub rank: Option<UserRank>,
    /// avatar revision (useful for getting someones avatar)
    /// consider using the avatar field instead...
    pub avatar_revision: Option<i64>,
    /// user pfp
    pub avatar: Option<Vec<u8>>,
}

impl TetrioUser {
    /// fetches user data\
    /// Examples:
    /// ```
    ///  let embed = TetrioUser::fetch(&username).await?.to_embed(); // turns data into a discord embed
    /// ```
    pub async fn fetch(username: &str) -> Result<Self, Error> {
        let client = InMemoryReqwestClient::default();

        // fetch user info
        let user_packet: Packet<UserInfo> = client
            .fetch_user_info(username)
            .await
            .map_err(|e| format!("failed to fetch user info: {e}"))?;

        let mut user = Self::from_packet(user_packet)?;

        // try to fetch league data if not already included
        if user.league.is_none() {
            user.league = Self::fetch_league(&user.id).await.ok();
        }

        user.rank = user.league.as_ref().and_then(|league| league.rank.clone());
        user.avatar = Some(user.get_avatar().await?);

        Ok(user)
    }

    async fn fetch_league(user_id: &str) -> Result<LeagueSummary, Error> {
        let client = InMemoryReqwestClient::default();
        let packet: Packet<LeagueSummary> = client
            .fetch_user_league_summaries(user_id)
            .await
            .map_err(|e| format!("failed to fetch league data!: {e}"))?;

        match packet {
            Packet {
                data: Some(data), ..
            } => Ok(data),
            Packet { error, .. } => {
                if let Some(err) = error {
                    // Convert tetrio_api error to string
                    Err(format!("API error!: {err:?}").into())
                } else {
                    Err("unknown error fetching league data!".into())
                }
            }
        }
    }

    fn from_packet(packet: Packet<UserInfo>) -> Result<Self, Error> {
        match packet {
            Packet {
                data: Some(data), ..
            } => Ok(Self {
                username: data.username,
                id: data.id,
                xp: data.xp,
                role: data.role,
                league: None, // we'll fetch this separately
                rank: None,   // unfinished?
                avatar_revision: data.avatar_revision,
                avatar: None,
            }),
            Packet { error, .. } => {
                if let Some(err) = error {
                    // Convert tetrio_api error to string (so it compiles)
                    Err(format!("API error!: {err:?}").into())
                } else {
                    Err("unknown error from tetrio API!".into())
                }
            }
        }
    }

    /// tetrio's special cursed formula for calculating levels...\
    /// ![the tetrio formula equation](https://latex2image-output.s3.amazonaws.com/img-Gu74frYXEMDj.png)
    #[must_use] 
    pub fn level(&self) -> f64 {
        (self.xp / 500.0).powf(0.6)
            + (self.xp / (5000.0 + f64::max(0.0, self.xp - 4000000.0) / 5000.0))
            + 1.0
    }

    /// turns data into a discord embed
    /// only if discord feature is enabled
    /// Examples:
    /// ```
    /// #[poise::command(prefix_command, slash_command, rename = "user")]
    /// async fn tetrio_user(ctx: Context<'_>, username: String) -> Result<(), Error> {
    /// // formatting is fucked ik
    ///        let embed = TetrioUser::fetch(&username).await?.to_embed();
    ///        ctx.send(poise::CreateReply::default().embed(embed).reply(true))
    ///            .await?;
    ///          Ok(())
    /// }
    /// ```
    #[cfg(feature = "discord")]
    pub async fn to_embed(&self) -> serenity::all::CreateEmbed {
        use crate::AverageColor;

        let mut embed = serenity::all::CreateEmbed::new()
            .author(
                serenity::all::CreateEmbedAuthor::new(&self.username)
                    .icon_url(self.get_avatar_url()),
            )
            .color(
                AverageColor::from_image_url(self.get_avatar_url().as_str())
                    .await
                    .unwrap_or(AverageColor::new(0, 0, 0))
                    .to_embed_color(),
            )
            .field("id", &self.id, false)
            .field("xp", format!("{} XP", self.xp), false)
            .field("level", self.level().to_string(), true)
            // .field("role", &self.role.into(), false)
            .field("rank:", self.rank_label(), false);

        if let Some(league) = &self.league {
            // handle TR (Tetra Rating)
            let tr_display = match league.tr {
                Some(tr_value) => {
                    if tr_value >= 0.0 {
                        format!("{tr_value:.2}")
                    } else {
                        "unranked".to_string()
                    }
                }
                None => "no data???".to_string(),
            };
            embed = embed.field("tr", tr_display, true);

            // handle GXE (basically how likely it is for someone to beat an average person)
            let gxe_display = match league.gxe {
                Some(gxe_value) => {
                    if gxe_value >= 0.0 {
                        format!("{gxe_value:.1}%")
                    } else {
                        "???".to_string()
                    }
                }
                None => "no data".to_string(),
            };
            embed = embed.field("GXE", gxe_display, true);

            // self explanatory
            if let Some(apm_value) = league.apm {
                embed = embed.field("apm", format!("{apm_value:.1}"), true);
            }

            // you too bro
            if let Some(pps_value) = league.pps {
                embed = embed.field("pps", format!("{pps_value:.2}"), true);
            }
        }

        embed
    }

    // unfinished
    /// may work
    #[must_use] 
    pub fn rank_label(&self) -> &'static str {
        match self.rank {
            Some(UserRank::XPlus) => "X+",
            Some(UserRank::X) => "X",
            Some(UserRank::U) => "U",
            Some(UserRank::SS) => "SS",
            Some(UserRank::SPlus) => "S+",
            Some(UserRank::S) => "S",
            Some(UserRank::SMinus) => "S-",
            Some(UserRank::APlus) => "A+",
            Some(UserRank::A) => "A",
            Some(UserRank::AMinus) => "A-",
            Some(UserRank::BPlus) => "B+",
            Some(UserRank::B) => "B",
            Some(UserRank::BMinus) => "B-",
            Some(UserRank::CPlus) => "C+",
            Some(UserRank::C) => "C",
            Some(UserRank::CMinus) => "C-",
            Some(UserRank::DPlus) => "D+",
            Some(UserRank::D) => "D",
            Some(UserRank::Z) => "Unranked",
            Some(UserRank::Unknown(_)) => "???",
            None => "???",
        }
    }

    /// get raw image data of a user's avatar
    /// NOT TESTED! USE AT YOUR OWN RISK!
    pub async fn get_avatar(&self) -> Result<Vec<u8>, Error> {
        let client = reqwest::ClientBuilder::new()
            .user_agent(std::env::var("USER_AGENT")?)
            .build()?;
        Ok(client
            .get(self.get_avatar_url())
            .send()
            .await?
            .bytes()
            .await?
            .to_vec())
    }

    /// says it in the namo bro
    #[must_use] 
    pub fn get_avatar_url(&self) -> String {
        format!(
            "https://tetr.io/user-content/avatars/{}.jpg?rv={}",
            self.id,
            self.avatar_revision.unwrap_or(1) // too risky to panic if avatar revision doesn't exist
        )
    }
}

// pub struct TetrioLeaderboard {}

/// use this struct for fetching server activity\
/// you can also use it to make a chart (KINDA similar to the one on ch.tetr.io)
/// Examples:
/// ```
/// // sorry but i was too lazy so i just used my bot, however its pretty easy to deduce
/// // You can help fix docs!
/// #[poise::command(slash_command, prefix_command, broadcast_typing)]
/// async fn activity(ctx: Context<'_>) -> Result<(), Error> {
///    let attachment = TetrioActivity::fetch().await?.create_chart()?; // notice how theres no "TetrioActivity::new().fetch"?
///    ctx.send(
///        poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(
///            attachment,
///            "tetrio_activity.png",
///        )),
///    )
///    .await?;
///    Ok(())
/// }
/// ```
pub struct TetrioActivity {
    /// Self explanatory (probably), DO NOT MODIFY THIS!
    pub data: Vec<f64>,
    /// you too man, DO NOT MODIFY THIS!\
    /// this however requires std (no idea why im mentioning that, my crate can NEVER work without std)\
    /// useful for cache invaildation if you like that  
    ///  
    /// why? just use moka bro please
    pub fetched_at: Instant,
}

impl TetrioActivity {
    /// obvious lol\
    /// DO NOT USE `TetrioActivity::new().fetch().await?` PLEASE!
    /// # errors
    /// - errors when tetrio feels like it probably
    pub async fn fetch() -> Result<Self, Error> {
        let client = InMemoryReqwestClient::default();
        let activity = client
            .fetch_general_activity()
            .await
            .map_err(|e| format!("failed to fetch activity: {e}"))?;

        match activity {
            Packet {
                data: Some(data), ..
            } => Ok(Self {
                data: data.activity.iter().map(|&x| x as f64).collect(),
                fetched_at: Instant::now(),
            }),
            Packet { error, .. } => {
                if let Some(err) = error {
                    Err(format!("API error: {err:?}").into())
                } else {
                    Err("unknown error from tetrio API".into())
                }
            }
        }
    }

    /// makes a chart using plotters and returns raw bytes\
    /// Don't forget to `?`!\
    /// Example:  
    /// ```
    /// let image_bytes = TetrioActivity::fetch().await?.create_chart()?;
    /// // no idea what comes next bro :sob:
    /// ```
    /// # errors:
    /// - errors out when you modify this method
    /// - failure to read the docs (bro i told you to use `TetrioActivity::fetch()` not whatever this is: `TetrioActivity::new().fetch()`)
    /// - a random bit switch
    pub fn create_chart(&self) -> Result<Vec<u8>, Error> {
        use image::ImageBuffer;
        use plotters::prelude::*;
        use plotters::style::colors::full_palette::ORANGE;
        use std::io::Cursor;
        const W: u32 = 800;
        const H: u32 = 400;
        const BYTES_PER_PIXEL: usize = 3;

        // 1. raw RGB buffer (plotters wants &mut [u8])
        let size = (W as usize)
            .checked_mul(H as usize)
            .and_then(|s| s.checked_mul(BYTES_PER_PIXEL))
            .ok_or("image size too large")?;
        let mut raw = vec![0u8; size];

        {
            // 2. draw into that raw buffer (probably)
            let root = BitMapBackend::with_buffer(&mut raw, (W, H)).into_drawing_area();
            root.fill(&WHITE)?;

            let (min_val, max_val) = match (
                self.data.iter().copied().reduce(f64::min),
                self.data.iter().copied().reduce(f64::max),
            ) {
                (Some(min), Some(max)) => (min, max),
                _ => return Err("no self.data".into()),
            };
            let pad = (max_val - min_val) * 0.1;
            let y_min = (min_val - pad).max(0.0);
            let y_max = max_val + pad;

            let mut chart = ChartBuilder::on(&root)
                .caption("tetrio server activity", ("sans-serif", 25))
                .margin(20)
                .x_label_area_size(40)
                .y_label_area_size(50)
                .build_cartesian_2d(0f64..self.data.len() as f64, y_min..y_max)?;

            chart
                .configure_mesh()
                .x_desc("time")
                .y_desc("players")
                .draw()?;

            chart.draw_series(LineSeries::new(
                self.data.iter().enumerate().map(|(i, &v)| (i as f64, v)),
                &ORANGE,
            ))?;

            root.present()?;
        }

        // 3. wrap raw rgb bytes in an `RgbImage` and encode to png
        let rgb_img: ImageBuffer<image::Rgb<u8>, _> =
            ImageBuffer::from_raw(W, H, raw).ok_or("buffer size mismatch")?;
        let mut png_bytes = Vec::new();
        rgb_img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)?;

        Ok(png_bytes)
    }
}
