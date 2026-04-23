<script lang="ts">
  import type { SavedRequest, HttpMethod } from "../types";
  import { createEventDispatcher } from "svelte";

  export let savedRequests: SavedRequest[];
  export let searchFilter: string = "";
  export let activeRequestId: string | null = null;

  const dispatch = createEventDispatcher<{
    load: SavedRequest;
    delete: string;
    edit: SavedRequest;
    saveNew: void;
    search: string;
    openSsl: void;
  }>();

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

  $: filteredRequests = savedRequests.filter(r => 
    r.name.toLowerCase().includes(searchFilter.toLowerCase()) ||
    r.url.toLowerCase().includes(searchFilter.toLowerCase())
  );

  function handleDelete(e: Event, id: string) {
    e.stopPropagation();
    dispatch("delete", id);
  }

  function handleEdit(e: Event, request: SavedRequest) {
    e.stopPropagation();
    dispatch("edit", request);
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="logo">
      <span class="curl-logo">curl</span>
      <span class="divider">|</span>
      <span class="app-name">GUI</span>
    </div>
    <div class="header-actions">
      <button class="ssl-btn" on:click={() => dispatch("openSsl")} title="SSL Settings">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
        </svg>
      </button>
      <button class="new-request-btn" on:click={() => dispatch("saveNew")} title="Save current request">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
    </div>
  </div>

  <div class="search-box">
    <input 
      type="text" 
      bind:value={searchFilter}
      on:input={() => dispatch("search", searchFilter)}
      placeholder="Search..." 
      class="search-input" 
    />
  </div>

  <div class="saved-requests">
    {#if filteredRequests.length === 0}
      <div class="empty-sidebar">
        {#if searchFilter}
          <p>No matches</p>
        {:else}
          <p>No saved requests</p>
          <p class="hint">Click + to save</p>
        {/if}
      </div>
    {:else}
      {#each filteredRequests as saved (saved.id)}
        <div class="saved-request-item" class:active={saved.id === activeRequestId} on:click={() => dispatch("load", saved)}>
          <div class="request-info">
            <span class="method-badge" style="color: {getMethodColor(saved.method)}">{saved.method}</span>
            <span class="request-name" title={saved.name}>{saved.name}</span>
          </div>
          <div class="request-actions">
            <button class="action-btn edit" on:click={(e) => handleEdit(e, saved)} title="Rename">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
              </svg>
            </button>
            <button class="action-btn delete" on:click={(e) => handleDelete(e, saved.id)} title="Delete">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6"></polyline>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
              </svg>
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    background: #16162a;
    border-right: 1px solid #2a2a3e;
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    padding: 12px 15px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #2a2a3e;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 16px;
    font-weight: 600;
  }

  .curl-logo {
    color: #61affe;
  }

  .divider {
    color: #666;
  }

  .app-name {
    color: #e4e4e7;
  }

  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .ssl-btn, .new-request-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .ssl-btn {
    background: #3a3a4e;
    color: #fca130;
  }

  .ssl-btn:hover {
    background: #4a4a5e;
    transform: scale(1.05);
  }

  .new-request-btn {
    background: #49cc90;
    color: #fff;
  }

  .new-request-btn:hover {
    opacity: 0.9;
    transform: scale(1.05);
  }

  .ssl-btn svg, .new-request-btn svg {
    width: 16px;
    height: 16px;
  }

  .action-btn svg {
    width: 14px;
    height: 14px;
  }

  .search-box {
    padding: 10px 15px;
    border-bottom: 1px solid #2a2a3e;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #1e1e2e;
    color: #e4e4e7;
    font-size: 13px;
    box-sizing: border-box;
  }

  .search-input:focus {
    outline: none;
    border-color: #61affe;
  }

  .saved-requests {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
  }

  .empty-sidebar {
    padding: 40px;
    text-align: center;
    color: #666;
  }

  .empty-sidebar .hint {
    font-size: 12px;
    margin-top: 10px;
    color: #888;
  }

  .saved-request-item {
    padding: 12px;
    border-radius: 8px;
    background: #1e1e2e;
    margin-bottom: 8px;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
    position: relative;
  }

  .saved-request-item:hover {
    background: #2a2a3e;
    border-color: #3a3a4e;
  }

  .saved-request-item.active {
    background: #2a2a3e;
    border-color: #61affe;
    box-shadow: 0 0 0 1px #61affe;
  }

  .request-info {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .method-badge {
    font-size: 11px;
    font-weight: 700;
    min-width: 50px;
  }

  .request-name {
    font-weight: 500;
    font-size: 13px;
    color: #e4e4e7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .request-url {
    font-size: 11px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-left: 58px;
  }

  .request-actions {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 4px;
    opacity: 0.5;
    transition: opacity 0.2s;
  }

  .saved-request-item:hover .request-actions,
  .saved-request-item.active .request-actions {
    opacity: 1;
  }

  .action-btn {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    border: none;
    background: #3a3a4e;
    color: #e4e4e7;
    font-size: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
  }

  .action-btn:hover {
    opacity: 1;
  }

  .action-btn.edit:hover {
    background: #61affe;
  }

  .action-btn.delete:hover {
    background: #f93e3e;
  }
</style>
