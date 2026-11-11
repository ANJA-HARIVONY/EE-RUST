<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { messages, uiLocale } from "../../lib/i18n/index.svelte";
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
  const m = $derived(messages());
  const loc = $derived(uiLocale());

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
      error = m.services.invalid;
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
      <h2>{m.services.title}</h2>
      <button class="btn primary" onclick={create}>{m.common.new}</button>
    </header>
    <label class="field rowish" style="padding: 8px 12px;">
      <input type="checkbox" bind:checked={includeInactive} onchange={load} />
      <span>{m.common.showInactive}</span>
    </label>
    <div class="scroll">
      {#each items as service (service.id)}
        <button class="row" class:active={selected === service.id} onclick={() => open(service)}>
          <b>{service.name}</b>
          <span>
            {service.durationMinutes} min · {formatMoney(service.priceCents, boot.settings.currency, loc)}
            {#if !service.active}{m.common.inactiveSuffix}{/if}
          </span>
        </button>
      {:else}
        <p class="empty">{m.services.empty}</p>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header><h2>{selected === "new" ? m.services.newTitle : m.services.cardTitle}</h2></header>
    {#if selected === null}
      <p class="empty">{m.services.pick}</p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <label class="field">{m.common.name} <input bind:value={name} required /></label>
        <div class="cols">
          <label class="field">{m.services.duration} <input type="number" min="1" bind:value={duration} /></label>
          <label class="field">{m.services.priceExcl(boot.settings.currency)} <input bind:value={price} /></label>
        </div>
        <label class="field">{m.common.description} <textarea bind:value={description}></textarea></label>
        <label class="field rowish"><input type="checkbox" bind:checked={active} /> {m.common.activeFem}</label>
        <div class="actions">
          <button class="btn primary" disabled={busy}>{m.common.save}</button>
        </div>
      </form>
    {/if}
  </section>
</section>
