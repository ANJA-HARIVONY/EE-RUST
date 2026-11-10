<script lang="ts">
  import { untrack } from "svelte";
  import { api } from "../../lib/api";
  import { errText } from "../../lib/labels";
  import { formatVat } from "../../lib/money";
  import type { Bootstrap, SalonSettings } from "../../lib/types";

  let {
    boot,
    onSaved,
  }: {
    boot: Bootstrap;
    onSaved: (settings: SalonSettings) => void;
  } = $props();

  let name = $state(untrack(() => boot.settings.name));
  let address = $state(untrack(() => boot.settings.address ?? ""));
  let phone = $state(untrack(() => boot.settings.phone ?? ""));
  let email = $state(untrack(() => boot.settings.email ?? ""));
  let siret = $state(untrack(() => boot.settings.siret ?? ""));
  let currency = $state(untrack(() => boot.settings.currency));
  let vat = $state(untrack(() => String(boot.settings.vatRateBps / 100).replace(".", ",")));
  let prefix = $state(untrack(() => boot.settings.invoicePrefix));
  let error = $state<string | null>(null);
  let saved = $state(false);
  let busy = $state(false);

  async function save() {
    const normalized = vat.trim().replace(",", ".");
    if (!/^\d+(\.\d{1,2})?$/.test(normalized)) {
      error = "Taux de TVA invalide.";
      return;
    }
    const vatRateBps = Math.round(Number(normalized) * 100);
    busy = true;
    error = null;
    saved = false;
    try {
      const settings = await api.settingsSave({
        name,
        address,
        phone,
        email,
        siret,
        currency,
        vatRateBps,
        invoicePrefix: prefix,
      });
      onSaved(settings);
      saved = true;
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }
</script>

<section class="detail">
  <header><h2>Réglages du salon</h2></header>
  <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
    <p class="muted">
      Ces informations ont été créées depuis le fichier <b>.env</b> au premier lancement.
      Les enregistrer ici met à jour la base locale. Les horaires d'agenda
      ({boot.agendaOpenHour}h–{boot.agendaCloseHour}h, créneaux de {boot.agendaSlotMinutes} min)
      et la locale ({boot.locale}) restent lus dans .env à chaque démarrage.
    </p>
    {#if error}<p class="error">{error}</p>{/if}
    {#if saved}<p class="ok">Réglages enregistrés. TVA actuelle : {formatVat(boot.settings.vatRateBps)}.</p>{/if}
    <label class="field">Nom du salon <input bind:value={name} required /></label>
    <label class="field">Adresse <input bind:value={address} /></label>
    <div class="cols">
      <label class="field">Téléphone <input bind:value={phone} /></label>
      <label class="field">E-mail <input bind:value={email} /></label>
    </div>
    <label class="field">SIRET <input bind:value={siret} /></label>
    <div class="cols">
      <label class="field">Devise (ISO) <input bind:value={currency} maxlength="3" /></label>
      <label class="field">TVA (%) <input bind:value={vat} /></label>
    </div>
    <div class="cols">
      <label class="field">Préfixe facture <input bind:value={prefix} /></label>
      <label class="field">Prochain numéro
        <input value={boot.settings.nextInvoiceSeq} readonly />
      </label>
    </div>
    <p class="muted">Fichier de données : {boot.databasePath}</p>
    <p class="muted">
      Utilisateur de session : {boot.currentUser.firstName} {boot.currentUser.lastName}
      ({boot.currentUser.email ?? "sans e-mail"}), reconnu via DEFAULT_USER_EMAIL.
    </p>
    <div class="actions">
      <button class="btn primary" disabled={busy}>Enregistrer</button>
    </div>
  </form>
</section>
