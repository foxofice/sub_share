#!/usr/bin/env node
/**
 * sync-db.mjs — 从 subs_list 目录树自动生成/补齐字幕数据库
 *
 * 设计原则：db.xml 是人工维护的“事实源”，本脚本只做增量：
 *   1) 扫描 subs_list（CI 里用 git ls-tree，不下载任何字幕文件）
 *   2) 对 db.xml 里已存在的记录一律不改（保留人工校对过的名称、来源、字幕组）
 *   3) 只为“新番剧 / 新字幕组”追加 <subs> 记录，名称优先读同目录 info.txt
 *   4) 顺带编译站点用的 subtitles.json，并输出一份诊断报告
 *
 * 用法：
 *   node scripts/sync-db.mjs [--repo .] [--db "Subtitles DataBase/Files/db.xml"]
 *                            [--json site/subtitles.json] [--report report.md]
 *                            [--prune] [--check] [--tree-file list.txt] [--no-git]
 */

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const argv = process.argv.slice(2);
const arg = (name, def = null) => {
  const i = argv.indexOf('--' + name);
  if (i === -1) return def;
  const v = argv[i + 1];
  return v && !v.startsWith('--') ? v : true;
};
const REPO = path.resolve(arg('repo', '.'));
const DB_PATH = path.resolve(REPO, arg('db', path.join('Subtitles DataBase', 'Files', 'db.xml')));
const JSON_PATH = arg('json', path.join('site', 'subtitles.json'));
const REPORT_PATH = arg('report', null);
const PRUNE = argv.includes('--prune');
const CHECK = argv.includes('--check');
const TREE_FILE = arg('tree-file', null);
const NO_GIT = argv.includes('--no-git') || !!TREE_FILE;
const DBJSON_PATH = arg('dbjson', null);
const COVERS_PATH = arg('covers', null);
const COVERS_LIMIT = parseInt(arg('covers-limit', '20'), 10) || 20;
const NO_COVERS = argv.includes('--no-covers');

const SRC_TOKENS = new Set([
  'bd', 'dvd', 'tv', 'web', 'webrip', 'hdtv', 'ova', 'oad', 'psp', 'r3dvd',
  'bdr', 'bdrip', '美版bd', '西班牙版bd', '日版bd', '台版bd', '港版bd',
]);

/* ---------------- 1. 取出文件清单（不含内容） ---------------- */
function listFiles() {
  if (TREE_FILE) {
    return fs.readFileSync(path.resolve(REPO, TREE_FILE), 'utf8').split('\n').map(s => s.trim()).filter(Boolean);
  }
  const out = execFileSync(
    'git',
    ['ls-tree', '-r', '--name-only', '-z', 'HEAD', '--', 'subs_list'],
    { cwd: REPO, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }
  );
  return out.split('\0').filter(Boolean);
}

/* ---------------- 2. 扫描目录，归并成“字幕组”粒度 ---------------- */
function scan(files) {
  const groups = new Map();     // key: show|source|sub
  const shows = new Map();      // path: {type,year,show,date}
  const depthIssues = [];
  let fileCount = 0;
  let skipped = 0;

  for (const raw of files) {
    const f = raw.replace(/\\/g, '/');
    if (!f.startsWith('subs_list/')) continue;
    const seg = f.split('/');
    // subs_list / type / year / show / ...
    if (seg.length < 5) { skipped++; continue; }           // 年份层、show 层的 info.txt
    const [, type, year, show] = seg;
    const rest = seg.slice(4);
    if (rest.length < 2) { skipped++; continue; }          // show 级文件：info.txt / 说明文件
    if (rest.some(s => s.startsWith('.'))) { skipped++; continue; }

    const ext = path.extname(rest[rest.length - 1]).toLowerCase();
    if (!ext) { skipped++; continue; }

    const isSourceDir = SRC_TOKENS.has(rest[0].toLowerCase());
    // 有 source 目录时要求 source/字幕组/文件 至少三层，否则是 source 级的说明 txt
    if (isSourceDir && rest.length < 3) { skipped++; continue; }
    if (!isSourceDir && rest.length < 2) { skipped++; continue; }
    const source = (isSourceDir ? rest[0] : 'BD').trim();
    const subName = (isSourceDir ? rest[1] : rest[0]).trim();
    if (path.extname(subName)) { skipped++; continue; }  // 字幕组位置其实是文件
    const showPath = `${type}/${year}/${show}`;
    const key = `${showPath}|${source}|${subName}`;

    if (!groups.has(key)) {
      groups.set(key, { showPath, type, year, show, source, subName, extensions: new Set() });
    }
    groups.get(key).extensions.add(ext);
    fileCount++;

    if (!shows.has(showPath)) {
      shows.set(showPath, { type, year, show, date: parseDate(year, show) });
    }
    if (rest.length > 6) depthIssues.push(f);
  }

  return { groups, shows, fileCount, skipped, depthIssues };
}

function parseDate(year, show) {
  const m = show.match(/^\((\d{4})(?:\.(\d{1,2}))?(?:\.(\d{1,2}))?\)/);
  if (m) {
    const y = m[1];
    const mo = String(m[2] || 1).padStart(2, '0');
    const d = String(m[3] || 1).padStart(2, '0');
    return `${y}-${mo}-${d}`;
  }
  return `${year}-01-01`;
}

/* ---------------- 3. 读取现有 db.xml（只解析，不改写旧记录） ---------------- */
const DB_RAW = fs.existsSync(DB_PATH) ? fs.readFileSync(DB_PATH, 'utf8') : '<DocumentElement>\n</DocumentElement>\n';

function unescapeXml(s) {
  return s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'").replace(/&#x27;/g, "'").replace(/&amp;/g, '&');
}

function existingRecords(xml) {
  const out = [];
  const re = /<subs>([\s\S]*?)<\/subs>/g;
  let m;
  while ((m = re.exec(xml))) {
    const b = m[1];
    const get = n => {
      const r = b.match(new RegExp('<' + n + '>([\\s\\S]*?)</' + n + '>'));
      return r ? unescapeXml(r[1].trim()) : '';
    };
    out.push({
      time: get('time'),
      chs: get('name_chs'),
      cht: get('name_cht'),
      jp: get('name_jp'),
      en: get('name_en'),
      rome: get('name_rome'),
      type: get('type') || 'animation',
      source: get('source'),
      sub_name: get('sub_name'),
      extension: get('extension'),
      providers: get('providers'),
      desc: get('desc'),
      path: get('path').replace(/\\/g, '/'),
    });
  }
  return out;
}

const oldRecords = existingRecords(DB_RAW);
const oldKeys = new Set(oldRecords.map(r => `${r.path}|${r.source}|${r.sub_name}`));
const oldShows = new Set(oldRecords.map(r => r.path));

/* ---------------- 4. 读 info.txt 补名称 ---------------- */
function readInfo(showPath) {
  const rel = `subs_list/${showPath}/info.txt`;
  let text = null;
  try {
    if (NO_GIT) {
      const abs = path.join(REPO, rel);
      if (fs.existsSync(abs)) text = fs.readFileSync(abs, 'utf8');
    } else {
      text = execFileSync('git', ['show', `HEAD:${rel}`], { cwd: REPO, encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 });
    }
  } catch (e) { /* 文件不存在或非 partial clone */ }

  const info = { chs: '', cht: '', jp: '', en: '', rome: '' };
  if (!text) return info;
  text = text.replace(/^\uFEFF/, '');
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(/^\s*(chs|cht|jp|en|rome)\s*:\s*(.*)$/i);
    if (m) info[m[1].toLowerCase()] = m[2].trim();
  }
  if (!info.chs) info.chs = info.jp || info.cht || '';
  return info;
}

/* ---------------- 5. 生成新增记录 ---------------- */
function xmlEscape(s) {
  return String(s == null ? '' : s)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;').replace(/'/g, '&apos;');
}

function block(r) {
  return [
    '  <subs>',
    `    <time>${xmlEscape(r.time)}</time>`,
    `    <name_chs>${xmlEscape(r.chs)}</name_chs>`,
    `    <name_cht>${xmlEscape(r.cht)}</name_cht>`,
    `    <name_jp>${xmlEscape(r.jp)}</name_jp>`,
    `    <name_en>${xmlEscape(r.en)}</name_en>`,
    `    <name_rome>${xmlEscape(r.rome)}</name_rome>`,
    `    <type>${xmlEscape(r.type)}</type>`,
    `    <source>${xmlEscape(r.source)}</source>`,
    `    <sub_name>${xmlEscape(r.sub_name)}</sub_name>`,
    `    <extension>${xmlEscape(r.extension)}</extension>`,
    '    <providers />',
    '    <desc />',
    `    <path>${xmlEscape(r.path.replace(/\//g, '\\'))}</path>`,
    '  </subs>',
  ].join('\n');
}

const scanRes = scan(listFiles());
const newRecords = [];
const newShows = [];
const missingInfo = [];

for (const [key, g] of scanRes.groups) {
  if (oldKeys.has(`${g.showPath}|${g.source}|${g.subName}`)) continue;
  const isNewShow = !oldShows.has(g.showPath);
  const info = readInfo(g.showPath);
  if (isNewShow && !info.chs) missingInfo.push(g.showPath);
  const time = isNewShow ? scanRes.shows.get(g.showPath).date : (oldRecords.find(r => r.path === g.showPath) || {}).time || parseDate(g.year, g.show);

  newRecords.push({
    time: `${time}${time.length === 10 ? 'T00:00:00+08:00' : ''}`,
    ...info,
    type: g.type,
    source: g.source,
    sub_name: g.subName,
    extension: [...g.extensions].sort().join(';'),
    path: g.showPath,
  });
  if (isNewShow && info.chs) newShows.push(g.showPath);
}

/* ---------------- 6. 写回 db.xml（只在末尾追加） ---------------- */
if (newRecords.length) {
  const closing = DB_RAW.lastIndexOf('</DocumentElement>');
  const head = closing === -1 ? DB_RAW + '\n' : DB_RAW.slice(0, closing);
  const tail = closing === -1 ? '</DocumentElement>\n' : DB_RAW.slice(closing);
  const merged = head.replace(/\n*$/, '\n') + newRecords.map(block).join('\n') + '\n' + tail;
  fs.mkdirSync(path.dirname(DB_PATH), { recursive: true });
  fs.writeFileSync(DB_PATH, merged);
}

/* ---------------- 7. 编译站点 subtitles.json ---------------- */
const allRecords = existingRecords(fs.readFileSync(DB_PATH, 'utf8'));
const json = allRecords.map((r, i) => ({
  id: i + 1,
  time: (r.time || '').split('T')[0],
  chs: r.chs, cht: r.cht, jp: r.jp, en: r.en, rome: r.rome,
  type: r.type, source: r.source, sub_name: r.sub_name,
  extension: r.extension,
  path: r.path.replace(/\//g, '\\'),
}));

if (DBJSON_PATH) {
  const dbJson = allRecords.map(r => ({
    time: (r.time || '').split('T')[0],
    name_chs: r.chs, name_cht: r.cht, name_jp: r.jp, name_en: r.en, name_rome: r.rome,
    type: r.type, source: r.source, sub_name: r.sub_name, extension: r.extension,
    providers: r.providers || '', desc: r.desc || '',
    path: r.path.replace(/\//g, '\\'),
  }));
  fs.mkdirSync(path.dirname(path.resolve(REPO, DBJSON_PATH)), { recursive: true });
  fs.writeFileSync(path.resolve(REPO, DBJSON_PATH), JSON.stringify(dbJson));
}

if (JSON_PATH) {
  fs.mkdirSync(path.dirname(path.resolve(REPO, JSON_PATH)), { recursive: true });
  fs.writeFileSync(path.resolve(REPO, JSON_PATH), JSON.stringify(json));
}

/* ---------------- 7.5 新番封面（Bangumi） ---------------- */
const coverStats = { added: 0, missing: 0, total: 0, remaining: 0 };

function cleanTitle(raw) {
  if (!raw) return '';
  let t = raw.split('/')[0].trim();
  t = t.replace(/（[^）]*）|\([^)]*\)|【[^】]*】|\[[^\]]*\]/g, ' ').trim();
  t = t.replace(/第[一二三四五六七八九十0-9]+[季部期]/g, '').trim();
  t = t.replace(/\b(S[0-9]+|Season\s*[0-9]+|TV)\b/gi, '').trim();
  return t;
}

function pickBest(list, query, year) {
  const q = (query || '').trim().toLowerCase();
  let best = null;
  let bestScore = -1;
  for (const it of list.slice(0, 8)) {
    const names = [it.name, it.name_cn].filter(Boolean).map(x => x.toLowerCase());
    let score = 0;
    if (names.some(n => n === q)) score += 4;
    else if (names.some(n => n.startsWith(q))) score += 2;
    else if (names.some(n => n.includes(q))) score += 1;
    const airYear = (it.air_date || '').slice(0, 4);
    if (year && /^\d{4}$/.test(airYear)) {
      const diff = Math.abs(parseInt(airYear, 10) - year);
      if (diff === 0) score += 2;
      else if (diff <= 1) score += 1;
      else if (diff >= 8) score -= 1;
    }
    if (score > bestScore) { bestScore = score; best = it; }
  }
  return bestScore >= 0 ? best : null;
}

async function bgmSearch(keyword, year) {
  const url = 'https://api.bgm.tv/search/subject/' + encodeURIComponent(keyword) + '?type=2&responseGroup=small';
  const ac = new AbortController();
  const timer = setTimeout(() => ac.abort(), 5000);
  try {
    const resp = await fetch(url, {
      headers: { 'User-Agent': 'sub_share-db-sync/1.0 (https://github.com/foxofice/sub_share)' },
      signal: ac.signal,
    });
    if (!resp.ok) return null;
    const data = await resp.json();
    const list = data.list || [];
    const first = pickBest(list, keyword, year);
    if (!first) return null;
    const images = first.images || {};
    const img = images.common || images.medium || images.large || '';
    return {
      id: first.id,
      name: first.name,
      name_cn: first.name_cn || first.name,
      cover: img ? String(img).replace(/^http:/i, 'https:') : null,
    };
  } catch (e) {
    return null;
  } finally {
    clearTimeout(timer);
  }
}

async function updateCovers() {
  const abs = path.resolve(REPO, COVERS_PATH);
  let seed = {};
  if (fs.existsSync(abs)) {
    try { seed = JSON.parse(fs.readFileSync(abs, 'utf8')); } catch (e) { seed = {}; }
  }
  const keyOf = p => p.replace(/\//g, '\\');
  const pending = [];
  for (const [showPath, meta] of scanRes.shows) {
    if (meta.type.toLowerCase() === 'r18') continue;
    if (seed[keyOf(showPath)]) continue;
    pending.push(showPath);
  }
  coverStats.total = Object.keys(seed).length;
  coverStats.remaining = pending.length;
  if (!pending.length) return;

  const batch = pending.slice(0, COVERS_LIMIT);
  for (const showPath of batch) {
    const info = readInfo(showPath);
    const folder = scanRes.shows.get(showPath).show;
    const candidates = [info.chs, info.jp, cleanTitle(folder), folder.replace(/^\([^)]*\)/, '')]
      .map(x => (x || '').trim())
      .filter((x, i, arr) => x.length >= 2 && arr.indexOf(x) === i)
      .slice(0, 2);

    let hit = null;
    for (const kw of candidates) {
      hit = await bgmSearch(kw, parseInt(scanRes.shows.get(showPath).year, 10));
      if (hit) break;
      await new Promise(r => setTimeout(r, 250));
    }
    if (hit && hit.cover) {
      seed[keyOf(showPath)] = hit;
      coverStats.added++;
    } else {
      coverStats.missing++;
    }
    await new Promise(r => setTimeout(r, 150));
  }
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, JSON.stringify(seed, null, 2) + '\n');
  coverStats.total = Object.keys(seed).length;
  coverStats.remaining = pending.length - batch.length;
}

if (COVERS_PATH && !NO_COVERS) await updateCovers();

/* ---------------- 8. 诊断 ---------------- */
const orphan = oldRecords.filter(r => !scanRes.shows.has(r.path));
const noInfo = [...scanRes.shows.keys()].filter(p => {
  const rel = path.join(REPO, 'subs_list', p, 'info.txt');
  return NO_GIT ? !fs.existsSync(rel) : false;
});

const lines = [
  '## 字幕数据库同步报告',
  '',
  `- 扫描文件：**${scanRes.fileCount}** 个（跳过 ${scanRes.skipped} 个非字幕文件）`,
  `- 番剧目录：**${scanRes.shows.size}** 个，字幕组记录：**${scanRes.groups.size}** 条`,
  `- 本次新增番剧：**${newShows.length}** 个，新增记录：**${newRecords.length}** 条`,
  `- 数据库总记录：**${allRecords.length}** 条，站点索引条目：**${json.length}** 条`,
  '',
  orphan.length ? `### ⚠️ 数据库中存在、但目录已不存在（${orphan.length} 条，未自动删除）\n` +
    orphan.slice(0, 20).map(r => `- \`${r.path}\` (${r.source} / ${r.sub_name})`).join('\n') : '### ✅ 无失效记录',
  '',
  missingInfo.length ? `### ⚠️ 新番剧缺少 info.txt（${missingInfo.length} 个，名称回退为目录名）\n` +
    missingInfo.slice(0, 20).map(p => `- \`${p}\``).join('\n') : '### ✅ 新增番剧均有 info.txt',
  '',
  scanRes.depthIssues.length ? `### ⚠️ 嵌套过深（${scanRes.depthIssues.length} 个文件，建议压平到统一层级）` : '### ✅ 目录层级正常',
  '',
  PRUNE ? '`--prune` 已开启（当前实现只报告，不会删除记录）' : '',
];

const report = lines.filter(Boolean).join('\n');
if (REPORT_PATH) fs.writeFileSync(path.resolve(REPO, REPORT_PATH), report + '\n');
console.log(report);

if (CHECK && (newRecords.length || orphan.length)) {
  console.error(`\n[CHECK] 发现 ${newRecords.length} 条待新增、${orphan.length} 条失效记录，退出码 1`);
  process.exit(1);
}
