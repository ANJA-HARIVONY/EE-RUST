import { es } from "./es";
import { fr } from "./fr";
import type { Lang, Messages } from "./types";

export type { Lang };

const STORAGE_KEY = "ee-salon-lang";

function storedLang(): Lang | null {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value === "fr" || value === "es") return value;
  } catch {
    // Hors navigateur, ou stockage indisponible.
  }
  return null;
}

let lang = $state<Lang>(storedLang() ?? "fr");
let chosen = storedLang() !== null;

function applyDocumentLang(next: Lang) {
  if (typeof document !== "undefined") document.documentElement.lang = next;
}

applyDocumentLang(storedLang() ?? "fr");

export function getLang(): Lang {
  return lang;
}

export function setLang(next: Lang) {
  lang = next;
  chosen = true;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Le choix reste valable pour cette session.
  }
  applyDocumentLang(next);
}

export function adoptLocale(locale: string) {
  if (chosen) return;
  lang = locale.toLowerCase().startsWith("es") ? "es" : "fr";
  applyDocumentLang(lang);
}

export function messages(): Messages {
  return lang === "es" ? es : fr;
}

export function uiLocale(): string {
  return lang === "es" ? "es-ES" : "fr-FR";
}

const roleCodes = ["admin", "manager", "receptionist", "stylist"] as const;

export function roleName(code: string, fallback: string): string {
  if ((roleCodes as readonly string[]).includes(code)) {
    return messages().roles[code as (typeof roleCodes)[number]];
  }
  return fallback;
}

export function appointmentStatusCodes() {
  return Object.keys(messages().appointmentStatus);
}

export function appointmentStatusLabel(code: string): string {
  const table = messages().appointmentStatus as Record<string, string>;
  return table[code] ?? code;
}

export function invoiceStatusLabel(code: string): string {
  const table = messages().invoiceStatus as Record<string, string>;
  return table[code] ?? code;
}

export function movementLabel(code: string): string {
  const table = messages().movement as Record<string, string>;
  return table[code] ?? code;
}

export function movementReason(reason: string | null): string {
  if (!reason) return "";
  const sale = /^vente (.+)$/.exec(reason);
  if (sale) return messages().products.saleReason(sale[1]);
  const cancel = /^annulation (.+)$/.exec(reason);
  if (cancel) return messages().products.cancelReason(cancel[1]);
  return reason;
}

const exactEs: Record<string, string> = {
  "client introuvable": "cliente no encontrado",
  "employé introuvable": "empleado no encontrado",
  "aucun employé actif": "ningún empleado activo",
  "la couleur doit être au format #RRGGBB": "el color debe tener el formato #RRGGBB",
  "rôle inconnu": "rol desconocido",
  "il doit rester au moins un administrateur actif": "debe quedar al menos un administrador activo",
  "SALON_NAME est vide": "SALON_NAME está vacío",
  "CURRENCY doit être un code ISO à 3 lettres, par exemple EUR":
    "CURRENCY debe ser un código ISO de 3 letras, por ejemplo EUR",
  "VAT_RATE_BPS doit être compris entre 0 et 10000 (10000 = 100 %)":
    "VAT_RATE_BPS debe estar entre 0 y 10000 (10000 = 100 %)",
  "INVOICE_PREFIX doit contenir 1 à 8 lettres ou chiffres":
    "INVOICE_PREFIX debe contener de 1 a 8 letras o cifras",
  "DEFAULT_USER_ROLE doit être admin, manager, receptionist ou stylist":
    "DEFAULT_USER_ROLE debe ser admin, manager, receptionist o stylist",
  "DEFAULT_USER_COLOR doit être une couleur hexadécimale, par exemple #2F5F8A":
    "DEFAULT_USER_COLOR debe ser un color hexadecimal, por ejemplo #2F5F8A",
  "DATABASE_FILENAME doit être un nom de fichier, sans dossier":
    "DATABASE_FILENAME debe ser un nombre de archivo, sin carpeta",
  "AGENDA_OPEN_HOUR et AGENDA_CLOSE_HOUR doivent être des heures valides":
    "AGENDA_OPEN_HOUR y AGENDA_CLOSE_HOUR deben ser horas válidas",
  "AGENDA_OPEN_HOUR doit être inférieur à AGENDA_CLOSE_HOUR":
    "AGENDA_OPEN_HOUR debe ser anterior a AGENDA_CLOSE_HOUR",
  "AGENDA_SLOT_MINUTES doit être 5, 10, 15, 20, 30 ou 60":
    "AGENDA_SLOT_MINUTES debe ser 5, 10, 15, 20, 30 o 60",
  "LOCALE est vide": "LOCALE está vacío",
  "LOW_STOCK_DEFAULT doit être positif ou nul": "LOW_STOCK_DEFAULT debe ser positivo o cero",
  "le prénom et le nom de l'utilisateur par défaut sont obligatoires":
    "el nombre y el apellido del usuario predeterminado son obligatorios",
  "prestation introuvable": "servicio no encontrado",
  "la durée doit être supérieure à 0": "la duración debe ser mayor que 0",
  "le prix ne peut pas être négatif": "el precio no puede ser negativo",
  "montant ou taux de TVA invalide": "importe o tipo de IVA no válido",
  "montant hors limite": "importe fuera de límite",
  "quantité ou prix invalide": "cantidad o precio no válido",
  "montant de ligne hors limite": "importe de línea fuera de límite",
  "réglages du salon introuvables": "ajustes del salón no encontrados",
  "la devise doit être un code ISO à 3 lettres, par exemple EUR":
    "la moneda debe ser un código ISO de 3 letras, por ejemplo EUR",
  "le taux de TVA doit être compris entre 0 et 10000 points de base":
    "el tipo de IVA debe estar entre 0 y 10000 puntos básicos",
  "le préfixe de facture doit contenir 1 à 8 lettres ou chiffres":
    "el prefijo de factura debe contener de 1 a 8 letras o cifras",
  "produit introuvable": "producto no encontrado",
  "le seuil de stock ne peut pas être négatif": "el umbral de stock no puede ser negativo",
  "stock insuffisant": "stock insuficiente",
  "rendez-vous introuvable": "cita no encontrada",
  "facture introuvable": "factura no encontrada",
  "seule une facture brouillon peut être modifiée": "solo se puede modificar un borrador",
  "ligne introuvable": "línea no encontrada",
  "la quantité entrée doit être positive": "la cantidad de entrada debe ser positiva",
  "la quantité sortie doit être positive": "la cantidad de salida debe ser positiva",
  "la quantité cible ne peut pas être négative": "la cantidad objetivo no puede ser negativa",
  "type de mouvement inconnu": "tipo de movimiento desconocido",
  "choisissez au moins une prestation": "elija al menos un servicio",
  "statut de rendez-vous inconnu": "estado de cita desconocido",
  "l'heure de fin doit être après l'heure de début": "la hora de fin debe ser posterior a la de inicio",
  "cet employé est inactif": "este empleado está inactivo",
  "une prestation ne peut figurer qu'une fois": "un servicio solo puede figurar una vez",
  "cet employé a déjà un rendez-vous sur ce créneau": "este empleado ya tiene una cita en este hueco",
  "date ou heure invalide": "fecha u hora no válida",
  "la quantité doit être positive": "la cantidad debe ser positiva",
  "prestation manquante": "falta el servicio",
  "cette prestation est inactive": "este servicio está inactivo",
  "produit manquant": "falta el producto",
  "ce produit est inactif": "este producto está inactivo",
  "type de ligne inconnu": "tipo de línea desconocido",
  "seule une facture brouillon peut être émise": "solo se puede emitir un borrador",
  "la facture ne contient aucune ligne": "la factura no contiene ninguna línea",
  "seule une facture émise peut être marquée payée": "solo una factura emitida puede marcarse como pagada",
  "cette facture est déjà annulée": "esta factura ya está anulada",
  "ce rendez-vous n'appartient pas à ce client": "esta cita no pertenece a este cliente",
  "un rendez-vous annulé ou absent ne peut pas être facturé":
    "una cita anulada o con ausencia no se puede facturar",
  "ce rendez-vous est déjà facturé": "esta cita ya está facturada",
  "ce rendez-vous n'a aucune prestation": "esta cita no tiene ningún servicio",
  "accès à la base interrompu": "acceso a la base interrumpido",
};

const fieldEs: Record<string, string> = {
  "Le prénom": "El nombre",
  "Le nom": "El apellido",
  "Le nom du salon": "El nombre del salón",
};

export function translateError(message: string): string {
  if (lang !== "es") return message;
  const known = exactEs[message];
  if (known) return known;

  let match = /^(.+) est obligatoire$/.exec(message);
  if (match) return `${fieldEs[match[1]] ?? match[1]} es obligatorio`;
  match = /^(.+) est trop long$/.exec(message);
  if (match) return `${fieldEs[match[1]] ?? match[1]} es demasiado largo`;
  match = /^la prestation « (.+) » est inactive$/.exec(message);
  if (match) return `el servicio « ${match[1]} » está inactivo`;
  match = /^stock insuffisant pour « (.+) »$/.exec(message);
  if (match) return `stock insuficiente para « ${match[1]} »`;
  match = /^impossible de créer le dossier de données : (.+)$/.exec(message);
  if (match) return `no se puede crear la carpeta de datos: ${match[1]}`;
  match = /^(.+) est vide dans \.env$/.exec(message);
  if (match) return `${match[1]} está vacío en .env`;
  match = /^(.+) manquant\. Vérifiez le fichier \.env à la racine du projet\.$/.exec(message);
  if (match) return `falta ${match[1]}. Compruebe el archivo .env en la raíz del proyecto.`;
  match = /^(.+) doit être un entier$/.exec(message);
  if (match) return `${match[1]} debe ser un número entero`;
  match = /^(.+) doit être un entier positif ou nul$/.exec(message);
  if (match) return `${match[1]} debe ser un entero positivo o cero`;
  match = /^rôle inconnu dans DEFAULT_USER_ROLE : (.+)$/.exec(message);
  if (match) return `rol desconocido en DEFAULT_USER_ROLE: ${match[1]}`;
  return message;
}
