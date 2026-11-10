import { invoke } from "@tauri-apps/api/core";
import type {
  Appointment,
  Bootstrap,
  Client,
  Employee,
  Invoice,
  InvoiceSummary,
  Product,
  Role,
  SalonSettings,
  Service,
  StockMovement,
} from "./types";

export const api = {
  bootstrap: () => invoke<Bootstrap>("app_bootstrap"),
  settingsSave: (input: {
    name: string;
    address: string;
    phone: string;
    email: string;
    siret: string;
    currency: string;
    vatRateBps: number;
    invoicePrefix: string;
  }) => invoke<SalonSettings>("settings_save", { input }),
  lowStockCount: () => invoke<number>("low_stock_count"),

  clientsList: (query: string) => invoke<Client[]>("clients_list", { query }),
  clientsSave: (input: {
    id: number | null;
    firstName: string;
    lastName: string;
    email: string;
    phone: string;
    notes: string;
  }) => invoke<Client>("clients_save", { input }),

  rolesList: () => invoke<Role[]>("roles_list"),
  employeesList: (includeInactive: boolean) =>
    invoke<Employee[]>("employees_list", { includeInactive }),
  employeesSave: (input: {
    id: number | null;
    firstName: string;
    lastName: string;
    email: string;
    phone: string;
    roleId: number;
    color: string;
    active: boolean;
    notes: string;
  }) => invoke<Employee>("employees_save", { input }),
  employeesSetActive: (id: number, active: boolean) =>
    invoke<Employee>("employees_set_active", { id, active }),

  servicesList: (includeInactive: boolean) =>
    invoke<Service[]>("services_list", { includeInactive }),
  servicesSave: (input: {
    id: number | null;
    name: string;
    description: string;
    durationMinutes: number;
    priceCents: number;
    active: boolean;
  }) => invoke<Service>("services_save", { input }),
  servicesSetActive: (id: number, active: boolean) =>
    invoke<Service>("services_set_active", { id, active }),

  productsList: (includeInactive: boolean) =>
    invoke<Product[]>("products_list", { includeInactive }),
  productsSave: (input: {
    id: number | null;
    sku: string;
    name: string;
    description: string;
    priceCents: number;
    lowStockThreshold: number;
    active: boolean;
  }) => invoke<Product>("products_save", { input }),
  productsSetActive: (id: number, active: boolean) =>
    invoke<Product>("products_set_active", { id, active }),
  stockAdjust: (input: {
    productId: number;
    kind: string;
    quantity: number;
    reason: string;
    employeeId: number | null;
  }) => invoke<Product>("stock_adjust", { input }),
  stockHistory: (productId: number) => invoke<StockMovement[]>("stock_history", { productId }),

  appointmentsList: (from: string, to: string, employeeId: number | null, clientId: number | null) =>
    invoke<Appointment[]>("appointments_list", { from, to, employeeId, clientId }),
  appointmentsSave: (input: {
    id: number | null;
    clientId: number;
    employeeId: number;
    startsAt: string;
    endsAt: string;
    status: string;
    notes: string;
    serviceIds: number[];
  }) => invoke<Appointment>("appointments_save", { input }),
  appointmentsSetStatus: (id: number, status: string) =>
    invoke<Appointment>("appointments_set_status", { id, status }),

  invoicesList: () => invoke<InvoiceSummary[]>("invoices_list"),
  invoicesGet: (id: number) => invoke<Invoice>("invoices_get", { id }),
  invoicesCreate: (input: {
    clientId: number;
    employeeId: number | null;
    appointmentId: number | null;
    notes: string;
  }) => invoke<Invoice>("invoices_create", { input }),
  invoicesAddLine: (input: {
    invoiceId: number;
    kind: string;
    serviceId: number | null;
    productId: number | null;
    quantity: number;
  }) => invoke<Invoice>("invoices_add_line", { input }),
  invoicesRemoveLine: (invoiceId: number, lineId: number) =>
    invoke<Invoice>("invoices_remove_line", { invoiceId, lineId }),
  invoicesSetNotes: (invoiceId: number, notes: string) =>
    invoke<Invoice>("invoices_set_notes", { invoiceId, notes }),
  invoicesIssue: (id: number) => invoke<Invoice>("invoices_issue", { id }),
  invoicesMarkPaid: (id: number) => invoke<Invoice>("invoices_mark_paid", { id }),
  invoicesCancel: (id: number) => invoke<Invoice>("invoices_cancel", { id }),
};
