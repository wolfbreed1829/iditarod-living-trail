function loadTrail(assets) {
  const root = document.documentElement;
  root.dataset.runtime = 'loading';
  // Links and the native mobile menu remain usable while enhanced controls wait.
  let status, message, retry, timer, disabled = [], starting = false, failed = false;
  const ready = new Promise(resolve => {
    if (document.readyState !== 'loading') resolve();
    else document.addEventListener('DOMContentLoaded', resolve, {once: true});
  });
  const show = text => { if (status) { message.textContent = text; status.hidden = false; } };
  function lock() {
    disabled = [...document.querySelectorAll('button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea:not(:disabled)')].filter(el => !el.closest('#runtime-status'));
    disabled.forEach(el => { el.disabled = true; });
  }
  function unlock() { disabled.forEach(el => { el.disabled = false; }); disabled = []; }
  ready.then(() => {
    // This shell lives after the Leptos root and is never part of its DOM tree.
    status = document.getElementById('runtime-status');
    message = document.getElementById('runtime-message');
    retry = document.getElementById('runtime-retry');
    retry.addEventListener('click', () => location.reload());
    lock();
    timer = setTimeout(() => show('Interactive tools are loading. You can read and follow links now.'), 1200);
    document.addEventListener('keydown', event => {
      if (event.key === 'Escape') {
        const menu = document.querySelector('.mobile-navigation[open]');
        if (menu) { menu.open = false; menu.querySelector('summary').focus(); }
      }
    });
    document.addEventListener('click', event => {
      const link = event.target.closest('a[href]');
      if (link?.closest('.mobile-navigation')) link.closest('details').open = false;
    });
  });
  // Fetch the module in parallel. Stream compilation uses the browser HTTP cache.
  // No service worker caches documents, CMS records, private previews or forms.
  const binary = fetch(assets.wasm, {credentials: 'same-origin'}).then(response => {
    if (!response.ok) throw Error('Application download failed');
    return response;
  });
  const module = import(assets.js);
  Promise.all([module, binary, ready]).then(async ([mod, response]) => {
    await mod.default({module_or_path: response});
    clearTimeout(timer);
    unlock();
    starting = true;
    mod.hydrate();
    root.dataset.runtime = 'ready';
    status.hidden = true;
    document.dispatchEvent(new Event('trail:ready'));
  }).catch(async () => {
    await ready;
    clearTimeout(timer);
    if (starting) lock();
    failed = true;
    root.dataset.runtime = 'error';
    show('Interactive tools could not load. Your connection may be interrupted. Pages and links are still available.');
    retry.hidden = false;
  });
  window.addEventListener('offline', () => {
    if (root.dataset.runtime !== 'ready') show('You are offline. This page is available to read; reconnect and try again for interactive tools.');
  });
  window.addEventListener('online', () => { if (failed) show('Your connection is back. Try loading the interactive tools again.'); });
}
