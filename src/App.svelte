<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api } from "./lib/api";
  import { adoptLocale, messages, roleName } from "./lib/i18n/index.svelte";
  import { errText, fullName } from "./lib/labels";
  import type { Bootstrap, ModuleId } from "./lib/types";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import AgendaView from "./modules/agenda/AgendaView.svelte";
  import ClientsView from "./modules/clients/ClientsView.svelte";
  import ServicesView from "./modules/services/ServicesView.svelte";
  import ProductsView from "./modules/products/ProductsView.svelte";
  import EmployeesView from "./modules/employees/EmployeesView.svelte";
  import InvoicesView from "./modules/invoices/InvoicesView.svelte";
  import SettingsView from "./modules/settings/SettingsView.svelte";

  let boot = $state<Bootstrap | null>(null);
  let error = $state<string | null>(null);
  let module = $state<ModuleId>("agenda");
  let lowStock = $state(0);
  const m = $derived(messages());

  async function refreshStock() {
    lowStock = await api.lowStockCount();
  }

  function applySettings(next: Bootstrap["settings"]) {
    if (!boot) return;
    boot = { ...boot, settings: next };
  }

  onMount(async () => {
    try {
      const loaded = await api.bootstrap();
      boot = loaded;
      lowStock = loaded.lowStockCount;
      adoptLocale(loaded.locale);
      try {
        await getCurrentWindow().setTitle(`EE Salon — ${loaded.settings.name}`);
      } catch {
        // Le titre natif n'est disponible que dans la fenêtre Tauri.
      }
    } catch (cause) {
      error = errText(cause);
    }
  });
</script>

<div class="shell">
  <header class="topbar">
    <div class="brand">
      <strong>{boot?.settings.name ?? "EE Salon"}</strong>
      <span>{m.app.subtitle}</span>
    </div>
    {#if boot}
      <div class="who">
        {fullName(boot.currentUser.firstName, boot.currentUser.lastName)}
        <div>{roleName(boot.currentUser.roleCode, boot.currentUser.roleLabel)}</div>
      </div>
    {/if}
  </header>
  {#if error}
    <div class="boot-error">
      <h1>{m.app.bootError}</h1>
      <p>{error}</p>
    </div>
  {:else if !boot}
    <p class="boot">{m.app.opening}</p>
  {:else}
    <div class="body">
      <Sidebar current={module} {lowStock} onChange={(id) => (module = id)} />
      <main class="workspace">
        {#key module}
          {#if module === "agenda"}
            <AgendaView {boot} />
          {:else if module === "clients"}
            <ClientsView {boot} />
          {:else if module === "services"}
            <ServicesView {boot} />
          {:else if module === "products"}
            <ProductsView {boot} onStockChange={refreshStock} />
          {:else if module === "employees"}
            <EmployeesView />
          {:else if module === "invoices"}
            <InvoicesView {boot} onStockChange={refreshStock} />
          {:else}
            <SettingsView {boot} onSaved={applySettings} />
          {/if}
        {/key}
      </main>
    </div>
  {/if}
</div>
