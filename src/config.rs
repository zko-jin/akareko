use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{error, warn};
use yosemite::RouterApi;

use crate::{
    db::user::I2PAddress,
    errors::TomlSaveError,
    hash::{PrivateKey, PublicKey},
    helpers::b32_from_pub_b64,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    private_key: PrivateKey,
    //todo: custom serialize to remove public_key
    public_key: PublicKey,
}

impl KeyPair {
    pub fn new(private_key: PrivateKey) -> Self {
        let public_key = private_key.public_key();

        Self {
            private_key,
            public_key,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuroraConfig {
    #[serde(flatten)]
    keypair: KeyPair,

    sam_port: u16,

    eepsite_key: String,
    eepsite_address: I2PAddress,

    dev_mode: bool,

    image_viewer_preferences: ImageViewerPreferences,

    is_relay: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageScale {
    /// Image will be stretched to fit the screen
    Fill,
    /// Image won't get biggeer than the screen horizontally
    FitHorizontally,
    /// Image won't get bigger than the screen vertically
    FitVertically,
    /// Renders image as is
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageVisualizationType {
    LeftToRight,
    RightToLeft,
    Scroll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageViewerPreferences {
    pub double_pages: bool,
    /// Percentage of the image size
    pub zoom: i8,
    pub scale: ImageScale,
    pub visualization_type: ImageVisualizationType,
}

impl Default for ImageViewerPreferences {
    fn default() -> Self {
        Self {
            double_pages: false,
            zoom: 100,
            scale: ImageScale::FitHorizontally,
            visualization_type: ImageVisualizationType::LeftToRight,
        }
    }
}

impl Default for AuroraConfig {
    fn default() -> Self {
        Self {
            keypair: KeyPair::new(PrivateKey::new()),
            sam_port: 7656,
            eepsite_key: String::new(),
            eepsite_address: I2PAddress::new(""),
            dev_mode: false,
            is_relay: false,
            image_viewer_preferences: ImageViewerPreferences::default(),
        }
    }
}

impl AuroraConfig {
    pub async fn save(&self) -> Result<(), TomlSaveError> {
        let config = toml::to_string(self)?;
        fs::write("config.toml", config).await?;
        Ok(())
    }

    async fn generate_eepsite_key() -> (I2PAddress, String) {
        let (destination, private_key) = RouterApi::default().generate_destination().await.unwrap();
        (b32_from_pub_b64(&destination).unwrap(), private_key)
    }

    /// can't fail, if the config is missing or is invalid it will just be created
    /// anyways
    pub async fn load() -> AuroraConfig {
        let mut should_save = false;

        let mut config = match fs::read_to_string("config.toml").await {
            Ok(config_str) => match toml::from_str(&config_str) {
                Ok(config) => config,
                Err(e) => {
                    error!("error loading config: {}", e);
                    AuroraConfig::default()
                }
            },
            Err(e) => {
                warn!("error opening config file: {}", e);
                should_save = true;
                AuroraConfig::default()
            }
        };

        if config.eepsite_key.is_empty() {
            let (address, key) = Self::generate_eepsite_key().await;
            config.eepsite_address = address;
            config.eepsite_key = key;
        }

        if should_save {
            match config.save().await {
                Ok(_) => {}
                Err(e) => {
                    error!("error saving config: {}", e);
                }
            }
        }

        config
    }

    pub fn eepsite_key(&self) -> &String {
        &self.eepsite_key
    }

    pub fn eepsite_address(&self) -> &I2PAddress {
        &self.eepsite_address
    }

    pub fn sam_port(&self) -> u16 {
        self.sam_port
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.keypair.public_key
    }

    pub fn private_key(&self) -> &PrivateKey {
        &self.keypair.private_key
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn set_dev_mode(&mut self, dev_mode: bool) {
        self.dev_mode = dev_mode;
    }

    pub fn is_relay(&self) -> bool {
        self.is_relay
    }

    pub fn set_is_relay(&mut self, is_relay: bool) {
        self.is_relay = is_relay;
    }
}
