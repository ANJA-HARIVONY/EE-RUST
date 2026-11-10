CREATE TABLE schema_migrations (
    version     INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    applied_at  TEXT NOT NULL
);

CREATE TABLE roles (
    id    INTEGER PRIMARY KEY,
    code  TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL
);

CREATE TABLE employees (
    id          INTEGER PRIMARY KEY,
    first_name  TEXT NOT NULL,
    last_name   TEXT NOT NULL,
    email       TEXT,
    phone       TEXT,
    role_id     INTEGER NOT NULL REFERENCES roles(id),
    color       TEXT NOT NULL DEFAULT '#2F5F8A',
    active      INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE clients (
    id          INTEGER PRIMARY KEY,
    first_name  TEXT NOT NULL,
    last_name   TEXT NOT NULL,
    email       TEXT,
    phone       TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE services (
    id                INTEGER PRIMARY KEY,
    name              TEXT NOT NULL,
    description       TEXT,
    duration_minutes  INTEGER NOT NULL CHECK (duration_minutes > 0),
    price_cents       INTEGER NOT NULL CHECK (price_cents >= 0),
    active            INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

CREATE TABLE products (
    id                   INTEGER PRIMARY KEY,
    sku                  TEXT UNIQUE,
    name                 TEXT NOT NULL,
    description          TEXT,
    price_cents          INTEGER NOT NULL CHECK (price_cents >= 0),
    stock_qty            INTEGER NOT NULL DEFAULT 0 CHECK (stock_qty >= 0),
    low_stock_threshold  INTEGER NOT NULL DEFAULT 3 CHECK (low_stock_threshold >= 0),
    active               INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL
);

CREATE TABLE stock_movements (
    id           INTEGER PRIMARY KEY,
    product_id   INTEGER NOT NULL REFERENCES products(id),
    kind         TEXT NOT NULL CHECK (kind IN ('in', 'out', 'adjustment', 'sale')),
    quantity     INTEGER NOT NULL CHECK (quantity <> 0),
    reason       TEXT,
    employee_id  INTEGER REFERENCES employees(id),
    invoice_id   INTEGER,
    created_at   TEXT NOT NULL
);

CREATE TABLE appointments (
    id           INTEGER PRIMARY KEY,
    client_id    INTEGER NOT NULL REFERENCES clients(id),
    employee_id  INTEGER NOT NULL REFERENCES employees(id),
    starts_at    TEXT NOT NULL,
    ends_at      TEXT NOT NULL,
    status       TEXT NOT NULL DEFAULT 'scheduled'
                 CHECK (status IN (
                     'scheduled', 'confirmed', 'in_progress',
                     'completed', 'cancelled', 'no_show'
                 )),
    notes        TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    CHECK (ends_at > starts_at)
);

CREATE TABLE appointment_services (
    id                INTEGER PRIMARY KEY,
    appointment_id    INTEGER NOT NULL REFERENCES appointments(id) ON DELETE CASCADE,
    service_id        INTEGER NOT NULL REFERENCES services(id),
    name_snapshot     TEXT NOT NULL,
    duration_minutes  INTEGER NOT NULL CHECK (duration_minutes > 0),
    price_cents       INTEGER NOT NULL CHECK (price_cents >= 0)
);

CREATE TABLE salon_settings (
    id                INTEGER PRIMARY KEY CHECK (id = 1),
    name              TEXT NOT NULL,
    address           TEXT,
    phone             TEXT,
    email             TEXT,
    siret             TEXT,
    currency          TEXT NOT NULL DEFAULT 'EUR',
    vat_rate_bps      INTEGER NOT NULL DEFAULT 2000 CHECK (vat_rate_bps >= 0 AND vat_rate_bps <= 10000),
    invoice_prefix    TEXT NOT NULL DEFAULT 'FAC',
    next_invoice_seq  INTEGER NOT NULL DEFAULT 1 CHECK (next_invoice_seq > 0)
);

CREATE TABLE invoices (
    id              INTEGER PRIMARY KEY,
    number          TEXT NOT NULL UNIQUE,
    client_id       INTEGER NOT NULL REFERENCES clients(id),
    employee_id     INTEGER REFERENCES employees(id),
    status          TEXT NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'issued', 'paid', 'cancelled')),
    issued_at       TEXT,
    paid_at         TEXT,
    notes           TEXT,
    currency        TEXT NOT NULL,
    vat_rate_bps    INTEGER NOT NULL CHECK (vat_rate_bps >= 0 AND vat_rate_bps <= 10000),
    subtotal_cents  INTEGER NOT NULL DEFAULT 0 CHECK (subtotal_cents >= 0),
    vat_cents       INTEGER NOT NULL DEFAULT 0 CHECK (vat_cents >= 0),
    total_cents     INTEGER NOT NULL DEFAULT 0 CHECK (total_cents >= 0),
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE invoice_lines (
    id                INTEGER PRIMARY KEY,
    invoice_id        INTEGER NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    kind              TEXT NOT NULL CHECK (kind IN ('service', 'product')),
    service_id        INTEGER REFERENCES services(id),
    product_id        INTEGER REFERENCES products(id),
    appointment_id    INTEGER REFERENCES appointments(id),
    description       TEXT NOT NULL,
    quantity          INTEGER NOT NULL CHECK (quantity > 0),
    unit_price_cents  INTEGER NOT NULL CHECK (unit_price_cents >= 0),
    line_total_cents  INTEGER NOT NULL CHECK (line_total_cents >= 0),
    CHECK (
        (kind = 'service' AND service_id IS NOT NULL AND product_id IS NULL)
        OR
        (kind = 'product' AND product_id IS NOT NULL AND service_id IS NULL)
    )
);

CREATE INDEX idx_employees_active ON employees(active);
CREATE INDEX idx_clients_name ON clients(last_name, first_name);
CREATE INDEX idx_appointments_range ON appointments(starts_at, ends_at);
CREATE INDEX idx_appointments_employee ON appointments(employee_id, starts_at);
CREATE INDEX idx_appointments_client ON appointments(client_id, starts_at);
CREATE INDEX idx_stock_product ON stock_movements(product_id, created_at);
CREATE INDEX idx_invoices_client ON invoices(client_id, created_at);
CREATE INDEX idx_invoice_lines_invoice ON invoice_lines(invoice_id);
CREATE INDEX idx_invoice_lines_appointment ON invoice_lines(appointment_id);

INSERT INTO roles (code, label) VALUES
    ('admin', 'Administrateur'),
    ('manager', 'Responsable'),
    ('receptionist', 'Réception'),
    ('stylist', 'Praticien');
