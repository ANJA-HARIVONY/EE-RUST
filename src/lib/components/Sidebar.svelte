<script lang="ts">
  import { messages } from "../i18n/index.svelte";
  import type { ModuleId } from "../types";

  let {
    current,
    lowStock,
    onChange,
  }: {
    current: ModuleId;
    lowStock: number;
    onChange: (id: ModuleId) => void;
  } = $props();

  const m = $derived(messages());
  const items = $derived<{ id: ModuleId; label: string }[]>([
    { id: "agenda", label: m.nav.agenda },
    { id: "clients", label: m.nav.clients },
    { id: "services", label: m.nav.services },
    { id: "products", label: m.nav.products },
    { id: "employees", label: m.nav.employees },
    { id: "invoices", label: m.nav.invoices },
    { id: "settings", label: m.nav.settings },
  ]);
</script>

<nav class="sidebar">
  {#each items as item (item.id)}
    <button class:active={current === item.id} onclick={() => onChange(item.id)}>
      {item.label}
      {#if item.id === "products" && lowStock > 0}
        <em class="badge">{lowStock}</em>
      {/if}
    </button>
  {/each}
</nav>
