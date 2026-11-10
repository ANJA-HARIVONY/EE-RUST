<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { errText, fullName } from "../../lib/labels";
  import type { Bootstrap, Client } from "../../lib/types";

  let { boot }: { boot: Bootstrap } = $props();
  let query = $state("");
  let items = $state<Client[]>([]);
  let selected = $state<number | "new" | null>(null);
  let firstName = $state("");
  let lastName = $state("");
  let email = $state("");
  let phone = $state("");
  let notes = $state("");
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function load() {
    items = await api.clientsList(query);
  }

  function fill(client: Client | null) {
    firstName = client?.firstName ?? "";
    lastName = client?.lastName ?? "";
    email = client?.email ?? "";
    phone = client?.phone ?? "";
    notes = client?.notes ?? "";
  }

  function open(client: Client) {
    selected = client.id;
    fill(client);
    error = null;
  }

  function create() {
    selected = "new";
    fill(null);
    error = null;
  }

  async function save() {
    busy = true;
    error = null;
    try {
      const saved = await api.clientsSave({
        id: selected === "new" || selected === null ? null : selected,
        firstName,
        lastName,
        email,
        phone,
        notes,
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
      <h2>Clients</h2>
      <button class="btn primary" onclick={create}>Nouveau</button>
    </header>
    <div style="padding: 8px 12px;">
      <input class="search" type="search" placeholder="Nom, téléphone, e-mail" bind:value={query} oninput={load} />
    </div>
    <div class="scroll">
      {#if items.length === 0}
        <p class="empty">Aucun client.</p>
      {/if}
      {#each items as client (client.id)}
        <button class="row" class:active={selected === client.id} onclick={() => open(client)}>
          <b>{fullName(client.firstName, client.lastName)}</b>
          <span>{client.phone || client.email || "Sans coordonnées"}</span>
        </button>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header><h2>{selected === "new" ? "Nouveau client" : "Fiche client"}</h2></header>
    {#if selected === null}
      <p class="empty">Sélectionnez un client ou créez-en un.</p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="cols">
          <label class="field">Prénom <input bind:value={firstName} required /></label>
          <label class="field">Nom <input bind:value={lastName} required /></label>
        </div>
        <div class="cols">
          <label class="field">Téléphone <input bind:value={phone} /></label>
          <label class="field">E-mail <input type="email" bind:value={email} /></label>
        </div>
        <label class="field">Notes <textarea bind:value={notes}></textarea></label>
        <div class="actions">
          <button class="btn primary" type="submit" disabled={busy}>Enregistrer</button>
        </div>
        <p class="muted">Les montants du salon s'affichent en {boot.settings.currency}.</p>
      </form>
    {/if}
  </section>
</section>
