// YouTube - site-specific customizations
PAKE.config.site = 'youtube.com';

(function() {
  'use strict';

  // ============================================
  // SHORTS HIDING
  // ============================================

  const SHORTS_SELECTORS = [
    'ytd-rich-shelf-renderer[is-shorts]',
    'ytd-reel-shelf-renderer',
    'ytd-rich-section-renderer:has([title="Shorts"])',
    'ytd-guide-entry-renderer:has(a[title="Shorts"])',
    'ytd-mini-guide-entry-renderer:has(a[title="Shorts"])',
    '[tab-title="Shorts"]',
    'ytd-video-renderer:has([overlay-style="SHORTS"])',
    'ytd-grid-video-renderer:has([overlay-style="SHORTS"])',
    'ytd-rich-item-renderer:has([overlay-style="SHORTS"])',
  ];

  function hideShorts() {
    // Hide by selector
    SHORTS_SELECTORS.forEach(selector => {
      try {
        document.querySelectorAll(selector).forEach(el => {
          el.style.display = 'none';
        });
      } catch (e) {}
    });

    // Fallback: find Shorts section by text content
    document.querySelectorAll('ytd-rich-section-renderer, ytd-reel-shelf-renderer, ytd-rich-shelf-renderer').forEach(el => {
      if (el.querySelector('[is-shorts]') || el.querySelector('span#title')?.textContent?.includes('Shorts')) {
        el.style.display = 'none';
      }
    });

    // Hide Shorts link in sidebar
    document.querySelectorAll('ytd-guide-entry-renderer, ytd-mini-guide-entry-renderer').forEach(el => {
      if (el.textContent?.includes('Shorts')) {
        el.style.display = 'none';
      }
    });

    // Hide individual short videos in feeds
    document.querySelectorAll('ytd-rich-item-renderer, ytd-video-renderer, ytd-grid-video-renderer').forEach(el => {
      if (el.querySelector('[overlay-style="SHORTS"]') || el.querySelector('a[href*="/shorts/"]')) {
        el.style.display = 'none';
      }
    });
  }

  // ============================================
  // AD SKIPPING (Option A: Auto-skip + banner hiding)
  // ============================================

  function skipAd() {
    // Click skip button if available
    const skipButtons = [
      '.ytp-skip-ad-button',
      '.ytp-ad-skip-button',
      '.ytp-ad-skip-button-modern',
      'button.ytp-ad-skip-button',
      '.videoAdUiSkipButton',
      '[id^="skip-button"]',
    ];

    for (const selector of skipButtons) {
      const btn = document.querySelector(selector);
      if (btn && btn.offsetParent !== null) {
        btn.click();
        console.log('[Pake] Clicked skip button');
        return true;
      }
    }

    // Click "Skip ads" text button
    const skipText = document.querySelector('.ytp-ad-text.ytp-ad-skip-button-text');
    if (skipText) {
      skipText.click();
      return true;
    }

    return false;
  }

  function handleAd() {
    // Check if ad is playing
    const adPlaying = document.querySelector('.ad-showing, .ad-interrupting');
    if (!adPlaying) return;

    // Try to skip
    if (skipAd()) return;

    // If can't skip, check for video element and try to speed through
    const video = document.querySelector('video.html5-main-video');
    if (video && video.duration && video.duration < 120) {
      // Only speed up short ads (< 2 min), skip to near end
      // Add small delay to avoid detection
      const adOverlay = document.querySelector('.ytp-ad-player-overlay');
      if (adOverlay) {
        // Mute during ad
        if (!video.muted) {
          video._pakeMuted = true;
          video.muted = true;
        }
        // Speed through ad
        video.playbackRate = 16;
        video.currentTime = Math.max(video.duration - 0.1, 0);
      }
    }
  }

  function restoreVideoState() {
    const video = document.querySelector('video.html5-main-video');
    if (video && video._pakeMuted) {
      const adPlaying = document.querySelector('.ad-showing, .ad-interrupting');
      if (!adPlaying) {
        video.muted = false;
        video._pakeMuted = false;
        video.playbackRate = 1;
      }
    }
  }

  // ============================================
  // MAIN OBSERVER
  // ============================================

  function runAll() {
    hideShorts();
    handleAd();
    restoreVideoState();
  }

  // Debounce to avoid running too frequently
  let timeout = null;
  function debouncedRun() {
    if (timeout) return;
    timeout = setTimeout(() => {
      runAll();
      timeout = null;
    }, 100);
  }

  // Watch for dynamic content
  const observer = new MutationObserver(debouncedRun);

  function startObserver() {
    if (document.body) {
      observer.observe(document.body, {
        childList: true,
        subtree: true
      });
      runAll();
    } else {
      requestAnimationFrame(startObserver);
    }
  }

  startObserver();

  // Also check periodically for ads (backup)
  setInterval(handleAd, 500);

  console.log('[Pake] YouTube enhancements loaded (Shorts hiding + Ad skip)');
})();
