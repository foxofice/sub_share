/* ============ 字幕共享计划 · 交互脚本 ============ */
(function(){
  'use strict';
  var $  = function(s,r){ return (r||document).querySelector(s); };
  var $$ = function(s,r){ return Array.prototype.slice.call((r||document).querySelectorAll(s)); };
  var reduce = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* ---------- 主题切换 ---------- */
  function initTheme(){
    var btn = $('#themeBtn');
    if(!btn) return;
    function sync(){
      var t = document.documentElement.dataset.theme;
      btn.setAttribute('aria-pressed', t === 'light' ? 'true' : 'false');
      btn.setAttribute('aria-label', t === 'light' ? '切换到深色' : '切换到浅色');
      var meta = document.querySelector('meta[name="theme-color"]');
      if(meta) meta.setAttribute('content', t === 'light' ? '#F7F5F0' : '#0B0F17');
    }
    sync();
    btn.addEventListener('click', function(){
      var next = document.documentElement.dataset.theme === 'light' ? 'dark' : 'light';
      document.documentElement.dataset.theme = next;
      try { localStorage.setItem('theme', next); } catch(e){}
      sync();
    });
    // 仅当用户未显式选择时跟随系统
    var mq = window.matchMedia && window.matchMedia('(prefers-color-scheme: light)');
    if(mq && mq.addEventListener){
      mq.addEventListener('change', function(e){
        var stored = null;
        try { stored = localStorage.getItem('theme'); } catch(err){}
        if(!stored){ document.documentElement.dataset.theme = e.matches ? 'light' : 'dark'; sync(); }
      });
    }
  }

  /* ---------- 复制反馈 ---------- */
  var toastTimer = null;
  function showToast(msg, ok){
    var toast = $('#toast'), live = $('#liveRegion');
    if(toast){
      $('span', toast).textContent = msg;
      toast.classList.toggle('toast--error', !ok);
      toast.classList.add('show');
      clearTimeout(toastTimer);
      toastTimer = setTimeout(function(){ toast.classList.remove('show'); }, 1600);
    }
    if(live) live.textContent = msg;
  }
  function fallbackCopy(text){
    try {
      var ta = document.createElement('textarea');
      ta.value = text;
      ta.style.position = 'fixed'; ta.style.opacity = '0';
      document.body.appendChild(ta); ta.select();
      var ok = document.execCommand('copy');
      document.body.removeChild(ta);
      return ok;
    } catch(e){ return false; }
  }
  function initCopy(){
    var usingClipboard = navigator.clipboard && navigator.clipboard.writeText && window.isSecureContext;
    document.addEventListener('click', function(e){
      var btn = e.target.closest('[data-copy]');
      if(!btn) return;
      e.preventDefault();
      var text = btn.getAttribute('data-copy');
      if(!text){
        var code = $('code', btn.closest('.term') || document);
        text = code ? code.textContent.trim() : '';
      }
      if(usingClipboard){
        navigator.clipboard.writeText(text).then(function(){
          showToast('已复制到剪贴板', true);
        }).catch(function(){
          showToast(fallbackCopy(text) ? '已复制到剪贴板' : '请手动复制', true);
        });
      } else {
        showToast(fallbackCopy(text) ? '已复制到剪贴板' : '请手动复制', true);
      }
    });
  }

  /* ---------- 分类 Tabs ---------- */
  function initTabs(){
    var tabs = $$('.tab'), r18Modal = $('#r18Modal');
    if(!tabs.length) return;
    var pendingR18Tab = null;

    function activate(tab){
      tabs.forEach(function(t){
        var on = t === tab;
        t.classList.toggle('is-active', on);
        t.setAttribute('aria-selected', on ? 'true' : 'false');
        t.tabIndex = on ? 0 : -1;
        var panel = document.getElementById(t.getAttribute('aria-controls'));
        if(panel) panel.hidden = !on;
      });
    }
    function request(tab){
      if(tab.dataset.cat === 'r18' && !tab.classList.contains('is-active') && r18Modal){
        pendingR18Tab = tab;
        openR18();
      } else {
        activate(tab);
      }
    }
    function openR18(){
      r18Modal.hidden = false;
      document.body.style.overflow = 'hidden';
      var cancel = $('[data-cancel-r18].btn', r18Modal) || $('#r18Confirm');
      if(cancel) cancel.focus();
      r18Modal._trigger = document.activeElement;
    }
    function closeR18(confirm){
      r18Modal.hidden = true;
      document.body.style.overflow = '';
      if(confirm && pendingR18Tab) activate(pendingR18Tab);
      var tr = r18Modal._trigger;
      if(tr) tr.focus();
      pendingR18Tab = null;
    }

    tabs.forEach(function(tab, idx){
      tab.addEventListener('click', function(){ request(tab); });
      tab.addEventListener('keydown', function(e){
        var next = null;
        if(e.key === 'ArrowRight') next = tabs[(idx+1) % tabs.length];
        else if(e.key === 'ArrowLeft') next = tabs[(idx-1+tabs.length) % tabs.length];
        else if(e.key === 'Home') next = tabs[0];
        else if(e.key === 'End') next = tabs[tabs.length-1];
        if(next){ e.preventDefault(); next.focus(); request(next); }
      });
    });

    if(r18Modal){
      $$('[data-cancel-r18]', r18Modal).forEach(function(el){ el.addEventListener('click', function(){ closeR18(false); }); });
      $('#r18Confirm').addEventListener('click', function(){ closeR18(true); });
      r18Modal.addEventListener('keydown', function(e){
        if(e.key === 'Escape'){ e.preventDefault(); closeR18(false); return; }
        if(e.key === 'Tab'){
          var focusables = $$('button', r18Modal);
          var first = focusables[0], last = focusables[focusables.length-1];
          if(e.shiftKey && document.activeElement === first){ e.preventDefault(); last.focus(); }
          else if(!e.shiftKey && document.activeElement === last){ e.preventDefault(); first.focus(); }
        }
      });
    }
  }

  /* ---------- 移动抽屉 ---------- */
  function initDrawer(){
    var btn = $('#menuBtn'), drawer = $('#mobileDrawer');
    if(!btn || !drawer) return;
    var panel = $('.drawer__panel', drawer);
    function open(){
      drawer.hidden = false;
      btn.setAttribute('aria-expanded','true');
      document.body.style.overflow = 'hidden';
      var first = $('a, button', panel); if(first) first.focus();
      document.addEventListener('keydown', onKey);
    }
    function close(){
      drawer.hidden = true;
      btn.setAttribute('aria-expanded','false');
      document.body.style.overflow = '';
      document.removeEventListener('keydown', onKey);
      btn.focus();
    }
    function onKey(e){
      if(e.key === 'Escape'){ e.preventDefault(); close(); return; }
      if(e.key === 'Tab'){
        var f = $$('a, button', panel); if(!f.length) return;
        var first = f[0], last = f[f.length-1];
        if(e.shiftKey && document.activeElement === first){ e.preventDefault(); last.focus(); }
        else if(!e.shiftKey && document.activeElement === last){ e.preventDefault(); first.focus(); }
      }
    }
    btn.addEventListener('click', function(){ drawer.hidden ? open() : close(); });
    $$('[data-close-drawer]', drawer).forEach(function(el){ el.addEventListener('click', close); });
    $$('.drawer__nav a', drawer).forEach(function(a){ a.addEventListener('click', close); });
    window.addEventListener('resize', function(){
      if(window.innerWidth >= 900 && !drawer.hidden){
        drawer.hidden = true; btn.setAttribute('aria-expanded','false');
        document.body.style.overflow = ''; document.removeEventListener('keydown', onKey);
      }
    });
  }

  /* ---------- 滚动渐显 ---------- */
  function initReveal(){
    var els = $$('.reveal');
    if(!els.length) return;
    if(reduce || !('IntersectionObserver' in window)){
      els.forEach(function(el){ el.classList.add('is-in'); });
      return;
    }
    var io = new IntersectionObserver(function(entries){
      entries.forEach(function(en){
        if(en.isIntersecting){ en.target.classList.add('is-in'); io.unobserve(en.target); }
      });
    }, { threshold:.1, rootMargin:'0px 0px -8% 0px' });
    els.forEach(function(el){ io.observe(el); });
  }

  /* ---------- 导航高亮 / 状态灯 / 返回顶部 ---------- */
  function initSpy(){
    var links = $$('.nav__link'), lamp = $('#statusLamp'), back = $('#backTop'), hero = $('#hero');
    var sections = links.map(function(l){ return document.getElementById(l.getAttribute('href').slice(1)); }).filter(Boolean);

    // 状态灯 + 返回顶部：离开 Hero 时显示
    function onScroll(){
      var show = hero && window.scrollY > hero.offsetHeight - 120;
      if(lamp) lamp.classList.toggle('on', !!show);
      if(back) back.classList.toggle('on', !!show);
    }
    var ticking = false;
    window.addEventListener('scroll', function(){
      if(!ticking){ requestAnimationFrame(function(){ onScroll(); ticking = false; }); ticking = true; }
    }, { passive:true });
    onScroll();

    // 当前分区高亮
    if('IntersectionObserver' in window && sections.length){
      var io = new IntersectionObserver(function(entries){
        var visible = entries.filter(function(e){ return e.isIntersecting; })
                             .sort(function(a,b){ return a.boundingClientRect.top - b.boundingClientRect.top; });
        if(visible.length){
          var id = visible[0].target.id;
          links.forEach(function(l){ l.classList.toggle('is-active', l.getAttribute('href') === '#'+id); });
        }
      }, { rootMargin:'-45% 0px -50% 0px', threshold:0 });
      sections.forEach(function(s){ io.observe(s); });
    }

    if(back) back.addEventListener('click', function(){
      window.scrollTo({ top:0, behavior: reduce ? 'auto' : 'smooth' });
    });
  }

  /* ---------- 启动 ---------- */
  function init(){
    initTheme();
    initCopy();
    initTabs();
    initDrawer();
    initReveal();
    initSpy();
  }
  if(document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();
})();
