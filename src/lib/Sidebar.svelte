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
    <button class="new-request-btn" on:click={() => dispatch("saveNew")} title="Save current request">+</button>
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
            <button class="action-btn edit" on:click={(e) => handleEdit(e, saved)} title="Rename">✎</button>
            <button class="action-btn delete" on:click={(e) => handleDelete(e, saved.id)} title="Delete">×</button>
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

  .new-request-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: none;
    background: #49cc90;
    color: #fff;
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity 0.2s;
  }

  .new-request-btn:hover {
    opacity: 0.9;
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

  .saved-request-item:hover .request-actions {
    opacity: 1;
  }

  .saved-request-item.active {
    background: #2a2a3e;
    border-color: #61affe;
    box-shadow: 0 0 0 1px #61affe;
  }

  .saved-request-item.active .request-actions {
    opacity: 1;
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
    opacity: 0;
    transition: opacity 0.2s;
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
