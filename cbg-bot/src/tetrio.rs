use crate::{Context, Error};
use cbg_core::tetrio::*;
use poise::serenity_prelude as serenity;

/// tetrio related commands, DO NOT USE STANDALONE!! YOU MUST SPECIFY A SUBCOMMAND!!
#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "user",
        // "records",
        // "league",
        // "stats",
        "activity",
        // "leaderboard"
    ),
    aliases("pentrio"),
    user_cooldown = 1
)]
pub async fn tetrio(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("you forgot the subcommand...").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn user(ctx: Context<'_>, username: String) -> Result<(), Error> {
    let embed = TetrioUser::fetch(&username).await?.to_embed().await;
    ctx.send(poise::CreateReply::default().embed(embed).reply(true))
        .await?;
    Ok(())
}

/// Shows general activity of tetrio
#[poise::command(slash_command, prefix_command, broadcast_typing)]
async fn activity(ctx: Context<'_>) -> Result<(), Error> {
    let attachment = TetrioActivity::fetch().await?.create_chart()?;
    ctx.send(
        poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(
            attachment,
            "tetrio_activity.png",
        )),
    )
    .await?;
    Ok(())
}

// / returns an embed of the top 20 players in tetra league
// #[poise::command(prefix_command, slash_command)]
// async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
//     let client = &InMemoryReqwestClient::default();
//     let tetrio_leaderboard = client
//         .fetch_leaderboard(
//             LeaderboardType::League,
//             ValueBoundQuery::NotBound {
//                 limit: None,
//                 country: None,
//             },
//             None,
//         )
//         .await?;

//     match tetrio_leaderboard {
//         Packet {
//             data: Some(data), ..
//         } => {
//             // build an embed with the top N entries
//             let mut embed = serenity::CreateEmbed::default()
//                 .title("tetrio leaderboard")
//                 .color(serenity::colours::branding::GREEN);

//             for (i, entry) in data.entries.iter().enumerate().take(20) {
//                 embed = embed.field(
//                     format!("#{} {}", i + 1, entry.username),
//                     format!(
//                         "tr: {:.2} | rank: {} | country: {}",
//                         entry.league.tr,
//                         rank_label(entry.league.rank.as_ref()),
//                         entry.country.clone().unwrap_or_else(|| "??".into())
//                     ),
//                     false,
//                 );
//             }

//             ctx.send(poise::CreateReply::default().embed(embed)).await?;
//             Ok(())
//         }
//         Packet { error, .. } => {
//             ctx.say(format!("error fetching leaderboard! {:?}", error))
//                 .await?;
//             Ok(())
//         }
//     }
// }
