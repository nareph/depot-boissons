// src/services/report_generator_service.rs

use crate::{
    error::{AppError, AppResult},
    queries::ReportData,
};
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, dictionary};
use rust_xlsxwriter::{Format, Workbook};
use std::fs;
use std::path::PathBuf;

/// Trouve un chemin de sauvegarde approprié pour les rapports.
fn get_save_path(extension: &str) -> AppResult<String> {
    let base_path = dirs::download_dir().unwrap_or_else(|| PathBuf::from("rapports"));
    fs::create_dir_all(&base_path)?;
    let file_name = format!(
        "rapport_ventes_{}.{}",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        extension
    );
    let full_path = base_path.join(file_name);
    Ok(full_path.to_str().unwrap_or("rapport.fallback").to_string())
}

/// Convertit une chaîne UTF-8 en bytes Latin-1 pour PDF
fn encode_text_for_pdf_bytes(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| match c {
            'é' => 0xE9_u8,
            'è' => 0xE8_u8,
            'ê' => 0xEA_u8,
            'ë' => 0xEB_u8,
            'à' => 0xE0_u8,
            'â' => 0xE2_u8,
            'ä' => 0xE4_u8,
            'ç' => 0xE7_u8,
            'î' => 0xEE_u8,
            'ï' => 0xEF_u8,
            'ô' => 0xF4_u8,
            'ö' => 0xF6_u8,
            'ù' => 0xF9_u8,
            'û' => 0xFB_u8,
            'ü' => 0xFC_u8,
            'ÿ' => 0xFF_u8,
            'É' => 0xC9_u8,
            'È' => 0xC8_u8,
            'Ê' => 0xCA_u8,
            'Ë' => 0xCB_u8,
            'À' => 0xC0_u8,
            'Â' => 0xC2_u8,
            'Ä' => 0xC4_u8,
            'Ç' => 0xC7_u8,
            'Î' => 0xCE_u8,
            'Ï' => 0xCF_u8,
            'Ô' => 0xD4_u8,
            'Ö' => 0xD6_u8,
            'Ù' => 0xD9_u8,
            'Û' => 0xDB_u8,
            'Ü' => 0xDC_u8,
            'Ÿ' => 0x9F_u8, // Ÿ en Windows-1252
            _ => {
                if c.is_ascii() {
                    c as u8
                } else {
                    b'?' // Caractère de remplacement pour les autres
                }
            }
        })
        .collect()
}

/// Alternative : conversion des caractères spéciaux en représentation octal
fn encode_text_for_pdf_octal(text: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for c in text.chars() {
        match c {
            'é' => result.push(0xE9),
            'è' => result.push(0xE8),
            'ê' => result.push(0xEA),
            'ë' => result.push(0xEB),
            'à' => result.push(0xE0),
            'â' => result.push(0xE2),
            'ä' => result.push(0xE4),
            'ç' => result.push(0xE7),
            'î' => result.push(0xEE),
            'ï' => result.push(0xEF),
            'ô' => result.push(0xF4),
            'ö' => result.push(0xF6),
            'ù' => result.push(0xF9),
            'û' => result.push(0xFB),
            'ü' => result.push(0xFC),
            'ÿ' => result.push(0xFF),
            'É' => result.push(0xC9),
            'È' => result.push(0xC8),
            'Ê' => result.push(0xCA),
            'Ë' => result.push(0xCB),
            'À' => result.push(0xC0),
            'Â' => result.push(0xC2),
            'Ä' => result.push(0xC4),
            'Ç' => result.push(0xC7),
            'Î' => result.push(0xCE),
            'Ï' => result.push(0xCF),
            'Ô' => result.push(0xD4),
            'Ö' => result.push(0xD6),
            'Ù' => result.push(0xD9),
            'Û' => result.push(0xDB),
            'Ü' => result.push(0xDC),
            _ => {
                if c.is_ascii() {
                    result.push(c as u8);
                } else {
                    result.push(b'?'); // Caractère de remplacement
                }
            }
        }
    }
    result
}

// --- GÉNÉRATION PDF ---

pub fn generate_pdf_report(data: &ReportData) -> AppResult<String> {
    let file_path = get_save_path("pdf")?;

    // Créer un nouveau document PDF
    let mut doc = Document::with_version("1.7");

    // Créer l'arborescence des pages
    let pages_id = doc.new_object_id();

    // Ajouter les polices avec encodage WinAnsi
    let font_regular = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });

    let font_bold = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
        "Encoding" => "WinAnsiEncoding",
    });

    // Ressources pour les polices
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_regular,
            "F2" => font_bold,
        },
    });

    // Créer le contenu de la page
    let mut content = Content {
        operations: Vec::new(),
    };

    // Fonction helper pour ajouter du texte avec gestion des accents
    fn add_text(content: &mut Content, text: &str, font: &str, size: f64, x: f64, y: f64) {
        // Encoder le texte en bytes pour le PDF
        let encoded_bytes = encode_text_for_pdf_bytes(text);

        content.operations.push(Operation::new("BT", vec![]));
        content
            .operations
            .push(Operation::new("Tf", vec![font.into(), size.into()]));
        content
            .operations
            .push(Operation::new("Td", vec![x.into(), y.into()]));

        // Créer un objet String à partir des bytes
        let text_object = Object::String(encoded_bytes, lopdf::StringFormat::Literal);
        content
            .operations
            .push(Operation::new("Tj", vec![text_object]));
        content.operations.push(Operation::new("ET", vec![]));
    }

    // Alternative avec bytes pour un contrôle plus fin
    fn add_text_with_bytes(
        content: &mut Content,
        text: &str,
        font: &str,
        size: f64,
        x: f64,
        y: f64,
    ) {
        let encoded_bytes = encode_text_for_pdf_octal(text);

        content.operations.push(Operation::new("BT", vec![]));
        content
            .operations
            .push(Operation::new("Tf", vec![font.into(), size.into()]));
        content
            .operations
            .push(Operation::new("Td", vec![x.into(), y.into()]));

        // Créer un objet string à partir des bytes
        let text_object = Object::String(encoded_bytes, lopdf::StringFormat::Literal);
        content
            .operations
            .push(Operation::new("Tj", vec![text_object]));
        content.operations.push(Operation::new("ET", vec![]));
    }

    // Position Y initiale (en partant du haut)
    let mut y_pos = 800.0;

    // Titre principal
    add_text(&mut content, "Rapport de Ventes", "F2", 24.0, 50.0, y_pos);
    y_pos -= 30.0;

    // Ligne de séparation
    content.operations.push(Operation::new("q", vec![]));
    content
        .operations
        .push(Operation::new("w", vec![1.0.into()]));
    content
        .operations
        .push(Operation::new("m", vec![50.0.into(), y_pos.into()]));
    content
        .operations
        .push(Operation::new("l", vec![545.0.into(), y_pos.into()]));
    content.operations.push(Operation::new("S", vec![]));
    content.operations.push(Operation::new("Q", vec![]));
    y_pos -= 20.0;

    // Section Statistiques Clés
    add_text(&mut content, "Statistiques Clés", "F2", 18.0, 50.0, y_pos);
    y_pos -= 25.0;

    add_text(
        &mut content,
        &format!("Chiffre d'Affaires: {} XAF", data.total_revenue),
        "F1",
        12.0,
        50.0,
        y_pos,
    );
    y_pos -= 20.0;

    add_text(
        &mut content,
        &format!("Nombre de Ventes: {}", data.total_sales),
        "F1",
        12.0,
        50.0,
        y_pos,
    );
    y_pos -= 30.0;

    // Section Top Produits
    add_text(
        &mut content,
        "Top 5 Produits Vendus (par quantité)",
        "F2",
        18.0,
        50.0,
        y_pos,
    );
    y_pos -= 25.0;

    for (i, (product, quantity)) in data.top_products.iter().enumerate() {
        if y_pos < 50.0 {
            break; // Éviter de dépasser le bas de page
        }
        add_text(
            &mut content,
            &format!(
                "#{}. {} - ({} unités vendues)",
                i + 1,
                product.name,
                quantity
            ),
            "F1",
            12.0,
            50.0,
            y_pos,
        );
        y_pos -= 15.0;
    }

    // Section additionnelle avec informations sur l'encodage
    y_pos -= 20.0;
    add_text(&mut content, "Généré avec succès", "F1", 10.0, 50.0, y_pos);
    y_pos -= 15.0;
    add_text(
        &mut content,
        &format!("Date: {}", chrono::Local::now().format("%d/%m/%Y à %H:%M")),
        "F1",
        10.0,
        50.0,
        y_pos,
    );

    // Ajouter le contenu à la page
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // Créer la page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    });

    // Ajouter la page à l'arborescence
    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    // Créer le catalogue
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });

    // Définir la racine du document
    doc.trailer.set("Root", catalog_id);

    // Compresser le document
    doc.compress();

    // Sauvegarder le document
    doc.save(&file_path)?;

    log::info!("Rapport PDF généré avec succès : {}", file_path);
    Ok(file_path)
}

// --- GÉNÉRATION EXCEL ---
pub fn generate_excel_report(data: &ReportData) -> AppResult<String> {
    let file_path = get_save_path("xlsx")?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let title_format = Format::new().set_bold().set_font_size(16.0);
    let header_format = Format::new().set_bold().set_background_color("#DDEBF7");
    let money_format = Format::new().set_num_format("#,##0 \"XAF\"");

    worksheet.set_column_width(0, 30.0)?;
    worksheet.set_column_width(1, 15.0)?;

    worksheet.write_string_with_format(0, 0, "Rapport de Ventes", &title_format)?;
    worksheet.write_string_with_format(2, 0, "Statistiques Clés", &header_format)?;
    worksheet.write_string(3, 0, "Chiffre d'Affaires Total")?;
    worksheet.write_number_with_format(
        3,
        1,
        data.total_revenue.to_string().parse().unwrap_or(0.0),
        &money_format,
    )?;
    worksheet.write_string(4, 0, "Nombre de Ventes")?;
    worksheet.write_number(4, 1, data.total_sales as f64)?;
    worksheet.write_string_with_format(6, 0, "Top 5 Produits Vendus", &header_format)?;
    worksheet.write_string(7, 0, "Produit")?;
    worksheet.write_string(7, 1, "Quantité Vendue")?;
    for (i, (product, quantity)) in data.top_products.iter().enumerate() {
        worksheet.write_string(8 + i as u32, 0, &product.name)?;
        worksheet.write_number(8 + i as u32, 1, *quantity as f64)?;
    }

    workbook.save(&file_path)?;
    log::info!("Rapport Excel généré avec succès : {}", file_path);
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_text_for_pdf_bytes() {
        let text = "Café français avec des accents: é è ê à ç";
        let encoded = encode_text_for_pdf_bytes(text);
        // Vérifier que les bytes correspondent aux caractères attendus
        assert!(encoded.contains(&0xE9)); // é
        assert!(encoded.contains(&0xE8)); // è
        assert!(encoded.contains(&0xEA)); // ê
        assert!(encoded.contains(&0xE0)); // à
        assert!(encoded.contains(&0xE7)); // ç
    }

    #[test]
    fn test_encode_cafe() {
        let text = "Café";
        let encoded = encode_text_for_pdf_bytes(text);
        assert_eq!(encoded, vec![67, 97, 102, 0xE9]); // C a f é
    }
}
