use aniapi::{export_plugin, Context, Event, Plugin, Result};
use async_trait::async_trait;
use tracing::info;

#[derive(Default)]
pub struct VoiceTempPlugin;

#[async_trait]
impl Plugin for VoiceTempPlugin {
    fn name(&self) -> &str {
        "VoiceTempPlugin"
    }

    async fn on_load(&mut self, _ctx: &Context) -> Result<()> {
        info!("VoiceTempPlugin loaded!");
        Ok(())
    }

    async fn on_unload(&mut self, _ctx: &Context) -> Result<()> {
        info!("VoiceTempPlugin unloaded!");
        Ok(())
    }

    async fn on_event(&mut self, event: &Event, _ctx: &Context) -> Result<()> {
        match event {
            Event::System(e) => {
                info!("VoiceTempPlugin received system event: {:?}", e);
            }
            _ => {}
        }
        Ok(())
    }
}

export_plugin!(VoiceTempPlugin);
