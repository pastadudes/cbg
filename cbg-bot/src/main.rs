// cbg, a discord bot (that does basic things)
// Copyright (C) 2025 pastaya
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Pointer;
use std::sync::Arc;

use cbg_core::osu::OsuClient;
use cbg_core::{AverageColor, imageops::*};
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use tokio::time::{Duration, Instant, sleep_until};

mod osu;
mod tetrio;

pub struct Data {
    // who knows i might add more user data here
    start_time: Instant,
    osu_client: Arc<OsuClient>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// 15 decimal pi.  
/// also watch out theres a 0.000000001454% chance of a mutated pi
#[poise::command(slash_command, prefix_command)]
async fn pi(ctx: Context<'_>) -> Result<(), Error> {
    let pi = cbg_core::pi().await;

    ctx.reply(format!("pi is: {pi}")).await?;
    Ok(())
}

/// shows a user's avatar
#[poise::command(slash_command, prefix_command, aliases("av"))]
async fn avatar(
    ctx: Context<'_>,
    #[description = "user mention"] user: Option<serenity::User>,
    #[description = "user ID"] user_id: Option<serenity::UserId>,
) -> Result<(), Error> {
    // decide whose avatar to show
    let user: serenity::User = if let Some(user) = user {
        user
    } else if let Some(uid) = user_id {
        ctx.serenity_context().http.get_user(uid).await?
    } else {
        ctx.author().clone()
    };

    let avatar_url = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    let embed_color = AverageColor::from_image_url(&avatar_url)
        .await?
        .to_embed_color();
    // build the embed
    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s avatar", user.name))
        .color(embed_color)
        .image(avatar_url);

    // send it
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// returns with the age of the discord account
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    // convert Timestamp -> chrono::DateTime<Utc>
    let datetime: DateTime<Utc> = u.created_at().to_utc();

    let timestamp = datetime.timestamp();

    let response = format!(
        "{}'s account was created at {} (<t:{}:R>)",
        u.name, datetime, timestamp
    );

    ctx.reply(response).await?;

    Ok(())
}

/// NOT FOR PUBLIC USE!!! well nothing is stopping you  
/// anyways it registers guild commands
#[poise::command(slash_command, prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// HELP!!! well uh it also tracks edits. you can use -help `command` to get more info about a command
#[poise::command(slash_command, track_edits, prefix_command)]
async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration::default();
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Returns the uptime of the bot
#[poise::command(slash_command, prefix_command, aliases("up", "ut"))]
async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let elapsed = ctx.data().start_time.elapsed();
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;

    ctx.reply(format!(
        "uptime: {:02}:{:02}:{:02}",
        hours, minutes, seconds
    ))
    .await?;
    Ok(())
}

/// Sends an embed of the user's info.
#[poise::command(slash_command, prefix_command, aliases("users", "u"))]
async fn user(
    ctx: Context<'_>,
    #[description = "User mention"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.unwrap_or_else(|| ctx.author().clone());

    let avatar_url = user
        .avatar_url()
        .unwrap_or_else(|| user.default_avatar_url());

    let author = serenity::CreateEmbedAuthor::new(&user.name).icon_url(
        user.avatar_url()
            .unwrap_or_else(|| user.default_avatar_url()),
    );

    let embed_color = AverageColor::from_image_url(&avatar_url)
        .await?
        .to_embed_color();
    let embed = serenity::CreateEmbed::new()
        // .author(|a: &mut serenity::CreateEmbedAuthor| a.name(&user.name).icon_url(avatar_url))
        .author(author)
        .color(embed_color)
        .thumbnail(
            user.avatar_url()
                .unwrap_or_else(|| user.default_avatar_url()),
        )
        .field(
            "user info",
            format!("id: {}\nusername: @{}", user.id, user.name),
            false,
        );

    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;

    Ok(())
}

/// calls saul
#[poise::command(slash_command, prefix_command)]
async fn call(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Calling Saul...").await?;
    sleep_until(Instant::now() + Duration::from_secs(5)).await;
    ctx.say("Saul: yo").await?;
    Ok(())
}

/// Ping pong! not the game tho it just tells you if the bot is responsive (IN TIME)
#[poise::command(prefix_command, slash_command, aliases("p"))]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let before_timestamp = ctx.created_at();

    // send an initial reply and get a handle to it
    let reply_handle = ctx.reply("Pong!").await?;

    // retrieve the full message object from the handle
    let message = reply_handle.message().await?;

    // get the timestamp of the bot's reply message
    let after_timestamp = message.timestamp;

    // convert both timestamps into chrono to subtract
    let before: chrono::DateTime<chrono::Utc> = before_timestamp.to_utc();
    let after: chrono::DateTime<chrono::Utc> = after_timestamp.to_utc();

    let latency = after - before;

    let response_content = format!("Pong! Latency: `{}ms`", latency.num_milliseconds());

    let builder = poise::CreateReply::default().content(response_content);

    reply_handle.edit(ctx, builder).await?;

    Ok(())
}

/// Perform operations on an image.
#[poise::command(slash_command, prefix_command)]
async fn imageop(
    ctx: Context<'_>,
    #[description = "the image to process"] img: serenity::Attachment,
    #[description = "the blur amount (optional)"] blur: Option<f32>,
    #[description = "the flip direction (optional)"] orientation: Option<ImageOrientation>, // Horizontal or Vertical
    #[description = "set to true for grayscale"] grayscale: Option<bool>,
) -> Result<(), Error> {
    // check if the file is an image.
    if !img
        .content_type
        .as_ref()
        .is_some_and(|ct| ct.starts_with("image/"))
    {
        ctx.say("please provide a valid image file!").await?;
        return Ok(());
    }

    if blur.is_none() && orientation.is_none() && grayscale.is_none() {
        ctx.say("??? bro i aint giving you the same image").await?;
        return Ok(());
    }

    let url = img.url;
    // Here we call our existing image processing function
    let result = ImageProcessor::new(url)
        .blur(blur)
        .flip(orientation)
        .grayscale(grayscale)
        .process()
        .await?;

    ctx.send(
        poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(
            result,
            format!("new!!!_{}", img.filename),
        )),
    )
    .await?;
    Ok(())
}

/// required by the agpl  
/// if a fork of my bot doesn't include this pls contact me  
/// @pastaya
#[poise::command(prefix_command, slash_command)]
async fn source(ctx: Context<'_>) -> Result<(), Error> {
    // ehh this doesn't need error handling
    ctx.reply("https://github.com/pastadudes/cbg").await?;
    Ok(())
}

/// Get a random ip address
#[poise::command(prefix_command, slash_command)]
async fn ipv4(ctx: Context<'_>) -> Result<(), Error> {
    let ip = cbg_core::get_random_ipv4().await;

    ctx.reply(format!(
        "heres a vaild ip address (may not be online): {}",
        ip
    ))
    .await?;
    Ok(())
}

/// shows 5 job listings from arbeitnow.com
#[poise::command(slash_command, prefix_command, aliases("j*bs"), owners_only)]
async fn jobs(ctx: Context<'_>) -> Result<(), Error> {
    let embed = cbg_core::jobs::JobListings::fetch()
        .await?
        .take(5)
        .to_embed();
    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn invite(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("invite me pls: https://discord.com/oauth2/authorize?client_id=734193707741347851")
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                pi(),
                avatar(),
                age(),
                register(),
                help(),
                uptime(),
                user(),
                call(),
                ping(),
                source(),
                imageop(),
                ipv4(),
                tetrio::tetrio(),
                jobs(),
                invite(),
                crate::osu::osu(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("-".into()),
                ..Default::default()
            },
            event_handler: |_ctx, _event, _framework, _data| {
                Box::pin(async move {
                    // if let serenity::FullEvent::Message { new_message } = event {
                    //     if new_message.author.bot || new_message.content.starts_with("-") {
                    //         return Ok(());
                    //     }
                    //
                    //     println!(
                    //         "event_handler triggered for message: {}",
                    //         new_message.content
                    //     );
                    //
                    //     let guild_id = match new_message.guild_id {
                    //         Some(g) => g,
                    //         None => return Ok(()), // skip DMs cuz why would it count in dms?
                    //     };
                    //
                    //     let censor = Censor::Standard + Censor::Sex;
                    //     if censor.check(&new_message.content) {
                    //         let mut map = data.swears.lock().await;
                    //         let key = SwearKey::from((guild_id, new_message.author.id));
                    //         *map.entry(key).or_insert(0) += 1;
                    //
                    //         if let Err(e) = data.save("swears.json").await {
                    //             eprintln!("failed to save swears.json: {:?}", e);
                    //         }
                    //     }
                    // }
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, ready, _framework| {
            Box::pin(async move {
                // let swears = Data::load("swears.json").await?;
                // Ok(Data {
                //     swears: Arc::new(Mutex::new(swears)),
                // })
                println!("logged in as {}!", ready.user.name);

                // TODO: maybe make it a env variable?
                ctx.set_presence(
                    Some(serenity::ActivityData::watching("you goon")),
                    serenity::OnlineStatus::Online,
                );

                Ok(Data {
                    start_time: Instant::now(),
                    osu_client: OsuClient::from_env().await?.into(),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        eprintln!("kablam! {:?}", why);
    }
}
