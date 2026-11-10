<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { addDays, formatWhen, startOfDay } from "../../lib/dates";
  import { errText, invoiceStatus } from "../../lib/labels";
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
      error = "Choisissez un client.";
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
      error = "Quantité invalide.";
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
      <h2>Factures</h2>
      <button class="btn primary" onclick={startCreate}>Nouvelle</button>
    </header>
    <div class="scroll">
      {#each items as item (item.id)}
        <button class="row" class:active={invoice?.id === item.id} onclick={() => open(item.id)}>
          <b>{item.status === "draft" ? "Brouillon" : item.number}</b>
          <span>
            {item.clientName} · {invoiceStatus[item.status] ?? item.status} ·
            {formatMoney(item.totalCents, item.currency, boot.locale)}
          </span>
        </button>
      {:else}
        <p class="empty">Aucune facture.</p>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header>
      <h2>
        {#if creating && !invoice}
          Nouvelle facture
        {:else if invoice}
          {invoice.status === "draft" ? "Brouillon" : invoice.number}
        {:else}
          Facture
        {/if}
      </h2>
      {#if invoice}<span class="status" data-status={invoice.status}>{invoiceStatus[invoice.status]}</span>{/if}
    </header>
    {#if error}<p class="error" style="margin: 12px 16px 0;">{error}</p>{/if}
    {#if creating && !invoice}
      <form class="form" onsubmit={(event) => { event.preventDefault(); createInvoice(); }}>
        <label class="field">Client
          <select bind:value={clientId} onchange={() => loadAppointments(Number(clientId))}>
            {#each clients as client (client.id)}
              <option value={client.id}>{client.firstName} {client.lastName}</option>
            {/each}
          </select>
        </label>
        <label class="field">Employé
          <select bind:value={employeeId}>
            <option value={0}>Non assigné</option>
            {#each employees as employee (employee.id)}
              <option value={employee.id}>{employee.firstName} {employee.lastName}</option>
            {/each}
          </select>
        </label>
        <label class="field">Importer un rendez-vous
          <select bind:value={appointmentId}>
            <option value={0}>Aucun</option>
            {#each appointments as appointment (appointment.id)}
              <option value={appointment.id}>
                {formatWhen(appointment.startsAt, boot.locale)} · {appointment.clientName}
              </option>
            {/each}
          </select>
        </label>
        <label class="field">Notes <textarea bind:value={notes}></textarea></label>
        <div class="actions"><button class="btn primary" disabled={busy}>Créer le brouillon</button></div>
      </form>
    {:else if invoice}
      <div class="form">
        <p><b>{invoice.clientName}</b> {#if invoice.employeeName}· {invoice.employeeName}{/if}</p>
        <table>
          <thead>
            <tr><th>Description</th><th>Qté</th><th>PU HT</th><th>Total HT</th><th></th></tr>
          </thead>
          <tbody>
            {#each invoice.lines as line (line.id)}
              <tr>
                <td>{line.description}</td>
                <td>{line.quantity}</td>
                <td>{formatMoney(line.unitPriceCents, invoice.currency, boot.locale)}</td>
                <td>{formatMoney(line.lineTotalCents, invoice.currency, boot.locale)}</td>
                <td>
                  {#if invoice.status === "draft"}
                    <button class="btn" type="button" onclick={() => removeLine(line.id)}>Retirer</button>
                  {/if}
                </td>
              </tr>
            {:else}
              <tr><td colspan="5" class="muted">Aucune ligne.</td></tr>
            {/each}
          </tbody>
        </table>
        <div class="totals">
          <div><span>Total HT</span><span>{formatMoney(invoice.subtotalCents, invoice.currency, boot.locale)}</span></div>
          <div><span>TVA {formatVat(invoice.vatRateBps)}</span><span>{formatMoney(invoice.vatCents, invoice.currency, boot.locale)}</span></div>
          <div><strong>Total TTC</strong><strong>{formatMoney(invoice.totalCents, invoice.currency, boot.locale)}</strong></div>
        </div>
        {#if invoice.status === "draft"}
          <div class="cols">
            <label class="field">Prestation
              <select bind:value={serviceId}>
                {#each services as service (service.id)}
                  <option value={service.id}>{service.name}</option>
                {/each}
              </select>
            </label>
            <div class="actions" style="align-items: end;">
              <button class="btn" type="button" disabled={busy || !serviceId} onclick={() => addLine("service")}>Ajouter</button>
            </div>
          </div>
          <div class="cols">
            <label class="field">Produit
              <select bind:value={productId}>
                {#each products as product (product.id)}
                  <option value={product.id}>{product.name} · stock {product.stockQty}</option>
                {/each}
              </select>
            </label>
            <label class="field">Quantité <input type="number" min="1" bind:value={quantity} /></label>
          </div>
          <div class="actions">
            <button class="btn" type="button" disabled={busy || !productId} onclick={() => addLine("product")}>Ajouter le produit</button>
          </div>
          <label class="field">Notes <textarea bind:value={notes}></textarea></label>
          <div class="actions">
            <button class="btn" type="button" disabled={busy} onclick={saveNotes}>Enregistrer la note</button>
            <button class="btn primary" type="button" disabled={busy} onclick={() => act("issue")}>Émettre</button>
            <button class="btn danger" type="button" disabled={busy} onclick={() => act("cancel")}>Annuler</button>
          </div>
        {:else}
          <p class="muted">{invoice.notes ?? ""}</p>
          <div class="actions">
            {#if invoice.status === "issued"}
              <button class="btn primary" type="button" disabled={busy} onclick={() => act("paid")}>Marquer payée</button>
            {/if}
            {#if invoice.status !== "cancelled"}
              <button class="btn danger" type="button" disabled={busy} onclick={() => act("cancel")}>Annuler la facture</button>
            {/if}
          </div>
        {/if}
      </div>
    {:else}
      <p class="empty">Sélectionnez une facture ou créez-en une. Les prestations d'un rendez-vous et les produits vendus y sont regroupés.</p>
    {/if}
  </section>
</section>
