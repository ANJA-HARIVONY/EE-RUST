<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { api } from "../../lib/api";
  import { messages, movementLabel, movementReason, uiLocale } from "../../lib/i18n/index.svelte";
  import { errText } from "../../lib/labels";
  import { centsToInput, formatMoney, inputToCents } from "../../lib/money";
  import { formatWhen } from "../../lib/dates";
  import type { Bootstrap, Product, StockMovement } from "../../lib/types";

  let { boot, onStockChange }: { boot: Bootstrap; onStockChange: () => Promise<void> } = $props();
  let items = $state<Product[]>([]);
  let history = $state<StockMovement[]>([]);
  let includeInactive = $state(true);
  let selected = $state<number | "new" | null>(null);
  let sku = $state("");
  let name = $state("");
  let description = $state("");
  let price = $state("0,00");
  let threshold = $state(untrack(() => String(boot.lowStockDefault)));
  let active = $state(true);
  let moveKind = $state("in");
  let moveQty = $state("1");
  let moveReason = $state("");
  let error = $state<string | null>(null);
  let busy = $state(false);
  const m = $derived(messages());
  const loc = $derived(uiLocale());

  const current = $derived(items.find((item) => item.id === selected) ?? null);

  async function load() {
    items = await api.productsList(includeInactive);
  }

  async function open(product: Product) {
    selected = product.id;
    sku = product.sku ?? "";
    name = product.name;
    description = product.description ?? "";
    price = centsToInput(product.priceCents);
    threshold = String(product.lowStockThreshold);
    active = product.active;
    history = await api.stockHistory(product.id);
    error = null;
  }

  function create() {
    selected = "new";
    sku = "";
    name = "";
    description = "";
    price = "0,00";
    threshold = String(boot.lowStockDefault);
    active = true;
    history = [];
    error = null;
  }

  async function save() {
    const priceCents = inputToCents(price);
    const lowStockThreshold = Number(threshold);
    if (priceCents === null || !Number.isInteger(lowStockThreshold) || lowStockThreshold < 0) {
      error = m.products.invalid;
      return;
    }
    busy = true;
    error = null;
    try {
      const saved = await api.productsSave({
        id: selected === "new" || selected === null ? null : selected,
        sku,
        name,
        description,
        priceCents,
        lowStockThreshold,
        active,
      });
      await load();
      await onStockChange();
      await open(saved);
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  async function move() {
    if (typeof selected !== "number") return;
    const quantity = Number(moveQty);
    if (!Number.isInteger(quantity) || quantity < 0) {
      error = m.products.invalidQty;
      return;
    }
    busy = true;
    error = null;
    try {
      const saved = await api.stockAdjust({
        productId: selected,
        kind: moveKind,
        quantity,
        reason: moveReason,
        employeeId: boot.currentUser.id,
      });
      moveReason = "";
      await load();
      await onStockChange();
      await open(saved);
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    load().catch((cause) => (error = errText(cause)));
  });
</script>

<section class="split">
  <aside class="list">
    <header>
      <h2>{m.products.title}</h2>
      <button class="btn primary" onclick={create}>{m.common.new}</button>
    </header>
    <label class="field rowish" style="padding: 8px 12px;">
      <input type="checkbox" bind:checked={includeInactive} onchange={load} />
      <span>{m.common.showInactive}</span>
    </label>
    <div class="scroll">
      {#each items as product (product.id)}
        <button class="row" class:active={selected === product.id} onclick={() => open(product)}>
          <b>{product.name}</b>
          <span class:warn={product.lowStock}>
            {m.products.stock(product.stockQty)}
            {#if product.lowStock}{m.products.low(product.lowStockThreshold)}{/if}
            · {formatMoney(product.priceCents, boot.settings.currency, loc)}
          </span>
        </button>
      {:else}
        <p class="empty">{m.products.empty}</p>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header><h2>{selected === "new" ? m.products.newTitle : m.products.cardTitle}</h2></header>
    {#if selected === null}
      <p class="empty">{m.products.pick}</p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="cols">
          <label class="field">{m.common.name} <input bind:value={name} required /></label>
          <label class="field">{m.products.sku} <input bind:value={sku} /></label>
        </div>
        <div class="cols">
          <label class="field">{m.products.priceExcl(boot.settings.currency)} <input bind:value={price} /></label>
          <label class="field">{m.products.threshold} <input type="number" min="0" bind:value={threshold} /></label>
        </div>
        <label class="field">{m.common.description} <textarea bind:value={description}></textarea></label>
        <label class="field rowish"><input type="checkbox" bind:checked={active} /> {m.common.active}</label>
        {#if current}
          <p>{m.products.currentStock} <b class:warn={current.lowStock}>{current.stockQty}</b></p>
        {/if}
        <div class="actions">
          <button class="btn primary" disabled={busy}>{m.common.save}</button>
        </div>
      </form>
      {#if typeof selected === "number"}
        <form class="form" onsubmit={(event) => { event.preventDefault(); move(); }}>
          <h3>{m.products.movementTitle}</h3>
          <div class="cols">
            <label class="field">{m.products.type}
              <select bind:value={moveKind}>
                <option value="in">{m.products.kindIn}</option>
                <option value="out">{m.products.kindOut}</option>
                <option value="adjustment">{m.products.kindAdjust}</option>
              </select>
            </label>
            <label class="field">{moveKind === "adjustment" ? m.products.targetStock : m.products.quantity}
              <input type="number" min="0" bind:value={moveQty} />
            </label>
          </div>
          <label class="field">{m.products.reason} <input bind:value={moveReason} /></label>
          <div class="actions">
            <button class="btn" disabled={busy}>{m.products.apply}</button>
          </div>
          {#if history.length}
            <table>
              <thead><tr><th>{m.products.when}</th><th>{m.products.type}</th><th>{m.products.qty}</th><th>{m.products.reason}</th></tr></thead>
              <tbody>
                {#each history as movement (movement.id)}
                  <tr>
                    <td>{formatWhen(movement.createdAt, loc)}</td>
                    <td>{movementLabel(movement.kind)}</td>
                    <td>{movement.quantity > 0 ? `+${movement.quantity}` : movement.quantity}</td>
                    <td>{movementReason(movement.reason)}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        </form>
      {/if}
    {/if}
  </section>
</section>
