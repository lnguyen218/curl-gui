import App from './App.svelte'
import './app.css'

const SMART_QUOTE_RE = /[\u2018\u2019\u201c\u201d\u2013\u2014]/;

function normalizeSmartQuotes(text: string): string {
  return text
    .replace(/\u2018/g, "'")
    .replace(/\u2019/g, "'")
    .replace(/\u201c/g, '"')
    .replace(/\u201d/g, '"')
    .replace(/\u2013/g, '-')
    .replace(/\u2014/g, '--');
}

// Disable macOS/WebKit smart substitution on all editable fields.
// This is the primary fix; it prevents the quote/dash replacement
// from happening in the first place.
function disableSmartSubstitution(el: HTMLElement) {
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
    el.setAttribute('autocorrect', 'off');
    el.setAttribute('autocapitalize', 'off');
    el.setAttribute('spellcheck', 'false');
  }
}

// Apply to dynamically focused elements
// (covers Svelte-created inputs after app mount).
document.addEventListener('focusin', (event) => {
  const target = event.target as HTMLElement | null;
  if (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target?.isContentEditable
  ) {
    disableSmartSubstitution(target);
  }
});

// Fallback: if substitution still sneaks through (e.g. via paste or
// an OS-level replacement), normalize the value after the fact.
let handling = false;

document.addEventListener('input', (event: Event) => {
  if (handling) return;

  const target = event.target as HTMLElement | null;
  if (!target) return;

  if (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement
  ) {
    const value = target.value;
    if (!SMART_QUOTE_RE.test(value)) return;

    handling = true;

    const start = target.selectionStart ?? value.length;
    const end = target.selectionEnd ?? value.length;
    const normalized = normalizeSmartQuotes(value);

    target.value = normalized;

    // Adjust cursor for length changes (e.g. em-dash 1 → 2 chars)
    const delta = normalized.length - value.length;
    target.setSelectionRange(
      Math.max(0, start + delta),
      Math.max(0, end + delta)
    );

    // Notify Svelte bind:value listeners
    target.dispatchEvent(new Event('input', { bubbles: true }));
    handling = false;
  }
});

const app = new App({
  target: document.getElementById('app')!,
})

export default app
