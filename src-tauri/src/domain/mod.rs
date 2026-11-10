pub mod appointment;
pub mod client;
pub mod employee;
pub mod invoice;
pub mod money;
pub mod product;
pub mod service;
pub mod settings;

pub use appointment::{Appointment, AppointmentInput, AppointmentServiceLine};
pub use client::{Client, ClientInput};
pub use employee::{Employee, EmployeeInput, Role};
pub use invoice::{Invoice, InvoiceCreate, InvoiceLine, InvoiceLineInput, InvoiceSummary};
pub use product::{Product, ProductInput, StockAdjust, StockMovement};
pub use service::{Service, ServiceInput};
pub use settings::{Bootstrap, SalonSettings, SettingsInput};
