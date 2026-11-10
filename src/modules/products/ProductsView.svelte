<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { api } from "../../lib/api";
  import { errText, movementKind } from "../../lib/labels";
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
      error = "Prix ou seuil invalide.";
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
      error = "Quantité invalide.";
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
      <h2>Produits</h2>
      <button class="btn primary" onclick={create}>Nouveau</button>
    </header>
    <label class="field rowish" style="padding: 8px 12px;">
      <input type="checkbox" bind:checked={includeInactive} onchange={load} />
      <span>Afficher les inactifs</span>
    </label>
    <div class="scroll">
      {#each items as product (product.id)}
        <button class="row" class:active={selected === product.id} onclick={() => open(product)}>
          <b>{product.name}</b>
          <span class:warn={product.lowStock}>
            Stock {product.stockQty}
            {#if product.lowStock} · seuil {product.lowStockThreshold}{/if}
            · {formatMoney(product.priceCents, boot.settings.currency, boot.locale)}
          </span>
        </button>
      {:else}
        <p class="empty">Aucun produit.</p>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header><h2>{selected === "new" ? "Nouveau produit" : "Fiche produit"}</h2></header>
    {#if selected === null}
      <p class="empty">Sélectionnez un produit. La vente se fait depuis Factures.</p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="cols">
          <label class="field">Nom <input bind:value={name} required /></label>
          <label class="field">SKU <input bind:value={sku} /></label>
        </div>
        <div class="cols">
          <label class="field">Prix HT ({boot.settings.currency}) <input bind:value={price} /></label>
          <label class="field">Seuil d'alerte <input type="number" min="0" bind:value={threshold} /></label>
        </div>
        <label class="field">Description <textarea bind:value={description}></textarea></label>
        <label class="field rowish"><input type="checkbox" bind:checked={active} /> Actif</label>
        {#if current}
          <p>Stock actuel : <b class:warn={current.lowStock}>{current.stockQty}</b></p>
        {/if}
        <div class="actions">
          <button class="btn primary" disabled={busy}>Enregistrer</button>
        </div>
      </form>
      {#if typeof selected === "number"}
        <form class="form" onsubmit={(event) => { event.preventDefault(); move(); }}>
          <h3>Mouvement de stock</h3>
          <div class="cols">
            <label class="field">Type
              <select bind:value={moveKind}>
                <option value="in">Entrée</option>
                <option value="out">Sortie</option>
                <option value="adjustment">Ajuster au stock cible</option>
              </select>
            </label>
            <label class="field">{moveKind === "adjustment" ? "Stock cible" : "Quantité"}
              <input type="number" min="0" bind:value={moveQty} />
            </label>
          </div>
          <label class="field">Motif <input bind:value={moveReason} /></label>
          <div class="actions">
            <button class="btn" disabled={busy}>Appliquer</button>
          </div>
          {#if history.length}
            <table>
              <thead><tr><th>Quand</th><th>Type</th><th>Qté</th><th>Motif</th></tr></thead>
              <tbody>
                {#each history as movement (movement.id)}
                  <tr>
                    <td>{formatWhen(movement.createdAt, boot.locale)}</td>
                    <td>{movementKind[movement.kind] ?? movement.kind}</td>
                    <td>{movement.quantity > 0 ? `+${movement.quantity}` : movement.quantity}</td>
                    <td>{movement.reason ?? ""}</td>
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
