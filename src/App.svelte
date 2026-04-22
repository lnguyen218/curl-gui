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
  let sslConfig: SslConfig = {
    verifySsl: true,
    certPath: "",
    keyPath: "",
    caPath: ""
  };
  let authConfig: AuthConfig = {
    type: "none",
    username: "",
    password: "",
    token: "",
    apiKeyName: "",
    apiKeyValue: "",
    apiKeyIn: "header"
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

    // Load current state
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
        sslConfig = parsed.sslConfig || { verifySsl: true, certPath: "", keyPath: "", caPath: "" };
        authConfig = parsed.authConfig || { type: "none", username: "", password: "", token: "", apiKeyName: "", apiKeyValue: "", apiKeyIn: "header" };
      } catch {
        // Keep defaults
      }
    }
  });

  // Auto-save: persist current state whenever it changes
  $: {
    const state = { method, url, headers, body, sslConfig, authConfig };
    localStorage.setItem("curl-gui-current", JSON.stringify(state));
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
              sslConfig,
              authConfig,
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
      authHeaders["Authorization"] = `Bearer ${authConfig.token}`;
      authCurlFlags = ` -H "Authorization: Bearer ${authConfig.token}"`;
    } else if (authConfig.type === "api-key" && authConfig.apiKeyName && authConfig.apiKeyValue) {
      if (authConfig.apiKeyIn === "header") {
        authHeaders[authConfig.apiKeyName] = authConfig.apiKeyValue;
        authCurlFlags = ` -H "${authConfig.apiKeyName}: ${authConfig.apiKeyValue}"`;
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
      sslConfig,
      authConfig,
      createdAt: editingRequest?.createdAt || Date.now(),
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
    sslConfig = { verifySsl: true, certPath: "", keyPath: "", caPath: "" };
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
    sslConfig = saved.sslConfig || { verifySsl: true, certPath: "", keyPath: "", caPath: "" };
    authConfig = saved.authConfig || { type: "none", username: "", password: "", token: "", apiKeyName: "", apiKeyValue: "", apiKeyIn: "header" };
    activeRequestId = saved.id;
    response = null;
    error = "";
    curlCommand = "";
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
  />

  <div class="main-content">
    <RequestPanel
      bind:method
      bind:url
      bind:headers
      bind:body
      bind:sslConfig
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
</style>
