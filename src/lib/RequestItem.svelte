<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { SavedRequest, HttpMethod, RequestFolder } from "../types";

  export let saved: SavedRequest;
  export let activeRequestId: string | null = null;
  export let folders: RequestFolder[] = [];

  const dispatch = createEventDispatcher<{
    load: SavedRequest;
    delete: string;
    edit: SavedRequest;
    move: string | null;
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
</script>

<div class="saved-request-item" class:active={saved.id === activeRequestId} on:click={() => dispatch("load", saved)}>
  <div class="request-info">
    <span class="method-badge" style="color: {getMethodColor(saved.method)}">{saved.method}</span>
    <span class="request-name" title={saved.name}>{saved.name}</span>
  </div>
  <div class="request-actions">
    {#if folders.length > 0}
      <select
        class="move-select"
        value={saved.folderId || ""}
        on:click={(e) => e.stopPropagation()}
        on:change={(e) => dispatch("move", e.currentTarget.value || null)}
        title="Move to folder"
      >
        <option value="">No folder</option>
        {#each folders as folder}
          <option value={folder.id}>{folder.name}</option>
        {/each}
      </select>
    {/if}
    
    <button class="action-btn edit" on:click={(e) => { e.stopPropagation(); dispatch("edit", saved); }} title="Rename">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
      </svg>
    </button>
    <button class="action-btn delete" on:click={(e) => { e.stopPropagation(); dispatch("delete", saved.id); }} title="Delete">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="3 6 5 6 21 6"></polyline>
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
      </svg>
    </button>
  </div>
</div>

<style>
  .saved-request-item {
    padding: 10px 12px;
    border-radius: 8px;
    background: #1e1e2e;
    margin-bottom: 6px;
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

  .request-actions {
    display: flex;
    align-items: center;
    gap: 4px;
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

  .move-select {
    background: #2a2a3e;
    border: 1px solid #3a3a4e;
    color: #888;
    border-radius: 4px;
    font-size: 11px;
    padding: 2px 4px;
    cursor: pointer;
    max-width: 80px;
  }

  .move-select:focus {
    outline: none;
    border-color: #61affe;
  }
</style>
