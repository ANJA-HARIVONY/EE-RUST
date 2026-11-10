<script lang="ts">
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

  const items: { id: ModuleId; label: string }[] = [
    { id: "agenda", label: "Agenda" },
    { id: "clients", label: "Clients" },
    { id: "services", label: "Services" },
    { id: "products", label: "Produits" },
    { id: "employees", label: "Employés" },
    { id: "invoices", label: "Factures" },
    { id: "settings", label: "Réglages" },
  ];
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
