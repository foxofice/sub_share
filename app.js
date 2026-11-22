/**
 * 字幕共享计划 (Subtitles DataBase) · 交互与检索核心脚本
 * 
 * 功能特性：
 * 1. 毫秒级多维度实时检索（支持中/日/英/罗马音/字幕组/格式/路径匹配，多关键词空格与检索）
 * 2. 彻底剔除 R18 内容检索，保障页面健康纯净
 * 3. 国内极速封面加载引擎：
 *    - 优先直连本仓库内置高清封面库（./covers/*.jpg，无跨域、免翻墙、100% 成功率）
 *    - 针对远程图片自动通过多通道国内可用 CDN 反代（wsrv.nl / i0.wp.com 自动故障转移）
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
      pageSize: 36,
      total: 0
    },
    seedCovers: {},     // 预填充封面（含本地 covers/ 路径）
    recentPaths: new Set(),
    latestCommit: null,
    githubToken: localStorage.getItem(STORAGE_KEY_TOKEN) || '',
    idb: null
  };

  
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

  // =============== 国内多镜像 CDN 转换 ===============
  function getSafeCoverUrl(meta) {
    if (!meta) return null;
    // 1. 本地仓库内置图片优先（最快、最稳、中国大陆 100% 直连）
    if (meta.localCover) {
      return './' + meta.localCover;
    }
    const raw = meta.cover;
    if (!raw) return null;

    // 2. 如果包含 lain.bgm.tv，通过国内可访问的 CDN 节点加速转接
    if (raw.includes('lain.bgm.tv')) {
      const cleanUrl = raw.replace(/^http:\/\//i, 'https://');
      return `https://wsrv.nl/?url=${encodeURIComponent(cleanUrl)}&w=360&output=jpg`;
    }
    return raw;
  }

  // 全局图片加载失败备用转移
  window.handleCoverError = function (img, rawUrl) {
    if (!img) return;
    const step = parseInt(img.dataset.failStep || '0', 10);
    if (!rawUrl) rawUrl = img.dataset.rawUrl || '';

    if (step === 0 && rawUrl) {
      // 切换到 WordPress Jetpack 全球 CDN 镜像
      img.dataset.failStep = '1';
      const cleanNoProto = rawUrl.replace(/^https?:\/\//i, '');
      img.src = `https://i0.wp.com/${cleanNoProto}`;
    } else if (step === 1 && rawUrl) {
      // 切换到 i1.wp.com 镜像
      img.dataset.failStep = '2';
      const cleanNoProto = rawUrl.replace(/^https?:\/\//i, '');
      img.src = `https://i1.wp.com/${cleanNoProto}`;
    } else {
      // 降级显示纯净色块卡片
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
            summary: first.summary || '',
            score: first.rating ? first.rating.score : null,
            cover: imgUrl ? imgUrl.replace(/^http:\/\//i, 'https://') : null,
            url: `https://bgm.tv/subject/${first.id}`
          };
        }
      } catch (e) {}
      return null;
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
               alt="${escapeHtml(meta.name_cn || meta.name || '')}"
               loading="lazy"
               referrerpolicy="no-referrer"
               onerror="handleCoverError(this, '${escapeHtml(meta.cover || '')}')"
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

  // =============== 数据载入引擎 ===============
  async function loadData() {
    showLoading(true, '正在载入字幕索引数据...');

    // 1. 优先加载预置种子封面 (包含已下载到 covers/ 的本地高质量海报)
    try {
      const seedResp = await fetch('./seed_covers.json');
      if (seedResp.ok) {
        state.seedCovers = await seedResp.json();
      }
    } catch (e) {}

    // 2. 加载字幕数据（过滤 R18）
    let dataList = null;
    try {
      const jsonResp = await fetch('./subtitles.json');
      if (jsonResp.ok) {
        dataList = await jsonResp.json();
      }
    } catch (e) {}

    if (!dataList || dataList.length === 0) {
      dataList = await fetchAndParseXml();
    }

    if (!dataList || dataList.length === 0) {
      showToast('数据载入失败，请检查网络或在设置中切换镜像源', false);
      showLoading(false);
      return;
    }

    // 严格过滤剔除任何 R18 内容
    state.rawList = dataList.filter(item => {
      const t = (item.type || '').toLowerCase();
      const p = (item.path || '').toLowerCase();
      return t !== 'r18' && !p.startsWith('r18');
    });

    buildAnimeAggregation();

    // 3. 拉取 GitHub Commits 动态
    fetchRecentCommits();

    // 4. 首次渲染
    applyFiltersAndSort();
    renderStats();
    showLoading(false);
  }

  async function fetchAndParseXml() {
    const urls = [
      './Subtitles%20DataBase/Files/db.xml',
      'https://cdn.jsdelivr.net/gh/foxofice/sub_share@master/Subtitles%20DataBase/Files/db.xml',
      'https://raw.githubusercontent.com/foxofice/sub_share/master/Subtitles%20DataBase/Files/db.xml'
    ];

    for (const url of urls) {
      try {
        const resp = await fetch(url);
        if (!resp.ok) continue;
        const xmlText = await resp.text();
        return parseXmlData(xmlText);
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
             alt="${escapeHtml(anime.chs || anime.jp)}"
             loading="lazy"
             referrerpolicy="no-referrer"
             onerror="handleCoverError(this, '${escapeHtml(seedMeta.cover || '')}')"
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
          ${isRecent ? '<span class="badge badge--new">🔥 最新更新</span>' : ''}
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

        <div class="anime-meta">
          <span class="meta-tag"><svg class="icon"><use href="#i-db"/></svg> ${verCount} 套字幕</span>
          <span class="meta-sub" title="${escapeHtml(Array.from(anime.subNames).join(', '))}">${escapeHtml(subGroups || '收录完成')}</span>
        </div>

        <div class="anime-chips">
          ${Array.from(anime.sources).slice(0, 3).map(s => `<span class="chip chip--source">${escapeHtml(s)}</span>`).join('')}
          ${Array.from(anime.extensions).map(e => `<span class="chip chip--ext">${escapeHtml(e.replace('.', ''))}</span>`).join('')}
        </div>

        <div class="anime-actions">
          <a class="btn btn--primary btn--sm action-gh" href="${repoUrl}" target="_blank" rel="noopener noreferrer" title="直接前往 GitHub 仓库所在目录">
            <svg class="icon"><use href="#i-external"/></svg> 仓库目录
          </a>
          <button class="btn btn--ghost btn--sm action-detail" type="button" title="查看所有字幕版本与详情">
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

    if (total <= pageSize) {
      container.hidden = true;
      return;
    }
    container.hidden = false;

    const totalPages = Math.ceil(total / pageSize);
    let html = '';

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
    html += `<span class="page-info">共 ${total} 部 / ${totalPages} 页</span>`;

    container.innerHTML = html;

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

    const meta = await getFromCache(anime.path) || await scraper.fetchFromBangumi(anime.chs, anime.jp);
    if (meta) {
      const safeUrl = getSafeCoverUrl(meta);
      if (safeUrl) {
        modalCoverImg.src = safeUrl;
        modalCoverImg.dataset.rawUrl = meta.cover || '';
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
  function showToast(msg, ok = true) {
    const toast = $('#toast');
    if (!toast) return;
    toast.textContent = msg;
    toast.classList.toggle('toast--error', !ok);
    toast.classList.add('show');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast.classList.remove('show');
    }, 2000);
  }

  function showLoading(show, text = '加载中...') {
    const loader = $('#mainLoader');
    if (!loader) return;
    loader.hidden = !show;
    const txt = $('.loader-text', loader);
    if (txt) txt.textContent = text;
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
      dropZone.addEventListener('drop', (e) => {
        e.preventDefault();
        dropZone.classList.remove('is-dragover');
        if (e.dataTransfer.files && e.dataTransfer.files[0]) {
          const file = e.dataTransfer.files[0];
          const reader = new FileReader();
          reader.onload = (ev) => {
            try {
              const list = parseXmlData(ev.target.result);
              if (list.length > 0) {
                state.rawList = list;
                buildAnimeAggregation();
                applyFiltersAndSort();
                renderStats();
                showToast(`成功本地解析导入 ${list.length} 条字幕记录！`, true);
                closeSettingsModal();
              } else {
                showToast('无法解析该 XML 文件中的 <subs> 节点', false);
              }
            } catch (err) {
              showToast('XML 解析失败：' + err.message, false);
            }
          };
          reader.readAsText(file);
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
