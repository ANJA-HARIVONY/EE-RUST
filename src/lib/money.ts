export function formatMoney(cents: number, currency = "EUR", locale = "fr-FR"): string {
  return new Intl.NumberFormat(locale, { style: "currency", currency }).format(cents / 100);
}

export function centsToInput(cents: number): string {
  return (cents / 100).toFixed(2).replace(".", ",");
}

export function inputToCents(value: string): number | null {
  const normalized = value.trim().replace(",", ".");
  if (!/^\d+(\.\d{1,2})?$/.test(normalized)) return null;
  const [whole, frac = ""] = normalized.split(".");
  return Number(whole) * 100 + Number(frac.padEnd(2, "0"));
}

export function formatVat(bps: number): string {
  return `${(bps / 100).toFixed(2).replace(".", ",")} %`;
}
