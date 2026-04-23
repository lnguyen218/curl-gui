<script lang="ts">
  import type { ResponseData } from "../types";
  import { createEventDispatcher } from "svelte";

  export let response: ResponseData | null;
  export let error: string;
  export let curlCommand: string;

  const dispatch = createEventDispatcher<{ clear: void }>();

  let responseTab: "body" | "headers" = "body";
  let copied = false;

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

  async function copyBody() {
    if (!response) return;
    await navigator.clipboard.writeText(formatBody(response.body));
    copied = true;
    setTimeout(() => copied = false, 2000);
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
      <button class="clear-btn" on:click={() => dispatch("clear")} title="Clear response">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      </button>
    </div>

    <div class="response-tabs">
      <button class="tab-btn" class:active={responseTab === "body"} on:click={() => responseTab = "body"}>Body</button>
      <button class="tab-btn" class:active={responseTab === "headers"} on:click={() => responseTab = "headers"}>
        Headers ({Object.keys(response.headers).length})
      </button>
    </div>

    <div class="tab-content">
      {#if responseTab === "body"}
        <div class="body-wrapper">
          <button class="copy-btn" on:click={copyBody} title="Copy to clipboard">
            {#if copied}
              <svg class="copy-icon" viewBox="0 0 24 24" fill="none" stroke="#49cc90" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
            {:else}
              <svg class="copy-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
            {/if}
          </button>
          <pre class="response-body">{formatBody(response.body)}</pre>
        </div>
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
    min-height: 0;
  }

  .body-wrapper {
    position: relative;
  }

  .copy-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: #3a3a4e;
    border: 1px solid #4a4a5e;
    border-radius: 6px;
    padding: 6px 8px;
    color: #888;
    cursor: pointer;
    transition: all 0.2s;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .copy-btn:hover {
    background: #4a4a5e;
    color: #e4e4e7;
  }

  .copy-icon {
    width: 14px;
    height: 14px;
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

  .clear-btn {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    border: none;
    background: #3a3a4e;
    color: #888;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    margin-left: auto;
  }

  .clear-btn:hover {
    background: #f93e3e;
    color: #fff;
  }

  .clear-btn svg {
    width: 14px;
    height: 14px;
  }
</style>
