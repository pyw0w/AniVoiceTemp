use aniapi::{export_plugin, Context, Event, Plugin, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const MASTER_VOICE_CHANNEL: u64 = 1366403705460621359;
const CATEGORY_ID: u64 = 1366403705460621357;
const ALLOWED_GUILD_ID: u64 = 1366403704130900018;

#[derive(Default)]
pub struct VoiceTempPlugin {
    // Map: user_id -> created channel_id
    temp_channels: Arc<Mutex<HashMap<u64, u64>>>,
}

#[async_trait]
impl Plugin for VoiceTempPlugin {
    fn name(&self) -> &str {
        "VoiceTempPlugin"
    }

    async fn on_load(&mut self, ctx: &Context) -> Result<()> {
        // Setup logging redirection via log crate
        if let Some(logger) = ctx.logger {
            log::set_logger(logger).ok();
            log::set_max_level(log::LevelFilter::Info);
        }

        log::info!("VoiceTempPlugin loaded! (log crate)");
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &Context) -> Result<()> {
        log::info!("VoiceTempPlugin unloaded! (log crate)");
        Ok(())
    }

    async fn on_event(&mut self, event: &Event, ctx: &Context) -> Result<()> {
        match event {
            Event::VoiceStateUpdate(state) => {
                let state_guild_id = state.guild_id.map(|id| id.get());
                if state_guild_id != Some(ALLOWED_GUILD_ID) {
                    return Ok(());
                }
                let user_id = state.user_id.get();
                let cur_chan = state.channel_id.map(|id| id.get());

                if let Some(channel_id) = cur_chan {
                    // User joined a channel
                    if channel_id == MASTER_VOICE_CHANNEL {
                        // User joined master channel: create temp channel and move them to it
                        if let Some(guild_mgr) = &ctx.guild {
                            if let Some(guild_id) = state.guild_id.map(|id| id.get()) {
                                let mut skip = false;
                                {
                                    let map = self.temp_channels.lock().await;
                                    if map.contains_key(&user_id) {
                                        skip = true;
                                    }
                                }
                                if skip {
                                    log::info!(
                                        "User {} already has a temp channel, skipping creation.",
                                        user_id
                                    );
                                    return Ok(());
                                }

                                match guild_mgr
                                    .create_voice_channel(
                                        guild_id,
                                        &format!("🔊 {}'s Room", user_id),
                                        Some(CATEGORY_ID),
                                    )
                                    .await
                                {
                                    Ok(temp_channel_id) => {
                                        // Move user into the new channel
                                        let _ = guild_mgr
                                            .move_member(guild_id, user_id, temp_channel_id)
                                            .await;
                                        log::info!(
                                            "Created temp channel {} and moved user {}",
                                            temp_channel_id, user_id
                                        );
                                        // Track user <-> channel mapping
                                        let mut map = self.temp_channels.lock().await;
                                        map.insert(user_id, temp_channel_id);
                                    }
                                    Err(e) => {
                                        log::warn!(
                                            "Failed to create temp channel for {}: {:?}",
                                            user_id, e
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        // User joined any other channel
                        // Only delete previous temp channel if they left it
                        let prev_chan_id = {
                            let map = self.temp_channels.lock().await;
                            map.get(&user_id).copied()
                        };
                        // If the user is moving out of their temp channel (i.e., prev_chan exists and is not the current channel)
                        if let Some(user_temp_channel_id) = prev_chan_id {
                            if channel_id != user_temp_channel_id {
                                // Remove mapping and delete temp channel
                                let mut map = self.temp_channels.lock().await;
                                map.remove(&user_id);
                                if let Some(guild_mgr) = &ctx.guild {
                                    if let Err(e) = guild_mgr.delete_channel(user_temp_channel_id).await {
                                        log::warn!(
                                            "Failed to delete temp channel {} after user {} switched: {:?}",
                                            user_temp_channel_id, user_id, e
                                        );
                                    } else {
                                        log::info!(
                                            "Deleted temp channel {} after user {} switched channel",
                                            user_temp_channel_id, user_id
                                        );
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // User left all voice channels
                    // Check if user had a temp channel, and delete if present
                    let prev_chan_id = {
                        let map = self.temp_channels.lock().await;
                        map.get(&user_id).copied()
                    };
                    if let Some(user_temp_channel_id) = prev_chan_id {
                        // User left all -- so they left their temp channel
                        let mut map = self.temp_channels.lock().await;
                        map.remove(&user_id);
                        if let Some(guild_mgr) = &ctx.guild {
                            if let Err(e) = guild_mgr.delete_channel(user_temp_channel_id).await {
                                log::warn!(
                                    "Failed to delete temp channel {} after user {} left: {:?}",
                                    user_temp_channel_id, user_id, e
                                );
                            } else {
                                log::info!(
                                    "Deleted temp channel {} after user {} left all channels",
                                    user_temp_channel_id, user_id
                                );
                            }
                        }
                    }
                }
            }
            Event::System(e) => {
                log::info!("VoiceTempPlugin received system event: {:?}", e);
            }
            _ => {}
        }
        Ok(())
    }
}

export_plugin!(VoiceTempPlugin);
