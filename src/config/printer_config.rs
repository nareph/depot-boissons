// src/config/printer_config.rs

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrinterType {
    Serial,
    USB,
    Network,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfig {
    pub name: String,
    pub port: String,
    pub printer_type: PrinterType,
    pub paper_width: u32,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct ConfigFile {
    printers: Vec<PrinterConfig>,
}

const CONFIG_PATH: &str = "printer_config.json";

pub fn load_printers() -> Vec<PrinterConfig> {
    if Path::new(CONFIG_PATH).exists() {
        fs::read_to_string(CONFIG_PATH)
            .ok()
            .and_then(|data| serde_json::from_str::<ConfigFile>(&data).ok())
            .map(|config| config.printers)
            .unwrap_or_else(|| vec![default_printer()]) // Fallback si le fichier est corrompu
    } else {
        // Retourne une config par défaut si le fichier n'existe pas
        //vec![default_printer()]
        vec![PrinterConfig {
            name: "Imprimante par défaut (USB)".to_string(),
            port: "/dev/ttyUSB0".to_string(),
            printer_type: PrinterType::USB,
            paper_width: 48,
            is_default: true,
        }]
    }
}

pub fn save_printers(printers: &[PrinterConfig]) -> std::io::Result<()> {
    let config_file = ConfigFile {
        printers: printers.to_vec(),
    };
    let data = serde_json::to_string_pretty(&config_file)?;
    fs::write(CONFIG_PATH, data)
}

fn default_printer() -> PrinterConfig {
    PrinterConfig {
        name: "Imprimante par défaut (Simulation)".to_string(),
        port: "/dev/null".to_string(), // Port neutre
        printer_type: PrinterType::USB,
        paper_width: 48,
        is_default: true,
    }
}
