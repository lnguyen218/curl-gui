<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writable } from "svelte/store";
  import { onMount } from "svelte";

  type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";

  interface Header {
    key: string;
    value: string;
  }

  interface SavedRequest {
    id: string;
    name: string;
    method: HttpMethod;
    url: string;
    headers: Header[];
    body: string;
    createdAt: number;
  }

  interface ResponseData {
    status_code: number;
    status_text: string;
    headers: Record<string, string>;
    body: string;
    time_ms: number;
  }

  // Create a custom store with actions
  const createRequestStore = () => {
    const { subscribe, set, update } = writable<{
      method: HttpMethod;
      url: string;
      headers: Header[];
      body: string;
    }>({
      method: "GET",
      url: "",
      headers: [{ key: "", value: "" }],
      body: "",
    });

    return {
      subscribe,
      setMethod: (m: HttpMethod) => update(s => ({ ...s, method: m })),
      setUrl: (u: string) => update(s => ({ ...s, url: u })),
      setHeaders: (h: Header[]) => set({ ...get({ subscribe }), headers: h }),
      setBody: (b: string) => update(s => ({ ...s, body: b })),
      load: (data: { method: HttpMethod; url: string; headers: Header[]; body: string }) => set(data),
      getState: () => {
        let state: any;
        subscribe(s => { state = s; })();
        return state;
      }
    };
  };

  // Helper to get current store value
  function get<T>(store: { subscribe: (fn: (v: T) => void) => void }): T {
    let value: T;
    store.subscribe(v => value = v)();
    return value!;
  }

  const request = createRequestStore();

  // Component state
  let savedRequests = writable<SavedRequest[]>([]);
  let showSaveDialog = false;
  let saveName = "";
  let editingRequest: SavedRequest | null = null;
  let response: ResponseData | null = null;
  let loading = false;
  let error = "";
  let activeTab: "headers" | "body" | "params" = "headers";
  let responseTab: "body" | "headers" = "body";
  let curlCommand = "";
  let searchFilter = "";

  const methods: HttpMethod[] = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

  const getMethodColor = (method: HttpMethod): string => {
    switch (method) {
      case "GET": return "#61affe";
      case "POST": return "#49cc90";
      case "PUT": return "#fca130";
      case "DELETE": return "#f93e3e";
      case "PATCH": return "#50e3c2";
      case "HEAD": return "#9012fe";
      case "OPTIONS": return "#0d5aa7";
      default: return "#999";
    }
  };

  const loadFromStorage = () => {
    const saved = localStorage.getItem("curl-gui-saved-requests");
    if (saved) {
      try {
        savedRequests.set(JSON.parse(saved));
      } catch {
        savedRequests.set([]);
      }
    }

    const current = localStorage.getItem("curl-gui-current");
    if (current) {
      try {
        const parsed = JSON.parse(current);
        request.load({
          method: parsed.method || "GET",
          url: parsed.url || "",
          headers: Array.isArray(parsed.headers) && parsed.headers.length > 0
            ? parsed.headers
            : [{ key: "", value: "" }],
          body: parsed.body || "",
        });
      } catch {
        // Keep defaults
      }
    }
  };

  const saveToStorage = () => {
    localStorage.setItem("curl-gui-current", JSON.stringify(get(request)));
  };

  const addHeader = () => {
    const state = get(request);
    request.setHeaders([...state.headers, { key: "", value: "" }]);
    saveToStorage();
  };

  const removeHeader = (index: number) => {
    const state = get(request);
    request.setHeaders(state.headers.filter((_, i) => i !== index));
    saveToStorage();
  };

  const updateHeader = (index: number, field: "key" | "value", value: string) => {
    const state = get(request);
    request.setHeaders(
      state.headers.map((h, i) => (i === index ? { ...h, [field]: value } : h))
    );
    saveToStorage();
  };

  const generateCurlCommand = (): string => {
    const state = get(request);
    let cmd = `curl -X ${state.method}`;
    state.headers
      .filter(h => h.key && h.value)
      .forEach(h => { cmd += ` -H "${h.key}: ${h.value}"`; });
    if (state.body && state.method !== "GET") {
      cmd += ` -d '${state.body}'`;
    }
    cmd += ` "${state.url}"`;
    return cmd;
  };

  const sendRequest = async () => {
    const state = get(request);
    if (!state.url) {
      error = "Please enter a URL";
      return;
    }

    loading = true;
    error = "";
    response = null;
    curlCommand = generateCurlCommand();

    const headers: Record<string, string> = {};
    state.headers.filter(h => h.key && h.value).forEach(h => { headers[h.key] = h.value; });

    try {
      const result = await invoke<ResponseData>("make_request", {
        request: {
          method: state.method,
          url: state.url,
          headers: Object.keys(headers).length > 0 ? headers : undefined,
          body: state.body || undefined,
        },
      });
      response = result;
    } catch (e: any) {
      error = e.error || e.toString();
    } finally {
      loading = false;
    }
  };

  const saveCurrentRequest = () => {
    if (!saveName.trim()) return;
    const state = get(request);

    const newReq: SavedRequest = {
      id: editingRequest?.id || crypto.randomUUID(),
      name: saveName.trim(),
      method: state.method,
      url: state.url,
      headers: JSON.parse(JSON.stringify(state.headers)),
      body: state.body,
      createdAt: editingRequest?.createdAt || Date.now(),
    };

    savedRequests.update(reqs => {
      if (editingRequest) {
        return reqs.map(r => r.id === editingRequest!.id ? newReq : r);
      }
      return [newReq, ...reqs];
    });

    localStorage.setItem("curl-gui-saved-requests", JSON.stringify(get(savedRequests)));
    showSaveDialog = false;
    saveName = "";
    editingRequest = null;
  };

  const loadRequest = (saved: SavedRequest) => {
    response = null;
    error = "";
    curlCommand = "";
    request.load({
      method: saved.method,
      url: saved.url,
      headers: JSON.parse(JSON.stringify(saved.headers)),
      body: saved.body,
    });
    saveToStorage();
  };

  const deleteRequest = (id: string, e: Event) => {
    e.stopPropagation();
    savedRequests.update(reqs => reqs.filter(r => r.id !== id));
    localStorage.setItem("curl-gui-saved-requests", JSON.stringify(get(savedRequests)));
  };

  const editRequestName = (saved: SavedRequest, e: Event) => {
    e.stopPropagation();
    editingRequest = saved;
    saveName = saved.name;
    showSaveDialog = true;
  };

  const openSaveDialog = () => {
    editingRequest = null;
    saveName = "";
    showSaveDialog = true;
  };

  const formatBody = (body: string): string => {
    try {
      return JSON.stringify(JSON.parse(body), null, 2);
    } catch {
      return body;
    }
  };

  const formatCurl = (cmd: string): string => {
    return cmd.replace(/ -H /g, " \\\n  -H ").replace(/ -d /g, " \\\n  -d ").replace(/ -X /g, " \\\n  -X ");
  };

  const filteredRequests = writable<SavedRequest[]>([]);
  savedRequests.subscribe(reqs => {
    filteredRequests.set(
      reqs.filter(r => 
        r.name.toLowerCase().includes(searchFilter.toLowerCase()) ||
        r.url.toLowerCase().includes(searchFilter.toLowerCase())
      )
    );
  });

  onMount(() => {
    loadFromStorage();
  });
</script>

<main class="app">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="logo">
        <span class="curl-logo">curl</span>
        <span class="divider">|</span>
        <span class="app-name">GUI</span>
      </div>
      <button class="new-request-btn" onclick={openSaveDialog} title="Save request">+</button>
    </div>

    <div class="search-box">
      <input type="text" bind:value={searchFilter} placeholder="Search..." class="search-input" />
    </div>

    <div class="saved-requests">
      {#if $filteredRequests.length === 0}
        <div class="empty-sidebar">
          {#if searchFilter}
            <p>No matches</p>
          {:else}
            <p>No saved requests</p>
            <p class="hint">Click + to save</p>
          {/if}
        </div>
      {:else}
        {#each $filteredRequests as saved (saved.id)}
          <div class="saved-request-item" onclick={() => loadRequest(saved)}>
            <div class="request-info">
              <span class="method-badge" style="color: {getMethodColor(saved.method)}">{saved.method}</span>
              <span class="request-name" title={saved.name}>{saved.name}</span>
            </div>
            <div class="request-url" title={saved.url}>{saved.url}</div>
            <div class="request-actions">
              <button class="action-btn edit" onclick={(e) => editRequestName(saved, e)} title="Rename">✎</button>
              <button class="action-btn delete" onclick={(e) => deleteRequest(saved.id, e)} title="Delete">×</button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </aside>

  <!-- Main Content -->
  <div class="main-content">
    <!-- Request Panel -->
    <div class="request-panel">
      <div class="url-bar">
        <select class="method-select" 
          value={$request.method}
          onchange={(e) => { request.setMethod(e.currentTarget.value as HttpMethod); saveToStorage(); }}
        >
          {#each methods as method}
            <option value={method} style="color: {getMethodColor(method)}">{method}</option>
          {/each}
        </select>

        <input type="text" value={$request.url} 
          oninput={(e) => { request.setUrl(e.currentTarget.value); saveToStorage(); }}
          placeholder="https://api.example.com/endpoint" class="url-input" 
        />

        <button onclick={sendRequest} class="send-btn" disabled={loading}>
          {#if loading}<span class="spinner"></span>{:else}Send{/if}
        </button>
        <button onclick={openSaveDialog} class="save-btn" title="Save">💾</button>
      </div>

      <div class="request-tabs">
        <button class="tab-btn" class:active={activeTab === "params"} onclick={() => activeTab = "params"}>Params</button>
        <button class="tab-btn" class:active={activeTab === "headers"} onclick={() => activeTab = "headers"}>
          Headers ({$request.headers.filter(h => h.key).length})
        </button>
        <button class="tab-btn" class:active={activeTab === "body"} onclick={() => activeTab = "body"}>Body</button>
      </div>

      <div class="tab-content">
        {#if activeTab === "headers"}
          <div class="headers-section">
            {#each $request.headers as header, i (i)}
              <div class="header-row">
                <input type="text" placeholder="Key" value={header.key} 
                  oninput={(e) => updateHeader(i, "key", e.currentTarget.value)} class="header-key" />
                <input type="text" placeholder="Value" value={header.value}
                  oninput={(e) => updateHeader(i, "value", e.currentTarget.value)} class="header-value" />
                <button onclick={() => removeHeader(i)} class="remove-btn">×</button>
              </div>
            {/each}
            <button onclick={addHeader} class="add-header-btn">+ Add Header</button>
          </div>
        {:else if activeTab === "body"}
          <div class="body-section">
            <textarea value={$request.body}
              oninput={(e) => { request.setBody(e.currentTarget.value); saveToStorage(); }}
              placeholder={`{\n  "key": "value"\n}`} class="body-editor"
            ></textarea>
          </div>
        {:else}
          <div class="params-section">
            <p class="placeholder">Query params parsed from URL</p>
          </div>
        {/if}
      </div>
    </div>

    <div class="divider-line"></div>

    <!-- Response Panel -->
    <div class="response-panel">
      {#if response}
        <div class="response-meta">
          <span class="status-code" class:success={response.status_code >= 200 && response.status_code < 300} 
            class:error={response.status_code >= 400}>
            {response.status_code} {response.status_text}
          </span>
          <span class="response-time">{response.time_ms}ms</span>
        </div>

        <div class="response-tabs">
          <button class="tab-btn" class:active={responseTab === "body"} onclick={() => responseTab = "body"}>Body</button>
          <button class="tab-btn" class:active={responseTab === "headers"} onclick={() => responseTab = "headers"}>
            Headers ({Object.keys(response.headers).length})
          </button>
        </div>

        <div class="tab-content">
          {#if responseTab === "body"}
            <pre class="response-body">{formatBody(response.body)}</pre>
          {:else}
            <div class="response-headers">
              {#each Object.entries(response.headers) as [key, value]}
                <div class="header-item">
                  <span class="header-key-name">{key}:</span>
                  <span class="header-value-text">{value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if error}
        <div class="error-message"><span class="error-icon">⚠</span>{error}</div>
      {:else if curlCommand}
        <div class="curl-preview">
          <h3>cURL Command</h3>
          <pre class="curl-code">{formatCurl(curlCommand)}</pre>
        </div>
      {:else}
        <div class="empty-state"><p>Enter URL and click Send</p></div>
      {/if}
    </div>
  </div>
</main>

<!-- Modal -->
{#if showSaveDialog}
  <div class="modal-overlay" onclick={() => showSaveDialog = false}>
    <div class="modal" onclick|stopPropagation>
      <h3>{editingRequest ? 'Rename' : 'Save Request'}</h3>
      <input type="text" bind:value={saveName} placeholder="Name..." class="modal-input" 
        onkeydown={(e) => e.key === 'Enter' && saveCurrentRequest()} />
      <div class="modal-actions">
        <button class="modal-btn secondary" onclick={() => { showSaveDialog = false; editingRequest = null; }}>Cancel</button>
        <button class="modal-btn primary" onclick={saveCurrentRequest} disabled={!saveName.trim()}>
          {editingRequest ? 'Update' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body) { margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #1a1a2e; color: #e4e4e7; }
  .app { height: 100vh; display: flex; }

  /* Sidebar */
  .sidebar { width: 280px; min-width: 280px; background: #16162a; border-right: 1px solid #2a2a3e; display: flex; flex-direction: column; }
  .sidebar-header { padding: 12px 15px; display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid #2a2a3e; }
  .logo { display: flex; align-items: center; gap: 6px; font-size: 16px; font-weight: 600; }
  .curl-logo { color: #61affe; }
  .divider { color: #666; }
  .app-name { color: #e4e4e7; }
  .new-request-btn { width: 28px; height: 28px; border-radius: 6px; border: none; background: #49cc90; color: #fff; font-size: 18px; cursor: pointer; }
  .new-request-btn:hover { opacity: 0.9; }
  .search-box { padding: 10px 15px; border-bottom: 1px solid #2a2a3e; }
  .search-input { width: 100%; padding: 8px 12px; border-radius: 6px; border: 1px solid #3a3a4e; background: #1e1e2e; color: #e4e4e7; font-size: 13px; box-sizing: border-box; }
  .saved-requests { flex: 1; overflow-y: auto; padding: 10px; }
  .empty-sidebar { padding: 40px; text-align: center; color: #666; }
  .empty-sidebar .hint { font-size: 12px; margin-top: 10px; color: #888; }
  .saved-request-item { padding: 12px; border-radius: 8px; background: #1e1e2e; margin-bottom: 8px; cursor: pointer; border: 1px solid transparent; position: relative; }
  .saved-request-item:hover { background: #2a2a3e; border-color: #3a3a4e; }
  .saved-request-item:hover .request-actions { opacity: 1; }
  .request-info { display: flex; align-items: center; gap: 8px; margin-bottom: 4px; }
  .method-badge { font-size: 11px; font-weight: 700; min-width: 50px; }
  .request-name { font-weight: 500; font-size: 13px; color: #e4e4e7; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex: 1; }
  .request-url { font-size: 11px; color: #888; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-left: 58px; }
  .request-actions { position: absolute; top: 8px; right: 8px; display: flex; gap: 4px; opacity: 0; transition: opacity 0.2s; }
  .action-btn { width: 24px; height: 24px; border-radius: 4px; border: none; background: #3a3a4e; color: #e4e4e7; font-size: 14px; cursor: pointer; }
  .action-btn:hover { background: #61affe; }
  .action-btn.delete:hover { background: #f93e3e; }

  /* Main */
  .main-content { flex: 1; display: flex; overflow: hidden; }
  .request-panel { flex: 1; display: flex; flex-direction: column; padding: 20px; background: #1e1e2e; min-width: 0; }
  .url-bar { display: flex; gap: 10px; margin-bottom: 20px; }
  .method-select { padding: 10px 15px; border-radius: 6px; border: 1px solid #3a3a4e; background: #2a2a3e; color: #e4e4e7; font-size: 14px; font-weight: 600; min-width: 100px; cursor: pointer; }
  .url-input { flex: 1; padding: 10px 15px; border-radius: 6px; border: 1px solid #3a3a4e; background: #2a2a3e; color: #e4e4e7; font-size: 14px; }
  .url-input:focus { outline: none; border-color: #61affe; }
  .send-btn { padding: 10px 30px; border-radius: 6px; border: none; background: #49cc90; color: #fff; font-size: 14px; font-weight: 600; cursor: pointer; }
  .send-btn:hover:not(:disabled) { opacity: 0.9; }
  .send-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .save-btn { padding: 10px 15px; border-radius: 6px; border: 1px solid #3a3a4e; background: #2a2a3e; color: #e4e4e7; font-size: 16px; cursor: pointer; }
  .save-btn:hover { border-color: #61affe; }
  .spinner { width: 16px; height: 16px; border: 2px solid #fff; border-top-color: transparent; border-radius: 50%; animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .request-tabs { display: flex; gap: 5px; border-bottom: 1px solid #3a3a4e; margin-bottom: 15px; }
  .tab-btn { padding: 10px 20px; background: transparent; border: none; color: #888; font-size: 13px; cursor: pointer; border-bottom: 2px solid transparent; }
  .tab-btn:hover { color: #e4e4e7; }
  .tab-btn.active { color: #61affe; border-bottom-color: #61affe; }
  .tab-content { flex: 1; overflow-y: auto; }
  .headers-section { display: flex; flex-direction: column; gap: 10px; }
  .header-row { display: flex; gap: 10px; align-items: center; }
  .header-key, .header-value { flex: 1; padding: 8px 12px; border-radius: 4px; border: 1px solid #3a3a4e; background: #2a2a3e; color: #e4e4e7; font-size: 13px; }
  .header-key:focus, .header-value:focus { outline: none; border-color: #61affe; }
  .remove-btn { width: 30px; height: 30px; border-radius: 4px; border: none; background: #3a3a4e; color: #e4e4e7; font-size: 18px; cursor: pointer; }
  .remove-btn:hover { background: #f93e3e; color: #fff; }
  .add-header-btn { padding: 10px; border-radius: 4px; border: 1px dashed #3a3a4e; background: transparent; color: #888; font-size: 13px; cursor: pointer; margin-top: 10px; }
  .add-header-btn:hover { border-color: #61affe; color: #61affe; }
  .body-section { height: 100%; }
  .body-editor { width: 100%; height: 100%; padding: 15px; border-radius: 6px; border: 1px solid #3a3a4e; background: #2a2a3e; color: #e4e4e7; font-family: Monaco, Menlo, monospace; font-size: 13px; resize: none; box-sizing: border-box; }
  .body-editor:focus { outline: none; border-color: #61affe; }
  .placeholder { color: #666; text-align: center; padding: 40px; }
  .divider-line { width: 2px; background: #2a2a3e; }
  .response-panel { flex: 1; display: flex; flex-direction: column; padding: 20px; background: #1a1a2e; min-width: 0; }
  .response-meta { display: flex; align-items: center; gap: 15px; margin-bottom: 20px; }
  .status-code { padding: 6px 12px; border-radius: 4px; font-size: 13px; font-weight: 600; background: #3a3a4e; }
  .status-code.success { background: #49cc9020; color: #49cc90; }
  .status-code.error { background: #f93e3e20; color: #f93e3e; }
  .response-time { color: #888; font-size: 13px; }
  .response-body { margin: 0; padding: 15px; background: #2a2a3e; border-radius: 6px; font-family: Monaco, Menlo, monospace; font-size: 12px; line-height: 1.5; overflow-x: auto; white-space: pre-wrap; }
  .response-headers { padding: 15px; background: #2a2a3e; border-radius: 6px; }
  .header-item { padding: 8px 0; border-bottom: 1px solid #3a3a4e; display: flex; gap: 10px; font-size: 13px; }
  .header-item:last-child { border-bottom: none; }
  .header-key-name { color: #61affe; font-weight: 500; min-width: 150px; }
  .header-value-text { color: #e4e4e7; }
  .error-message { padding: 20px; background: #f93e3e20; border-radius: 6px; color: #f93e3e; display: flex; align-items: center; gap: 10px; }
  .error-icon { font-size: 20px; }
  .curl-preview { padding: 20px; }
  .curl-preview h3 { margin: 0 0 15px 0; color: #888; font-size: 14px; font-weight: 500; }
  .curl-code { margin: 0; padding: 15px; background: #2a2a3e; border-radius: 6px; font-family: Monaco, Menlo, monospace; font-size: 12px; line-height: 1.6; overflow-x: auto; white-space: pre-wrap; color: #a6e22e; }
  .empty-state { flex: 1; display: flex; align-items: center; justify-content: center; color: #666; }

  /* Modal */
  .modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000; }
  .modal { background: #1e1e2e; border-radius: 12px; padding: 24px; width: 400px; max-width: 90%; border: 1px solid #3a3a4e; }
  .modal h3 { margin: 0 0 16px 0; color: #e4e4e7; font-size: 18px; }
  .modal-input { width: 100%; padding: 12px 16px; border-radius: 8px; border: 1px solid #3a3a4e; background: #2a2a3e; color: #e4e4e7; font-size: 14px; box-sizing: border-box; margin-bottom: 20px; }
  .modal-input:focus { outline: none; border-color: #61affe; }
  .modal-actions { display: flex; gap: 10px; justify-content: flex-end; }
  .modal-btn { padding: 10px 20px; border-radius: 6px; border: none; font-size: 14px; font-weight: 500; cursor: pointer; }
  .modal-btn:hover:not(:disabled) { opacity: 0.9; }
  .modal-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .modal-btn.secondary { background: #3a3a4e; color: #e4e4e7; }
  .modal-btn.primary { background: #61affe; color: #fff; }
</style>