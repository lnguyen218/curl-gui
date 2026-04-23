<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writable } from "svelte/store";
  import { onMount } from "svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import RequestPanel from "./lib/RequestPanel.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import type { Header, HttpMethod, SavedRequest, ResponseData, SslConfig, AuthConfig } from "./types";

  // Request state
  let method: HttpMethod = "GET";
  let url: string = "";
  let headers: Header[] = [{ key: "", value: "" }];
  let body: string = "";
  let authConfig: AuthConfig = {
    type: "none",
    username: "",
    password: "",
    token: "",
    apiKeyName: "",
    apiKeyValue: "",
    apiKeyIn: "header"
  };

  // Global SSL settings (shared across all requests)
  let sslConfig: SslConfig = {
    verifySsl: true,
    certPath: "",
    keyPath: "",
    caPath: ""
  };

  // Saved requests
  const savedRequests = writable<SavedRequest[]>([]);
  
  // UI state
  let activeRequestId: string | null = null;
  let response: ResponseData | null = null;
  let loading = false;
  let error = "";
  let curlCommand = "";
  let searchFilter = "";
  
  // Modal state
  let showModal = false;
  let modalName = "";
  let editingRequest: SavedRequest | null = null;

  // SSL settings modal
  let showSslModal = false;

  onMount(() => {
    // Load saved requests
    const saved = localStorage.getItem("curl-gui-saved-requests");
    if (saved) {
      try {
        savedRequests.set(JSON.parse(saved));
      } catch {
        savedRequests.set([]);
      }
    }

    // Load current request state
    const current = localStorage.getItem("curl-gui-current");
    if (current) {
      try {
        const parsed = JSON.parse(current);
        method = parsed.method || "GET";
        url = parsed.url || "";
        headers = Array.isArray(parsed.headers) && parsed.headers.length > 0
          ? parsed.headers
          : [{ key: "", value: "" }];
        body = parsed.body || "";
        authConfig = parsed.authConfig || { type: "none", username: "", password: "", token: "", apiKeyName: "", apiKeyValue: "", apiKeyIn: "header" };
      } catch {
        // Keep defaults
      }
    }

    // Load global SSL settings
    const savedSsl = localStorage.getItem("curl-gui-ssl");
    if (savedSsl) {
      try {
        sslConfig = JSON.parse(savedSsl);
      } catch {
        // Keep defaults
      }
    }
  });

  // Auto-save: persist current request state whenever it changes
  $: {
    const state = { method, url, headers, body, authConfig };
    localStorage.setItem("curl-gui-current", JSON.stringify(state));
  }

  // Auto-save: persist global SSL settings whenever they change
  $: {
    localStorage.setItem("curl-gui-ssl", JSON.stringify(sslConfig));
  }

  function persistRequests() {
    localStorage.setItem("curl-gui-saved-requests", JSON.stringify($savedRequests));
  }

  function autoSaveRequest() {
    // If there's an active saved request, update it automatically
    if (activeRequestId) {
      savedRequests.update(reqs => {
        return reqs.map(r => {
          if (r.id === activeRequestId) {
            return {
              ...r,
              method,
              url,
              headers: JSON.parse(JSON.stringify(headers)),
              body,
              authConfig,
              response: response || undefined,
              curlCommand: curlCommand || undefined,
            };
          }
          return r;
        });
      });
      persistRequests();
    }
  }

  async function sendRequest() {
    if (!url) {
      error = "Please enter a URL";
      return;
    }

    loading = true;
    error = "";
    response = null;
    
    const hdrs: Record<string, string> = {};
    headers.filter(h => h.key && h.value).forEach(h => { hdrs[h.key] = h.value; });
    
    // Apply authentication
    let authHeaders: Record<string, string> = {};
    let authCurlFlags = "";
    
    if (authConfig.type === "basic" && authConfig.username) {
      const encoded = btoa(`${authConfig.username}:${authConfig.password}`);
      authHeaders["Authorization"] = `Basic ${encoded}`;
      authCurlFlags = ` -u "${authConfig.username}:${authConfig.password}"`;
    } else if (authConfig.type === "bearer" && authConfig.token) {
      const token = authConfig.token.trim();
      authHeaders["Authorization"] = `Bearer ${token}`;
      authCurlFlags = ` -H "Authorization: Bearer ${token}"`;
    } else if (authConfig.type === "api-key" && authConfig.apiKeyName && authConfig.apiKeyValue) {
      const apiKeyValue = authConfig.apiKeyValue.trim();
      if (authConfig.apiKeyIn === "header") {
        authHeaders[authConfig.apiKeyName] = apiKeyValue;
        authCurlFlags = ` -H "${authConfig.apiKeyName}: ${apiKeyValue}"`;
      }
    }
    
    // Merge auth headers with user headers
    const allHeaders = { ...authHeaders, ...hdrs };
    
    // Build curl command
    curlCommand = `curl -X ${method}${Object.entries(hdrs).map(([k, v]) => ` -H "${k}: ${v}"`).join("")}${authCurlFlags}${body && method !== "GET" ? ` -d '${body}'` : ""}${!sslConfig.verifySsl ? " --insecure" : ""}${sslConfig.certPath ? ` --cert ${sslConfig.certPath}` : ""}${sslConfig.keyPath ? ` --key ${sslConfig.keyPath}` : ""}${sslConfig.caPath ? ` --cacert ${sslConfig.caPath}` : ""} "${url}"`;

    try {
      response = await invoke<ResponseData>("make_request", {
        request: {
          method,
          url,
          headers: Object.keys(allHeaders).length > 0 ? allHeaders : undefined,
          body: body || undefined,
          verify_ssl: sslConfig.verifySsl,
          ssl_cert: sslConfig.certPath || undefined,
          ssl_key: sslConfig.keyPath || undefined,
          ssl_ca: sslConfig.caPath || undefined,
        },
      });
    } catch (e: any) {
      error = e.error || e.toString();
    } finally {
      loading = false;
    }
  }

  function saveRequest() {
    if (!modalName.trim()) return;
    
    const newReq: SavedRequest = {
      id: editingRequest?.id || crypto.randomUUID(),
      name: modalName.trim(),
      method,
      url,
      headers: JSON.parse(JSON.stringify(headers)),
      body,
      authConfig,
      createdAt: editingRequest?.createdAt || Date.now(),
      response: response || undefined,
      curlCommand: curlCommand || undefined,
    };

    savedRequests.update(reqs => {
      if (editingRequest) {
        return reqs.map(r => r.id === editingRequest.id ? newReq : r);
      }
      return [newReq, ...reqs];
    });

    persistRequests();
    closeModal();

    // If creating a new request (not editing), select it and reset form to fresh state
    if (!editingRequest) {
      activeRequestId = newReq.id;
      resetForm();
    }
  }

  function resetForm() {
    method = "GET";
    url = "";
    headers = [{ key: "", value: "" }];
    body = "";
    authConfig = { type: "none", username: "", password: "", token: "", apiKeyName: "", apiKeyValue: "", apiKeyIn: "header" };
    activeRequestId = null;
    response = null;
    error = "";
    curlCommand = "";
  }

  function loadRequest(saved: SavedRequest) {
    method = saved.method;
    url = saved.url;
    headers = JSON.parse(JSON.stringify(saved.headers));
    body = saved.body;
    authConfig = saved.authConfig || { type: "none", username: "", password: "", token: "", apiKeyName: "", apiKeyValue: "", apiKeyIn: "header" };
    activeRequestId = saved.id;
    response = saved.response || null;
    error = "";
    curlCommand = saved.curlCommand || "";
  }

  function deleteRequest(id: string) {
    savedRequests.update(reqs => reqs.filter(r => r.id !== id));
    persistRequests();
  }

  function editRequestName(saved: SavedRequest) {
    editingRequest = saved;
    modalName = saved.name;
    showModal = true;
  }

  function openSaveModal() {
    editingRequest = null;
    modalName = "";
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    modalName = "";
    editingRequest = null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') saveRequest();
    if (e.key === 'Escape') closeModal();
  }

  function openSslModal() {
    showSslModal = true;
  }

  function closeSslModal() {
    showSslModal = false;
  }

  import { open } from "@tauri-apps/plugin-dialog";

  async function pickFile(field: "certPath" | "keyPath" | "caPath") {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Certificate/Key Files",
          extensions: ["pem", "crt", "cert", "key", "der", "pfx", "p12"],
        },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (selected) {
      sslConfig = { ...sslConfig, [field]: selected };
    }
  }

  function clearFile(field: "certPath" | "keyPath" | "caPath") {
    sslConfig = { ...sslConfig, [field]: "" };
  }
</script>

<main class="app">
  <Sidebar
    savedRequests={$savedRequests}
    bind:searchFilter
    {activeRequestId}
    on:load={(e) => loadRequest(e.detail)}
    on:delete={(e) => deleteRequest(e.detail)}
    on:edit={(e) => editRequestName(e.detail)}
    on:saveNew={openSaveModal}
    on:search={(e) => searchFilter = e.detail}
    on:openSsl={openSslModal}
  />

  <div class="main-content">
    <RequestPanel
      bind:method
      bind:url
      bind:headers
      bind:body
      bind:authConfig
      {loading}
      on:send={sendRequest}
      on:update={autoSaveRequest}
    />

    <div class="divider-line"></div>

    <ResponsePanel
      {response}
      {error}
      {curlCommand}
    />
  </div>
</main>

{#if showModal}
  <div class="modal-overlay" on:click={closeModal}>
    <div class="modal" on:click|stopPropagation>
      <h3>{editingRequest ? 'Rename Request' : 'Save Request'}</h3>
      <input 
        type="text" 
        bind:value={modalName} 
        placeholder="Request name..." 
        class="modal-input"
        on:keydown={handleKeydown}
      />
      <div class="modal-actions">
        <button class="modal-btn secondary" on:click={closeModal}>Cancel</button>
        <button class="modal-btn primary" on:click={saveRequest} disabled={!modalName.trim()}>
          {editingRequest ? 'Update' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showSslModal}
  <div class="modal-overlay" on:click={closeSslModal}>
    <div class="modal ssl-modal" on:click|stopPropagation>
      <h3>🔒 SSL Settings</h3>

      <div class="ssl-section">
        <div class="ssl-option">
          <label class="ssl-label">
            <input
              type="checkbox"
              bind:checked={sslConfig.verifySsl}
              class="ssl-checkbox"
            />
            <span>Verify SSL Certificate</span>
          </label>
          <p class="ssl-hint">
            {#if sslConfig.verifySsl}
              SSL certificates will be verified (recommended for production)
            {:else}
              ⚠️ SSL verification disabled — insecure, use only for development
            {/if}
          </p>
        </div>

        <div class="ssl-divider"></div>

        <div class="ssl-option">
          <label class="ssl-label">Client Certificate (Cert)</label>
          <div class="file-input-row">
            <input
              type="text"
              bind:value={sslConfig.certPath}
              placeholder="/path/to/client-cert.pem"
              class="file-input"
              readonly
            />
            <button on:click={() => pickFile('certPath')} class="file-btn">Browse</button>
            {#if sslConfig.certPath}
              <button on:click={() => clearFile('certPath')} class="clear-btn">Clear</button>
            {/if}
          </div>
          <p class="ssl-hint">Client certificate for mutual TLS authentication</p>
        </div>

        <div class="ssl-option">
          <label class="ssl-label">Client Private Key</label>
          <div class="file-input-row">
            <input
              type="text"
              bind:value={sslConfig.keyPath}
              placeholder="/path/to/client-key.pem"
              class="file-input"
              readonly
            />
            <button on:click={() => pickFile('keyPath')} class="file-btn">Browse</button>
            {#if sslConfig.keyPath}
              <button on:click={() => clearFile('keyPath')} class="clear-btn">Clear</button>
            {/if}
          </div>
          <p class="ssl-hint">Private key corresponding to the client certificate</p>
        </div>

        <div class="ssl-option">
          <label class="ssl-label">CA Certificate</label>
          <div class="file-input-row">
            <input
              type="text"
              bind:value={sslConfig.caPath}
              placeholder="/path/to/ca-cert.pem"
              class="file-input"
              readonly
            />
            <button on:click={() => pickFile('caPath')} class="file-btn">Browse</button>
            {#if sslConfig.caPath}
              <button on:click={() => clearFile('caPath')} class="clear-btn">Clear</button>
            {/if}
          </div>
          <p class="ssl-hint">Custom Certificate Authority for server verification</p>
        </div>
      </div>

      <div class="modal-actions">
        <button class="modal-btn primary" on:click={closeSslModal}>Done</button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #1a1a2e;
    color: #e4e4e7;
  }

  .app {
    height: 100vh;
    display: flex;
  }

  .main-content {
    flex: 1;
    display: flex;
    min-width: 0;
    overflow: hidden;
  }

  .divider-line {
    width: 2px;
    background: #2a2a3e;
  }

  /* Modal styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #1e1e2e;
    border-radius: 12px;
    padding: 24px;
    width: 400px;
    max-width: 90%;
    border: 1px solid #3a3a4e;
  }

  .ssl-modal {
    width: 520px;
  }

  .modal h3 {
    margin: 0 0 16px 0;
    color: #e4e4e7;
    font-size: 18px;
  }

  .modal-input {
    width: 100%;
    padding: 12px 16px;
    border-radius: 8px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 14px;
    box-sizing: border-box;
    margin-bottom: 20px;
  }

  .modal-input:focus {
    outline: none;
    border-color: #61affe;
  }

  .modal-actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
  }

  .modal-btn {
    padding: 10px 20px;
    border-radius: 6px;
    border: none;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .modal-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .modal-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-btn.secondary {
    background: #3a3a4e;
    color: #e4e4e7;
  }

  .modal-btn.primary {
    background: #61affe;
    color: #fff;
  }

  /* SSL Modal Styles */
  .ssl-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 10px 0;
    margin-bottom: 20px;
  }

  .ssl-option {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .ssl-label {
    color: #e4e4e7;
    font-size: 13px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }

  .ssl-checkbox {
    width: 16px;
    height: 16px;
    accent-color: #49cc90;
    cursor: pointer;
  }

  .ssl-hint {
    color: #888;
    font-size: 12px;
    margin: 0;
    margin-left: 26px;
  }

  .ssl-divider {
    height: 1px;
    background: #3a3a4e;
    margin: 10px 0;
  }

  .file-input-row {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .file-input {
    flex: 1;
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 13px;
    font-family: Monaco, Menlo, monospace;
  }

  .file-input::placeholder {
    color: #666;
  }

  .file-btn, .clear-btn {
    padding: 10px 16px;
    border-radius: 6px;
    border: none;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.2s;
    white-space: nowrap;
  }

  .file-btn {
    background: #61affe;
    color: #fff;
  }

  .clear-btn {
    background: #3a3a4e;
    color: #e4e4e7;
  }

  .file-btn:hover, .clear-btn:hover {
    opacity: 0.9;
  }
</style>
