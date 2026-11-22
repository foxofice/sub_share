/**
 * 字幕共享计划 (Subtitles DataBase) · 交互与检索核心脚本
 * 
 * 功能特性：
 * 1. 毫秒级多维度实时检索（支持中/日/英/罗马音/字幕组/格式/路径匹配，多关键词空格与检索）
 * 2. 彻底剔除 R18 内容检索，保障页面健康纯净
 * 3. 国内极速封面加载引擎（在线优先）：
 *    - 仓库内置封面走 jsDelivr 国内 CDN 在线加载（稳定、免防盗链、随仓库更新）
 *    - 其余封面直连 Bangumi 官方图域 lain.bgm.tv（配合 no-referrer，覆盖全量番剧）
 *    - 多级故障转移：wsrv.nl 反代 -> Jetpack CDN -> 同域本地封面 -> 色块占位
 * 4. 智能并发限流与本地持久化缓存（IndexedDB / localStorage，视口懒加载）
 * 5. 点击直接跳转 GitHub 仓库对应字幕目录（附带一键复制相对路径与稀疏检出命令）
 * 6. 优先展示最近更新批次，聚合番剧海报视图与单条字幕视图自由切换
 */

(function () {
  'use strict';

  // =============== 工具函数 ===============
  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => Array.from(root.querySelectorAll(selector));

  const GITHUB_REPO = 'foxofice/sub_share';
  const GITHUB_TREE_BASE = `https://github.com/${GITHUB_REPO}/tree/master/subs_list/`;
  const STORAGE_KEY_TOKEN = 'subshare_github_token';
  const STORAGE_KEY_THEME = 'subshare_theme';
  const STORAGE_KEY_VIEW = 'subshare_view_mode';
  const STORAGE_KEY_MIRROR = 'subshare_mirror_source';
  const IDB_DB_NAME = 'subshare_bangumi_cache_v2';
  const IDB_STORE_NAME = 'anime_covers';

  // =============== zst 数据源与加载策略（地址来自 TECH_BRIEF 实测，不得臆测） ===============
  const ZST_URLS = [
    'https://cdn.jsdelivr.net/gh/foxofice/sub_share@master/Subtitles%20DataBase/Files/db.json.zst',
    'https://raw.githubusercontent.com/foxofice/sub_share/master/Subtitles%20DataBase/Files/db.json.zst'
  ];
  const FZSTD_URL = 'https://cdn.jsdelivr.net/npm/fzstd@0.1.1/umd/index.js';
  const TIMEOUT = {
    zst: 6000,        // zst 单源
    fzstd: 5000,      // fzstd 脚本
    localJson: 4000,  // 本地 subtitles.json
    xml: 4000,        // XML 每源
    seed: 4000,       // seed_covers.json
    global: 12000     // 全局护栏
  };
  const STAGE_TEXT = {
    init: '正在初始化数据索引...',
    download: '正在下载字幕数据...',
    decompress: '正在解压字幕数据...',
    finalize: '正在整理字幕条目...',
    local: '正在读取本地索引...',
    xml: '正在从备用数据源读取...'
  };
  const ZST_EXPECTED_BYTES = 234497;  // TECH_BRIEF 实测压缩文件大小，供下载进度条按真实字节计算
  const MIN_ZST_BYTES = 1000;         // 小于此字节数判为无效
  const MIN_VALID_COUNT = 4000;       // 少于此条数触发降级（4684 条属正常，不降级）
  const MAX_CORE_MISSING_RATIO = 0.3; // 核心字段缺失超过 30% 判为无效
  const TOAST_DEGRADE_MS = 2500;      // 降级弱提示 2.5s
  const TOAST_FAILURE_MS = 2500;      // 失败提示 2.5s

  // =============== 全局状态 ===============
  const state = {
    rawList: [],        // 4,749 条单条字幕记录（已彻底过滤 R18）
    animeList: [],      // 2,104 部聚合番剧
    filteredAnime: [],  // 过滤后的番剧
    filteredFlat: [],   // 过滤后的单条记录
    viewMode: localStorage.getItem(STORAGE_KEY_VIEW) || 'group', // 'group' (聚合) | 'flat' (平铺)
    sortBy: 'recent',   // 'recent' | 'air_desc' | 'air_asc' | 'title_asc'
    filters: {
      keyword: '',
      type: 'all',      // 'all' | 'animation' | 'movie'
      source: 'all',    // 'all' | 'BD' | 'TV' | 'Web' ...
      ext: 'all',       // 'all' | 'ass' | 'srt'
      year: 'all'
    },
    pagination: {
      page: 1,
      pageSize: 60,
      total: 0
    },
    seedCovers: {},     // 预填充封面（含本地 covers/ 路径）
    recentPaths: new Set(),
    latestCommit: null,
    githubToken: localStorage.getItem(STORAGE_KEY_TOKEN) || '',
    idb: null
  };

  // =============== 数据加载并发控制 ===============
  let isLoading = false;          // 加载中重复点击防护锁
  let loadGeneration = 0;         // 请求序号：拖拽导入时自增，作废旧 zst 回调
  let detailLoadGeneration = 0;   // 详情弹窗请求序号：切换番剧时作废过期简介回填
  let fzstdLoadPromise = null;    // fzstd 脚本预取缓存（失败后可重试）
  let activeGuard = null;         // 当前全局护栏 AbortController

  // =============== 辅助路径与元数据匹配函数 ===============
  function getRepoFolderUrl(path) {
    if (!path) return `https://github.com/${GITHUB_REPO}/tree/master/subs_list`;
    const cleanPath = path.replace(/[\\/]+/g, '/').replace(/^\/+/g, '');
    const segments = cleanPath.split('/').map(seg => encodeURIComponent(seg)).join('/');
    return `${GITHUB_TREE_BASE}${segments}`;
  }

  function findSeedMeta(path) {
    if (!path || !state.seedCovers) return null;
    if (state.seedCovers[path]) return state.seedCovers[path];
    const forward = path.replace(/[\\/]+/g, '/');
    if (state.seedCovers[forward]) return state.seedCovers[forward];
    const backward = path.replace(/[\\/]+/g, '\\');
    if (state.seedCovers[backward]) return state.seedCovers[backward];
    return null;
  }

  // =============== 在线封面地址解析（在线优先，国内可靠源） ===============
  function getSafeCoverUrl(meta) {
    if (!meta || meta.notFound) return null;
    // 1. 仓库内置封面 -> jsDelivr 国内 CDN 在线加载（稳定、免防盗链、不依赖第三方图床存活）
    if (meta.localCover) {
      const name = String(meta.localCover).replace(/\\/g, '/').split('/').pop();
      if (name) return `https://cdn.jsdelivr.net/gh/${GITHUB_REPO}@gh-pages/covers/${name}`;
    }
    // 2. 其余走 Bangumi 官方图域在线直连（配合 img 的 referrerpolicy=no-referrer 绕过防盗链）
    const raw = meta.cover;
    if (raw) return String(raw).replace(/^http:\/\//i, 'https://');
    return null;
  }

  // 全局图片加载失败备用转移（多级在线兜底）
  window.handleCoverError = function (img, rawUrl) {
    if (!img) return;
    const step = parseInt(img.dataset.failStep || '0', 10);
    if (!rawUrl) rawUrl = img.dataset.rawUrl || '';
    const localCover = img.dataset.localCover || '';
    const localName = localCover ? String(localCover).replace(/\\/g, '/').split('/').pop() : '';
    const cleanUrl = String(rawUrl).replace(/^http:\/\//i, 'https://');

    if (step === 0 && cleanUrl) {
      // 一级：wsrv.nl 反代 Bangumi 原图（剥离 Referer，绕过防盗链）
      img.dataset.failStep = '1';
      img.src = 'https://wsrv.nl/?url=' + encodeURIComponent(cleanUrl) + '&w=400&output=jpg';
    } else if (step === 1 && cleanUrl) {
      // 二级：Jetpack CDN
      img.dataset.failStep = '2';
      img.src = 'https://i0.wp.com/' + cleanUrl.replace(/^https?:\/\//i, '');
    } else if (step === 2 && localName) {
      // 三级：同域本地封面（线上 gh-pages 与本地预览均存在）
      img.dataset.failStep = '3';
      img.src = './covers/' + localName;
    } else {
      // 最终降级：纯净色块占位
      img.style.display = 'none';
      const thumb = img.closest('.poster-thumb');
      if (thumb) {
        thumb.classList.add('is-fallback');
      }
    }
  };

  // =============== IndexedDB 缓存管理器 ===============
  async function initIndexedDB() {
    return new Promise((resolve) => {
      if (!window.indexedDB) {
        resolve(null);
        return;
      }
      const request = window.indexedDB.open(IDB_DB_NAME, 1);
      request.onupgradeneeded = (e) => {
        const db = e.target.result;
        if (!db.objectStoreNames.contains(IDB_STORE_NAME)) {
          db.createObjectStore(IDB_STORE_NAME, { keyPath: 'key' });
        }
      };
      request.onsuccess = (e) => {
        state.idb = e.target.result;
        resolve(state.idb);
      };
      request.onerror = () => resolve(null);
    });
  }

  async function getFromCache(key) {
    if (state.seedCovers[key]) {
      return state.seedCovers[key];
    }
    if (state.idb) {
      return new Promise((resolve) => {
        try {
          const tx = state.idb.transaction(IDB_STORE_NAME, 'readonly');
          const store = tx.objectStore(IDB_STORE_NAME);
          const req = store.get(key);
          req.onsuccess = () => resolve(req.result ? req.result.data : null);
          req.onerror = () => resolve(null);
        } catch (e) {
          resolve(null);
        }
      });
    }
    try {
      const stored = localStorage.getItem('bgm_' + key);
      return stored ? JSON.parse(stored) : null;
    } catch (e) {
      return null;
    }
  }

  async function saveToCache(key, data) {
    if (state.idb) {
      try {
        const tx = state.idb.transaction(IDB_STORE_NAME, 'readwrite');
        const store = tx.objectStore(IDB_STORE_NAME);
        store.put({ key, data, time: Date.now() });
      } catch (e) {}
    } else {
      try {
        localStorage.setItem('bgm_' + key, JSON.stringify(data));
      } catch (e) {}
    }
  }

  // =============== 番剧元数据抓取器 ===============
  class BangumiScraper {
    constructor() {
      this.queue = [];
      this.running = 0;
      this.maxConcurrent = 2;
      this.delayBetween = 200;
      this.observer = null;
      this.callbacks = new Map();
      this.initObserver();
    }

    initObserver() {
      if ('IntersectionObserver' in window) {
        this.observer = new IntersectionObserver((entries) => {
          entries.forEach(entry => {
            if (entry.isIntersecting) {
              const el = entry.target;
              const path = el.dataset.path;
              const chs = el.dataset.chs;
              const jp = el.dataset.jp;
              if (path && !el.dataset.fetching) {
                el.dataset.fetching = 'true';
                this.request(path, chs, jp, (meta) => {
                  this.updateCardCover(el, meta);
                });
                this.observer.unobserve(el);
              }
            }
          });
        }, { rootMargin: '200px 0px' });
      }
    }

    observeCard(cardEl) {
      if (this.observer) {
        this.observer.observe(cardEl);
      } else {
        const path = cardEl.dataset.path;
        this.request(path, cardEl.dataset.chs, cardEl.dataset.jp, (meta) => {
          this.updateCardCover(cardEl, meta);
        });
      }
    }

    cleanTitle(rawTitle) {
      if (!rawTitle) return '';
      let t = rawTitle.split('/')[0].trim();
      t = t.replace(/（[^）]*）|\([^)]*\)|【[^】]*】|\[[^\]]*\]/g, ' ').trim();
      t = t.replace(/第[一二三四五六七八九十0-9]+[季部期]/g, '').trim();
      t = t.replace(/\b(S[0-9]+|Season\s*[0-9]+|TV)\b/gi, '').trim();
      return t;
    }

    async request(path, chs, jp, callback) {
      const seed = findSeedMeta(path);
      if (seed) {
        callback(seed);
        return;
      }
      const cached = await getFromCache(path);
      if (cached) {
        callback(cached);
        return;
      }

      if (this.callbacks.has(path)) {
        this.callbacks.get(path).push(callback);
        return;
      }
      this.callbacks.set(path, [callback]);

      this.queue.push({ path, chs, jp });
      this.processNext();
    }

    async processNext() {
      if (this.running >= this.maxConcurrent || this.queue.length === 0) {
        return;
      }

      this.running++;
      const item = this.queue.shift();

      try {
        let meta = await this.fetchFromBangumi(item.chs, item.jp);
        if (!meta) {
          meta = { notFound: true, name: item.chs };
        }
        await saveToCache(item.path, meta);

        const cbs = this.callbacks.get(item.path) || [];
        cbs.forEach(cb => cb(meta));
        this.callbacks.delete(item.path);
      } catch (err) {
        console.warn('Metadata fetch warning:', item.chs, err);
      } finally {
        setTimeout(() => {
          this.running--;
          this.processNext();
        }, this.delayBetween);
      }
    }

    async fetchFromBangumi(chs, jp) {
      const cleanChs = this.cleanTitle(chs);
      const queryList = [cleanChs, jp, chs].filter(Boolean);

      for (const q of queryList) {
        if (!q || q.length < 2) continue;
        const res = await this.doBangumiSearch(q);
        if (res) return res;
      }
      return null;
    }

    async doBangumiSearch(keyword) {
      try {
        const url = `https://api.bgm.tv/search/subject/${encodeURIComponent(keyword)}?type=2&responseGroup=small`;
        const resp = await fetch(url, {
          headers: { 'User-Agent': 'foxofice-subshare/3.0 (browser client)' }
        });
        if (!resp.ok) return null;
        const data = await resp.json();
        if (data && data.list && data.list.length > 0) {
          const first = data.list[0];
          const imgUrl = (first.images && (first.images.common || first.images.medium || first.images.large)) || '';
          return {
            id: first.id,
            name: first.name,
            name_cn: first.name_cn || first.name,
            // 搜索接口 responseGroup=small 不带简介，稍后用详情接口补全
            summary: first.summary || '',
            score: first.rating ? first.rating.score : null,
            cover: imgUrl ? imgUrl.replace(/^http:\/\//i, 'https://') : null,
            url: `https://bgm.tv/subject/${first.id}`
          };
        }
      } catch (e) {}
      return null;
    }

    // 用 Bangumi v0 条目详情接口补全简介（搜索接口不带 summary，CORS 全开）
    async fetchSubjectSummary(id) {
      if (!id) return '';
      try {
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), 6000);
        try {
          const resp = await fetch(`https://api.bgm.tv/v0/subjects/${encodeURIComponent(id)}`, {
            headers: { 'User-Agent': 'foxofice-subshare/3.0 (browser client)' },
            signal: controller.signal
          });
          if (!resp.ok) return '';
          const data = await resp.json();
          return typeof data.summary === 'string' ? data.summary.trim() : '';
        } finally {
          clearTimeout(timer);
        }
      } catch (e) {}
      return '';
    }

    updateCardCover(cardEl, meta) {
      if (!cardEl) return;
      const imgWrap = $('.poster-thumb', cardEl);
      if (!imgWrap) return;

      const safeUrl = getSafeCoverUrl(meta);
      if (safeUrl && !meta.notFound) {
        imgWrap.innerHTML = `
          <img class="poster-img"
               src="${safeUrl}"
               data-raw-url="${meta.cover || ''}"
               data-local-cover="${meta.localCover || ''}"
               alt="${escapeHtml(meta.name_cn || meta.name || '')}"
               loading="lazy"
               referrerpolicy="no-referrer"
               onerror="handleCoverError(this, this.dataset.rawUrl)"
          >
        `;
        cardEl.classList.add('has-cover');
      }
      if (meta && meta.score) {
        const rateEl = $('.poster-score', cardEl);
        if (rateEl) {
          rateEl.textContent = '★ ' + meta.score;
          rateEl.hidden = false;
        }
      }
    }
  }

  const scraper = new BangumiScraper();

  // =============== 数据载入引擎（zst 优先 + 无感多级降级） ===============

  // 稳定的阶段文案节点：只创建一次，后续仅更新 textContent，不重建节点
  function getStageNode() {
    const loader = $('#mainLoader');
    if (!loader) return null;
    let stage = loader.__stageNode;
    if (!stage || !stage.isConnected) {
      stage = document.createElement('span');
      stage.className = 'loader-stage-text';
      stage.setAttribute('role', 'status');
      stage.setAttribute('aria-live', 'polite');
      loader.appendChild(stage);
      loader.__stageNode = stage;
    }
    return stage;
  }

  // 设置阶段文案；传空串隐藏 loader。文案容器 role=status / aria-live=polite
  function setStage(text) {
    const loader = $('#mainLoader');
    if (!loader) return;
    const stage = getStageNode();
    if (!text) {
      loader.hidden = true;
      loader.style.display = 'none';
      return;
    }
    if (stage && stage.textContent !== text) stage.textContent = text;
    loader.hidden = false;
    // 作者样式 .main-loader{display:flex} 会压过 hidden 属性，必须内联显式显示
    loader.style.display = 'flex';
  }

  // 双 rAF：强制绘制一帧，让同步解压前的文案真正落地（reduced-motion 下同样保留）
  function afterTwoFrames() {
    return new Promise(resolve => {
      requestAnimationFrame(() => requestAnimationFrame(resolve));
    });
  }

  // 带超时的 fetch；父信号（全局护栏）可提前 abort；timer 与 listener 均清理
  function fetchWithTimeout(url, options, timeoutMs, parentSignal) {
    return new Promise((resolve, reject) => {
      if (parentSignal && parentSignal.aborted) {
        reject(new Error('aborted by guard'));
        return;
      }
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), timeoutMs);
      const onParentAbort = () => controller.abort();
      if (parentSignal) parentSignal.addEventListener('abort', onParentAbort, { once: true });
      const cleanup = () => {
        clearTimeout(timer);
        if (parentSignal) parentSignal.removeEventListener('abort', onParentAbort);
      };
      fetch(url, Object.assign({}, options || {}, { signal: controller.signal }))
        .then(res => { cleanup(); resolve(res); }, err => { cleanup(); reject(err); });
    });
  }

  // 预取 fzstd（UMD 暴露 window.fzstd）；失败返回 null 由调用方降级，缓存失败后允许重试
  function loadFzstdScript() {
    if (window.fzstd && typeof window.fzstd.decompress === 'function') {
      return Promise.resolve(window.fzstd);
    }
    if (fzstdLoadPromise) return fzstdLoadPromise;
    fzstdLoadPromise = new Promise((resolve) => {
      const script = document.createElement('script');
      script.src = FZSTD_URL;
      script.async = true;
      script.setAttribute('data-fzstd-loader', '1');
      let settled = false;
      const finish = (result) => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        script.onload = null;
        script.onerror = null;
        if (!result) fzstdLoadPromise = null;
        resolve(result);
      };
      const timer = setTimeout(() => {
        if (script.parentNode) script.parentNode.removeChild(script);
        finish(null);
      }, TIMEOUT.fzstd);
      script.onload = () => finish(window.fzstd && typeof window.fzstd.decompress === 'function' ? window.fzstd : null);
      script.onerror = () => finish(null);
      document.head.appendChild(script);
    });
    return fzstdLoadPromise;
  }

  // 下载单个 zst 源 + 流式真实进度 + 字节数校验
  async function fetchZstBytes(url, parentSignal, onProgress) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), TIMEOUT.zst);
    const onParentAbort = () => controller.abort();
    if (parentSignal) {
      if (parentSignal.aborted) controller.abort();
      else parentSignal.addEventListener('abort', onParentAbort, { once: true });
    }
    const cleanup = () => {
      clearTimeout(timer);
      if (parentSignal) parentSignal.removeEventListener('abort', onParentAbort);
    };

    try {
      const resp = await fetch(url, { cache: 'no-store', signal: controller.signal });
      if (!resp.ok) throw new Error('HTTP ' + resp.status);

      let buf;
      if (resp.body && typeof resp.body.getReader === 'function') {
        const reader = resp.body.getReader();
        const chunks = [];
        let loaded = 0;
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          chunks.push(value);
          loaded += value.length;
          if (onProgress) onProgress(loaded);
        }
        const merged = new Uint8Array(loaded);
        let offset = 0;
        for (const chunk of chunks) {
          merged.set(chunk, offset);
          offset += chunk.length;
        }
        buf = merged.buffer;
      } else {
        buf = await resp.arrayBuffer();
        if (onProgress) onProgress(buf.byteLength);
      }
      if (!buf || buf.byteLength < MIN_ZST_BYTES) throw new Error('zst too small');
      return buf;
    } finally {
      cleanup();
    }
  }

  // 同步解压（阻塞主线程）：调用前必须已 setStage(decompress) 且经过双 rAF
  function decompressZst(arrayBuffer) {
    const fzstd = window.fzstd;
    if (!fzstd || typeof fzstd.decompress !== 'function') throw new Error('fzstd unavailable');
    const uint8 = fzstd.decompress(new Uint8Array(arrayBuffer));
    if (!uint8 || !uint8.length) throw new Error('decompress empty');
    return new TextDecoder('utf-8').decode(uint8);
  }

  // 单条字段适配：name_* -> chs/cht/jp/en/rome；核心字段缺失逐条回退，不整条丢弃
  function adaptZstItem(raw, index) {
    const rawPath = typeof raw.path === 'string' ? raw.path : '';
    const path = rawPath.replace(/\\/g, '/') || `unknown_${index + 1}`;
    const rawTime = typeof raw.time === 'string' ? raw.time : '';
    const time = rawTime.includes('T') ? rawTime.split('T')[0] : rawTime;
    const tail = path.split('/').filter(Boolean).pop() || '';
    const chs = raw.name_chs || raw.name_jp || raw.name_en || tail || `unknown_${index + 1}`;
    return {
      id: index + 1,
      time,
      chs,
      cht: raw.name_cht || '',
      jp: raw.name_jp || '',
      en: raw.name_en || '',
      rome: raw.name_rome || '',
      type: raw.type || 'animation',
      source: raw.source || 'BD',
      sub_name: raw.sub_name || '字幕组',
      extension: raw.extension || '.ass',
      path,
      providers: raw.providers,
      desc: raw.desc
    };
  }

  // JSON 解析 + 结构/质量校验 + 字段适配 + R18 过滤；返回 null 表示需降级
  function parseZstData(jsonText) {
    let rawList;
    try {
      rawList = JSON.parse(jsonText);
    } catch (e) {
      return null;
    }
    if (!Array.isArray(rawList) || rawList.length < MIN_VALID_COUNT) return null;

    let missing = 0;
    for (const raw of rawList) {
      if (!raw || typeof raw !== 'object') { missing++; continue; }
      const pathTail = typeof raw.path === 'string'
        ? raw.path.replace(/\\/g, '/').split('/').filter(Boolean).pop()
        : '';
      const hasChs = !!(raw.name_chs || raw.name_jp || raw.name_en || pathTail);
      const hasPath = typeof raw.path === 'string' && raw.path.trim().length > 0;
      const hasTime = typeof raw.time === 'string' && raw.time.trim().length > 0;
      if (!hasChs || !hasPath || !hasTime) missing++;
    }
    if (missing / rawList.length > MAX_CORE_MISSING_RATIO) return null;

    return rawList
      .map((raw, index) => adaptZstItem(raw || {}, index))
      .filter(item => {
        const t = (item.type || '').toLowerCase();
        const p = (item.path || '').toLowerCase();
        return t !== 'r18' && !p.startsWith('r18');
      });
  }

  // zst 链路：fzstd 脚本与二进制并行预取；主源失败才串行取备源；解压前双 rAF
  async function loadFromZst(gen, parentSignal) {
    setStage(STAGE_TEXT.download);
    const scriptReady = loadFzstdScript();
    let arrayBuffer = null;
    for (const url of ZST_URLS) {
      if (gen !== loadGeneration || parentSignal.aborted) return null;
      try {
        setDownloadProgress(0); // 换源时进度归零，避免上一源的残留宽度
        arrayBuffer = await fetchZstBytes(url, parentSignal, setDownloadProgress);
        if (arrayBuffer) break;
      } catch (e) {
        arrayBuffer = null;
      }
    }
    if (!arrayBuffer) return null;

    const fzstd = await scriptReady;
    if (!fzstd || gen !== loadGeneration || parentSignal.aborted) return null;

    setStage(STAGE_TEXT.decompress);
    await afterTwoFrames();
    if (gen !== loadGeneration || parentSignal.aborted) return null;

    let jsonText;
    try {
      jsonText = decompressZst(arrayBuffer);
    } catch (e) {
      return null;
    }
    return parseZstData(jsonText);
  }

  // 降级 1：本地 subtitles.json
  async function loadFromLocalJson(parentSignal) {
    try {
      const resp = await fetchWithTimeout('./subtitles.json', {}, TIMEOUT.localJson, parentSignal);
      if (!resp.ok) return null;
      const list = await resp.json();
      if (!Array.isArray(list) || list.length === 0) return null;
      return list;
    } catch (e) {
      return null;
    }
  }

  // 降级 2：XML 三源（复用现有 fetchAndParseXml，追加超时与全局护栏信号）
  async function loadFromXml(parentSignal) {
    const list = await fetchAndParseXml(parentSignal);
    return list && list.length > 0 ? list : null;
  }

  // 种子封面并行预取；失败静默，返回空对象
  function fetchSeedCovers(parentSignal) {
    return fetchWithTimeout('./seed_covers.json', {}, TIMEOUT.seed, parentSignal)
      .then(resp => resp.ok ? resp.json() : {})
      .catch(() => ({}));
  }

  // 加载前重置数据态；保留 filters / viewMode / theme / seedCovers
  function resetLoadState() {
    state.rawList = [];
    state.animeList = [];
    state.filteredAnime = [];
    state.filteredFlat = [];
    state.recentPaths = new Set();
    state.latestCommit = null;
    state.pagination.page = 1;
  }

  // 失败空态：复用 .empty-state 结构 + 原生重试按钮（键盘可达）
  function renderLoadFailure() {
    const grid = $('#animeGrid');
    if (grid) {
      grid.innerHTML = `
        <div class="empty-state">
          <div class="empty-icon"><svg class="icon" viewBox="0 0 24 24"><use href="#i-db"/></svg></div>
          <h3>暂时无法加载数据</h3>
          <p>请检查网络后重试，或稍后再来</p>
          <button class="btn btn--ghost" type="button" id="retryLoadBtn">重试加载</button>
        </div>
      `;
      $('#retryLoadBtn')?.addEventListener('click', () => { loadData(); });
    }
    const pagination = $('#pagination');
    if (pagination) pagination.hidden = true;
    showToast('字幕数据加载失败，请稍后重试', false, TOAST_FAILURE_MS, true);
  }

  // 成功：统计区幂等增强「已载入 N 条字幕」，数字用 strong 继承 --accent，不破坏原三个统计位
  function showLoadedStats(count) {
    const nav = $('#navStats');
    if (!nav) return;
    let hint = $('#loadedStatsHint');
    if (!hint) {
      const sep = document.createElement('span');
      sep.setAttribute('aria-hidden', 'true');
      sep.textContent = '·';
      hint = document.createElement('span');
      hint.id = 'loadedStatsHint';
      hint.appendChild(document.createTextNode('已载入 '));
      hint.appendChild(document.createElement('strong'));
      hint.appendChild(document.createTextNode(' 条字幕'));
      nav.appendChild(sep);
      nav.appendChild(hint);
    }
    const num = $('strong', hint);
    if (num) num.textContent = Number(count).toLocaleString();
  }

  async function loadData() {
    if (isLoading) return;
    isLoading = true;
    const gen = ++loadGeneration;
    const guard = new AbortController();
    activeGuard = guard;
    const guardTimer = setTimeout(() => guard.abort(), TIMEOUT.global);

    try {
      resetLoadState();
      showLoading(true, STAGE_TEXT.init);
      const seedPromise = fetchSeedCovers(guard.signal);

      let dataList = null;

      // 1. zst（jsDelivr 主 -> raw 备，串行），fzstd 并行预取
      if (!guard.signal.aborted) {
        try {
          dataList = await loadFromZst(gen, guard.signal);
        } catch (e) {
          dataList = null;
        }
      }
      if (gen !== loadGeneration) return;

      // 2. 降级本地 subtitles.json
      if (!dataList && !guard.signal.aborted) {
        setStage(STAGE_TEXT.local);
        dataList = await loadFromLocalJson(guard.signal);
        if (dataList) showToast('已自动切换数据源，不影响使用', true, TOAST_DEGRADE_MS);
      }
      if (gen !== loadGeneration) return;

      // 3. 降级 XML 三源
      if (!dataList && !guard.signal.aborted) {
        setStage(STAGE_TEXT.xml);
        dataList = await loadFromXml(guard.signal);
        if (dataList) showToast('已自动切换数据源，不影响使用', true, TOAST_DEGRADE_MS);
      }
      if (gen !== loadGeneration) return;

      // 4. 全部失败
      if (!dataList || dataList.length === 0) {
        renderLoadFailure();
        return;
      }

      // 5. 成功整理：先更新文案并双 rAF，再做同步过滤/聚合/渲染
      setStage(STAGE_TEXT.finalize);
      await afterTwoFrames();
      if (gen !== loadGeneration) return;

      state.seedCovers = await seedPromise;
      if (gen !== loadGeneration) return;

      state.rawList = dataList.filter(item => {
        const t = (item.type || '').toLowerCase();
        const p = (item.path || '').toLowerCase();
        return t !== 'r18' && !p.startsWith('r18');
      });

      buildAnimeAggregation();
      fetchRecentCommits();
      applyFiltersAndSort(); // 内部调用 renderCurrentPage，保留用户搜索词后再渲染
      renderStats();
      showLoadedStats(state.rawList.length);
      // 成功不弹 toast，条数在统计区以 --accent 数字呈现
    } catch (e) {
      if (gen === loadGeneration) renderLoadFailure();
    } finally {
      clearTimeout(guardTimer);
      if (gen === loadGeneration) {
        activeGuard = null;
        setStage('');
        showLoading(false);
        isLoading = false;
      }
    }
  }

  async function fetchAndParseXml(parentSignal) {
    const urls = [
      './Subtitles%20DataBase/Files/db.xml',
      'https://cdn.jsdelivr.net/gh/foxofice/sub_share@master/Subtitles%20DataBase/Files/db.xml',
      'https://raw.githubusercontent.com/foxofice/sub_share/master/Subtitles%20DataBase/Files/db.xml'
    ];

    for (const url of urls) {
      if (parentSignal && parentSignal.aborted) break;
      try {
        const resp = await fetchWithTimeout(url, {}, TIMEOUT.xml, parentSignal);
        if (!resp.ok) continue;
        const xmlText = await resp.text();
        const parsed = parseXmlData(xmlText);
        if (parsed.length > 0) return parsed;
      } catch (e) {}
    }
    return [];
  }

  function parseXmlData(xmlText) {
    const parser = new DOMParser();
    const xmlDoc = parser.parseFromString(xmlText, 'text/xml');
    const subsNodes = xmlDoc.getElementsByTagName('subs');
    const result = [];

    for (let i = 0; i < subsNodes.length; i++) {
      const node = subsNodes[i];
      const getVal = (tag) => {
        const el = node.getElementsByTagName(tag)[0];
        return el ? el.textContent.trim() : '';
      };
      const type = getVal('type') || 'animation';
      const rawPath = getVal('path');

      // 剔除 R18
      if (type.toLowerCase() === 'r18' || rawPath.toLowerCase().startsWith('r18')) {
        continue;
      }

      result.push({
        id: i + 1,
        time: (getVal('time') || '').split('T')[0],
        chs: getVal('name_chs'),
        cht: getVal('name_cht'),
        jp: getVal('name_jp'),
        en: getVal('name_en'),
        rome: getVal('name_rome'),
        type: type,
        source: getVal('source') || 'BD',
        sub_name: getVal('sub_name') || '字幕组',
        extension: getVal('extension') || '.ass',
        path: rawPath.replace(/\\/g, '/')
      });
    }
    return result;
  }

  // =============== 数据聚合（把同番剧多字幕组聚合到 1 个卡片） ===============
  function buildAnimeAggregation() {
    const map = new Map();

    state.rawList.forEach((item, index) => {
      const p = item.path || `unknown_${index}`;
      if (!map.has(p)) {
        const yearMatch = (item.time || '').match(/^(\d{4})/);
        const year = yearMatch ? parseInt(yearMatch[1], 10) : 0;
        map.set(p, {
          path: p,
          chs: item.chs,
          cht: item.cht,
          jp: item.jp,
          en: item.en,
          rome: item.rome,
          type: item.type || 'animation',
          time: item.time,
          year: year,
          orderIndex: index,
          versions: [],
          sources: new Set(),
          extensions: new Set(),
          subNames: new Set()
        });
      }

      const anime = map.get(p);
      anime.versions.push({
        id: item.id,
        sub_name: item.sub_name,
        source: item.source,
        extension: item.extension,
        time: item.time
      });
      if (item.source) anime.sources.add(item.source);
      if (item.extension) anime.extensions.add(item.extension);
      if (item.sub_name) anime.subNames.add(item.sub_name);
    });

    state.animeList = Array.from(map.values());
  }

  // =============== GitHub API Commits 实时检测 ===============
  async function fetchRecentCommits() {
    try {
      const headers = { 'User-Agent': 'foxofice-subshare' };
      if (state.githubToken) {
        headers['Authorization'] = `token ${state.githubToken}`;
      }
      const resp = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/commits?per_page=10`, { headers });
      if (!resp.ok) return;
      const commits = await resp.json();
      if (!commits || !commits.length) return;

      state.latestCommit = commits[0];
      renderCommitBanner(commits[0]);

      // 将最新入库的 80 部番剧标记为最新更新
      const totalCount = state.animeList.length;
      state.animeList.slice(Math.max(0, totalCount - 80)).forEach(a => {
        state.recentPaths.add(a.path);
      });
    } catch (e) {
      console.warn('Commits warning:', e);
    }
  }

  function renderCommitBanner(commit) {
    const banner = $('#commitBanner');
    if (!banner || !commit) return;
    const authorName = commit.commit.author.name || 'Maintainer';
    const msg = (commit.commit.message || '').split('\n')[0];
    const dateStr = formatDateRel(commit.commit.author.date);
    const sha = commit.sha.slice(0, 7);

    banner.innerHTML = `
      <div class="commit-pill">
        <span class="commit-dot" aria-hidden="true"></span>
        <span class="commit-tag">最新同步</span>
        <a class="commit-msg" href="${commit.html_url}" target="_blank" rel="noopener noreferrer" title="${msg}">
          ${escapeHtml(msg)}
        </a>
        <span class="commit-time">(${dateStr} by ${escapeHtml(authorName)})</span>
        <a class="commit-sha" href="${commit.html_url}" target="_blank" rel="noopener noreferrer">${sha}</a>
      </div>
    `;
    banner.hidden = false;
  }

  // =============== 筛选与排序引擎 ===============
  function applyFiltersAndSort() {
    const kw = state.filters.keyword.trim().toLowerCase();
    const keywords = kw ? kw.split(/\s+/).filter(Boolean) : [];
    const typeFilter = state.filters.type;
    const srcFilter = state.filters.source;
    const extFilter = state.filters.ext;

    // 1. 过滤番剧 (Grouped)
    state.filteredAnime = state.animeList.filter(anime => {
      if (typeFilter !== 'all') {
        if (anime.type.toLowerCase() !== typeFilter.toLowerCase()) return false;
      }
      if (srcFilter !== 'all') {
        const hasSrc = Array.from(anime.sources).some(s => s.toLowerCase().includes(srcFilter.toLowerCase()));
        if (!hasSrc) return false;
      }
      if (extFilter !== 'all') {
        const hasExt = Array.from(anime.extensions).some(e => e.toLowerCase().includes(extFilter.toLowerCase()));
        if (!hasExt) return false;
      }
      if (keywords.length > 0) {
        const searchCorpus = [
          anime.chs, anime.cht, anime.jp, anime.en, anime.rome, anime.path,
          Array.from(anime.subNames).join(' '),
          Array.from(anime.sources).join(' ')
        ].join(' ').toLowerCase();

        for (const word of keywords) {
          if (!searchCorpus.includes(word)) return false;
        }
      }
      return true;
    });

    sortAnimeList(state.filteredAnime, state.sortBy);

    // 2. 过滤单条字幕 (Flat)
    if (state.viewMode === 'flat') {
      state.filteredFlat = state.rawList.filter(item => {
        if (typeFilter !== 'all') {
          if ((item.type || '').toLowerCase() !== typeFilter.toLowerCase()) return false;
        }
        if (srcFilter !== 'all') {
          if (!(item.source || '').toLowerCase().includes(srcFilter.toLowerCase())) return false;
        }
        if (extFilter !== 'all') {
          if (!(item.extension || '').toLowerCase().includes(extFilter.toLowerCase())) return false;
        }
        if (keywords.length > 0) {
          const searchCorpus = [
            item.chs, item.cht, item.jp, item.en, item.rome, item.sub_name, item.source, item.extension, item.path
          ].join(' ').toLowerCase();
          for (const word of keywords) {
            if (!searchCorpus.includes(word)) return false;
          }
        }
        return true;
      });
      sortFlatList(state.filteredFlat, state.sortBy);
    }

    state.pagination.page = 1;
    renderCurrentPage();
  }

  function sortAnimeList(list, sortBy) {
    if (sortBy === 'recent') {
      list.sort((a, b) => {
        const aRecent = state.recentPaths.has(a.path) ? 1 : 0;
        const bRecent = state.recentPaths.has(b.path) ? 1 : 0;
        if (aRecent !== bRecent) return bRecent - aRecent;
        return b.orderIndex - a.orderIndex;
      });
    } else if (sortBy === 'air_desc') {
      list.sort((a, b) => (b.time || '').localeCompare(a.time || ''));
    } else if (sortBy === 'air_asc') {
      list.sort((a, b) => (a.time || '').localeCompare(b.time || ''));
    } else if (sortBy === 'title_asc') {
      list.sort((a, b) => (a.chs || '').localeCompare(b.chs || '', 'zh-Hans-CN'));
    }
  }

  function sortFlatList(list, sortBy) {
    if (sortBy === 'recent') {
      list.sort((a, b) => b.id - a.id);
    } else if (sortBy === 'air_desc') {
      list.sort((a, b) => (b.time || '').localeCompare(a.time || ''));
    } else if (sortBy === 'air_asc') {
      list.sort((a, b) => (a.time || '').localeCompare(b.time || ''));
    } else if (sortBy === 'title_asc') {
      list.sort((a, b) => (a.chs || '').localeCompare(b.chs || '', 'zh-Hans-CN'));
    }
  }

  // =============== 视图渲染 ===============
  function renderCurrentPage() {
    const isGroup = state.viewMode === 'group';
    const dataset = isGroup ? state.filteredAnime : state.filteredFlat;
    const total = dataset.length;
    state.pagination.total = total;

    const page = state.pagination.page;
    const pageSize = state.pagination.pageSize;
    const startIdx = (page - 1) * pageSize;
    const endIdx = Math.min(startIdx + pageSize, total);
    const slice = dataset.slice(startIdx, endIdx);

    const grid = $('#animeGrid');
    if (!grid) return;

    if (total === 0) {
      grid.innerHTML = `
        <div class="empty-state">
          <div class="empty-icon"><svg class="icon" viewBox="0 0 24 24"><use href="#i-search"/></svg></div>
          <h3>未找到匹配的字幕记录</h3>
          <p>尝试减少检索词，或清除分类与来源筛选条件</p>
          <button class="btn btn--ghost" type="button" id="resetFilterBtn">清空所有筛选</button>
        </div>
      `;
      $('#resetFilterBtn')?.addEventListener('click', resetAllFilters);
      renderPagination(0, 0, 0);
      return;
    }

    const frag = document.createDocumentFragment();

    if (isGroup) {
      slice.forEach(anime => {
        frag.appendChild(createAnimeCard(anime));
      });
    } else {
      slice.forEach(item => {
        frag.appendChild(createFlatCard(item));
      });
    }

    grid.innerHTML = '';
    grid.appendChild(frag);

    // 对尚未配置本地封面的卡片开启视口探测
    $$('.anime-card:not(.has-cover)', grid).forEach(card => scraper.observeCard(card));

    renderPagination(page, pageSize, total);
  }

  function createAnimeCard(anime) {
    const card = document.createElement('article');
    card.className = 'anime-card';
    card.dataset.path = anime.path;
    card.dataset.chs = anime.chs;
    card.dataset.jp = anime.jp;

    const isRecent = state.recentPaths.has(anime.path);
    const repoUrl = getRepoFolderUrl(anime.path);
    const firstLetter = (anime.chs || anime.jp || '?')[0].toUpperCase();
    const verCount = anime.versions.length;
    const subGroups = Array.from(anime.subNames).slice(0, 2).join(' / ');
    const yearStr = anime.year > 0 ? anime.year : (anime.time || '').slice(0, 4);

    // 核心优化：直接检查是否已有预置种子封面（无需等待异步请求即可直出封面）
    const seedMeta = findSeedMeta(anime.path);
    const initialCoverUrl = getSafeCoverUrl(seedMeta);

    let thumbHtml = '';
    if (initialCoverUrl) {
      card.classList.add('has-cover');
      thumbHtml = `
        <img class="poster-img"
             src="${initialCoverUrl}"
             data-raw-url="${seedMeta.cover || ''}"
             data-local-cover="${seedMeta.localCover || ''}"
             alt="${escapeHtml(anime.chs || anime.jp)}"
             loading="lazy"
             referrerpolicy="no-referrer"
             onerror="handleCoverError(this, this.dataset.rawUrl)"
        >
      `;
    } else {
      thumbHtml = `
        <div class="poster-placeholder">
          <span class="pl-letter">${escapeHtml(firstLetter)}</span>
          <span class="pl-title">${escapeHtml(anime.chs || anime.jp)}</span>
        </div>
      `;
    }

    card.innerHTML = `
      <div class="poster-box">
        <div class="poster-thumb" data-letter="${escapeHtml(firstLetter)}">
          ${thumbHtml}
        </div>
        <div class="poster-badges">
          ${isRecent ? '<span class="badge badge--new">🔥 最新</span>' : ''}
          ${anime.type === 'movie' ? '<span class="badge badge--gold">电影</span>' : ''}
          <span class="badge badge--dark">${yearStr || '经典'}</span>
        </div>
        <span class="poster-score" ${seedMeta && seedMeta.score ? '' : 'hidden'}>
          ${seedMeta && seedMeta.score ? '★ ' + seedMeta.score : ''}
        </span>
      </div>

      <div class="anime-body">
        <h3 class="anime-title" title="${escapeHtml(anime.chs || anime.jp)}">
          ${highlightKeyword(anime.chs || anime.jp, state.filters.keyword)}
        </h3>
        <p class="anime-jp" title="${escapeHtml(anime.jp || anime.en || '')}">
          ${escapeHtml(anime.jp || anime.en || '—')}
        </p>

        <div class="anime-chips">
          <span class="chip chip--count" title="${escapeHtml(Array.from(anime.subNames).join(', '))}">
            <svg class="icon"><use href="#i-db"/></svg> ${verCount}套
          </span>
          ${Array.from(anime.sources).slice(0, 2).map(s => `<span class="chip chip--source">${escapeHtml(s)}</span>`).join('')}
          ${Array.from(anime.extensions).slice(0, 2).map(e => `<span class="chip chip--ext">${escapeHtml(e.replace('.', ''))}</span>`).join('')}
        </div>

        <div class="anime-actions">
          <a class="btn btn--primary btn--sm action-gh" href="${repoUrl}" target="_blank" rel="noopener noreferrer" title="前往 GitHub 仓库目录">
            <svg class="icon"><use href="#i-external"/></svg> 仓库
          </a>
          <button class="btn btn--ghost btn--sm action-detail" type="button" title="查看所有字幕与详情">
            详情
          </button>
        </div>
      </div>
    `;

    $('.action-detail', card)?.addEventListener('click', (e) => {
      e.stopPropagation();
      openDetailModal(anime);
    });

    card.addEventListener('click', (e) => {
      if (!e.target.closest('a, button')) {
        openDetailModal(anime);
      }
    });

    return card;
  }

  function createFlatCard(item) {
    const card = document.createElement('article');
    card.className = 'anime-card anime-card--flat';
    card.dataset.path = item.path;
    card.dataset.chs = item.chs;
    card.dataset.jp = item.jp;

    const repoUrl = getRepoFolderUrl(item.path);

    card.innerHTML = `
      <div class="anime-body">
        <div class="flat-header">
          <span class="chip chip--source">${escapeHtml(item.source || 'BD')}</span>
          <span class="chip chip--ext">${escapeHtml(item.extension || '.ass')}</span>
          <span class="flat-time">${escapeHtml(item.time || '')}</span>
        </div>
        <h3 class="anime-title" title="${escapeHtml(item.chs)}">
          ${highlightKeyword(item.chs, state.filters.keyword)}
        </h3>
        <p class="anime-jp">${escapeHtml(item.jp || item.en || '')}</p>
        <p class="flat-sub">字幕制作：<b>${escapeHtml(item.sub_name || '默认')}</b></p>
        <p class="flat-path" title="${escapeHtml(item.path)}"><code>${escapeHtml(item.path)}</code></p>
        <div class="anime-actions">
          <a class="btn btn--primary btn--sm" href="${repoUrl}" target="_blank" rel="noopener noreferrer">
            <svg class="icon"><use href="#i-external"/></svg> 打开目录
          </a>
          <button class="btn btn--ghost btn--sm btn--copy-path" type="button" data-path="${escapeHtml(item.path)}">
            <svg class="icon"><use href="#i-copy"/></svg> 复制路径
          </button>
        </div>
      </div>
    `;

    $('.btn--copy-path', card)?.addEventListener('click', (e) => {
      e.stopPropagation();
      copyToClipboard(item.path, '已复制相对路径到剪贴板');
    });

    return card;
  }

  // =============== 分页控件 ===============
  
  function renderPagination(page, pageSize, total) {
    const container = $('#pagination');
    if (!container) return;

    if (total <= pageSize && pageSize === 60) {
      container.innerHTML = `<div class="pagination-summary">· 已显示全部 ${total} 部番剧 ·</div>`;
      container.hidden = false;
      return;
    }

    const totalPages = Math.ceil(total / pageSize);
    let html = '';

    html += `<div class="pagination-summary">
      <span>第 <strong>${page}</strong> / <strong>${totalPages}</strong> 页</span>
      <span>·</span>
      <span>共 <strong>${total}</strong> 部</span>
      <span>·</span>
      <span class="page-size-selector">
        每页
        <button type="button" class="page-size-btn ${pageSize === 30 ? 'is-active' : ''}" data-size="30">30</button>
        <button type="button" class="page-size-btn ${pageSize === 60 ? 'is-active' : ''}" data-size="60">60</button>
        <button type="button" class="page-size-btn ${pageSize === 120 ? 'is-active' : ''}" data-size="120">120</button>
      </span>
    </div>`;

    html += `<div class="pagination-controls">`;
    html += `<button class="page-btn" ${page <= 1 ? 'disabled' : ''} data-page="${page - 1}">‹ 上一页</button>`;

    let pages = [];
    if (totalPages <= 7) {
      for (let i = 1; i <= totalPages; i++) pages.push(i);
    } else {
      if (page <= 4) {
        pages = [1, 2, 3, 4, 5, '...', totalPages];
      } else if (page >= totalPages - 3) {
        pages = [1, '...', totalPages - 4, totalPages - 3, totalPages - 2, totalPages - 1, totalPages];
      } else {
        pages = [1, '...', page - 1, page, page + 1, '...', totalPages];
      }
    }

    pages.forEach(p => {
      if (p === '...') {
        html += `<span class="page-ellipsis">…</span>`;
      } else {
        html += `<button class="page-btn ${p === page ? 'is-active' : ''}" data-page="${p}">${p}</button>`;
      }
    });

    html += `<button class="page-btn" ${page >= totalPages ? 'disabled' : ''} data-page="${page + 1}">下一页 ›</button>`;
    html += `</div>`;

    container.innerHTML = html;
    container.hidden = false;

    $$('.page-btn[data-page]', container).forEach(btn => {
      btn.addEventListener('click', () => {
        const p = parseInt(btn.dataset.page, 10);
        if (p >= 1 && p <= totalPages && p !== state.pagination.page) {
          state.pagination.page = p;
          renderCurrentPage();
          window.scrollTo({ top: $('#searchSection').offsetTop - 70, behavior: 'smooth' });
        }
      });
    });

    $$('.page-size-btn[data-size]', container).forEach(btn => {
      btn.addEventListener('click', () => {
        const sz = parseInt(btn.dataset.size, 10);
        if (sz && sz !== state.pagination.pageSize) {
          state.pagination.pageSize = sz;
          state.pagination.page = 1;
          renderCurrentPage();
          window.scrollTo({ top: $('#searchSection').offsetTop - 70, behavior: 'smooth' });
        }
      });
    });
  }

  // =============== 详情弹窗 (Modal) ===============
  async function openDetailModal(anime) {
    const modal = $('#detailModal');
    if (!modal) return;

    const repoUrl = getRepoFolderUrl(anime.path);
    const gitSparseCmd = `git clone --filter=blob:none --sparse https://github.com/${GITHUB_REPO}.git && cd sub_share && git sparse-checkout set "subs_list/${anime.path.replace(/[\\/]+/g, "/")}"`;

    $('#modalTitle').textContent = anime.chs || anime.jp;
    $('#modalJpTitle').textContent = anime.jp ? `日文名：${anime.jp}` : '';
    $('#modalPathCode').textContent = `subs_list/${anime.path}`;
    $('#modalGhLink').href = repoUrl;
    $('#modalCopyPathBtn').setAttribute('data-copy', anime.path);
    $('#modalCopyCmdBtn').setAttribute('data-copy', gitSparseCmd);

    const listBody = $('#modalVersionList');
    listBody.innerHTML = anime.versions.map(v => `
      <tr>
        <td><b>${escapeHtml(v.sub_name || '默认')}</b></td>
        <td><span class="chip chip--source">${escapeHtml(v.source || 'BD')}</span></td>
        <td><span class="chip chip--ext">${escapeHtml(v.extension || '.ass')}</span></td>
        <td>
          <a class="text-link" href="${repoUrl}" target="_blank" rel="noopener noreferrer">
            打开目录 <svg class="icon"><use href="#i-external"/></svg>
          </a>
        </td>
      </tr>
    `).join('');

    const cleanSearchTitle = (anime.chs || anime.jp || '').split('/')[0].trim();
    $('#modalBgmLink').href = `https://bgm.tv/subject_search/${encodeURIComponent(cleanSearchTitle)}?cat=2`;
    $('#modalBiliLink').href = `https://search.bilibili.com/all?keyword=${encodeURIComponent(cleanSearchTitle)}`;
    $('#modalDoubanLink').href = `https://www.douban.com/search?cat=1002&q=${encodeURIComponent(cleanSearchTitle)}`;
    $('#modalTmdbLink').href = `https://www.themoviedb.org/search?query=${encodeURIComponent(cleanSearchTitle)}`;

    const modalCoverImg = $('#modalCover');
    modalCoverImg.src = '';
    modalCoverImg.hidden = true;
    $('#modalSummary').textContent = '正在获取番剧数据...';
    $('#modalRating').hidden = true;

    modal.hidden = false;
    document.body.style.overflow = 'hidden';

    const detailGen = ++detailLoadGeneration;
    let meta = await getFromCache(anime.path) || await scraper.fetchFromBangumi(anime.chs, anime.jp);
    if (detailGen !== detailLoadGeneration) return; // 用户已切到别的番剧，丢弃过期结果

    if (meta && !meta.summary && meta.id) {
      // 简介缺失但已有 Bangumi id：同步补全（详情接口），保证本次打开就能看到简介
      const summary = await scraper.fetchSubjectSummary(meta.id);
      if (detailGen !== detailLoadGeneration) return;
      if (summary) {
        meta.summary = summary;
        saveToCache(anime.path, meta); // 写回缓存，下次秒开
      }
    }

    if (meta) {
      const safeUrl = getSafeCoverUrl(meta);
      if (safeUrl) {
        modalCoverImg.dataset.rawUrl = meta.cover || '';
        modalCoverImg.dataset.localCover = meta.localCover || '';
        modalCoverImg.dataset.failStep = '0';
        modalCoverImg.onerror = function () { window.handleCoverError(this, this.dataset.rawUrl); };
        modalCoverImg.src = safeUrl;
        modalCoverImg.hidden = false;
      }
      if (meta.summary) {
        $('#modalSummary').textContent = meta.summary;
      } else {
        $('#modalSummary').textContent = '收录于《字幕共享计划》数据库，暂无附加简介。';
      }
      if (meta.score) {
        $('#modalRating').textContent = `Bangumi 评分：★ ${meta.score}`;
        $('#modalRating').hidden = false;
      }
      if (meta.url) {
        $('#modalBgmLink').href = meta.url;
      }
    } else {
      $('#modalSummary').textContent = '收录于《字幕共享计划》数据库，点击上方链接可在各平台查看番剧详情。';
    }
  }

  function closeDetailModal() {
    const modal = $('#detailModal');
    if (!modal) return;
    modal.hidden = true;
    document.body.style.overflow = '';
  }

  // =============== 设置弹窗 ===============
  function openSettingsModal() {
    const modal = $('#settingsModal');
    if (!modal) return;
    $('#tokenInput').value = state.githubToken;
    modal.hidden = false;
    document.body.style.overflow = 'hidden';
  }

  function closeSettingsModal() {
    const modal = $('#settingsModal');
    if (!modal) return;
    modal.hidden = true;
    document.body.style.overflow = '';
  }

  // =============== 辅助与交互 ===============
  function renderStats() {
    const statsTotal = $('#statTotalSubs');
    const statsAnime = $('#statTotalAnime');
    const statsUpdate = $('#statLastUpdate');
    if (statsTotal) statsTotal.textContent = state.rawList.length.toLocaleString();
    if (statsAnime) statsAnime.textContent = state.animeList.length.toLocaleString();
    if (statsUpdate && state.rawList.length > 0) {
      statsUpdate.textContent = state.rawList[state.rawList.length - 1].time || '2026';
    }
  }

  function highlightKeyword(text, keyword) {
    if (!keyword || !text) return escapeHtml(text || '');
    const escaped = escapeHtml(text);
    const kw = escapeHtml(keyword.trim().split(/\s+/)[0]);
    if (!kw) return escaped;
    const regex = new RegExp(`(${escapeRegex(kw)})`, 'gi');
    return escaped.replace(regex, '<mark class="highlight">$1</mark>');
  }

  function escapeHtml(str) {
    if (!str) return '';
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  function escapeRegex(str) {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  function formatDateRel(dateStr) {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      const diff = Date.now() - d.getTime();
      const hours = Math.floor(diff / (1000 * 60 * 60));
      if (hours < 1) return '刚刚';
      if (hours < 24) return `${hours} 小时前`;
      const days = Math.floor(hours / 24);
      if (days < 30) return `${days} 天前`;
      return d.toISOString().split('T')[0];
    } catch (e) {
      return dateStr;
    }
  }

  function copyToClipboard(text, successMsg = '已复制到剪贴板') {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(() => {
        showToast(successMsg, true);
      }).catch(() => fallbackCopy(text, successMsg));
    } else {
      fallbackCopy(text, successMsg);
    }
  }

  function fallbackCopy(text, msg) {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    try {
      document.execCommand('copy');
      showToast(msg, true);
    } catch (e) {
      showToast('复制失败，请手动选择复制', false);
    }
    document.body.removeChild(ta);
  }

  let toastTimer = null;
  // ok 决定错误样式，duration 可选（默认 2000ms 不破坏既有调用），urgent 临时升级 assertive
  function showToast(msg, ok = true, duration = 2000, urgent = false) {
    const toast = $('#toast');
    if (!toast) return;
    toast.textContent = msg;
    toast.classList.toggle('toast--error', !ok);
    if (urgent) {
      toast.setAttribute('role', 'alert');
      toast.setAttribute('aria-live', 'assertive');
    } else {
      toast.setAttribute('role', 'status');
      toast.setAttribute('aria-live', 'polite');
    }
    toast.classList.add('show');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast.classList.remove('show');
      toast.setAttribute('role', 'status');
      toast.setAttribute('aria-live', 'polite');
    }, duration);
  }

  
  let progressTimer = null;
  // text 参数现在真正生效：阶段文案写入 #mainLoader 的稳定子节点，显示于骨架卡上方
  function showLoading(show, text) {
    const bar = $('#topProgressBar');

    if (show) {
      if (bar) {
        bar.style.opacity = '1';
        bar.style.width = '25%';
        clearInterval(progressTimer);
        let current = 25;
        progressTimer = setInterval(() => {
          if (current < 85) {
            current += (85 - current) * 0.18;
            bar.style.width = current + '%';
          }
        }, 150);
      }
      renderSkeletons(15);
      setStage(text || STAGE_TEXT.init);
    } else {
      clearInterval(progressTimer);
      if (bar) {
        bar.style.width = '100%';
        setTimeout(() => {
          bar.style.opacity = '0';
          setTimeout(() => {
            bar.style.width = '0%';
          }, 350);
        }, 200);
      }
      setStage('');
    }
  }

  // 下载阶段按真实字节更新顶部进度条（已接收字节 / 234497）；首次调用停掉渐进假动画
  function setDownloadProgress(loaded) {
    const bar = $('#topProgressBar');
    if (!bar) return;
    clearInterval(progressTimer);
    const ratio = Math.max(0, Math.min(1, Number(loaded) / ZST_EXPECTED_BYTES));
    bar.style.opacity = '1';
    bar.style.width = (ratio * 100) + '%';
  }

  
  function renderSkeletons(count = 18) {
    const grid = $('#animeGrid');
    if (!grid) return;
    if (state.animeList && state.animeList.length > 0 && grid.children.length > 0 && !grid.querySelector('.skeleton-card')) {
      return;
    }
    const frag = document.createDocumentFragment();
    for (let i = 0; i < count; i++) {
      const card = document.createElement('div');
      card.className = 'skeleton-card';
      card.setAttribute('aria-hidden', 'true');
      card.innerHTML = `
        <div class="skeleton-thumb"></div>
        <div class="skeleton-body">
          <div class="skeleton-line skeleton-line--title"></div>
          <div class="skeleton-line skeleton-line--sub"></div>
          <div class="skeleton-line skeleton-line--chips"></div>
          <div class="skeleton-line skeleton-line--actions"></div>
        </div>
      `;
      frag.appendChild(card);
    }
    grid.innerHTML = '';
    grid.appendChild(frag);
  }

  function resetAllFilters() {
    state.filters.keyword = '';
    state.filters.type = 'all';
    state.filters.source = 'all';
    state.filters.ext = 'all';
    $('#searchInput').value = '';
    $$('.filter-tab').forEach(t => t.classList.toggle('is-active', t.dataset.type === 'all'));
    $('#sourceSelect').value = 'all';
    $('#extSelect').value = 'all';
    applyFiltersAndSort();
  }

  // =============== 主题与初始化 ===============
  function initTheme() {
    const themeBtn = $('#themeBtn');
    let theme = localStorage.getItem(STORAGE_KEY_THEME);
    if (!theme) {
      theme = window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    }
    document.documentElement.dataset.theme = theme;

    if (themeBtn) {
      themeBtn.addEventListener('click', () => {
        const next = document.documentElement.dataset.theme === 'light' ? 'dark' : 'light';
        document.documentElement.dataset.theme = next;
        localStorage.setItem(STORAGE_KEY_THEME, next);
      });
    }
  }

  function bindEvents() {
    let searchDebounce = null;
    const searchInput = $('#searchInput');
    if (searchInput) {
      searchInput.addEventListener('input', (e) => {
        clearTimeout(searchDebounce);
        searchDebounce = setTimeout(() => {
          state.filters.keyword = e.target.value;
          applyFiltersAndSort();
        }, 180);
      });
      window.addEventListener('keydown', (e) => {
        if (e.key === '/' && document.activeElement !== searchInput && !$('#detailModal:not([hidden])')) {
          e.preventDefault();
          searchInput.focus();
        }
      });
    }

    $('#clearSearchBtn')?.addEventListener('click', () => {
      if (searchInput) {
        searchInput.value = '';
        state.filters.keyword = '';
        applyFiltersAndSort();
        searchInput.focus();
      }
    });

    $$('.quick-tag').forEach(tag => {
      tag.addEventListener('click', () => {
        const kw = tag.dataset.kw;
        if (searchInput) {
          searchInput.value = kw;
          state.filters.keyword = kw;
          applyFiltersAndSort();
        }
      });
    });

    $$('.filter-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        $$('.filter-tab').forEach(t => t.classList.remove('is-active'));
        tab.classList.add('is-active');
        state.filters.type = tab.dataset.type;
        applyFiltersAndSort();
      });
    });

    $('#sourceSelect')?.addEventListener('change', (e) => {
      state.filters.source = e.target.value;
      applyFiltersAndSort();
    });

    $('#extSelect')?.addEventListener('change', (e) => {
      state.filters.ext = e.target.value;
      applyFiltersAndSort();
    });

    $('#sortSelect')?.addEventListener('change', (e) => {
      state.sortBy = e.target.value;
      applyFiltersAndSort();
    });

    $$('.view-mode-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const mode = btn.dataset.mode;
        state.viewMode = mode;
        localStorage.setItem(STORAGE_KEY_VIEW, mode);
        $$('.view-mode-btn').forEach(b => b.classList.remove('is-active'));
        btn.classList.add('is-active');
        applyFiltersAndSort();
      });
    });

    document.addEventListener('click', (e) => {
      const copyBtn = e.target.closest('[data-copy]');
      if (copyBtn) {
        const text = copyBtn.getAttribute('data-copy');
        if (text) {
          copyToClipboard(text, copyBtn.dataset.copyMsg || '已复制');
        }
      }
    });

    $$('[data-close-modal]').forEach(btn => {
      btn.addEventListener('click', () => {
        closeDetailModal();
        closeSettingsModal();
      });
    });
    $('.modal-backdrop')?.addEventListener('click', () => {
      closeDetailModal();
      closeSettingsModal();
    });
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        closeDetailModal();
        closeSettingsModal();
      }
    });

    $('#settingsBtn')?.addEventListener('click', openSettingsModal);
    $('#saveTokenBtn')?.addEventListener('click', () => {
      const token = $('#tokenInput').value.trim();
      state.githubToken = token;
      if (token) {
        localStorage.setItem(STORAGE_KEY_TOKEN, token);
        showToast('GitHub Token 已保存到本地浏览器', true);
      } else {
        localStorage.removeItem(STORAGE_KEY_TOKEN);
        showToast('已清除本地 Token', true);
      }
      closeSettingsModal();
      fetchRecentCommits();
    });

    const dropZone = $('#dropZone');
    if (dropZone) {
      dropZone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropZone.classList.add('is-dragover');
      });
      dropZone.addEventListener('dragleave', () => dropZone.classList.remove('is-dragover'));
      dropZone.addEventListener('drop', async (e) => {
        e.preventDefault();
        dropZone.classList.remove('is-dragover');
        if (!e.dataTransfer.files || !e.dataTransfer.files[0]) return;

        // 用户主动导入优先：自增请求序号 + 中断护栏，彻底作废旧 zst 回调
        const wasLoading = isLoading;
        const gen = ++loadGeneration;
        isLoading = false;
        if (activeGuard) {
          activeGuard.abort();
          activeGuard = null;
        }
        if (wasLoading) showLoading(false); else setStage('');

        const file = e.dataTransfer.files[0];
        const name = (file.name || '').toLowerCase();
        const isZst = name.endsWith('.zst') || name.endsWith('.zstd') || name.endsWith('.json.zst');
        const isXml = name.endsWith('.xml');

        if (!isZst && !isXml) {
          showToast('仅支持 db.json.zst 或 db.xml 文件', false);
          return;
        }

        let list = [];
        try {
          if (isZst) {
            // zst：按需加载 fzstd（页面主链路可能未加载），解压后走字段适配
            setStage('正在解压导入文件...');
            const fzstd = await loadFzstdScript();
            if (!fzstd) throw new Error('解压组件加载失败，请检查网络');
            if (gen !== loadGeneration) return;
            const arrayBuffer = await file.arrayBuffer();
            if (gen !== loadGeneration) return;
            const jsonText = decompressZst(arrayBuffer);
            if (gen !== loadGeneration) return;
            // 导入场景放宽条数下限：允许小样本/部分数据集，仅要求非空数组与字段合法
            const rawParsed = JSON.parse(jsonText);
            if (!Array.isArray(rawParsed) || rawParsed.length === 0) {
              throw new Error('文件中没有可导入的字幕条目');
            }
            let missing = 0;
            for (const raw of rawParsed) {
              if (!raw || typeof raw !== 'object') { missing++; continue; }
              const pathTail = typeof raw.path === 'string'
                ? raw.path.replace(/\\/g, '/').split('/').filter(Boolean).pop() : '';
              if (!(raw.name_chs || raw.name_jp || raw.name_en || pathTail) ||
                  !(typeof raw.path === 'string' && raw.path.trim()) ||
                  !(typeof raw.time === 'string' && raw.time.trim())) missing++;
            }
            if (rawParsed.length > 0 && missing / rawParsed.length > MAX_CORE_MISSING_RATIO) {
              throw new Error('文件字段格式不符合字幕数据结构');
            }
            list = rawParsed
              .map((raw, index) => adaptZstItem(raw || {}, index))
              .filter(item => {
                const t = (item.type || '').toLowerCase();
                const p = (item.path || '').toLowerCase();
                return t !== 'r18' && !p.startsWith('r18');
              });
            setStage('');
          } else {
            // xml：沿用原解析逻辑
            const text = await file.text();
            if (gen !== loadGeneration) return;
            list = parseXmlData(text);
          }
        } catch (err) {
          setStage('');
          showToast((isZst ? 'ZST 导入失败：' : 'XML 导入失败：') + err.message, false);
          return;
        }

        if (gen !== loadGeneration) return;
        if (list.length > 0) {
          state.rawList = list;
          buildAnimeAggregation();
          applyFiltersAndSort();
          renderStats();
          showLoadedStats(list.length);
          showToast(`成功本地解析导入 ${list.length} 条字幕记录！`, true);
          closeSettingsModal();
        } else {
          showToast(isZst
            ? '未从该 ZST 文件中解析出有效字幕记录'
            : '无法解析该 XML 文件中的 <subs> 节点', false);
        }
      });
    }

    const backtop = $('#backTop');
    if (backtop) {
      window.addEventListener('scroll', () => {
        backtop.classList.toggle('is-visible', window.scrollY > 400);
      }, { passive: true });
      backtop.addEventListener('click', () => {
        window.scrollTo({ top: 0, behavior: 'smooth' });
      });
    }
  }

  // =============== 启动流程 ===============
  async function init() {
    initTheme();
    bindEvents();
    await initIndexedDB();
    await loadData();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
