<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { SavedRequest, RequestFolder } from "../types";
  import RequestItem from "./RequestItem.svelte";

  export let savedRequests: SavedRequest[];
  export let folders: RequestFolder[] = [];
  export let searchFilter: string = "";
  export let activeRequestId: string | null = null;
  export let selectedFolderId: string | null = null;

  const dispatch = createEventDispatcher<{
    load: SavedRequest;
    delete: string;
    edit: SavedRequest;
    saveNew: void;
    search: string;
    openSsl: void;
    createFolder: void;
    renameFolder: RequestFolder;
    deleteFolder: RequestFolder;
    selectFolder: string | null;
    moveRequest: { requestId: string; folderId: string | null };
    reorderRequest: { requestId: string; beforeId: string | null; folderId?: string | null };
    reorderFolder: { folderId: string; beforeId: string | null };
  }>();

  let expandedFolderIds: string[] = [];

  $: filteredRequests = savedRequests.filter(r =>
    r.name.toLowerCase().includes(searchFilter.toLowerCase()) ||
    r.url.toLowerCase().includes(searchFilter.toLowerCase())
  );

  $: rootRequests = filteredRequests.filter(r => !r.folderId).sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
  $: folderRequests = (folderId: string) =>
    filteredRequests.filter(r => r.folderId === folderId).sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
  $: sortedFolders = folders.slice().sort((a, b) => (a.order ?? 0) - (b.order ?? 0));

  import { flip } from "svelte/animate";
  import { quintOut } from "svelte/easing";

  const flipDuration = 200;

  function flipProps() {
    return { duration: flipDuration, easing: quintOut };
  }

  function isExpanded(folderId: string): boolean {
    return expandedFolderIds.includes(folderId);
  }

  function toggleFolder(folderId: string) {
    expandedFolderIds = expandedFolderIds.includes(folderId)
      ? expandedFolderIds.filter(id => id !== folderId)
      : [...expandedFolderIds, folderId];
  }

  function handleFolderAction(e: Event, action: string, folder: RequestFolder) {
    e.stopPropagation();
    if (action === "rename") dispatch("renameFolder", folder);
    if (action === "delete") dispatch("deleteFolder", folder);
  }

  let draggingRequestId: string | null = null;
  let draggingFolderId: string | null = null;
  let dragOverFolderId: string | null = null;
  let dragOverRoot = false;
  let dragOverRequestId: string | null = null;
  let dragOverRequestTop = false;
  let dragPos = { x: 0, y: 0 };
  let dragMode: "request" | "folder" | null = null;

  function onRequestDragStart(requestId: string) {
    draggingRequestId = requestId;
    dragMode = "request";
  }

  function onFolderDragStart(folderId: string) {
    draggingFolderId = folderId;
    dragMode = "folder";
  }

  function onFolderPointerDown(folderId: string) {
    return (e: PointerEvent) => {
      const target = e.target as HTMLElement;
      const closest = target.closest(".folder-actions, .chevron-btn");
      if (closest) return;
      e.preventDefault();
      e.stopPropagation();
      onFolderDragStart(folderId);
    };
  }

  function onPointerMoveGlobal(e: PointerEvent) {
    if (!draggingRequestId && !draggingFolderId) return;
    dragPos = { x: e.clientX, y: e.clientY };

    const elements = document.elementsFromPoint(e.clientX, e.clientY);
    const folderEl = elements.find(el => el instanceof HTMLElement && el.dataset.dropFolderId) as HTMLElement | undefined;
    const rootEl = elements.find(el => el instanceof HTMLElement && el.dataset.dropRoot) as HTMLElement | undefined;
    const requestEl = elements.find(el => el instanceof HTMLElement && el.dataset.dropRequestId) as HTMLElement | undefined;
    const listEl = elements.find(el => el instanceof HTMLElement && el.dataset.dropList === "requests") as HTMLElement | undefined;

    dragOverFolderId = folderEl?.dataset.dropFolderId || null;
    dragOverRoot = !!rootEl || (!!listEl && !requestEl);
    if (requestEl) {
      dragOverRequestId = requestEl.dataset.dropRequestId || null;
      const rect = requestEl.getBoundingClientRect();
      dragOverRequestTop = e.clientY - rect.top < rect.height / 2;
    } else {
      dragOverRequestId = null;
      dragOverRequestTop = false;
    }
  }

  function onPointerUpGlobal() {
    if (dragMode === "request" && draggingRequestId) {
      if (dragOverRequestId && dragOverRequestId !== draggingRequestId) {
        // Reorder around the target request: if top half, insert before target; otherwise after
        // and adopt the target's folder when dropping onto a request in a folder
        const targetRequest = savedRequests.find(r => r.id === dragOverRequestId);
        dispatch("reorderRequest", { requestId: draggingRequestId, beforeId: dragOverRequestTop ? dragOverRequestId : null, folderId: targetRequest?.folderId ?? null });
      } else if (dragOverFolderId) {
        dispatch("moveRequest", { requestId: draggingRequestId, folderId: dragOverFolderId });
      } else if (dragOverRoot) {
        dispatch("moveRequest", { requestId: draggingRequestId, folderId: null });
      }
    } else if (dragMode === "folder" && draggingFolderId) {
      if (dragOverRequestId) {
        // Ignore: can't drop folder onto request
      } else if (dragOverFolderId && dragOverFolderId !== draggingFolderId) {
        dispatch("reorderFolder", { folderId: draggingFolderId, beforeId: dragOverFolderId });
      }
    }

    draggingRequestId = null;
    draggingFolderId = null;
    dragOverFolderId = null;
    dragOverRoot = false;
    dragOverRequestId = null;
    dragOverRequestTop = false;
    dragMode = null;
  }

  function onRequestClick(saved: SavedRequest) {
    if (draggingRequestId) return;
    dispatch("load", saved);
  }
</script>

<svelte:window 
  on:pointermove={onPointerMoveGlobal}
  on:pointerup={onPointerUpGlobal}
/>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="logo">
      <span class="curl-logo">curl</span>
      <span class="divider">|</span>
      <span class="app-name">GUI</span>
    </div>
    <div class="header-actions">
      <button class="folder-btn" on:click={() => dispatch("createFolder")} title="New folder">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          <line x1="12" y1="11" x2="12" y2="17"></line>
          <line x1="9" y1="14" x2="15" y2="14"></line>
        </svg>
      </button>
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

  <div 
    class="saved-requests"
    class:dragging-request={!!draggingRequestId}
  >
    {#if filteredRequests.length === 0 && folders.length === 0}
      <div class="empty-sidebar">
        {#if searchFilter}
          <p>No matches</p>
        {:else}
          <p>No saved requests</p>
          <p class="hint">Click + to save, folder icon to organize</p>
        {/if}
      </div>
    {:else}
      <div 
        class="root-section"
        class:active={selectedFolderId === null}
        class:drop-target={dragOverRoot}
        class:drag-active={!!draggingRequestId}
        data-drop-root="true"
        on:click={() => dispatch("selectFolder", null)}
      >
        <span class="section-name">All Requests</span>
        <span class="section-count">{filteredRequests.length}</span>
      </div>

      {#each sortedFolders as folder (folder.id)}
        <div class="folder" animate:flip={flipProps()}>
          <div 
            class="folder-header"
            class:active={selectedFolderId === folder.id}
            class:drop-target={dragOverFolderId === folder.id}
            class:drag-active={!!draggingRequestId || !!draggingFolderId}
            class:dragging={draggingFolderId === folder.id}
            data-drop-folder-id={folder.id}
            on:pointerdown={onFolderPointerDown(folder.id)}
          >
            <button
              type="button"
              class="chevron-btn"
              class:expanded={expandedFolderIds.includes(folder.id)}
              on:click={(e) => { e.stopPropagation(); toggleFolder(folder.id); }}
              title={expandedFolderIds.includes(folder.id) ? "Collapse" : "Expand"}
            >
              <svg 
                class="folder-chevron"
                class:expanded={expandedFolderIds.includes(folder.id)}
                viewBox="0 0 24 24" 
                fill="none" 
                stroke="currentColor" 
                stroke-width="2" 
                stroke-linecap="round" 
                stroke-linejoin="round"
              >
                <polyline points="9 18 15 12 9 6"></polyline>
              </svg>
            </button>
            <div class="folder-label" on:click={() => { if (!draggingFolderId) dispatch("selectFolder", folder.id); }}>
              <svg class="folder-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
              </svg>
              <span class="folder-name" title={folder.name}>{folder.name}</span>
              <span class="folder-count">{folderRequests(folder.id).length}</span>
            </div>
            <div class="folder-actions">
              <button 
                type="button"
                class="action-btn mini" 
                on:click={(e) => handleFolderAction(e, "rename", folder)} 
                title="Rename folder"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </button>
              <button 
                type="button"
                class="action-btn mini delete" 
                on:click={(e) => handleFolderAction(e, "delete", folder)} 
                title="Delete folder"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="3 6 5 6 21 6"></polyline>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                </svg>
              </button>
            </div>
          </div>
          {#if expandedFolderIds.includes(folder.id)}
            <div class="folder-children">
              {#each folderRequests(folder.id) as saved (saved.id)}
                <div animate:flip={flipProps()}>
                  <RequestItem 
                    {saved}
                    {activeRequestId}
                    dropRequestId={saved.id}
                    isDropBefore={dragOverRequestId === saved.id && dragOverRequestTop}
                    isDropAfter={dragOverRequestId === saved.id && !dragOverRequestTop}
                    on:load={() => onRequestClick(saved)}
                    on:delete={(e) => dispatch("delete", e.detail)}
                    on:edit={(e) => dispatch("edit", e.detail)}
                    on:dragstart={(e) => onRequestDragStart(e.detail)}
                  />
                </div>
              {/each}
              {#if folderRequests(folder.id).length === 0}
                <div class="empty-folder">No requests</div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}

      {#if selectedFolderId === null}
        {#each rootRequests as saved (saved.id)}
          <div animate:flip={flipProps()}>
            <RequestItem 
              {saved}
              {activeRequestId}
              dropRequestId={saved.id}
              isDropBefore={dragOverRequestId === saved.id && dragOverRequestTop}
              isDropAfter={dragOverRequestId === saved.id && !dragOverRequestTop}
              on:load={() => onRequestClick(saved)}
              on:delete={(e) => dispatch("delete", e.detail)}
              on:edit={(e) => dispatch("edit", e.detail)}
              on:dragstart={(e) => onRequestDragStart(e.detail)}
            />
          </div>
        {/each}
      {/if}
    {/if}
  </div>
</aside>

{#if draggingRequestId}
  <div class="drag-ghost" style="left: {dragPos.x}px; top: {dragPos.y}px;">
    <span>Moving request...</span>
  </div>
{/if}

<style>
  .sidebar {
    width: 300px;
    min-width: 300px;
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

  .ssl-btn, .new-request-btn, .folder-btn {
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

  .folder-btn {
    background: #3a3a4e;
    color: #61affe;
  }

  .folder-btn:hover {
    background: #4a4a5e;
    transform: scale(1.05);
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

  .ssl-btn svg, .new-request-btn svg, .folder-btn svg {
    width: 16px;
    height: 16px;
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

  .root-section {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
    color: #888;
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 6px;
    border: 2px dashed transparent;
  }

  .root-section:hover, .root-section.active {
    background: #2a2a3e;
    color: #e4e4e7;
  }

  .root-section.drag-active {
    border-color: #444;
  }

  .root-section.drop-target {
    background: #2a2a3e;
    border-color: #61affe;
    color: #e4e4e7;
  }

  .section-count {
    font-size: 11px;
    background: #3a3a4e;
    padding: 2px 6px;
    border-radius: 4px;
    color: #aaa;
  }

  .folder {
    margin-bottom: 4px;
    user-select: none;
  }

  .folder-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 6px;
    color: #888;
    transition: all 0.2s;
    position: relative;
    border: 2px dashed transparent;
  }

  .folder-header:hover, .folder-header.active {
    background: #2a2a3e;
    color: #e4e4e7;
  }

  .folder-header.drag-active {
    border-color: #444;
  }

  .folder-header.drop-target {
    background: #2a2a3e;
    border-color: #61affe;
    color: #e4e4e7;
  }

  .folder-header.dragging {
    opacity: 0.5;
  }

  .chevron-btn {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .folder-chevron {
    width: 14px;
    height: 14px;
    transition: transform 0.2s;
  }

  .folder-chevron.expanded {
    transform: rotate(90deg);
  }

  .folder-label {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .folder-icon {
    width: 16px;
    height: 16px;
    color: #61affe;
    flex-shrink: 0;
  }

  .folder-name {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .folder-count {
    font-size: 11px;
    background: #3a3a4e;
    padding: 2px 6px;
    border-radius: 4px;
    color: #aaa;
    flex-shrink: 0;
  }

  .folder-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .folder-children {
    padding-left: 16px;
    margin-top: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .empty-folder {
    padding: 8px 12px;
    color: #666;
    font-size: 12px;
    font-style: italic;
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

  .action-btn.mini {
    width: 20px;
    height: 20px;
  }

  .action-btn.delete:hover {
    background: #f93e3e;
  }

  .action-btn svg {
    width: 14px;
    height: 14px;
  }

  .drag-ghost {
    position: fixed;
    pointer-events: none;
    background: #2a2a3e;
    border: 1px solid #61affe;
    border-radius: 6px;
    padding: 6px 12px;
    color: #e4e4e7;
    font-size: 12px;
    z-index: 1000;
    transform: translate(-50%, -50%);
  }

  .move-select {
    display: none;
  }

  .saved-request-item {
    user-select: none;
  }
</style>
