<script lang="ts">
  import type { Header, HttpMethod, SslConfig, AuthConfig } from "../types";
  import { createEventDispatcher } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  export let method: HttpMethod;
  export let url: string;
  export let headers: Header[];
  export let body: string;
  export let sslConfig: SslConfig = {
    verifySsl: true,
    certPath: "",
    keyPath: "",
    caPath: ""
  };
  export let authConfig: AuthConfig = {
    type: "none",
    username: "",
    password: "",
    token: "",
    apiKeyName: "",
    apiKeyValue: "",
    apiKeyIn: "header"
  };
  export let loading: boolean = false;

  const dispatch = createEventDispatcher<{
    send: void;
    update: void;
  }>();

  let activeTab: "params" | "auth" | "headers" | "body" | "ssl" = "headers";

  const methods: HttpMethod[] = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

  const getMethodColor = (m: HttpMethod): string => {
    switch (m) {
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

  function addHeader() {
    headers = [...headers, { key: "", value: "" }];
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index);
  }

  function updateHeader(index: number, field: "key" | "value", value: string) {
    headers = headers.map((h, i) => (i === index ? { ...h, [field]: value } : h));
  }

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
    console.log({selected})
    if (selected) {
      sslConfig = { ...sslConfig, [field]: selected };
      dispatch("update");
    }
  }

  function clearFile(field: "certPath" | "keyPath" | "caPath") {
    sslConfig = { ...sslConfig, [field]: "" };
    dispatch("update");
  }

  // Auto-save: notify parent whenever any request field changes
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  $: {
    // Trigger when any bound value changes
    method, url, headers, body, sslConfig, authConfig;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      dispatch("update");
    }, 300);
  }
</script>

<div class="request-panel">
  <div class="url-bar">
    <select 
      class="method-select"
      bind:value={method}
    >
      {#each methods as m}
        <option value={m} style="color: {getMethodColor(m)}">{m}</option>
      {/each}
    </select>

    <input 
      type="text" 
      bind:value={url}
      placeholder="https://api.example.com/endpoint" 
      class="url-input" 
    />

    <button on:click={() => dispatch("send")} class="send-btn" disabled={loading}>
      {#if loading}<span class="spinner"></span>{:else}<svg class="send-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="2" x2="11" y2="13"></line><polygon points="22,2 15,22 11,13 2,9 22,2"></polygon></svg> Send{/if}
    </button>
  </div>

  <div class="request-tabs">
    <button class="tab-btn" class:active={activeTab === "params"} on:click={() => activeTab = "params"}>Params</button>
    <button class="tab-btn auth-tab" class:active={activeTab === "auth"} on:click={() => activeTab = "auth"}>
      Auth
    </button>
    <button class="tab-btn" class:active={activeTab === "headers"} on:click={() => activeTab = "headers"}>
      Headers ({headers.filter(h => h.key).length})
    </button>
    <button class="tab-btn" class:active={activeTab === "body"} on:click={() => activeTab = "body"}>Body</button>
    <button class="tab-btn ssl-tab" class:active={activeTab === "ssl"} on:click={() => activeTab = "ssl"}>
      🔒 SSL
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === "headers"}
      <div class="headers-section">
        {#each headers as header, i (i)}
          <div class="header-row">
            <input 
              type="text" 
              placeholder="Key" 
              bind:value={header.key}
              on:input={() => updateHeader(i, "key", header.key)} 
              class="header-key" 
            />
            <input 
              type="text" 
              placeholder="Value" 
              bind:value={header.value}
              on:input={() => updateHeader(i, "value", header.value)} 
              class="header-value" 
            />
            <button on:click={() => removeHeader(i)} class="remove-btn">×</button>
          </div>
        {/each}
        <button on:click={addHeader} class="add-header-btn">+ Add Header</button>
      </div>
    {:else if activeTab === "body"}
      <div class="body-section">
        <textarea 
          bind:value={body}
          placeholder={`{\n  "key": "value"\n}`} 
          class="body-editor"
        ></textarea>
      </div>
    {:else if activeTab === "auth"}
      <div class="auth-section">
        <div class="auth-type-row">
          <label class="auth-label">Authorization Type</label>
          <select bind:value={authConfig.type} class="auth-select">
            <option value="none">No Auth</option>
            <option value="basic">Basic Auth</option>
            <option value="bearer">Bearer Token</option>
            <option value="api-key">API Key</option>
          </select>
        </div>

        {#if authConfig.type === "basic"}
          <div class="auth-fields">
            <div class="auth-field">
              <label class="auth-label">Username</label>
              <input 
                type="text" 
                bind:value={authConfig.username}
                placeholder="username"
                class="auth-input"
              />
            </div>
            <div class="auth-field">
              <label class="auth-label">Password</label>
              <input 
                type="password" 
                bind:value={authConfig.password}
                placeholder="password"
                class="auth-input"
              />
            </div>
          </div>
        {:else if authConfig.type === "bearer"}
          <div class="auth-fields">
            <div class="auth-field">
              <label class="auth-label">Token</label>
              <textarea 
                bind:value={authConfig.token}
                placeholder="Bearer token..."
                class="auth-textarea"
              ></textarea>
            </div>
          </div>
        {:else if authConfig.type === "api-key"}
          <div class="auth-fields">
            <div class="auth-field">
              <label class="auth-label">Key Name</label>
              <input 
                type="text" 
                bind:value={authConfig.apiKeyName}
                placeholder="X-API-Key"
                class="auth-input"
              />
            </div>
            <div class="auth-field">
              <label class="auth-label">Key Value</label>
              <input 
                type="text" 
                bind:value={authConfig.apiKeyValue}
                placeholder="your-api-key"
                class="auth-input"
              />
            </div>
            <div class="auth-field">
              <label class="auth-label">Add To</label>
              <div class="auth-radio-group">
                <label class="auth-radio">
                  <input 
                    type="radio" 
                    bind:group={authConfig.apiKeyIn} 
                    value="header"
                  />
                  Header
                </label>
                <label class="auth-radio">
                  <input 
                    type="radio" 
                    bind:group={authConfig.apiKeyIn} 
                    value="query"
                  />
                  Query Param
                </label>
              </div>
            </div>
          </div>
        {:else}
          <p class="placeholder">Select an authorization type to configure authentication</p>
        {/if}
      </div>
    {:else if activeTab === "ssl"}
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
              ⚠️ SSL verification disabled - insecure, use only for development
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
    {:else}
      <div class="params-section">
        <p class="placeholder">Query params parsed from URL</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .request-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    background: #1e1e2e;
    min-width: 0;
  }

  .url-bar {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
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

  .request-tabs {
    display: flex;
    gap: 5px;
    border-bottom: 1px solid #3a3a4e;
    margin-bottom: 15px;
  }

  .tab-btn {
    padding: 10px 20px;
    background: transparent;
    border: none;
    color: #888;
    font-size: 13px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    color: #e4e4e7;
  }

  .tab-btn.active {
    color: #61affe;
    border-bottom-color: #61affe;
  }

  .tab-btn.auth-tab.active {
    color: #fca130;
    border-bottom-color: #fca130;
  }

  .tab-btn.ssl-tab.active {
    color: #fca130;
    border-bottom-color: #fca130;
  }

  .tab-badge {
    display: inline-block;
    background: #fca130;
    color: #1a1a2e;
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    margin-left: 6px;
    text-transform: uppercase;
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
  }

  .headers-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .header-row {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .header-key, .header-value {
    flex: 1;
    padding: 8px 12px;
    border-radius: 4px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 13px;
  }

  .header-key:focus, .header-value:focus {
    outline: none;
    border-color: #61affe;
  }

  .remove-btn {
    width: 30px;
    height: 30px;
    border-radius: 4px;
    border: none;
    background: #3a3a4e;
    color: #e4e4e7;
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .remove-btn:hover {
    background: #f93e3e;
    color: #fff;
  }

  .add-header-btn {
    padding: 10px;
    border-radius: 4px;
    border: 1px dashed #3a3a4e;
    background: transparent;
    color: #888;
    font-size: 13px;
    cursor: pointer;
    margin-top: 10px;
  }

  .add-header-btn:hover {
    border-color: #61affe;
    color: #61affe;
  }

  .body-section {
    height: 100%;
  }

  .body-editor {
    width: 100%;
    height: 100%;
    padding: 15px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-family: Monaco, Menlo, monospace;
    font-size: 13px;
    resize: none;
    box-sizing: border-box;
  }

  .body-editor:focus {
    outline: none;
    border-color: #61affe;
  }

  /* SSL Section Styles */
  .ssl-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 10px 0;
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

  .file-input:placeholder {
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

  /* Auth Section Styles */
  .auth-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 10px 0;
  }

  .auth-type-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .auth-label {
    color: #e4e4e7;
    font-size: 13px;
    font-weight: 500;
  }

  .auth-select {
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 14px;
    cursor: pointer;
  }

  .auth-select:focus {
    outline: none;
    border-color: #fca130;
  }

  .auth-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .auth-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .auth-input {
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 14px;
  }

  .auth-input:focus {
    outline: none;
    border-color: #61affe;
  }

  .auth-textarea {
    width: 100%;
    min-height: 80px;
    padding: 10px 12px;
    border-radius: 6px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-family: Monaco, Menlo, monospace;
    font-size: 13px;
    resize: vertical;
    box-sizing: border-box;
  }

  .auth-textarea:focus {
    outline: none;
    border-color: #61affe;
  }

  .auth-radio-group {
    display: flex;
    gap: 20px;
    padding: 8px 0;
  }

  .auth-radio {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #e4e4e7;
    font-size: 13px;
    cursor: pointer;
  }

  .auth-radio input[type="radio"] {
    accent-color: #61affe;
    cursor: pointer;
  }

  .placeholder {
    color: #666;
    text-align: center;
    padding: 40px;
  }
</style>
