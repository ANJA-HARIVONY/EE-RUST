use crate::error::AppError;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub salon_name: String,
    pub salon_address: String,
    pub salon_phone: String,
    pub salon_email: String,
    pub salon_siret: String,
    pub currency: String,
    pub vat_rate_bps: i64,
    pub invoice_prefix: String,
    pub default_user_first_name: String,
    pub default_user_last_name: String,
    pub default_user_email: String,
    pub default_user_phone: String,
    pub default_user_role: String,
    pub default_user_color: String,
    pub database_filename: String,
    pub agenda_open_hour: u32,
    pub agenda_close_hour: u32,
    pub agenda_slot_minutes: u32,
    pub locale: String,
    pub low_stock_default: i64,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        load_dotenv();
        let cfg = Self {
            salon_name: required("SALON_NAME")?,
            salon_address: optional("SALON_ADDRESS"),
            salon_phone: optional("SALON_PHONE"),
            salon_email: optional("SALON_EMAIL"),
            salon_siret: optional("SALON_SIRET"),
            currency: required("CURRENCY")?.to_uppercase(),
            vat_rate_bps: required_i64("VAT_RATE_BPS")?,
            invoice_prefix: required("INVOICE_PREFIX")?.to_uppercase(),
            default_user_first_name: required("DEFAULT_USER_FIRST_NAME")?,
            default_user_last_name: required("DEFAULT_USER_LAST_NAME")?,
            default_user_email: optional("DEFAULT_USER_EMAIL"),
            default_user_phone: optional("DEFAULT_USER_PHONE"),
            default_user_role: required("DEFAULT_USER_ROLE")?.to_lowercase(),
            default_user_color: required("DEFAULT_USER_COLOR")?,
            database_filename: required("DATABASE_FILENAME")?,
            agenda_open_hour: required_u32("AGENDA_OPEN_HOUR")?,
            agenda_close_hour: required_u32("AGENDA_CLOSE_HOUR")?,
            agenda_slot_minutes: required_u32("AGENDA_SLOT_MINUTES")?,
            locale: required("LOCALE")?,
            low_stock_default: required_i64("LOW_STOCK_DEFAULT")?,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.salon_name.trim().is_empty() {
            return Err(AppError::message("SALON_NAME est vide"));
        }
        if self.currency.len() != 3 || !self.currency.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(AppError::message(
                "CURRENCY doit être un code ISO à 3 lettres, par exemple EUR",
            ));
        }
        if !(0..=10_000).contains(&self.vat_rate_bps) {
            return Err(AppError::message(
                "VAT_RATE_BPS doit être compris entre 0 et 10000 (10000 = 100 %)",
            ));
        }
        if self.invoice_prefix.is_empty()
            || self.invoice_prefix.len() > 8
            || !self
                .invoice_prefix
                .chars()
                .all(|c| c.is_ascii_alphanumeric())
        {
            return Err(AppError::message(
                "INVOICE_PREFIX doit contenir 1 à 8 lettres ou chiffres",
            ));
        }
        const ROLES: [&str; 4] = ["admin", "manager", "receptionist", "stylist"];
        if !ROLES.contains(&self.default_user_role.as_str()) {
            return Err(AppError::message(
                "DEFAULT_USER_ROLE doit être admin, manager, receptionist ou stylist",
            ));
        }
        if !is_hex_color(&self.default_user_color) {
            return Err(AppError::message(
                "DEFAULT_USER_COLOR doit être une couleur hexadécimale, par exemple #2F5F8A",
            ));
        }
        if self.database_filename.trim().is_empty()
            || self.database_filename.contains('/')
            || self.database_filename.contains('\\')
        {
            return Err(AppError::message(
                "DATABASE_FILENAME doit être un nom de fichier, sans dossier",
            ));
        }
        if self.agenda_open_hour >= 24 || self.agenda_close_hour > 24 {
            return Err(AppError::message(
                "AGENDA_OPEN_HOUR et AGENDA_CLOSE_HOUR doivent être des heures valides",
            ));
        }
        if self.agenda_open_hour >= self.agenda_close_hour {
            return Err(AppError::message(
                "AGENDA_OPEN_HOUR doit être inférieur à AGENDA_CLOSE_HOUR",
            ));
        }
        const SLOTS: [u32; 6] = [5, 10, 15, 20, 30, 60];
        if !SLOTS.contains(&self.agenda_slot_minutes) {
            return Err(AppError::message(
                "AGENDA_SLOT_MINUTES doit être 5, 10, 15, 20, 30 ou 60",
            ));
        }
        if self.locale.trim().is_empty() {
            return Err(AppError::message("LOCALE est vide"));
        }
        if self.low_stock_default < 0 {
            return Err(AppError::message("LOW_STOCK_DEFAULT doit être positif ou nul"));
        }
        if self.default_user_first_name.trim().is_empty()
            || self.default_user_last_name.trim().is_empty()
        {
            return Err(AppError::message(
                "le prénom et le nom de l'utilisateur par défaut sont obligatoires",
            ));
        }
        Ok(())
    }
}

pub fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

fn load_dotenv() {
    let mut paths: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join(".env"));
        paths.push(cwd.join("../.env"));
    }
    paths.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env"));
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join(".env"));
            paths.push(dir.join("../Resources/.env"));
        }
    }
    for path in paths {
        if path.is_file() {
            let _ = dotenvy::from_path(path);
            return;
        }
    }
}

fn required(key: &str) -> Result<String, AppError> {
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                Err(AppError::message(format!("{key} est vide dans .env")))
            } else {
                Ok(trimmed)
            }
        }
        Err(_) => Err(AppError::message(format!(
            "{key} manquant. Vérifiez le fichier .env à la racine du projet."
        ))),
    }
}

fn optional(key: &str) -> String {
    env::var(key).unwrap_or_default().trim().to_string()
}

fn required_i64(key: &str) -> Result<i64, AppError> {
    let raw = required(key)?;
    raw.parse::<i64>()
        .map_err(|_| AppError::message(format!("{key} doit être un entier")))
}

fn required_u32(key: &str) -> Result<u32, AppError> {
    let raw = required(key)?;
    raw.parse::<u32>()
        .map_err(|_| AppError::message(format!("{key} doit être un entier positif ou nul")))
}
