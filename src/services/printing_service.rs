// src/services/printing_service.rs

use crate::{
    config::printer_config::{PrinterConfig, PrinterType},
    error::{AppError, AppResult},
    models::Receipt,
};
use escpos_rs::{Instruction, Justification, PrintData, Printer, PrinterProfile, command::Font};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::net::TcpStream;
use std::process::Command;

// Le service d'impression ne gère plus la configuration, il ne fait qu'imprimer.

fn create_printer_from_config(config: &PrinterConfig) -> AppResult<Option<Printer>> {
    let printer_profile = match config.printer_type {
        PrinterType::USB => {
            // For USB printers, we need vendor and product IDs
            // These should ideally be part of the PrinterConfig
            // For now, using placeholder values - you'll need to configure these
            PrinterProfile::usb_builder(0x04b8, 0x0202) // Example: Epson TM-T20
                .with_font_width(Font::FontA, config.paper_width as u8)
                .build()
        }
        PrinterType::Network => {
            // For network printers, escpos-rs doesn't have a network_builder
            // We'll need to handle network printing differently
            // Fall back to file-based printing for now
            return Ok(None);
        }
        PrinterType::Serial => {
            // For serial printers, escpos-rs doesn't have a serial_builder
            // We'll need to handle serial printing differently
            // Fall back to file-based printing for now
            return Ok(None);
        }
        PrinterType::Windows => {
            // For Windows printers, we'll fall back to file-based printing
            // since escpos-rs doesn't directly support Windows printer names
            return Ok(None);
        }
    };

    match Printer::new(printer_profile) {
        Ok(maybe_printer) => Ok(maybe_printer),
        Err(e) => {
            log::error!("Failed to create printer: {}", e);
            Err(AppError::from(e))
        }
    }
}

fn print_receipt_with_escpos(receipt: &Receipt, config: &PrinterConfig) -> AppResult<()> {
    let printer = match create_printer_from_config(config)? {
        Some(printer) => printer,
        None => {
            // Fall back to alternative printing methods
            return print_receipt_alternative(receipt, config);
        }
    };

    // Print header
    let header_instruction =
        Instruction::text("DEPOT BOISSONS", Font::FontB, Justification::Center, None);
    printer.instruction(&header_instruction, None)?;

    let subheader_instruction = Instruction::text(
        "Votre Partenaire Fraîcheur",
        Font::FontA,
        Justification::Center,
        None,
    );
    printer.instruction(&subheader_instruction, None)?;

    // Print receipt info
    printer.println("")?;
    printer.println(&format!("Ticket N°: {}", receipt.sale_number))?;
    printer.println(&format!("Date:      {}", receipt.date))?;
    printer.println(&format!("Vendeur:   {}", receipt.seller_name))?;
    printer.println("")?;

    // Print separator
    let separator = "-".repeat(config.paper_width as usize);
    printer.println(&separator)?;

    // Print items header
    let header_line = format!(
        "{:<w1$} {:>w2$} {:>w3$} {:>w4$}",
        "PRODUIT",
        "QTE",
        "P.U.",
        "TOTAL",
        w1 = config.paper_width as usize - 20,
        w2 = 3,
        w3 = 7,
        w4 = 8
    );
    printer.println(&header_line)?;
    printer.println(&separator)?;

    // Print items
    for item in &receipt.items {
        printer.println(&item.product_name)?;
        let item_line = format!(
            "{:>w1$} x {:>w2$} = {:>w3$}",
            item.quantity,
            item.unit_price,
            item.total_price,
            w1 = config.paper_width as usize - 22,
            w2 = 8,
            w3 = 10
        );
        printer.println(&item_line)?;
    }

    // Print total
    printer.println(&separator)?;
    printer.println("")?;

    let total_instruction = Instruction::text(
        &format!("TOTAL: {} XAF", receipt.total_amount),
        Font::FontB,
        Justification::Right,
        None,
    );
    printer.instruction(&total_instruction, None)?;

    printer.println("")?;
    printer.println("")?;

    // Print footer
    let footer_instruction = Instruction::text(
        "Merci pour votre achat !",
        Font::FontA,
        Justification::Center,
        None,
    );
    printer.instruction(&footer_instruction, None)?;

    printer.println("")?;
    printer.println("")?;
    printer.println("")?;

    // Cut paper if supported
    // Note: escpos-rs might have a cut method, check the documentation
    printer.println("\x1D\x56\x00")?; // ESC/POS cut command

    Ok(())
}

fn print_test_with_escpos(config: &PrinterConfig) -> AppResult<()> {
    let printer = match create_printer_from_config(config)? {
        Some(printer) => printer,
        None => {
            return print_test_alternative(config);
        }
    };

    let test_instruction = Instruction::text(
        "TEST D'IMPRESSION",
        Font::FontB,
        Justification::Center,
        None,
    );
    printer.instruction(&test_instruction, None)?;

    printer.println("")?;
    printer.println(&format!("Imprimante: {}", config.name))?;
    printer.println("")?;
    printer.println("")?;
    printer.println("")?;

    // Cut paper
    printer.println("\x1D\x56\x00")?;

    Ok(())
}

// Alternative printing methods for network, serial, and Windows printers
fn print_receipt_alternative(receipt: &Receipt, config: &PrinterConfig) -> AppResult<()> {
    let content = format_receipt_as_text(receipt, config);

    match config.printer_type {
        PrinterType::Network => print_to_network(&content, config),
        PrinterType::Serial => print_to_serial(&content, config),
        PrinterType::Windows => print_receipt_to_file(receipt, config),
        _ => {
            // Fallback to file
            print_receipt_to_file(receipt, config)
        }
    }
}

fn print_test_alternative(config: &PrinterConfig) -> AppResult<()> {
    let content = format!("TEST D'IMPRESSION\n\nImprimante: {}\n\n\n", config.name);

    match config.printer_type {
        PrinterType::Network => print_to_network(&content, config),
        PrinterType::Serial => print_to_serial(&content, config),
        PrinterType::Windows => print_test_to_file(config),
        _ => {
            // Fallback to file
            print_test_to_file(config)
        }
    }
}

fn print_to_network(content: &str, config: &PrinterConfig) -> AppResult<()> {
    // Parse IP and port from config.port (e.g., "192.168.1.100:9100")
    let parts: Vec<&str> = config.port.split(':').collect();
    let ip = parts.get(0).unwrap_or(&"192.168.1.100");
    let port = parts
        .get(1)
        .unwrap_or(&"9100")
        .parse::<u16>()
        .unwrap_or(9100);

    let address = format!("{}:{}", ip, port);

    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            // Send ESC/POS commands directly
            let esc_pos_content = convert_to_escpos(content);
            stream.write_all(esc_pos_content.as_bytes())?;
            stream.flush()?;
            log::info!("Successfully printed to network printer at {}", address);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to connect to network printer {}: {}", address, e);
            Err(AppError::from(e))
        }
    }
}

fn print_to_serial(content: &str, config: &PrinterConfig) -> AppResult<()> {
    // For serial printing, we'll try to write directly to the serial port
    match OpenOptions::new().write(true).open(&config.port) {
        Ok(mut file) => {
            let esc_pos_content = convert_to_escpos(content);
            file.write_all(esc_pos_content.as_bytes())?;
            file.flush()?;
            log::info!("Successfully printed to serial printer at {}", config.port);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to open serial port {}: {}", config.port, e);
            Err(AppError::from(e))
        }
    }
}

fn convert_to_escpos(content: &str) -> String {
    // Convert plain text to ESC/POS commands
    let mut esc_pos = String::new();

    // Initialize printer
    esc_pos.push_str("\x1B\x40"); // ESC @ - Initialize printer

    // Add the content
    esc_pos.push_str(content);

    // Cut paper
    esc_pos.push_str("\x1D\x56\x00"); // GS V 0 - Cut paper

    esc_pos
}

// Fallback methods for file-based printing (Windows or when direct printing fails)
fn print_receipt_to_file(receipt: &Receipt, config: &PrinterConfig) -> AppResult<()> {
    let content = format_receipt_as_text(receipt, config);
    let filename = format!("{}_receipt.txt", config.name);
    std::fs::write(&filename, content)?;
    log::info!("Receipt saved to file: {}", filename);

    // Try to send to printer via system command
    send_file_to_printer(&filename, config)
}

fn print_test_to_file(config: &PrinterConfig) -> AppResult<()> {
    let content = format!("TEST D'IMPRESSION\n\nImprimante: {}\n\n\n", config.name);
    let filename = format!("{}_test.txt", config.name);
    std::fs::write(&filename, content)?;
    log::info!("Test page saved to file: {}", filename);

    // Try to send to printer via system command
    send_file_to_printer(&filename, config)
}

fn format_receipt_as_text(receipt: &Receipt, config: &PrinterConfig) -> String {
    let width = config.paper_width as usize;
    let mut content = String::new();

    // Header
    content.push_str(&format!("{:^width$}\n", "DEPOT BOISSONS", width = width));
    content.push_str(&format!(
        "{:^width$}\n",
        "Votre Partenaire Fraîcheur",
        width = width
    ));
    content.push_str("\n");

    // Receipt info
    content.push_str(&format!("Ticket N°: {}\n", receipt.sale_number));
    content.push_str(&format!("Date:      {}\n", receipt.date));
    content.push_str(&format!("Vendeur:   {}\n", receipt.seller_name));
    content.push_str("\n");

    // Items
    content.push_str(&format!("{}\n", "-".repeat(width)));
    content.push_str(&format!(
        "{:<w1$} {:>w2$} {:>w3$} {:>w4$}\n",
        "PRODUIT",
        "QTE",
        "P.U.",
        "TOTAL",
        w1 = width - 20,
        w2 = 3,
        w3 = 7,
        w4 = 8
    ));
    content.push_str(&format!("{}\n", "-".repeat(width)));

    for item in &receipt.items {
        content.push_str(&format!("{}\n", item.product_name));
        content.push_str(&format!(
            "{:>w1$} x {:>w2$} = {:>w3$}\n",
            item.quantity,
            item.unit_price,
            item.total_price,
            w1 = width - 22,
            w2 = 8,
            w3 = 10
        ));
    }

    // Total
    content.push_str(&format!("{}\n", "-".repeat(width)));
    content.push_str("\n");
    content.push_str(&format!(
        "{:>width$}\n",
        format!("TOTAL: {} XAF", receipt.total_amount),
        width = width
    ));
    content.push_str("\n\n");

    // Footer
    content.push_str(&format!(
        "{:^width$}\n",
        "Merci pour votre achat !",
        width = width
    ));
    content.push_str("\n\n\n");

    content
}

fn send_file_to_printer(filename: &str, config: &PrinterConfig) -> AppResult<()> {
    match config.printer_type {
        PrinterType::Windows => {
            #[cfg(windows)]
            {
                let output = Command::new("copy")
                    .arg(filename)
                    .arg(&config.port)
                    .arg("/B")
                    .output();

                match output {
                    Ok(result) => {
                        if result.status.success() {
                            log::info!("File printed successfully to {}", config.port);
                        } else {
                            log::error!("Print error: {}", String::from_utf8_lossy(&result.stderr));
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to execute print command: {}", e);
                    }
                }
            }

            #[cfg(not(windows))]
            {
                let output = Command::new("lpr")
                    .arg("-P")
                    .arg(&config.port)
                    .arg(filename)
                    .output();

                match output {
                    Ok(result) => {
                        if result.status.success() {
                            log::info!("File printed successfully to {}", config.port);
                        } else {
                            log::error!("Print error: {}", String::from_utf8_lossy(&result.stderr));
                        }
                    }
                    Err(e) => {
                        log::warn!("lpr not available: {}", e);
                    }
                }
            }
        }
        _ => {
            // For other printer types, the file has already been created
            // The escpos-rs library should handle direct printing
            log::info!("File created: {}", filename);
        }
    }

    Ok(())
}

/// Imprime un reçu en utilisant la configuration fournie.
pub fn print_receipt(receipt: &Receipt, config: &PrinterConfig) -> AppResult<()> {
    log::info!("Printing receipt for sale #{}", receipt.sale_number);
    print_receipt_with_escpos(receipt, config)
}

/// Imprime une page de test en utilisant la configuration fournie.
pub fn print_test_page(config: &PrinterConfig) -> AppResult<()> {
    log::info!("Printing test page for printer '{}'", config.name);
    print_test_with_escpos(config)
}

/// Vérifie si une imprimante est disponible
pub fn test_printer_connection(config: &PrinterConfig) -> AppResult<bool> {
    log::info!("Testing connection for printer '{}'", config.name);

    match config.printer_type {
        PrinterType::USB => {
            // Try to create a USB printer instance
            match create_printer_from_config(config) {
                Ok(Some(_printer)) => {
                    log::info!("USB printer connection successful");
                    Ok(true)
                }
                Ok(None) => Ok(false),
                Err(e) => {
                    log::error!("USB printer connection failed: {}", e);
                    Ok(false)
                }
            }
        }
        PrinterType::Network => {
            // Test network connection
            let parts: Vec<&str> = config.port.split(':').collect();
            let ip = parts.get(0).unwrap_or(&"192.168.1.100");
            let port = parts
                .get(1)
                .unwrap_or(&"9100")
                .parse::<u16>()
                .unwrap_or(9100);

            let address = format!("{}:{}", ip, port);

            match TcpStream::connect(&address) {
                Ok(_) => {
                    log::info!("Network printer connection successful");
                    Ok(true)
                }
                Err(e) => {
                    log::error!("Network printer connection failed: {}", e);
                    Ok(false)
                }
            }
        }
        PrinterType::Serial => {
            // Test serial connection
            match OpenOptions::new().read(true).open(&config.port) {
                Ok(_) => {
                    log::info!("Serial printer connection successful");
                    Ok(true)
                }
                Err(e) => {
                    log::error!("Serial printer connection failed: {}", e);
                    Ok(false)
                }
            }
        }
        PrinterType::Windows => {
            // Test Windows printer availability
            #[cfg(windows)]
            {
                let output = Command::new("wmic")
                    .args(&["printer", "get", "name"])
                    .output();

                match output {
                    Ok(result) => {
                        let output_str = String::from_utf8_lossy(&result.stdout);
                        Ok(output_str.contains(&config.port))
                    }
                    Err(_) => Ok(false),
                }
            }

            #[cfg(not(windows))]
            {
                let output = Command::new("lpstat").arg("-p").arg(&config.port).output();

                match output {
                    Ok(result) => Ok(result.status.success()),
                    Err(_) => Ok(false),
                }
            }
        }
    }
}

/// Liste les imprimantes disponibles sur le système
pub fn list_available_printers() -> AppResult<Vec<String>> {
    let mut printers = Vec::new();

    #[cfg(windows)]
    {
        // List Windows printers
        let output = Command::new("wmic")
            .args(&["printer", "get", "name"])
            .output();

        if let Ok(result) = output {
            let output_str = String::from_utf8_lossy(&result.stdout);
            for line in output_str.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed != "Name" {
                    printers.push(trimmed.to_string());
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        // List Unix/Linux printers
        let output = Command::new("lpstat").arg("-p").output();

        if let Ok(result) = output {
            let output_str = String::from_utf8_lossy(&result.stdout);
            for line in output_str.lines() {
                if let Some(name) = line.split_whitespace().nth(1) {
                    printers.push(name.to_string());
                }
            }
        }
    }

    Ok(printers)
}
