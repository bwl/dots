// Pake inject harness
// Shared code loaded before per-site customizations

window.PAKE = window.PAKE || {};
PAKE.config = {};
PAKE.version = '0.1.0';

// Stub: which-key feature (future)
// Will dynamically discover shortcuts from site's registered events or help overlay
PAKE.showHelp = function() {
  console.log('[Pake] Which-key help: not implemented');
  console.log('[Pake] Site:', PAKE.config.site || 'unknown');
};

// Register Ctrl+? or Cmd+? to show help stub
document.addEventListener('keydown', function(e) {
  if ((e.ctrlKey || e.metaKey) && e.key === '?') {
    e.preventDefault();
    PAKE.showHelp();
  }
});

console.log('[Pake] Harness loaded');
