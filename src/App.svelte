<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS";

  interface RequestState {
    method: HttpMethod;
    url: string;
    headers: { key: string; value: string }[];
    body: string;
  }

  interface ResponseData {
    status_code: number;
    status_text: string;
    headers: Record<string, string>;
    body: string;
    time_ms: number;
  }

  let request: RequestState = $state({
    method: "GET",
    url: "",
    headers: [{ key: "", value: "" }],
    body: "",
  });

  let response: ResponseData | null = $state(null);
  let loading = $state(false);
  let error = $state("");
  let activeTab = $state<"headers" | "body" | "params">("headers");
  let responseTab = $state<"body" | "headers">("body");
  let curlCommand = $state("");

  const methods: HttpMethod[] = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

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

  const addHeader = () => {
    request.headers = [...request.headers, { key: "", value: "" }];
  };

  const removeHeader = (index: number) => {
    request.headers = request.headers.filter((_, i) => i !== index);
  };

  const updateHeader = (index: number, field: "key" | "value", value: string) => {
    const newHeaders = [...request.headers];
    newHeaders[index][field] = value;
    request.headers = newHeaders;
  };

  const generateCurlCommand = (): string => {
    let cmd = `curl -X ${request.method}`;
    
    request.headers
      .filter((h) => h.key && h.value)
      .forEach((h) => {
        cmd += ` -H "${h.key}: ${h.value}"`;
      });
    
    if (request.body && request.method !== "GET") {
      cmd += ` -d '${request.body}'`;
    }
    
    cmd += ` "${request.url}"`;
    return cmd;
  };

  const sendRequest = async () => {
    if (!request.url) {
      error = "Please enter a URL";
      return;
    }

    loading = true;
    error = "";
    response = null;
    curlCommand = generateCurlCommand();

    const headers: Record<string, string> = {};
    request.headers
      .filter((h) => h.key && h.value)
      .forEach((h) => {
        headers[h.key] = h.value;
      });

    try {
      const result = await invoke<ResponseData>("make_request", {
        request: {
          method: request.method,
          url: request.url,
          headers: Object.keys(headers).length > 0 ? headers : undefined,
          body: request.body || undefined,
        },
      });
      response = result;
    } catch (e: any) {
      error = e.error || e.toString();
    } finally {
      loading = false;
    }
  };

  const formatBody = (body: string): string => {
    try {
      const parsed = JSON.parse(body);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return body;
    }
  };

  const formatCurl = (cmd: string): string => {
    // Add line breaks for readability
    return cmd.replace(/ -H /g, " \\\n  -H ").replace(/ -d /g, " \\\n  -d ").replace(/ -X /g, " \\\n  -X ");
  };

  onMount(() => {
    // Load from localStorage if available
    const saved = localStorage.getItem("curl-insomnia-request");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        request = { ...request, ...parsed };
      } catch {
        // Ignore invalid saved data
      }
    }
  });

  // Save to localStorage when request changes
  $effect(() => {
    localStorage.setItem("curl-insomnia-request", JSON.stringify(request));
  });
</script>

<main class="app">
  <header class="app-header">
    <div class="logo">
      <span class="curl-logo">curl</span>
      <span class="divider">|</span>
      <span class="app-name">GUI</span>
    </div>
  </header>

  <div class="main-content">
    <!-- Request Panel -->
    <div class="request-panel">
      <div class="url-bar">
        <select bind:value={request.method} class="method-select">
          {#each methods as method}
            <option value={method} style="color: {getMethodColor(method)}">
              {method}
            </option>
          {/each}
        </select>
        <input
          type="text"
          bind:value={request.url}
          placeholder="https://api.example.com/endpoint"
          class="url-input"
        />
        <button on:click={sendRequest} class="send-btn" disabled={loading}>
          {#if loading}
            <span class="spinner"></span>
          {:else}
            Send
          {/if}
        </button>
      </div>

      <div class="request-tabs">
        <button
          class="tab-btn"
          class:active={activeTab === "params"}
          on:click={() => (activeTab = "params")}
        >
          Params
        </button>
        <button
          class="tab-btn"
          class:active={activeTab === "headers"}
          on:click={() => (activeTab = "headers")}
        >
          Headers
        </button>
        <button
          class="tab-btn"
          class:active={activeTab === "body"}
          on:click={() => (activeTab = "body")}
        >
          Body
        </button>
      </div>

      <div class="tab-content">
        {#if activeTab === "headers"}
          <div class="headers-section">
            {#each request.headers as header, i}
              <div class="header-row">
                <input
                  type="text"
                  placeholder="Key"
                  value={header.key}
                  on:input={(e) => updateHeader(i, "key", e.currentTarget.value)}
                  class="header-key"
                />
                <input
                  type="text"
                  placeholder="Value"
                  value={header.value}
                  on:input={(e) => updateHeader(i, "value", e.currentTarget.value)}
                  class="header-value"
                />
                <button on:click={() => removeHeader(i)} class="remove-btn" title="Remove">
                  ×
                </button>
              </div>
            {/each}
            <button on:click={addHeader} class="add-header-btn">+ Add Header</button>
          </div>
        {:else if activeTab === "body"}
          <div class="body-section">
            <textarea
              bind:value={request.body}
              placeholder={`{\n  "key": "value"\n}`}
              class="body-editor"
            ></textarea>
          </div>
        {:else}
          <div class="params-section">
            <p class="placeholder">Query params will be parsed from the URL</p>
          </div>
        {/if}
      </div>
    </div>

    <!-- Divider -->
    <div class="divider-line"></div>

    <!-- Response Panel -->
    <div class="response-panel">
      {#if response}
        <div class="response-meta">
          <span
            class="status-code"
            class:success={response.status_code >= 200 && response.status_code < 300}
            class:error={response.status_code >= 400}
          >
            {response.status_code} {response.status_text}
          </span>
          <span class="response-time">{response.time_ms}ms</span>
        </div>

        <div class="response-tabs">
          <button
            class="tab-btn"
            class:active={responseTab === "body"}
            on:click={() => (responseTab = "body")}
          >
            Body
          </button>
          <button
            class="tab-btn"
            class:active={responseTab === "headers"}
            on:click={() => (responseTab = "headers")}
          >
            Headers
          </button>
        </div>

        <div class="tab-content">
          {#if responseTab === "body"}
            <pre class="response-body">{formatBody(response.body)}</pre>
          {:else}
            <div class="response-headers">
              {#each Object.entries(response.headers) as [key, value]}
                <div class="header-item">
                  <span class="header-key-name">{key}:</span>
                  <span class="header-value-text">{value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if error}
        <div class="error-message">
          <span class="error-icon">⚠</span>
          {error}
        </div>
      {:else if curlCommand}
        <div class="curl-preview">
          <h3>Equivalent cURL Command</h3>
          <pre class="curl-code">{formatCurl(curlCommand)}</pre>
        </div>
      {:else}
        <div class="empty-state">
          <p>Enter a URL and click Send to make a request</p>
        </div>
      {/if}
    </div>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, sans-serif;
    background: #1a1a2e;
    color: #e4e4e7;
  }

  .app {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .app-header {
    background: #16162a;
    padding: 12px 20px;
    border-bottom: 1px solid #2a2a3e;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 18px;
    font-weight: 600;
  }

  .curl-logo {
    color: #61affe;
  }

  .divider {
    color: #666;
    font-weight: 300;
  }

  .app-name {
    color: #e4e4e7;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .request-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    background: #1e1e2e;
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

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid #fff;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
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

  .header-key,
  .header-value {
    flex: 1;
    padding: 8px 12px;
    border-radius: 4px;
    border: 1px solid #3a3a4e;
    background: #2a2a3e;
    color: #e4e4e7;
    font-size: 13px;
  }

  .header-key:focus,
  .header-value:focus {
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
    font-family: "Monaco", "Menlo", monospace;
    font-size: 13px;
    resize: none;
    box-sizing: border-box;
  }

  .body-editor:focus {
    outline: none;
    border-color: #61affe;
  }

  .placeholder {
    color: #666;
    text-align: center;
    padding: 40px;
  }

  .divider-line {
    width: 2px;
    background: #2a2a3e;
  }

  .response-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    background: #1a1a2e;
  }

  .response-meta {
    display: flex;
    align-items: center;
    gap: 15px;
    margin-bottom: 20px;
  }

  .status-code {
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 600;
    background: #3a3a4e;
  }

  .status-code.success {
    background: #49cc9020;
    color: #49cc90;
  }

  .status-code.error {
    background: #f93e3e20;
    color: #f93e3e;
  }

  .response-time {
    color: #888;
    font-size: 13px;
  }

  .response-tabs {
    display: flex;
    gap: 5px;
    border-bottom: 1px solid #3a3a4e;
    margin-bottom: 15px;
  }

  .response-body {
    margin: 0;
    padding: 15px;
    background: #2a2a3e;
    border-radius: 6px;
    font-family: "Monaco", "Menlo", monospace;
    font-size: 12px;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .response-headers {
    padding: 15px;
    background: #2a2a3e;
    border-radius: 6px;
  }

  .header-item {
    padding: 8px 0;
    border-bottom: 1px solid #3a3a4e;
    display: flex;
    gap: 10px;
    font-size: 13px;
  }

  .header-item:last-child {
    border-bottom: none;
  }

  .header-key-name {
    color: #61affe;
    font-weight: 500;
    min-width: 150px;
  }

  .header-value-text {
    color: #e4e4e7;
  }

  .error-message {
    padding: 20px;
    background: #f93e3e20;
    border-radius: 6px;
    color: #f93e3e;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .error-icon {
    font-size: 20px;
  }

  .curl-preview {
    padding: 20px;
  }

  .curl-preview h3 {
    margin: 0 0 15px 0;
    color: #888;
    font-size: 14px;
    font-weight: 500;
  }

  .curl-code {
    margin: 0;
    padding: 15px;
    background: #2a2a3e;
    border-radius: 6px;
    font-family: "Monaco", "Menlo", monospace;
    font-size: 12px;
    line-height: 1.6;
    overflow-x: auto;
    white-space: pre-wrap;
    color: #a6e22e;
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
  }
</style>