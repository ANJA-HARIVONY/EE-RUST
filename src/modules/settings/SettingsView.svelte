<script lang="ts">
  import { untrack } from "svelte";
  import { api } from "../../lib/api";
  import { getLang, messages, setLang, type Lang } from "../../lib/i18n/index.svelte";
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
  const m = $derived(messages());

  async function save() {
    const normalized = vat.trim().replace(",", ".");
    if (!/^\d+(\.\d{1,2})?$/.test(normalized)) {
      error = m.settings.invalidVat;
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
  <header><h2>{m.settings.title}</h2></header>
  <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
    <p class="muted">{m.settings.intro(boot.agendaOpenHour, boot.agendaCloseHour, boot.agendaSlotMinutes)}</p>
    <label class="field">{m.settings.language}
      <select
        value={getLang()}
        onchange={(event) => setLang(event.currentTarget.value as Lang)}
      >
        <option value="fr">Français</option>
        <option value="es">Español</option>
      </select>
    </label>
    <p class="muted">{m.settings.languageHint}</p>
    {#if error}<p class="error">{error}</p>{/if}
    {#if saved}<p class="ok">{m.settings.saved(formatVat(boot.settings.vatRateBps))}</p>{/if}
    <label class="field">{m.settings.salonName} <input bind:value={name} required /></label>
    <label class="field">{m.settings.address} <input bind:value={address} /></label>
    <div class="cols">
      <label class="field">{m.common.phone} <input bind:value={phone} /></label>
      <label class="field">{m.common.email} <input bind:value={email} /></label>
    </div>
    <label class="field">{m.settings.siret} <input bind:value={siret} /></label>
    <div class="cols">
      <label class="field">{m.settings.currency} <input bind:value={currency} maxlength="3" /></label>
      <label class="field">{m.settings.vat} <input bind:value={vat} /></label>
    </div>
    <div class="cols">
      <label class="field">{m.settings.prefix} <input bind:value={prefix} /></label>
      <label class="field">{m.settings.nextNumber}
        <input value={boot.settings.nextInvoiceSeq} readonly />
      </label>
    </div>
    <p class="muted">{m.settings.dataFile(boot.databasePath)}</p>
    <p class="muted">
      {m.settings.sessionUser(
        boot.currentUser.firstName,
        boot.currentUser.lastName,
        boot.currentUser.email ?? m.settings.noEmail,
      )}
    </p>
    <div class="actions">
      <button class="btn primary" disabled={busy}>{m.common.save}</button>
    </div>
  </form>
</section>
