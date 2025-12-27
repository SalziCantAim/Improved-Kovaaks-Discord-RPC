use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
const CLIENT_ID: &str = "1321990331083784202";
pub struct DiscordRPC {
    client: DiscordIpcClient,
    connected: bool,
}
impl DiscordRPC {
    pub fn new() -> Result<Self, String> {
        let client = DiscordIpcClient::new(CLIENT_ID)
            .map_err(|e| format!("Failed to create Discord RPC client: {}", e))?;
        Ok(Self {
            client,
            connected: false,
        })
    }
    pub fn connect(&mut self) -> Result<(), String> {
        if !self.connected {
            self.client.connect()
                .map_err(|e| format!("Failed to connect Discord RPC: {}", e))?;
            self.connected = true;

        }
        Ok(())
    }
    pub fn disconnect(&mut self) -> Result<(), String> {
        if self.connected {
            self.client.close()
                .map_err(|e| format!("Failed to disconnect Discord RPC: {}", e))?;
            self.connected = false;

        }
        Ok(())
    }
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    pub fn update_presence(
        &mut self,
        scenario_name: &str,
        start_time: Option<i64>,
        highscore: f64,
        session_highscore: f64,
        _installation_path: &str,
        share_code: Option<String>,
    ) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }
        if scenario_name.is_empty() || scenario_name == "Unknown Scenario" {

            return Ok(());
        }
        let details_text = format!("Playing: {}", scenario_name);
        let highscore_display = format!("{:.1}", highscore);
        let state_text = format!("Highscore: {}", highscore_display);
        let large_text = if session_highscore > 0.0 {
            let session_display = format!("{:.1}", session_highscore);
            format!("Session Best: {}", session_display)
        } else {
            "No session plays yet".to_string()
        };
        let mut activity_builder = activity::Activity::new()
            .details(&details_text)
            .state(&state_text);
        if let Some(timestamp) = start_time {
            activity_builder = activity_builder.timestamps(
                activity::Timestamps::new().start(timestamp)
            );
        }
        activity_builder = activity_builder.assets(
            activity::Assets::new()
                .large_image("kovaak_image")
                .large_text(&large_text)
                .small_text(&large_text)
        );
        let button_url = if let Some(code) = &share_code {
            format!("steam://run/824270/?action=jump-to-playlist;sharecode={}", code)
        } else {
            let encoded_scenario = scenario_name.replace(' ', "%20").replace('&', "%26");
            format!("steam://run/824270/?action=jump-to-scenario;name={}", encoded_scenario)
        };
        let button_label = if share_code.is_some() {
            "Play Playlist"
        } else {
            "Play Scenario"
        };
        let button = activity::Button::new(button_label, &button_url);
        activity_builder = activity_builder.buttons(vec![button]);
        self.client.set_activity(activity_builder)
            .map_err(|e| format!("Failed to update Discord RPC activity: {}", e))?;
        Ok(())
    }
    pub fn clear_presence(&mut self) -> Result<(), String> {
        if self.connected {
            self.client.clear_activity()
                .map_err(|e| format!("Failed to clear Discord RPC activity: {}", e))?;
        }
        Ok(())
    }
}
impl Drop for DiscordRPC {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}