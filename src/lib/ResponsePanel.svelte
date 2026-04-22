<script lang="ts">
  import type { ResponseData } from "../types";

  export let response: ResponseData | null;
  export let error: string;
  export let curlCommand: string;

  let responseTab: "body" | "headers" = "body";

  function formatBody(body: string): string {
    try {
      return JSON.stringify(JSON.parse(body), null, 2);
    } catch {
      return body;
    }
  }

  function formatCurl(cmd: string): string {
    return cmd.replace(/ -H /g, " \\\n  -H ").replace(/ -d /g, " \\\n  -d ").replace(/ -X /g, " \\\n  -X ");
  }
</script>

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
      <button class="tab-btn" class:active={responseTab === "body"} on:click={() => responseTab = "body"}>Body</button>
      <button class="tab-btn" class:active={responseTab === "headers"} on:click={() => responseTab = "headers"}>
        Headers ({Object.keys(response.headers).length})
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
      <h3>cURL Command</h3>
      <pre class="curl-code">{formatCurl(curlCommand)}</pre>
    </div>
  {:else}
    <div class="empty-state">
      <p>Enter URL and click Send</p>
    </div>
  {/if}
</div>

<style>
  .response-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    background: #1a1a2e;
    min-width: 0;
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

  .response-body {
    margin: 0;
    padding: 15px;
    background: #2a2a3e;
    border-radius: 6px;
    font-family: Monaco, Menlo, monospace;
    font-size: 12px;
    line-height: 1.5;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    text-align: left;
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
    font-family: Monaco, Menlo, monospace;
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