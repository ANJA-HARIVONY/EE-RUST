<script lang="ts">
  import { api } from "../../lib/api";
  import {
    addDays,
    formatDay,
    formatWhen,
    sameDay,
    startOfDay,
    startOfMonth,
    startOfWeek,
    toLocalInput,
  } from "../../lib/dates";
  import { appointmentStatus, errText, fullName } from "../../lib/labels";
  import { formatMoney } from "../../lib/money";
  import type { Appointment, Bootstrap, Client, Employee, Service } from "../../lib/types";

  let { boot }: { boot: Bootstrap } = $props();

  const hourPx = 56;
  let mode = $state<"day" | "week" | "month">("week");
  let cursor = $state(startOfDay(new Date()));
  let employeeFilter = $state(0);
  let appointments = $state<Appointment[]>([]);
  let clients = $state<Client[]>([]);
  let employees = $state<Employee[]>([]);
  let services = $state<Service[]>([]);
  let selectedId = $state<number | null>(null);
  let editing = $state(false);
  let clientId = $state(0);
  let employeeId = $state(0);
  let startValue = $state("");
  let endValue = $state("");
  let status = $state("scheduled");
  let notes = $state("");
  let serviceIds = $state<number[]>([]);
  let error = $state<string | null>(null);
  let busy = $state(false);

  const filterId = $derived(Number(employeeFilter) || 0);
  const activeEmployees = $derived(employees.filter((employee) => employee.active));
  const visibleEmployees = $derived(
    filterId ? activeEmployees.filter((employee) => employee.id === filterId) : activeEmployees,
  );
  const selected = $derived(appointments.find((item) => item.id === selectedId) ?? null);

  const range = $derived.by(() => {
    if (mode === "day") return { from: startOfDay(cursor), to: addDays(startOfDay(cursor), 1) };
    if (mode === "week") return { from: startOfWeek(cursor), to: addDays(startOfWeek(cursor), 7) };
    const monthStart = startOfMonth(cursor);
    const gridStart = startOfWeek(monthStart);
    return { from: gridStart, to: addDays(gridStart, 42) };
  });

  const title = $derived.by(() => {
    if (mode === "day") return formatDay(cursor, boot.locale);
    if (mode === "week") {
      const start = startOfWeek(cursor);
      const end = addDays(start, 6);
      return `${start.toLocaleDateString(boot.locale, { day: "numeric", month: "short" })} – ${end.toLocaleDateString(boot.locale, { day: "numeric", month: "long", year: "numeric" })}`;
    }
    return cursor.toLocaleDateString(boot.locale, { month: "long", year: "numeric" });
  });

  const columns = $derived.by(() => {
    if (mode === "day") {
      return visibleEmployees.map((employee) => ({
        key: `e-${employee.id}`,
        label: fullName(employee.firstName, employee.lastName),
        color: employee.color,
        day: startOfDay(cursor),
        employeeId: employee.id,
      }));
    }
    return Array.from({ length: 7 }, (_, index) => {
      const day = addDays(startOfWeek(cursor), index);
      return {
        key: `d-${day.toISOString()}`,
        label: day.toLocaleDateString(boot.locale, { weekday: "short", day: "numeric" }),
        color: "",
        day,
        employeeId: filterId || null,
      };
    });
  });

  function inColumn(appointment: Appointment, column: (typeof columns)[number]): boolean {
    const when = new Date(appointment.startsAt);
    if (!sameDay(when, column.day)) return false;
    if (column.employeeId && appointment.employeeId !== column.employeeId) return false;
    if (!column.employeeId && filterId && appointment.employeeId !== filterId) return false;
    return true;
  }

  function topOf(iso: string): number {
    const date = new Date(iso);
    const minutes = (date.getHours() - boot.agendaOpenHour) * 60 + date.getMinutes();
    return (minutes / 60) * hourPx;
  }

  function heightOf(start: string, end: string): number {
    const minutes = (new Date(end).getTime() - new Date(start).getTime()) / 60000;
    return Math.max(22, (minutes / 60) * hourPx);
  }

  async function load() {
    const [nextAppointments, nextClients, nextEmployees, nextServices] = await Promise.all([
      api.appointmentsList(range.from.toISOString(), range.to.toISOString(), filterId || null, null),
      api.clientsList(""),
      api.employeesList(true),
      api.servicesList(false),
    ]);
    appointments = nextAppointments;
    clients = nextClients;
    employees = nextEmployees;
    services = nextServices;
  }

  function draft(start: Date, assignedEmployee: number | null) {
    editing = true;
    selectedId = null;
    clientId = clients[0]?.id ?? 0;
    employeeId = assignedEmployee || filterId || boot.currentUser.id;
    startValue = toLocalInput(start.toISOString());
    const end = new Date(start.getTime() + 60 * 60000);
    endValue = toLocalInput(end.toISOString());
    status = "scheduled";
    notes = "";
    serviceIds = [];
    error = null;
  }

  function openAppointment(appointment: Appointment) {
    editing = true;
    selectedId = appointment.id;
    clientId = appointment.clientId;
    employeeId = appointment.employeeId;
    startValue = toLocalInput(appointment.startsAt);
    endValue = toLocalInput(appointment.endsAt);
    status = appointment.status;
    notes = appointment.notes ?? "";
    serviceIds = appointment.services.map((service) => service.serviceId);
    error = null;
  }

  function toggleService(id: number) {
    serviceIds = serviceIds.includes(id) ? serviceIds.filter((item) => item !== id) : [...serviceIds, id];
    const minutes = services
      .filter((service) => serviceIds.includes(service.id))
      .reduce((sum, service) => sum + service.durationMinutes, 0);
    if (minutes > 0 && startValue) {
      const end = new Date(new Date(startValue).getTime() + minutes * 60000);
      endValue = toLocalInput(end.toISOString());
    }
  }

  function slotFromClick(event: MouseEvent, day: Date, assignedEmployee: number | null) {
    const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const y = event.clientY - bounds.top;
    const minutesTotal = Math.max(0, Math.round((y / hourPx) * 60));
    const snapped = Math.floor(minutesTotal / boot.agendaSlotMinutes) * boot.agendaSlotMinutes;
    const start = new Date(day);
    start.setHours(boot.agendaOpenHour, 0, 0, 0);
    start.setMinutes(start.getMinutes() + snapped);
    draft(start, assignedEmployee);
  }

  async function save() {
    const start = new Date(startValue);
    const end = new Date(endValue);
    if (!clientId || !employeeId) {
      error = "Client et employé sont obligatoires.";
      return;
    }
    if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) {
      error = "Date ou heure invalide.";
      return;
    }
    busy = true;
    error = null;
    try {
      const saved = await api.appointmentsSave({
        id: selectedId,
        clientId: Number(clientId),
        employeeId: Number(employeeId),
        startsAt: start.toISOString(),
        endsAt: end.toISOString(),
        status,
        notes,
        serviceIds: serviceIds.map(Number),
      });
      await load();
      openAppointment(saved);
    } catch (cause) {
      error = errText(cause);
    } finally {
      busy = false;
    }
  }

  function shift(direction: number) {
    if (mode === "month") {
      const next = new Date(cursor);
      next.setMonth(next.getMonth() + direction);
      cursor = startOfDay(next);
    } else {
      cursor = addDays(cursor, direction * (mode === "week" ? 7 : 1));
    }
  }

  const hours = $derived(
    Array.from({ length: boot.agendaCloseHour - boot.agendaOpenHour }, (_, index) => boot.agendaOpenHour + index),
  );
  const monthCells = $derived(
    Array.from({ length: 42 }, (_, index) => addDays(startOfWeek(startOfMonth(cursor)), index)),
  );

  $effect(() => {
    mode;
    cursor.getTime();
    filterId;
    load().catch((cause) => (error = errText(cause)));
  });
</script>

<section class="agenda">
  <div class="calendar">
    <div class="cal-toolbar">
      <button class="btn" onclick={() => shift(-1)} aria-label="Précédent">‹</button>
      <button class="btn" onclick={() => (cursor = startOfDay(new Date()))}>Aujourd'hui</button>
      <button class="btn" onclick={() => shift(1)} aria-label="Suivant">›</button>
      <h2>{title}</h2>
      <select bind:value={employeeFilter}>
        <option value={0}>Tous les employés</option>
        {#each activeEmployees as employee (employee.id)}
          <option value={employee.id}>{fullName(employee.firstName, employee.lastName)}</option>
        {/each}
      </select>
      <div class="views">
        <button class="btn" class:on={mode === "day"} onclick={() => (mode = "day")}>Jour</button>
        <button class="btn" class:on={mode === "week"} onclick={() => (mode = "week")}>Semaine</button>
        <button class="btn" class:on={mode === "month"} onclick={() => (mode = "month")}>Mois</button>
      </div>
    </div>
    <div class="legend">
      {#each activeEmployees as employee (employee.id)}
        <span><i class="swatch" style:background={employee.color}></i>{fullName(employee.firstName, employee.lastName)}</span>
      {/each}
    </div>
    <div class="grid-wrap">
      {#if mode === "month"}
        <div class="month">
          {#each ["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."] as label}
            <div class="day-head">{label}</div>
          {/each}
          {#each monthCells as day}
            <button
              class="month-cell"
              class:out={day.getMonth() !== cursor.getMonth()}
              onclick={() => { cursor = startOfDay(day); mode = "day"; }}
            >
              <b>{day.getDate()}</b>
              <ul>
                {#each appointments.filter((item) => sameDay(new Date(item.startsAt), day)).slice(0, 3) as appointment (appointment.id)}
                  <li><i class="swatch" style:background={appointment.employeeColor}></i>{appointment.clientName}</li>
                {/each}
              </ul>
            </button>
          {/each}
        </div>
      {:else}
        <div class="week" style:grid-template-columns={`56px repeat(${Math.max(columns.length, 1)}, minmax(120px, 1fr))`}>
          <div class="day-head"></div>
          {#each columns as column (column.key)}
            <div class="day-head" class:today={sameDay(column.day, new Date())}>
              {#if column.color}<i class="swatch" style:background={column.color}></i>{/if}
              {column.label}
            </div>
          {/each}
          <div class="hours" style:height={`${hours.length * hourPx}px`}>
            {#each hours as hour}
              <div class="hline" style:top={`${(hour - boot.agendaOpenHour) * hourPx}px`}>{hour}h</div>
            {/each}
          </div>
          {#each columns as column (column.key)}
            <div
              class="slot-col"
              style:height={`${hours.length * hourPx}px`}
              role="presentation"
              onclick={(event) => slotFromClick(event, column.day, column.employeeId)}
            >
              {#each hours as hour}
                <div class="hline" style:top={`${(hour - boot.agendaOpenHour) * hourPx}px`}></div>
              {/each}
              {#each appointments.filter((item) => inColumn(item, column)) as appointment (appointment.id)}
                <button
                  class="apt"
                  class:on={selectedId === appointment.id}
                  style={`top:${topOf(appointment.startsAt)}px;height:${heightOf(appointment.startsAt, appointment.endsAt)}px;--c:${appointment.employeeColor}`}
                  onclick={(event) => { event.stopPropagation(); openAppointment(appointment); }}
                >
                  <b>{appointment.clientName}</b>
                  <span>{new Date(appointment.startsAt).toLocaleTimeString(boot.locale, { hour: "2-digit", minute: "2-digit" })} · {appointment.employeeName}</span>
                </button>
              {/each}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
  <aside class="detail" style="width: 340px; flex: none; border-left: 1px solid var(--line);">
    <header>
      <h2>{selected ? "Rendez-vous" : editing ? "Nouveau rendez-vous" : "Détail"}</h2>
      <button class="btn primary" onclick={() => draft(new Date(), null)}>Nouveau</button>
    </header>
    {#if !editing}
      <p class="empty">
        {#if selected}
          {selected.clientName} · {formatWhen(selected.startsAt, boot.locale)}
        {:else}
          Cliquez un créneau pour créer un rendez-vous, ou un bloc pour l'ouvrir.
        {/if}
      </p>
    {:else}
      <form class="form" onsubmit={(event) => { event.preventDefault(); save(); }}>
        {#if error}<p class="error">{error}</p>{/if}
        <label class="field">Client
          <select bind:value={clientId}>
            {#each clients as client (client.id)}
              <option value={client.id}>{fullName(client.firstName, client.lastName)}</option>
            {/each}
          </select>
        </label>
        {#if clients.length === 0}<p class="muted">Ajoutez d'abord un client.</p>{/if}
        <label class="field">Employé
          <select bind:value={employeeId}>
            {#each activeEmployees as employee (employee.id)}
              <option value={employee.id}>{fullName(employee.firstName, employee.lastName)}</option>
            {/each}
          </select>
        </label>
        <label class="field">Début <input type="datetime-local" bind:value={startValue} required /></label>
        <label class="field">Fin <input type="datetime-local" bind:value={endValue} required /></label>
        <label class="field">Statut
          <select bind:value={status}>
            {#each Object.entries(appointmentStatus) as [value, label]}
              <option {value}>{label}</option>
            {/each}
          </select>
        </label>
        <div class="field">
          <span>Prestations</span>
          {#each services as service (service.id)}
            <label class="check">
              <input
                type="checkbox"
                checked={serviceIds.includes(service.id)}
                onchange={() => toggleService(service.id)}
              />
              <span>{service.name} · {service.durationMinutes} min · {formatMoney(service.priceCents, boot.settings.currency, boot.locale)}</span>
            </label>
          {:else}
            <p class="muted">Aucune prestation active.</p>
          {/each}
        </div>
        <label class="field">Notes <textarea bind:value={notes}></textarea></label>
        <div class="actions">
          <button class="btn primary" disabled={busy}>Enregistrer</button>
        </div>
      </form>
    {/if}
  </aside>
</section>
