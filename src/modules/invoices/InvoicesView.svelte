<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { addDays, formatDate, formatWhen, startOfDay } from "../../lib/dates";
  import { invoiceStatusLabel, messages, uiLocale } from "../../lib/i18n/index.svelte";
  import { errText } from "../../lib/labels";
  import { formatMoney, formatVat } from "../../lib/money";
  import type { Appointment, Bootstrap, Client, Employee, Invoice, InvoiceSummary, Product, Service } from "../../lib/types";

  let { boot, onStockChange }: { boot: Bootstrap; onStockChange: () => Promise<void> } = $props();
  let items = $state<InvoiceSummary[]>([]);
  let invoice = $state<Invoice | null>(null);
  let clients = $state<Client[]>([]);
  let employees = $state<Employee[]>([]);
  let services = $state<Service[]>([]);
  let products = $state<Product[]>([]);
  let appointments = $state<Appointment[]>([]);
  let creating = $state(false);
  let clientId = $state(0);
  let employeeId = $state(0);
  let appointmentId = $state(0);
  let notes = $state("");
  let serviceId = $state(0);
  let productId = $state(0);
  let quantity = $state("1");
  let error = $state<string | null>(null);
  let busy = $state(false);
  const m = $derived(messages());
  const loc = $derived(uiLocale());
  const salon = $derived(boot.settings);
  const salonInitial = $derived((salon.name.trim().charAt(0) || "S").toUpperCase());
  const billedClient = $derived.by(() => {
    const current = invoice;
    if (!current) return null;
    return clients.find((client) => client.id === current.clientId) ?? null;
  });

  function sheetDate(current: Invoice): string {
    return current.issuedAt ? formatDate(current.issuedAt, loc) : "—";
  }

  function paymentText(current: Invoice): string {
    if (current.status === "paid") {
      return m.invoices.paidOn(current.paidAt ? formatDate(current.paidAt, loc) : "—");
    }
    if (current.status === "issued") return m.invoices.awaitingPayment;
    if (current.status === "cancelled") return m.invoices.cancelledNotice;
    return m.invoices.draftNotice;
  }

  function printInvoice() {
    window.print();
  }

  async function loadList() {
    items = await api.invoicesList();
  }

  async function open(id: number) {
    invoice = await api.invoicesGet(id);
    notes = invoice.notes ?? "";
    creating = false;
    error = null;
  }

  async function loadAppointments(id: number) {
    if (!id) {
      appointments = [];
      return;
    }
    const from = addDays(startOfDay(new Date()), -90).toISOString();
    const to = addDays(startOfDay(new Date()), 14).toISOString();
    appointments = await api.appointmentsList(from, to, null, id);
  }

  function startCreate() {
    creating = true;
    invoice = null;
    clientId = clients[0]?.id ?? 0;
    employeeId = boot.currentUser.id;
    appointmentId = 0;
    notes = "";
    error = null;
    loadAppointments(clientId);
  }

  async function createInvoice() {
    if (!clientId) {
      error = m.invoices.chooseClient;
      return;
    }
    busy = true;
    error = null;
    try {
      const created = await api.invoicesCreate({
        clientId: Number(clientId),
        employeeId: Number(employeeId) || null,
        appointmentId: Number(appointmentId) || null,
        notes,
      });
      await loadList();
      await open(created.id);
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  async function addLine(kind: "service" | "product") {
    if (!invoice) return;
    const qty = Number(quantity);
    if (!Number.isInteger(qty) || qty <= 0) {
      error = m.invoices.invalidQty;
      return;
    }
    busy = true;
    error = null;
    try {
      invoice = await api.invoicesAddLine({
        invoiceId: invoice.id,
        kind,
        serviceId: kind === "service" ? Number(serviceId) : null,
        productId: kind === "product" ? Number(productId) : null,
        quantity: qty,
      });
      await loadList();
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  async function removeLine(lineId: number) {
    if (!invoice) return;
    busy = true;
    error = null;
    try {
      invoice = await api.invoicesRemoveLine(invoice.id, lineId);
      await loadList();
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  async function saveNotes() {
    if (!invoice) return;
    busy = true;
    error = null;
    try {
      invoice = await api.invoicesSetNotes(invoice.id, notes);
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  async function act(action: "issue" | "paid" | "cancel") {
    if (!invoice) return;
    busy = true;
    error = null;
    try {
      invoice =
        action === "issue"
          ? await api.invoicesIssue(invoice.id)
          : action === "paid"
            ? await api.invoicesMarkPaid(invoice.id)
            : await api.invoicesCancel(invoice.id);
      await loadList();
      await onStockChange();
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  onMount(async () => {
    try {
      [clients, employees, services, products] = await Promise.all([
        api.clientsList(""),
        api.employeesList(false),
        api.servicesList(false),
        api.productsList(false),
      ]);
      serviceId = services[0]?.id ?? 0;
      productId = products[0]?.id ?? 0;
      await loadList();
    } catch (cause) {
      error = errText(cause);
    }
  });
</script>

<section class="split">
  <aside class="list">
    <header>
      <h2>{m.invoices.title}</h2>
      <button class="btn primary" onclick={startCreate}>{m.invoices.newButton}</button>
    </header>
    <div class="scroll">
      {#each items as item (item.id)}
        <button class="row" class:active={invoice?.id === item.id} onclick={() => open(item.id)}>
          <b>{item.status === "draft" ? m.invoiceStatus.draft : item.number}</b>
          <span>
            {item.clientName} · {invoiceStatusLabel(item.status)} ·
            {formatMoney(item.totalCents, item.currency, loc)}
          </span>
        </button>
      {:else}
        <p class="empty">{m.invoices.empty}</p>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header>
      <h2>
        {#if creating && !invoice}
          {m.invoices.newTitle}
        {:else if invoice}
          {invoice.status === "draft" ? m.invoiceStatus.draft : invoice.number}
        {:else}
          {m.invoices.cardTitle}
        {/if}
      </h2>
      {#if invoice}<span class="status" data-status={invoice.status}>{invoiceStatusLabel(invoice.status)}</span>{/if}
      {#if invoice?.status === "paid"}
        <button class="btn primary no-print" type="button" onclick={printInvoice}>{m.invoices.print}</button>
      {/if}
    </header>
    {#if error}<p class="error" style="margin: 12px 16px 0;">{error}</p>{/if}
    {#if creating && !invoice}
      <form class="form" onsubmit={(event) => { event.preventDefault(); createInvoice(); }}>
        <label class="field">{m.invoices.client}
          <select bind:value={clientId} onchange={() => loadAppointments(Number(clientId))}>
            {#each clients as client (client.id)}
              <option value={client.id}>{client.firstName} {client.lastName}</option>
            {/each}
          </select>
        </label>
        <label class="field">{m.invoices.employee}
          <select bind:value={employeeId}>
            <option value={0}>{m.invoices.unassigned}</option>
            {#each employees as employee (employee.id)}
              <option value={employee.id}>{employee.firstName} {employee.lastName}</option>
            {/each}
          </select>
        </label>
        <label class="field">{m.invoices.importAppointment}
          <select bind:value={appointmentId}>
            <option value={0}>{m.invoices.none}</option>
            {#each appointments as appointment (appointment.id)}
              <option value={appointment.id}>
                {formatWhen(appointment.startsAt, loc)} · {appointment.clientName}
              </option>
            {/each}
          </select>
        </label>
        <label class="field">{m.common.notes} <textarea bind:value={notes}></textarea></label>
        <div class="actions"><button class="btn primary" disabled={busy}>{m.invoices.createDraft}</button></div>
      </form>
    {:else if invoice}
      <div class="invoice-scroll">
        <div class="sheet-stage">
          <article class="invoice-sheet">
            <header class="sheet-head">
              <div class="brand-lockup">
                <span class="mark" aria-hidden="true">{salonInitial}</span>
                <div>
                  <strong>{salon.name}</strong>
                  <span>{m.invoices.tagline}</span>
                </div>
              </div>
              <div class="sheet-title">
                <h3>{m.invoices.heading}</h3>
                <p>{m.invoices.dateLabel} : {sheetDate(invoice)}</p>
                <p class="sheet-number">{m.invoices.numberLabel} {invoice.number}</p>
              </div>
            </header>

            <div class="parties">
              <div>
                <h4>{m.invoices.from}</h4>
                <p>{salon.name}</p>
                {#if salon.address}<p>{salon.address}</p>{/if}
                {#if salon.phone}<p>{salon.phone}</p>{/if}
                {#if salon.email}<p>{salon.email}</p>{/if}
                {#if salon.siret}<p>{m.settings.siret} {salon.siret}</p>{/if}
              </div>
              <div>
                <h4>{m.invoices.to}</h4>
                <p>{invoice.clientName}</p>
                {#if billedClient?.phone}<p>{billedClient.phone}</p>{/if}
                {#if billedClient?.email}<p>{billedClient.email}</p>{/if}
                {#if invoice.employeeName}<p>{m.invoices.practitioner} · {invoice.employeeName}</p>{/if}
              </div>
            </div>

            <table class="sheet-table">
              <thead>
                <tr>
                  <th>{m.invoices.description}</th>
                  <th class="num">{m.invoices.unitPrice}</th>
                  <th class="num">{m.invoices.qty}</th>
                  <th class="num total">{m.invoices.lineTotal}</th>
                </tr>
              </thead>
              <tbody>
                {#each invoice.lines as line (line.id)}
                  <tr>
                    <td>
                      {line.description}
                      {#if invoice.status === "draft"}
                        <button class="linkish no-print" type="button" onclick={() => removeLine(line.id)}>{m.invoices.remove}</button>
                      {/if}
                    </td>
                    <td class="num">{formatMoney(line.unitPriceCents, invoice.currency, loc)}</td>
                    <td class="num">{line.quantity}</td>
                    <td class="num total">{formatMoney(line.lineTotalCents, invoice.currency, loc)}</td>
                  </tr>
                {:else}
                  <tr><td colspan="4" class="muted">{m.invoices.noLines}</td></tr>
                {/each}
              </tbody>
            </table>

            <div class="sheet-bottom">
              <div>
                <h4>{m.invoices.payment}</h4>
                <p>{paymentText(invoice)}</p>
              </div>
              <div class="sheet-totals">
                <div><span>{m.invoices.subtotal}</span><span>{formatMoney(invoice.subtotalCents, invoice.currency, loc)}</span></div>
                <div><span>{m.invoices.vat(formatVat(invoice.vatRateBps))}</span><span>{formatMoney(invoice.vatCents, invoice.currency, loc)}</span></div>
                <div class="grand"><span>{m.invoices.total}</span><span>{formatMoney(invoice.totalCents, invoice.currency, loc)}</span></div>
              </div>
            </div>

            <footer class="sheet-foot">
              <div class="sheet-contact">
                {#if salon.phone}<p>{salon.phone}</p>{/if}
                {#if salon.email}<p>{salon.email}</p>{/if}
                {#if salon.siret}<p>{m.settings.siret} {salon.siret}</p>{/if}
                {#if !salon.phone && !salon.email && !salon.siret}<p>{salon.name}</p>{/if}
              </div>
              <div class="sheet-terms">
                <h4>{m.invoices.terms}</h4>
                <p>{invoice.notes?.trim() || m.invoices.termsDefault}</p>
              </div>
            </footer>
          </article>
        </div>

        {#if invoice.status === "draft"}
          <div class="invoice-tools">
            <div class="cols">
              <label class="field">{m.invoices.service}
                <select bind:value={serviceId}>
                  {#each services as service (service.id)}
                    <option value={service.id}>{service.name}</option>
                  {/each}
                </select>
              </label>
              <div class="actions" style="align-items: end;">
                <button class="btn" type="button" disabled={busy || !serviceId} onclick={() => addLine("service")}>{m.invoices.add}</button>
              </div>
            </div>
            <div class="cols">
              <label class="field">{m.invoices.product}
                <select bind:value={productId}>
                  {#each products as product (product.id)}
                    <option value={product.id}>{product.name} · {m.invoices.stock(product.stockQty)}</option>
                  {/each}
                </select>
              </label>
              <label class="field">{m.products.quantity} <input type="number" min="1" bind:value={quantity} /></label>
            </div>
            <div class="actions">
              <button class="btn" type="button" disabled={busy || !productId} onclick={() => addLine("product")}>{m.invoices.addProduct}</button>
            </div>
            <label class="field">{m.common.notes} <textarea bind:value={notes}></textarea></label>
            <div class="actions">
              <button class="btn" type="button" disabled={busy} onclick={saveNotes}>{m.invoices.saveNote}</button>
              <button class="btn primary" type="button" disabled={busy} onclick={() => act("issue")}>{m.invoices.issue}</button>
              <button class="btn danger" type="button" disabled={busy} onclick={() => act("cancel")}>{m.invoices.cancel}</button>
            </div>
          </div>
        {:else}
          <div class="invoice-tools">
            <div class="actions">
              {#if invoice.status === "issued"}
                <button class="btn primary" type="button" disabled={busy} onclick={() => act("paid")}>{m.invoices.markPaid}</button>
              {/if}
              {#if invoice.status !== "cancelled"}
                <button class="btn danger" type="button" disabled={busy} onclick={() => act("cancel")}>{m.invoices.cancelInvoice}</button>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <p class="empty">{m.invoices.pick}</p>
    {/if}
  </section>
</section>

<style>
  .invoice-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
    background: var(--panel);
  }
  .sheet-stage {
    background: #5c5678;
    padding: 28px 20px 32px;
  }
  .invoice-sheet {
    max-width: 720px;
    margin: 0 auto;
    background: #f4f4f6;
    color: #1b1b2f;
    padding: 36px 40px 32px;
    box-shadow: 0 18px 40px rgba(20, 16, 40, 0.28);
  }
  .sheet-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 24px;
  }
  .brand-lockup {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .mark {
    width: 46px;
    height: 46px;
    border-radius: 50%;
    background: #1b1b2f;
    color: #f4f4f6;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 22px;
    font-weight: 750;
    flex: none;
  }
  .brand-lockup strong {
    display: block;
    font-size: 22px;
    line-height: 1.1;
    font-weight: 750;
  }
  .brand-lockup span {
    display: block;
    margin-top: 2px;
    font-size: 12px;
    color: #3a3a52;
  }
  .sheet-title {
    text-align: right;
    flex: none;
  }
  .sheet-title h3 {
    margin: 0;
    font-size: 42px;
    line-height: 0.9;
    letter-spacing: -0.04em;
    font-weight: 800;
  }
  .sheet-title p {
    margin: 6px 0 0;
    font-size: 13px;
  }
  .sheet-number {
    font-weight: 700;
  }
  .parties {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 28px;
    margin: 28px 0 22px;
  }
  .parties h4,
  .sheet-bottom h4,
  .sheet-terms h4 {
    margin: 0 0 6px;
    font-size: 14px;
    font-weight: 750;
  }
  .parties p,
  .sheet-bottom p,
  .sheet-contact p,
  .sheet-terms p {
    margin: 0;
    line-height: 1.45;
  }
  .sheet-table {
    width: 100%;
    border-collapse: collapse;
  }
  .sheet-table th,
  .sheet-table td {
    border-bottom: 0;
    padding: 10px 12px 10px 0;
    vertical-align: top;
    color: #1b1b2f;
    text-align: left;
  }
  .sheet-table th {
    font-size: 14px;
    font-weight: 750;
    border-bottom: 1.5px solid #1b1b2f;
    padding-top: 0;
  }
  .sheet-table .num {
    width: 92px;
    text-align: center;
    border-left: 1.5px solid #1b1b2f;
    padding-left: 8px;
  }
  .sheet-table .total {
    text-align: right;
    padding-right: 0;
  }
  .linkish {
    display: block;
    margin-top: 4px;
    padding: 0;
    border: 0;
    background: transparent;
    color: #5c5678;
    font-size: 12px;
    text-decoration: underline;
  }
  .sheet-bottom {
    display: grid;
    grid-template-columns: 1.15fr 0.85fr;
    gap: 28px;
    margin-top: 8px;
    padding-top: 14px;
    border-top: 1.5px solid #1b1b2f;
  }
  .sheet-totals div {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 3px 0;
    font-weight: 750;
  }
  .sheet-totals .grand {
    margin-top: 6px;
    padding-top: 8px;
    border-top: 1.5px solid #1b1b2f;
    font-size: 15px;
  }
  .sheet-foot {
    display: grid;
    grid-template-columns: 0.7fr 1.3fr;
    gap: 28px;
    margin-top: 28px;
    padding-top: 16px;
    border-top: 1.5px solid #1b1b2f;
  }
  .sheet-terms {
    text-align: right;
  }
  .invoice-tools {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 20px 28px;
    background: var(--panel);
  }
</style>
