const stateUrl = "/api/state";
const summaryUrl = "/api/summary";
const eventsUrl = "/api/events";
const actionsUrl = "/api/actions";

const statusDot = document.getElementById("status-dot");
const statusText = document.getElementById("status-text");
const summaryRun = document.getElementById("summary-run");
const summaryBudget = document.getElementById("summary-budget");
const summaryHistory = document.getElementById("summary-history");
const summaryBest = document.getElementById("summary-best");
const summaryLatest = document.getElementById("summary-latest");
const historyList = document.getElementById("history-list");
const eventList = document.getElementById("event-list");
const actionList = document.getElementById("action-list");
const filterEvent = document.getElementById("filter-event");
const filterSearch = document.getElementById("filter-search");

const chartCanvas = document.getElementById("trend-chart");
const chartCtx = chartCanvas.getContext("2d");

let lastHistory = [];
let eventsFilter = { event: "", q: "" };

async function fetchJson(url, params = {}) {
  const query = new URLSearchParams(params);
  const full = query.toString() ? `${url}?${query}` : url;
  const response = await fetch(full);
  if (!response.ok) throw new Error("Network error");
  return response.json();
}

function setStatus(ok) {
  statusDot.style.background = ok ? "#42ffb7" : "#ff4f4f";
  statusText.textContent = ok ? "Live" : "Offline";
}

function renderSummary(summary) {
  summaryRun.textContent = summary.run_id || "—";
  summaryBudget.textContent = summary.budget ?? "—";
  summaryHistory.textContent = summary.history_len ?? "—";
  summaryBest.textContent = summary.best ?? "—";
  summaryLatest.textContent = summary.latest ?? "—";
}

function renderHistory(history) {
  historyList.innerHTML = "";
  if (!history.length) {
    historyList.appendChild(makeItem("No evaluations yet"));
    return;
  }
  history.slice(-8).reverse().forEach((entry) => {
    const params = Object.entries(entry.params || {})
      .map(([k, v]) => `${k}=${Number(v).toFixed(4)}`)
      .join(", ");
    historyList.appendChild(makeItem(`value=${entry.value.toFixed(6)} | ${params}`));
  });
}

function renderEvents(events) {
  eventList.innerHTML = "";
  if (!events.length) {
    eventList.appendChild(makeItem("No events"));
    return;
  }
  events.slice(-10).reverse().forEach((event) => {
    const type = event.event || event.event_type || "event";
    const ts = event.timestamp_us || 0;
    const detail = event.value ?? "";
    eventList.appendChild(makeItem(`${ts} ${type} ${detail}`));
  });
}

function renderActions(actions) {
  actionList.innerHTML = "";
  if (!actions.length) {
    actionList.appendChild(makeItem("No actions queued"));
    return;
  }
  actions.slice(-6).reverse().forEach((action) => {
    const ts = action.timestamp_us || 0;
    const name = action.action || "action";
    const reason = action.reason || "";
    actionList.appendChild(makeItem(`${ts} ${name} ${reason}`));
  });
}

function makeItem(text) {
  const div = document.createElement("div");
  div.className = "list-item";
  div.textContent = text;
  return div;
}

function drawChart(values) {
  chartCtx.clearRect(0, 0, chartCanvas.width, chartCanvas.height);
  if (!values.length) return;
  const width = chartCanvas.width;
  const height = chartCanvas.height;
  const padding = 12;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  chartCtx.strokeStyle = "#7c5cff";
  chartCtx.lineWidth = 2;
  chartCtx.beginPath();
  values.forEach((value, index) => {
    const x = padding + (index / (values.length - 1)) * (width - padding * 2);
    const y = height - padding - ((value - min) / range) * (height - padding * 2);
    if (index === 0) chartCtx.moveTo(x, y);
    else chartCtx.lineTo(x, y);
  });
  chartCtx.stroke();
}

async function refresh() {
  try {
    const summary = await fetchJson(summaryUrl);
    renderSummary(summary);
    const state = await fetchJson(stateUrl);
    lastHistory = state.history || [];
    renderHistory(lastHistory);
    drawChart(lastHistory.map((entry) => entry.value));
    const events = await fetchJson(eventsUrl, eventsFilter);
    renderEvents(events.events || []);
    const actions = await fetchJson(actionsUrl);
    renderActions(actions.actions || []);
    setStatus(true);
  } catch (err) {
    setStatus(false);
  }
}

document.getElementById("filter-apply").addEventListener("click", () => {
  eventsFilter = {
    event: filterEvent.value.trim(),
    q: filterSearch.value.trim(),
  };
  refresh();
});

document.getElementById("action-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  const action = document.getElementById("action-name").value.trim();
  const reason = document.getElementById("action-reason").value.trim();
  if (!action) return;
  await fetch(actionsUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action, reason }),
  });
  document.getElementById("action-name").value = "";
  document.getElementById("action-reason").value = "";
  refresh();
});

setInterval(refresh, 1500);
refresh();
