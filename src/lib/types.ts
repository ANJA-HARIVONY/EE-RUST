export type Role = {
  id: number;
  code: string;
  label: string;
};

export type Employee = {
  id: number;
  firstName: string;
  lastName: string;
  email: string | null;
  phone: string | null;
  roleId: number;
  roleCode: string;
  roleLabel: string;
  color: string;
  active: boolean;
  notes: string | null;
};

export type Client = {
  id: number;
  firstName: string;
  lastName: string;
  email: string | null;
  phone: string | null;
  notes: string | null;
};

export type Service = {
  id: number;
  name: string;
  description: string | null;
  durationMinutes: number;
  priceCents: number;
  active: boolean;
};

export type Product = {
  id: number;
  sku: string | null;
  name: string;
  description: string | null;
  priceCents: number;
  stockQty: number;
  lowStockThreshold: number;
  active: boolean;
  lowStock: boolean;
};

export type StockMovement = {
  id: number;
  kind: string;
  quantity: number;
  reason: string | null;
  createdAt: string;
};

export type AppointmentService = {
  serviceId: number;
  name: string;
  durationMinutes: number;
  priceCents: number;
};

export type Appointment = {
  id: number;
  clientId: number;
  clientName: string;
  employeeId: number;
  employeeName: string;
  employeeColor: string;
  startsAt: string;
  endsAt: string;
  status: string;
  notes: string | null;
  services: AppointmentService[];
};

export type InvoiceLine = {
  id: number;
  kind: string;
  serviceId: number | null;
  productId: number | null;
  appointmentId: number | null;
  description: string;
  quantity: number;
  unitPriceCents: number;
  lineTotalCents: number;
};

export type Invoice = {
  id: number;
  number: string;
  clientId: number;
  clientName: string;
  employeeId: number | null;
  employeeName: string | null;
  status: string;
  issuedAt: string | null;
  paidAt: string | null;
  notes: string | null;
  currency: string;
  vatRateBps: number;
  subtotalCents: number;
  vatCents: number;
  totalCents: number;
  lines: InvoiceLine[];
};

export type InvoiceSummary = {
  id: number;
  number: string;
  clientName: string;
  status: string;
  totalCents: number;
  currency: string;
  updatedAt: string;
};

export type SalonSettings = {
  name: string;
  address: string | null;
  phone: string | null;
  email: string | null;
  siret: string | null;
  currency: string;
  vatRateBps: number;
  invoicePrefix: string;
  nextInvoiceSeq: number;
};

export type Bootstrap = {
  settings: SalonSettings;
  currentUser: Employee;
  agendaOpenHour: number;
  agendaCloseHour: number;
  agendaSlotMinutes: number;
  locale: string;
  lowStockDefault: number;
  databasePath: string;
  lowStockCount: number;
};

export type ModuleId =
  | "agenda"
  | "clients"
  | "services"
  | "products"
  | "employees"
  | "invoices"
  | "settings";
