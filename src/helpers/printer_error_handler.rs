// src/helpers/printer_error_handler.rs

use crate::config::printer_config::PrinterType;
use slint::SharedString;

#[derive(Debug, Clone)]
pub enum PrinterError {
    ValidationError(String),
    ConnectionError(String),
    ConfigurationError(String),
    NetworkError(String),
    SystemError(String),
    DuplicateError(String),
}

impl PrinterError {
    pub fn message(&self) -> String {
        match self {
            PrinterError::ValidationError(msg) => format!("❌ Erreur de validation : {}", msg),
            PrinterError::ConnectionError(msg) => format!("🔌 Erreur de connexion : {}", msg),
            PrinterError::ConfigurationError(msg) => {
                format!("⚙️ Erreur de configuration : {}", msg)
            }
            PrinterError::NetworkError(msg) => format!("🌐 Erreur réseau : {}", msg),
            PrinterError::SystemError(msg) => format!("💻 Erreur système : {}", msg),
            PrinterError::DuplicateError(msg) => format!("🔄 Doublon : {}", msg),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PrinterError::ValidationError(_) => "❌",
            PrinterError::ConnectionError(_) => "🔌",
            PrinterError::ConfigurationError(_) => "⚙️",
            PrinterError::NetworkError(_) => "🌐",
            PrinterError::SystemError(_) => "💻",
            PrinterError::DuplicateError(_) => "🔄",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            PrinterError::ValidationError(_) => "#e74c3c",
            PrinterError::ConnectionError(_) => "#f39c12",
            PrinterError::ConfigurationError(_) => "#9b59b6",
            PrinterError::NetworkError(_) => "#3498db",
            PrinterError::SystemError(_) => "#34495e",
            PrinterError::DuplicateError(_) => "#e67e22",
        }
    }
}

pub struct PrinterErrorHandler;

impl PrinterErrorHandler {
    /// Validation complète d'une configuration d'imprimante
    pub fn validate_printer_config(
        name: &str,
        port: &str,
        printer_type: &str,
        paper_width: &str,
        existing_names: &[String],
    ) -> Result<(), PrinterError> {
        // Validation du nom
        Self::validate_name(name, existing_names)?;

        // Validation du port selon le type
        Self::validate_port(port, printer_type)?;

        // Validation de la largeur du papier
        Self::validate_paper_width(paper_width)?;

        Ok(())
    }

    fn validate_name(name: &str, existing_names: &[String]) -> Result<(), PrinterError> {
        let name = name.trim();

        if name.is_empty() {
            return Err(PrinterError::ValidationError(
                "Le nom de l'imprimante ne peut pas être vide".to_string(),
            ));
        }

        if name.len() > 50 {
            return Err(PrinterError::ValidationError(
                "Le nom ne peut pas dépasser 50 caractères".to_string(),
            ));
        }

        if name.len() < 3 {
            return Err(PrinterError::ValidationError(
                "Le nom doit contenir au moins 3 caractères".to_string(),
            ));
        }

        // Caractères interdits
        let forbidden_chars = [
            '<', '>', ':', '"', '|', '?', '*', '\\', '/', '\n', '\r', '\t',
        ];
        if let Some(bad_char) = name.chars().find(|c| forbidden_chars.contains(c)) {
            return Err(PrinterError::ValidationError(format!(
                "Le caractère '{}' n'est pas autorisé dans le nom",
                bad_char
            )));
        }

        // Vérification des doublons
        if existing_names
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(name))
        {
            return Err(PrinterError::DuplicateError(format!(
                "Une imprimante nommée '{}' existe déjà",
                name
            )));
        }

        Ok(())
    }

    fn validate_port(port: &str, printer_type: &str) -> Result<(), PrinterError> {
        let port = port.trim();

        if port.is_empty() {
            return Err(PrinterError::ValidationError(
                "Le port ne peut pas être vide".to_string(),
            ));
        }

        match printer_type {
            "Serial" => Self::validate_serial_port(port),
            "USB" => Self::validate_usb_port(port),
            "Network" => Self::validate_network_port(port),
            "Windows" => Self::validate_windows_port(port),
            _ => Err(PrinterError::ValidationError(
                "Type d'imprimante non reconnu".to_string(),
            )),
        }
    }

    fn validate_serial_port(port: &str) -> Result<(), PrinterError> {
        if cfg!(windows) {
            // Windows: COM1, COM2, etc.
            if !port.to_uppercase().starts_with("COM") {
                return Err(PrinterError::ValidationError(
                    "Port série Windows invalide. Format attendu: COM1, COM2, etc.".to_string(),
                ));
            }

            if let Some(num_part) = port.get(3..) {
                if num_part.parse::<u32>().is_err() {
                    return Err(PrinterError::ValidationError(
                        "Numéro de port série invalide. Ex: COM1, COM2".to_string(),
                    ));
                }
            }
        } else {
            // Unix/Linux: /dev/ttyUSB0, /dev/ttyACM0, etc.
            if !port.starts_with("/dev/") {
                return Err(PrinterError::ValidationError(
                    "Port série Unix invalide. Format attendu: /dev/ttyUSB0, /dev/ttyACM0"
                        .to_string(),
                ));
            }

            if port.len() < 10 {
                return Err(PrinterError::ValidationError(
                    "Nom de port série trop court".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_usb_port(port: &str) -> Result<(), PrinterError> {
        if port.len() > 100 {
            return Err(PrinterError::ValidationError(
                "Nom de port USB trop long".to_string(),
            ));
        }

        // Pour USB, on accepte différents formats
        // La validation précise se fera à la connexion
        Ok(())
    }

    fn validate_network_port(port: &str) -> Result<(), PrinterError> {
        if !port.contains(':') {
            return Err(PrinterError::NetworkError(
                "Format réseau invalide. Utilisez IP:PORT (ex: 192.168.1.100:9100)".to_string(),
            ));
        }

        let parts: Vec<&str> = port.split(':').collect();
        if parts.len() != 2 {
            return Err(PrinterError::NetworkError(
                "Format réseau invalide. Un seul ':' autorisé".to_string(),
            ));
        }

        // Validation de l'IP
        if parts[0].parse::<std::net::IpAddr>().is_err() {
            return Err(PrinterError::NetworkError(format!(
                "Adresse IP invalide: '{}'",
                parts[0]
            )));
        }

        // Validation du port
        match parts[1].parse::<u16>() {
            Ok(port_num) => {
                if port_num == 0 {
                    return Err(PrinterError::NetworkError(
                        "Le numéro de port ne peut pas être 0".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(PrinterError::NetworkError(format!(
                    "Numéro de port invalide: '{}'",
                    parts[1]
                )));
            }
        }

        Ok(())
    }

    fn validate_windows_port(port: &str) -> Result<(), PrinterError> {
        if port.len() > 100 {
            return Err(PrinterError::ValidationError(
                "Nom d'imprimante Windows trop long (max 100 caractères)".to_string(),
            ));
        }

        // Caractères interdits dans les noms d'imprimante Windows
        let forbidden_chars = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];
        if let Some(bad_char) = port.chars().find(|c| forbidden_chars.contains(c)) {
            return Err(PrinterError::ValidationError(format!(
                "Le caractère '{}' n'est pas autorisé dans le nom d'imprimante Windows",
                bad_char
            )));
        }

        Ok(())
    }

    fn validate_paper_width(width_str: &str) -> Result<(), PrinterError> {
        match width_str.parse::<u32>() {
            Ok(width) => match width {
                32 | 48 | 64 => Ok(()),
                _ => Err(PrinterError::ValidationError(
                    "Largeur de papier non supportée. Choisissez 32, 48 ou 64 caractères"
                        .to_string(),
                )),
            },
            Err(_) => Err(PrinterError::ValidationError(
                "Largeur de papier invalide".to_string(),
            )),
        }
    }

    /// Génère des suggestions d'aide basées sur le type d'imprimante
    pub fn get_port_suggestions(printer_type: &str) -> Vec<String> {
        match printer_type {
            "Serial" => {
                if cfg!(windows) {
                    vec![
                        "COM1".to_string(),
                        "COM2".to_string(),
                        "COM3".to_string(),
                        "COM4".to_string(),
                    ]
                } else {
                    vec![
                        "/dev/ttyUSB0".to_string(),
                        "/dev/ttyUSB1".to_string(),
                        "/dev/ttyACM0".to_string(),
                        "/dev/ttyACM1".to_string(),
                    ]
                }
            }
            "Network" => vec![
                "192.168.1.100:9100".to_string(),
                "192.168.1.101:9100".to_string(),
                "192.168.0.100:9100".to_string(),
                "10.0.0.100:9100".to_string(),
            ],
            "Windows" => vec![
                "Imprimante par défaut".to_string(),
                "HP LaserJet".to_string(),
                "Canon PIXMA".to_string(),
                "Epson L3150".to_string(),
            ],
            "USB" => vec![
                "USB001".to_string(),
                "USB002".to_string(),
                "AUTO".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Génère un message d'aide contextuel
    pub fn get_help_message(printer_type: &str) -> String {
        match printer_type {
            "Serial" => {
                if cfg!(windows) {
                    "Ports série Windows : COM1, COM2, COM3, etc. Vérifiez le Gestionnaire de périphériques.".to_string()
                } else {
                    "Ports série Unix/Linux : /dev/ttyUSB0, /dev/ttyACM0, etc. Utilisez 'ls /dev/tty*' pour lister.".to_string()
                }
            }
            "USB" => {
                "Imprimantes USB directes. Généralement détectées automatiquement.".to_string()
            }
            "Network" => {
                "Imprimantes réseau au format IP:PORT. Port standard: 9100. Ex: 192.168.1.100:9100"
                    .to_string()
            }
            "Windows" => {
                "Nom de l'imprimante système Windows. Visible dans Paramètres > Imprimantes."
                    .to_string()
            }
            _ => "Sélectionnez un type d'imprimante pour voir l'aide correspondante.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_validation() {
        let existing = vec!["Imprimante1".to_string()];

        // Tests valides
        assert!(PrinterErrorHandler::validate_name("Imprimante Test", &existing).is_ok());
        assert!(PrinterErrorHandler::validate_name("ABC", &existing).is_ok());

        // Tests invalides
        assert!(PrinterErrorHandler::validate_name("", &existing).is_err());
        assert!(PrinterErrorHandler::validate_name("AB", &existing).is_err());
        assert!(PrinterErrorHandler::validate_name("Test<>", &existing).is_err());
        assert!(PrinterErrorHandler::validate_name("imprimante1", &existing).is_err()); // Doublon
    }

    #[test]
    fn test_network_port_validation() {
        assert!(PrinterErrorHandler::validate_network_port("192.168.1.100:9100").is_ok());
        assert!(PrinterErrorHandler::validate_network_port("192.168.1.100").is_err());
        assert!(PrinterErrorHandler::validate_network_port("invalid_ip:9100").is_err());
        assert!(PrinterErrorHandler::validate_network_port("192.168.1.100:0").is_err());
    }

    #[test]
    fn test_paper_width_validation() {
        assert!(PrinterErrorHandler::validate_paper_width("32").is_ok());
        assert!(PrinterErrorHandler::validate_paper_width("48").is_ok());
        assert!(PrinterErrorHandler::validate_paper_width("64").is_ok());
        assert!(PrinterErrorHandler::validate_paper_width("100").is_err());
        assert!(PrinterErrorHandler::validate_paper_width("invalid").is_err());
    }
}
