use aniapi::{export_plugin, CommandSpec, Context, Event, Plugin, Result};
use async_trait::async_trait;
use tracing::info;

#[derive(Default)]
pub struct VoiceTempPlugin;

#[async_trait]
impl Plugin for VoiceTempPlugin {
    fn name(&self) -> &str {
        "VoiceTempPlugin"
    }

    fn commands(&self) -> Vec<CommandSpec> {
        vec![
            CommandSpec::new("create_temp", "Create a temporary voice channel"),
        ]
    }

    async fn on_load(&mut self, _ctx: &Context) -> Result<()> {
        info!("VoiceTempPlugin loaded!");
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &Context) -> Result<()> {
        info!("VoiceTempPlugin unloaded!");
        Ok(())
    }

    async fn on_event(&mut self, event: &Event, ctx: &Context) -> Result<()> {
        match event {
            Event::Interaction(interaction) => {
                if let serenity::model::application::Interaction::Command(command) = interaction.as_ref() {
                    match command.data.name.as_str() {
                        "create_temp" => {
                            if let Some(guild_id) = command.guild_id {
                                if let Some(guild_mgr) = &ctx.guild {
                                    match guild_mgr.create_voice_channel(guild_id.get(), "Temp Channel", None).await {
                                        Ok(channel_id) => {
                                            // Move user to new channel if they are in one
                                            // Note: We need to know if user is in a channel. 
                                            // Without full cache, we assume user might be in the channel they triggered this from?
                                            // Or we just create it.
                                            
                                            let response = serenity::builder::CreateInteractionResponseMessage::new()
                                                .content(format!("Created temporary channel: <#{}>", channel_id));
                                            let builder = serenity::builder::CreateInteractionResponse::Message(response);
                                            if let Some(responder) = &ctx.responder {
                                                let _ = responder.respond(command.id.get(), &command.token, builder).await;
                                            }
                                        }
                                        Err(e) => {
                                            let response = serenity::builder::CreateInteractionResponseMessage::new()
                                                .content(format!("Failed to create channel: {:?}", e));
                                            let builder = serenity::builder::CreateInteractionResponse::Message(response);
                                            if let Some(responder) = &ctx.responder {
                                                let _ = responder.respond(command.id.get(), &command.token, builder).await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::VoiceStateUpdate(state) => {
                // Logic for temp channels:
                // If user joins "Master" channel -> Create Temp -> Move User
                // If Temp channel empty -> Delete
                
                // For now just log
                 if let Some(channel_id) = state.channel_id {
                    info!("User {} joined channel {}", state.user_id, channel_id);
                 }
            }
            Event::System(e) => {
                info!("VoiceTempPlugin received system event: {:?}", e);
            }
            _ => {}
        }
        Ok(())
    }
}

export_plugin!(VoiceTempPlugin);
