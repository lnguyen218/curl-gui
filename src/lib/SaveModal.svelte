<script lang="ts">
  interface Props {
    isOpen: boolean;
    isEditing: boolean;
    initialName: string;
    onSave: (name: string) => void;
    onCancel: () => void;
  }

  let { isOpen, isEditing, initialName, onSave, onCancel }: Props = $props();
  console.log({ isOpen, isEditing, initialName, onSave, onCancel })
  let name = $state("");

  // Update local name when initialName changes
  $effect(() => {
    name = initialName;
  });

  function handleSave() {
    if (name.trim()) {
      onSave(name.trim());
      name = "";
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleSave();
    if (e.key === 'Escape') onCancel();
  }
</script>

{#if isOpen}
  <div class="modal-overlay" onclick={onCancel}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h3>{isEditing ? 'Rename Request' : 'Save Request'}</h3>
      <input 
        type="text" 
        bind:value={name} 
        placeholder="Request name..." 
        class="modal-input"
        onkeydown={handleKeydown}
        autofocus
      />
      <div class="modal-actions">
        <button class="modal-btn secondary" onclick={onCancel}>Cancel</button>
        <button class="modal-btn primary" onclick={handleSave} disabled={!name.trim()}>
          {isEditing ? 'Update' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
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
