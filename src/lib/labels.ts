export const appointmentStatus: Record<string, string> = {
  scheduled: "Planifié",
  confirmed: "Confirmé",
  in_progress: "En cours",
  completed: "Terminé",
  cancelled: "Annulé",
  no_show: "Absent",
};

export const invoiceStatus: Record<string, string> = {
  draft: "Brouillon",
  issued: "Émise",
  paid: "Payée",
  cancelled: "Annulée",
};

export const movementKind: Record<string, string> = {
  in: "Entrée",
  out: "Sortie",
  adjustment: "Ajustement",
  sale: "Vente",
};

export function errText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Erreur inattendue";
}

export function fullName(first: string, last: string): string {
  return `${first} ${last}`.trim();
}
