<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../../lib/api";
  import { errText, fullName } from "../../lib/labels";
  import type { Employee, Role } from "../../lib/types";

  let items = $state<Employee[]>([]);
  let roles = $state<Role[]>([]);
  let includeInactive = $state(true);
  let selected = $state<number | "new" | null>(null);
  let firstName = $state("");
  let lastName = $state("");
  let email = $state("");
  let phone = $state("");
  let roleId = $state(0);
  let color = $state("#2F5F8A");
  let active = $state(true);
  let notes = $state("");
  let error = $state<string | null>(null);
  let busy = $state(false);

  async function load() {
    items = await api.employeesList(includeInactive);
  }

  function open(employee: Employee) {
    selected = employee.id;
    firstName = employee.firstName;
    lastName = employee.lastName;
    email = employee.email ?? "";
    phone = employee.phone ?? "";
    roleId = employee.roleId;
    color = employee.color;
    active = employee.active;
    notes = employee.notes ?? "";
    error = null;
  }

  function create() {
    selected = "new";
    firstName = "";
    lastName = "";
    email = "";
    phone = "";
    roleId = roles.find((role) => role.code === "stylist")?.id ?? roles[0]?.id ?? 0;
    color = "#8A4B2F";
    active = true;
    notes = "";
    error = null;
  }

  async function save() {
    busy = true;
    error = null;
    try {
      const saved = await api.employeesSave({
        id: selected === "new" || selected === null ? null : selected,
        firstName,
        lastName,
        email,
        phone,
        roleId: Number(roleId),
        color,
        active,
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

  onMount(async () => {
    try {
      roles = await api.rolesList();
      await load();
    } catch (cause) {
      error = errText(cause);
    }
  });
</script>

<section class="split">
  <aside class="list">
    <header>
      <h2>Employés</h2>
      <button class="btn primary" onclick={create}>Nouveau</button>
    </header>
    <label class="field rowish" style="padding: 8px 12px;">
      <input type="checkbox" bind:checked={includeInactive} onchange={load} />
      <span>Afficher les inactifs</span>
    </label>
    <div class="scroll">
      {#each items as employee (employee.id)}
        <button class="row" class:active={selected === employee.id} onclick={() => open(employee)}>
          <b><i class="swatch" style:background={employee.color}></i>{fullName(employee.firstName, employee.lastName)}</b>
          <span>{employee.roleLabel}{employee.active ? "" : " · inactif"}</span>
        </button>
      {/each}
    </div>
  </aside>
  <section class="detail">
    <header><h2>{selected === "new" ? "Nouvel employé" : "Fiche employé"}</h2></header>
    {#if error && selected === null}<p class="error" style="margin: 12px;">{error}</p>{/if}
    {#if selected === null}
      <p class="empty">Sélectionnez un employé. L'utilisateur initial vient du fichier .env.</p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <div class="cols">
          <label class="field">Prénom <input bind:value={firstName} required /></label>
          <label class="field">Nom <input bind:value={lastName} required /></label>
        </div>
        <div class="cols">
          <label class="field">Téléphone <input bind:value={phone} /></label>
          <label class="field">E-mail <input bind:value={email} /></label>
        </div>
        <div class="cols">
          <label class="field">Rôle
            <select bind:value={roleId}>
              {#each roles as role (role.id)}
                <option value={role.id}>{role.label}</option>
              {/each}
            </select>
          </label>
          <label class="field">Couleur agenda <input type="color" bind:value={color} /></label>
        </div>
        <label class="field">Notes <textarea bind:value={notes}></textarea></label>
        <label class="field rowish"><input type="checkbox" bind:checked={active} /> Actif</label>
        <div class="actions">
          <button class="btn primary" disabled={busy}>Enregistrer</button>
        </div>
      </form>
    {/if}
  </section>
</section>
