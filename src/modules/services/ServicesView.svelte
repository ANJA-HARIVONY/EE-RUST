<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { errText } from "../../lib/labels";
  import { centsToInput, formatMoney, inputToCents } from "../../lib/money";
  import type { Bootstrap, Service } from "../../lib/types";

  let { boot }: { boot: Bootstrap } = $props();
  let items = $state<Service[]>([]);
  let includeInactive = $state(true);
  let selected = $state<number | "new" | null>(null);
  let name = $state("");
  let description = $state("");
  let duration = $state("60");
  let price = $state("0,00");
  let active = $state(true);
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function load() {
    items = await api.servicesList(includeInactive);
  }

  function open(service: Service) {
    selected = service.id;
    name = service.name;
    description = service.description ?? "";
    duration = String(service.durationMinutes);
    price = centsToInput(service.priceCents);
    active = service.active;
    error = null;
  }

  function create() {
    selected = "new";
    name = "";
    description = "";
    duration = "60";
    price = "0,00";
    active = true;
    error = null;
  }

  async function save() {
    const priceCents = inputToCents(price);
    const durationMinutes = Number(duration);
    if (priceCents === null || !Number.isInteger(durationMinutes) || durationMinutes <= 0) {
      error = "Durée ou prix invalide.";
      return;
    }
    busy = true;
    error = null;
    try {
      const saved = await api.servicesSave({
        id: selected === "new" || selected === null ? null : selected,
        name,
        description,
        durationMinutes,
        priceCents,
        active,
      });
      await load();
      open(saved);
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
      <h2>Services</h2>
      <button class="btn primary" onclick={create}>Nouveau</button>
    </header>
    <label class="field rowish" style="padding: 8px 12px;">
      <input type="checkbox" bind:checked={includeInactive} onchange={load} />
      <span>Afficher les inactifs</span>
    </label>
    <div class="scroll">
      {#each items as service (service.id)}
        <button class="row" class:active={selected === service.id} onclick={() => open(service)}>
          <b>{service.name}</b>
          <span>
            {service.durationMinutes} min · {formatMoney(service.priceCents, boot.settings.currency, boot.locale)}
            {#if !service.active} · inactif{/if}
          </span>
        </button>
      {:else}
        <p class="empty">Aucune prestation.</p>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header><h2>{selected === "new" ? "Nouvelle prestation" : "Prestation"}</h2></header>
    {#if selected === null}
      <p class="empty">Sélectionnez une prestation.</p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <label class="field">Nom <input bind:value={name} required /></label>
        <div class="cols">
          <label class="field">Durée (minutes) <input type="number" min="1" bind:value={duration} /></label>
          <label class="field">Prix HT ({boot.settings.currency}) <input bind:value={price} /></label>
        </div>
        <label class="field">Description <textarea bind:value={description}></textarea></label>
        <label class="field rowish"><input type="checkbox" bind:checked={active} /> Active</label>
        <div class="actions">
          <button class="btn primary" disabled={busy}>Enregistrer</button>
        </div>
      </form>
    {/if}
  </section>
</section>
