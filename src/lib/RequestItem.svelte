<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { SavedRequest, HttpMethod } from "../types";

  export let saved: SavedRequest;
  export let activeRequestId: string | null = null;
  export let dropRequestId: string | undefined = undefined;

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  type $$Props = {
    saved: SavedRequest;
    activeRequestId?: string | null;
    dropRequestId?: string;
    // allow parent to set data attribute for drop detection
    "data-drop-request-id"?: string;
  };

  const dispatch = createEventDispatcher<{
    load: SavedRequest;
    delete: string;
    edit: SavedRequest;
    dragstart: string;
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

  let dragging = false;

  function onPointerDown(e: PointerEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".drag-handle")) return;
    e.preventDefault();
    e.stopPropagation();
    dragging = true;
    dispatch("dragstart", saved.id);
  }

  function onClick(e: MouseEvent) {
    if (dragging) {
      dragging = false;
      e.preventDefault();
      e.stopPropagation();
      return;
    }
    dispatch("load", saved);
  }
</script>

<div 
  class="saved-request-item" 
  class:active={saved.id === activeRequestId} 
  class:dragging
  data-drop-request-id={dropRequestId || saved.id}
  on:pointerdown={onPointerDown}
  on:click={onClick}
>
  <div class="request-info">
    <span class="drag-handle" title="Drag to folder">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="9" cy="12" r="1"></circle>
        <circle cx="9" cy="5" r="1"></circle>
        <circle cx="9" cy="19" r="1"></circle>
        <circle cx="15" cy="12" r="1"></circle>
        <circle cx="15" cy="5" r="1"></circle>
        <circle cx="15" cy="19" r="1"></circle>
      </svg>
    </span>
    <span class="method-badge" style="color: {getMethodColor(saved.method)}">{saved.method}</span>
    <span class="request-name" title={saved.name}>{saved.name}</span>
  </div>
  <div class="request-actions">
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
    padding: 8px 10px;
    border-radius: 8px;
    background: #1e1e2e;
    margin-bottom: 6px;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
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

  .saved-request-item.dragging {
    opacity: 0.5;
  }

  .request-info {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .drag-handle {
    color: #666;
    cursor: grab;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .drag-handle svg {
    width: 14px;
    height: 14px;
  }

  .drag-handle:active {
    cursor: grabbing;
  }

  .method-badge {
    font-size: 11px;
    font-weight: 700;
    min-width: 50px;
    flex-shrink: 0;
  }

  .request-name {
    font-weight: 500;
    font-size: 13px;
    color: #e4e4e7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .request-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
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

  .action-btn svg {
    width: 14px;
    height: 14px;
  }
</style>
