<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { writable } from "svelte/store";
  import { onMount } from "svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import RequestPanel from "./lib/RequestPanel.svelte";
  import ResponsePanel from "./lib/ResponsePanel.svelte";
  import type { Header, HttpMethod, SavedRequest, ResponseData } from "./types";

  // Request state
  let method: HttpMethod = "GET";
  let url: string = "";
  let headers: Header[] = [{ key: "", value: "" }];
  let body: string = "";

  // Saved requests
  const savedRequests = writable<SavedRequest[]>([]);
  
  // UI state
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
      } catch {
        // Keep defaults
      }
    }
  });

  // Auto-save: persist current state whenever it changes
  $: {
    const state = { method, url, headers, body };
    localStorage.setItem("curl-gui-current", JSON.stringify(state));
  }

  function persistRequests() {
    localStorage.setItem("curl-gui-saved-requests", JSON.stringify($savedRequests));
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

    curlCommand = `curl -X ${method}${Object.entries(hdrs).map(([k, v]) => ` -H "${k}: ${v}"`).join("")}${body && method !== "GET" ? ` -d '${body}'` : ""} "${url}"`;

    try {
      response = await invoke<ResponseData>("make_request", {
        request: {
          method,
          url,
          headers: Object.keys(hdrs).length > 0 ? hdrs : undefined,
          body: body || undefined,
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
  }

  function loadRequest(saved: SavedRequest) {
    method = saved.method;
    url = saved.url;
    headers = JSON.parse(JSON.stringify(saved.headers));
    body = saved.body;
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
      {loading}
      on:send={sendRequest}
      on:save={openSaveModal}
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
