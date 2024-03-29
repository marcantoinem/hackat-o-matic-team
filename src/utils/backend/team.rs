use std::{
    collections::{hash_map::Iter, HashMap},
    fmt,
};

use super::{events::Events, participant::Participant};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{ChannelId, GuildId, ScheduledEventId, UserId},
    builder::*,
    prelude::*,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Team {
    pub name: String,
    pub description: String,
    team: Vec<Participant>,
    pub text_channel: ChannelId,
    pub vocal_channel: ChannelId,
}

impl Team {
    pub fn new<Text: Into<String>>(
        name: Text,
        description: Text,
        team: Vec<Participant>,
        text_channel: ChannelId,
        vocal_channel: ChannelId,
    ) -> Team {
        Team {
            name: name.into(),
            description: description.into(),
            team,
            text_channel,
            vocal_channel,
        }
    }
    pub fn contains(&self, user_id: UserId) -> bool {
        self.team
            .iter()
            .any(|participant| participant.id == user_id)
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "**{}**: {}", self.name, self.description)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TeamId(pub u64);

impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Teams {
    teams: HashMap<TeamId, Team>,
    capacity: Option<u32>,
}

impl Teams {
    pub fn add_team(&mut self, team: Team) {
        let team_id = TeamId(self.teams.len() as u64);
        self.teams.insert(team_id, team);
    }
    pub fn iter(&self) -> Iter<'_, TeamId, Team> {
        self.teams.iter()
    }
    pub fn get_team(&self, id: &TeamId) -> Option<Team> {
        self.teams.get(id).cloned()
    }
    pub fn len(&self) -> usize {
        self.teams.len()
    }
    pub fn is_empty(&self) -> bool {
        self.teams.is_empty()
    }
    pub fn delete(&mut self, team_id: &TeamId) -> Option<Team> {
        self.teams.remove(team_id)
    }
    pub fn add_participant(
        &mut self,
        team_id: TeamId,
        participant: Participant,
    ) -> Result<(), SerenityError> {
        if let (Some(team), Some(capacity)) = (self.teams.get(&team_id), self.capacity) {
            if team.team.len() >= capacity as usize {
                return Err(SerenityError::Other(
                    "L'équipe a atteint sa capacité maximale",
                ));
            }
        }
        self.teams
            .entry(team_id)
            .and_modify(|x| x.team.push(participant));
        Ok(())
    }
    pub fn remove_participant(&mut self, team_id: TeamId, user_id: UserId) {
        self.teams.entry(team_id).and_modify(|x| {
            x.team.retain(|participant| participant.id != user_id);
        });
    }
    pub async fn menu_without_user(
        ctx: &Context,
        guild_id: GuildId,
        event_id: ScheduledEventId,
        user_id: UserId,
    ) -> CreateSelectMenu {
        let event = Events::get(ctx, guild_id, &event_id).await.unwrap();
        let options: Vec<CreateSelectMenuOption> = event
            .teams
            .iter()
            .filter(|(_, team)| !team.contains(user_id))
            .map(|(id, team)| CreateSelectMenuOption::new(team.name.clone(), id.to_string()))
            .collect();
        let select_menu = CreateSelectMenuKind::String { options };
        CreateSelectMenu::new("team", select_menu)
    }
    pub async fn menu(
        ctx: &Context,
        guild_id: GuildId,
        event_id: ScheduledEventId,
    ) -> CreateSelectMenu {
        let event = Events::get(ctx, guild_id, &event_id).await.unwrap();
        let options: Vec<CreateSelectMenuOption> = event
            .teams
            .iter()
            .map(|(id, team)| CreateSelectMenuOption::new(team.name.clone(), id.to_string()))
            .collect();
        let select_menu = CreateSelectMenuKind::String { options };
        CreateSelectMenu::new("team", select_menu)
    }
}

impl Default for Teams {
    fn default() -> Self {
        let teams = HashMap::new();
        Teams {
            teams,
            capacity: None,
        }
    }
}

impl fmt::Display for Teams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (_, team) in self.teams.iter() {
            writeln!(f, "{}", team)?;
            write!(f, "Participants: ")?;
            team.team.iter().fold(Ok(()), |result, participant| {
                result.and_then(|_| write!(f, "{} ", participant))
            })?;
            writeln!(f)?;
        }
        Ok(())
    }
}
