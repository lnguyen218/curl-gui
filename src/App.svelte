<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writable } from "svelte/store";
  import { onMount } from "svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import RequestPanel from "./lib/RequestPanel.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import type { Header, HttpMethod, SavedRequest, SslConfig, AuthConfig, RequestFolder, ResponseData } from "./types";

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

  // Saved requests and folders
  const savedRequests = writable<SavedRequest[]>([]);
  const folders = writable<RequestFolder[]>([]);
  
  // UI state
  let activeRequestId: string | null = null;
  let response: ResponseData | null = null;
  let loading = false;
  let error = "";
  let curlCommand = "";
  let searchFilter = "";
  let selectedFolderId: string | null = null;
  
  // Modal state
  let showModal = false;
  let modalName = "";
  let editingRequest: SavedRequest | null = null;
  let modalFolderId: string | null = null;

  // SSL settings modal
  let showSslModal = false;

  // Folder modal
  let showFolderModal = false;
  let folderModalName = "";
  let editingFolder: RequestFolder | null = null;

  onMount(() => {
    // Load folders
    const savedFolders = localStorage.getItem("curl-gui-folders");
    if (savedFolders) {
      try {
        folders.set(JSON.parse(savedFolders));
      } catch {
        folders.set([]);
      }
    }

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

  // Auto-save: persist folders and saved requests whenever they change
  $: {
    persistRequests();
  }

  function persistRequests() {
    localStorage.setItem("curl-gui-folders", JSON.stringify($folders));
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
              error: error || undefined,
            };
          }
          return r;
        });
      });
      // Auto-save reactive block handles persistence
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
      // Auto-save response to the active request
      autoSaveRequest();
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
      error: error || undefined,
      folderId: modalFolderId,
    };

    savedRequests.update(reqs => {
      if (editingRequest) {
        const id = editingRequest.id;
        return reqs.map(r => r.id === id ? newReq : r);
      }
      const maxOrder = reqs.reduce((max, r) => Math.max(max, r.order ?? 0), 0);
      return [{ ...newReq, order: maxOrder + 1 }, ...reqs];
    });

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
    error = saved.error || "";
    curlCommand = saved.curlCommand || "";
  }

  function deleteRequest(id: string) {
    savedRequests.update(reqs => reqs.filter(r => r.id !== id));
    if (activeRequestId === id) {
      activeRequestId = null;
    }
  }

  function editRequestName(saved: SavedRequest) {
    editingRequest = saved;
    modalName = saved.name;
    modalFolderId = saved.folderId || null;
    showModal = true;
  }

  function openSaveModal() {
    editingRequest = null;
    modalName = "";
    modalFolderId = selectedFolderId;
    showModal = true;
  }

  function createFolder() {
    editingFolder = null;
    folderModalName = "";
    showFolderModal = true;
  }

  function saveFolder() {
    if (!folderModalName.trim()) return;
    if (editingFolder) {
      folders.update(f => f.map(x => x.id === editingFolder!.id ? { ...x, name: folderModalName.trim() } : x));
    } else {
      folders.update(f => {
        const maxOrder = f.reduce((max, x) => Math.max(max, x.order ?? 0), 0);
        return [...f, { id: crypto.randomUUID(), name: folderModalName.trim(), order: maxOrder + 1, createdAt: Date.now() }];
      });
    }
    closeFolderModal();
  }

  function renameFolder(folder: RequestFolder) {
    editingFolder = folder;
    folderModalName = folder.name;
    showFolderModal = true;
  }

  function deleteFolder(folder: RequestFolder) {
    if (!confirm(`Delete folder "${folder.name}"? Requests inside will become uncategorized.`)) return;
    savedRequests.update(reqs => reqs.map(r => r.folderId === folder.id ? { ...r, folderId: null } : r));
    folders.update(f => f.filter(x => x.id !== folder.id));
    if (selectedFolderId === folder.id) {
      selectedFolderId = null;
    }
  }

  function closeFolderModal() {
    showFolderModal = false;
    folderModalName = "";
    editingFolder = null;
  }

  function moveRequestToFolder(e: CustomEvent<{ requestId: string; folderId: string | null }>) {
    const { requestId, folderId } = e.detail;
    savedRequests.update(reqs => reqs.map(r => r.id === requestId ? { ...r, folderId } : r));
  }

  function reorderRequest(e: CustomEvent<{ requestId: string; beforeId: string | null; folderId?: string | null }>) {
    const { requestId, beforeId, folderId: dropFolderId } = e.detail;
    savedRequests.update(reqs => {
      const list = reqs.slice();
      const idx = list.findIndex(r => r.id === requestId);
      if (idx === -1) return list;
      const [moved] = list.splice(idx, 1);
      // If dropped onto a request in a folder, adopt that folder
      const targetFolderId = dropFolderId !== undefined ? dropFolderId : moved.folderId;
      const sameFolderList = list.filter(r => r.folderId === targetFolderId);
      let insertIdx;
      if (beforeId) {
        insertIdx = list.findIndex(r => r.id === beforeId);
      } else {
        const lastInFolderIdx = list.map((r, i) => ({ i, r })).filter(({ r }) => r.folderId === targetFolderId).pop()?.i ?? -1;
        insertIdx = lastInFolderIdx + 1;
      }
      if (insertIdx === -1) insertIdx = list.length;
      const withFolder = { ...moved, folderId: targetFolderId };
      list.splice(insertIdx, 0, withFolder);
      return list.map((r, i) => ({ ...r, order: i }));
    });
  }

  function reorderFolder(e: CustomEvent<{ folderId: string; beforeId: string | null }>) {
    const { folderId, beforeId } = e.detail;
    folders.update(list => {
      const copy = list.slice();
      const idx = copy.findIndex(f => f.id === folderId);
      if (idx === -1) return copy;
      const [moved] = copy.splice(idx, 1);
      const targetIdx = beforeId ? copy.findIndex(f => f.id === beforeId) : copy.length;
      const insertIdx = targetIdx === -1 ? copy.length : targetIdx;
      copy.splice(insertIdx, 0, moved);
      return copy.map((f, i) => ({ ...f, order: i }));
    });
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

  function clearResponse() {
    response = null;
    error = "";
    curlCommand = "";
    autoSaveRequest();
  }

  function onSidebarLoad(e: CustomEvent<SavedRequest>) { loadRequest(e.detail); }
  function onSidebarDelete(e: CustomEvent<string>) { deleteRequest(e.detail); }
  function onSidebarEdit(e: CustomEvent<SavedRequest>) { editRequestName(e.detail); }
  function onSidebarSearch(e: CustomEvent<string>) { searchFilter = e.detail; }
  function onSidebarRenameFolder(e: CustomEvent<RequestFolder>) { renameFolder(e.detail); }
  function onSidebarDeleteFolder(e: CustomEvent<RequestFolder>) { deleteFolder(e.detail); }
  function onSidebarSelectFolder(e: CustomEvent<string | null>) { selectedFolderId = e.detail; }

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
    folders={$folders}
    bind:searchFilter
    {activeRequestId}
    {selectedFolderId}
    on:load={onSidebarLoad}
    on:delete={onSidebarDelete}
    on:edit={onSidebarEdit}
    on:saveNew={openSaveModal}
    on:search={onSidebarSearch}
    on:openSsl={openSslModal}
    on:createFolder={createFolder}
    on:renameFolder={onSidebarRenameFolder}
    on:deleteFolder={onSidebarDeleteFolder}
    on:selectFolder={onSidebarSelectFolder}
    on:moveRequest={moveRequestToFolder}
    on:reorderRequest={reorderRequest}
    on:reorderFolder={reorderFolder}
  />

  <div class="main-content">
    <div class="url-bar">
      <select class="method-select" bind:value={method} on:change={autoSaveRequest}>
        <option value="GET" style="color: #61affe">GET</option>
        <option value="POST" style="color: #49cc90">POST</option>
        <option value="PUT" style="color: #fca130">PUT</option>
        <option value="DELETE" style="color: #f93e3e">DELETE</option>
        <option value="PATCH" style="color: #50e3c2">PATCH</option>
        <option value="HEAD" style="color: #9012fe">HEAD</option>
        <option value="OPTIONS" style="color: #0d5aa7">OPTIONS</option>
      </select>

      <input
        type="text"
        class="url-input"
        bind:value={url}
        on:input={autoSaveRequest}
        placeholder="https://api.example.com/endpoint"
      />
      <button class="send-btn" on:click={sendRequest} disabled={loading}>
        {#if loading}
          <span class="spinner"></span>
        {:else}
          <svg class="send-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="22" y1="2" x2="11" y2="13"></line>
            <polygon points="22,2 15,22 11,13 2,9 22,2"></polygon>
          </svg>
          Send
        {/if}
      </button>
    </div>

    <div class="panels-row">
      <RequestPanel
        bind:headers
        bind:body
        bind:authConfig
        on:update={autoSaveRequest}
      />

      <div class="divider-line"></div>

      <ResponsePanel
        {response}
        {error}
        {curlCommand}
        on:clear={clearResponse}
      />
    </div>
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
      <div class="modal-field">
        <label for="folder-select">Folder</label>
        <select id="folder-select" class="modal-select" bind:value={modalFolderId}>
          <option value={null}>No folder</option>
          {#each $folders as folder}
            <option value={folder.id}>{folder.name}</option>
          {/each}
        </select>
      </div>
      
      <div class="modal-actions">
        <button class="modal-btn secondary" on:click={closeModal}>Cancel</button>
        <button class="modal-btn primary" on:click={saveRequest} disabled={!modalName.trim()}>
          {editingRequest ? 'Update' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showFolderModal}
  <div class="modal-overlay" on:click={closeFolderModal}>
    <div class="modal folder-modal" on:click|stopPropagation>
      <h3>{editingFolder ? 'Rename Folder' : 'New Folder'}</h3>
      <input
        type="text"
        bind:value={folderModalName}
        placeholder="Folder name..."
        class="modal-input"
        on:keydown={(e) => { if (e.key === 'Enter') saveFolder(); if (e.key === 'Escape') closeFolderModal(); }}
      />
      <div class="modal-actions">
        <button class="modal-btn secondary" on:click={closeFolderModal}>Cancel</button>
        <button class="modal-btn primary" on:click={saveFolder} disabled={!folderModalName.trim()}>
          {editingFolder ? 'Update' : 'Create'}
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
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .url-bar {
    display: flex;
    gap: 10px;
    padding: 20px;
    background: #1a1a2e;
    border-bottom: 1px solid #3a3a4e;
  }

  .method-select {
    padding: 10px 15px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 14px;
    font-weight: 600;
    min-width: 100px;
    cursor: pointer;
  }

  .method-select option {
    background: #2a2a3e;
    color: #e4e4e7;
  }

  .url-input {
    flex: 1;
    padding: 10px 15px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 14px;
  }

  .url-input:focus {
    outline: none;
    border-color: #61affe;
  }

  .send-btn {
    padding: 10px 30px;
    border-radius: 6px;
    border: none;
    background: #49cc90;
    color: #fff;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.2s;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .send-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .send-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .send-btn .send-icon {
    width: 16px;
    height: 16px;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid #fff;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .panels-row {
    flex: 1;
    display: flex;
    min-height: 0;
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
    margin-bottom: 16px;
  }

  .modal-input:focus {
    outline: none;
    border-color: #61affe;
  }

  .modal-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 16px;
  }

  .modal-field label {
    color: #888;
    font-size: 12px;
    font-weight: 500;
  }

  .modal-select {
    width: 100%;
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 14px;
    box-sizing: border-box;
    cursor: pointer;
  }

  .modal-select:focus {
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
