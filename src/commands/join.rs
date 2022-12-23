use std::num::NonZeroU64;

use serenity::{
    builder::*, collector::ComponentInteractionCollector, model::prelude::*, prelude::*,
};

use crate::utils::{
    events::Events,
    participant::Participant,
    team::{TeamId, Teams},
    traits::SendOrEdit,
};

async fn select_event(
    ctx: &Context,
    interaction: &CommandInteraction,
) -> Result<Option<(ComponentInteraction, ScheduledEventId)>, serenity::Error> {
    let guild_id = interaction.guild_id.unwrap();
    let Some(menu) = Events::menu_nonzero_team(ctx, guild_id).await else {
        CreateInteractionResponseMessage::new()
            .content("Veuillez créer une équipe avant d'essayer de rejoindre une équipe.")
            .build_and_send(ctx, interaction.id, &interaction.token)
            .await?;
        return Ok(None);
    };
    CreateInteractionResponseMessage::new()
        .select_menu(menu)
        .content("Sélectionnez l'événement que vous voulez rejoindre.")
        .ephemeral(true)
        .build_and_send(ctx, interaction.id, &interaction.token)
        .await?;
    let interaction = ComponentInteractionCollector::new(&ctx.shard)
        .next()
        .await
        .ok_or(SerenityError::Other("Event selection failed."))?;
    let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind else {
            return Err(SerenityError::Other("Event selection failed."));
    };
    let event_id = ScheduledEventId(values[0].parse::<NonZeroU64>().unwrap());
    Ok(Some((interaction, event_id)))
}

async fn select_team(
    ctx: &Context,
    interaction: &ComponentInteraction,
    event_id: &ScheduledEventId,
) -> Result<(ComponentInteraction, TeamId), serenity::Error> {
    let guild_id = interaction.guild_id.unwrap();
    let menu = Teams::menu(ctx, guild_id, *event_id).await;
    CreateInteractionResponseMessage::new()
        .content("Sélectionnez l'équipe que vous voulez rejoindre.")
        .components(vec![CreateActionRow::SelectMenu(menu)])
        .build_and_edit(ctx, interaction.id, &interaction.token)
        .await?;
    let interaction = ComponentInteractionCollector::new(&ctx.shard)
        .next()
        .await
        .ok_or(SerenityError::Other("Team selection failed."))?;
    let ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind else {
        return Err(SerenityError::Other("Team selection failed."));
    };
    let team_id = TeamId(values[0].parse::<u64>().unwrap());
    Ok((interaction, team_id))
}

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<(), serenity::Error> {
    let guild_id = interaction.guild_id.unwrap();
    let Some((interaction, event_id)) = select_event(ctx, interaction).await? else {
        return Ok(())
    };
    let (interaction, team_id) = select_team(ctx, &interaction, &event_id).await?;
    let mut event = Events::get(ctx, guild_id, &event_id)
        .await
        .ok_or(SerenityError::Other("Event joining failed."))?;
    let team = event
        .teams
        .get_team(&team_id)
        .ok_or(SerenityError::Other("Event joining failed."))?;
    let participant = Participant::from_user(interaction.user);
    let permissions = vec![PermissionOverwrite {
        allow: Permissions::VIEW_CHANNEL,
        deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(participant.id),
    }];
    println!("Test1");
    println!("{} {}", team.text_channel, team.vocal_channel);
    let builder = EditChannel::new().permissions(permissions.clone());
    team.text_channel.edit(ctx, builder.clone()).await?;
    team.vocal_channel.edit(ctx, builder).await?;
    // EditChannel::new()
    //     .permissions(permissions)
    //     .execute(ctx, team.vocal_channel, interaction.guild_id)
    //     .await?;

    let msg = match event.teams.add_participant(team_id, participant) {
        Ok(_) => format! {"Vous avez été rajouté à l'équipe: {}", team.name},
        Err(error) => format! {"Vous n'avez pas été rajouté à l'équipe: {}", error},
    };
    println!("Test2");
    Events::refresh_event(ctx, guild_id, &event).await;
    CreateInteractionResponseMessage::new()
        .content(msg)
        .components(vec![])
        .build_and_edit(ctx, interaction.id, &interaction.token)
        .await?;
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("join").description("Join a team.")
}
