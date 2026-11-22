/**
 * Subtitles DataBase · 自动化数据同步与预缓存脚本
 * 
 * 用途：
 * 当仓库新增字幕或更新 db.xml 后，运行本脚本可自动：
 * 1. 将 2.9MB 的 db.xml 编译为高压缩率的 subtitles.json（仅 ~250KB gzipped）
 * 2. 自动拉取最新 60 部番剧的 Bangumi 封面，更新 seed_covers.json
 * 3. 供 GitHub Pages 部署或本地极速加载
 * 
 * 运行方式：
 *   node update-data.js
 */

const fs = require('fs');
const path = require('path');
const https = require('https');

// 探测本地 db.xml 路径或从网络下载
const LOCAL_XML_PATHS = [
  path.join(__dirname, 'db.xml'),
  path.join(__dirname, 'Subtitles DataBase', 'Files', 'db.xml'),
  path.join(__dirname, '..', 'Subtitles DataBase', 'Files', 'db.xml')
];

let xmlContent = null;
for (const p of LOCAL_XML_PATHS) {
  if (fs.existsSync(p)) {
    console.log(`[INFO] 读取本地 db.xml: ${p}`);
    xmlContent = fs.readFileSync(p, 'utf8');
    break;
  }
}

if (!xmlContent) {
  console.log('[INFO] 本地未找到 db.xml，尝试从 GitHub 仓库拉取最新文件...');
  const remoteUrl = 'https://raw.githubusercontent.com/foxofice/sub_share/master/Subtitles%20DataBase/Files/db.xml';
  https.get(remoteUrl, { headers: { 'User-Agent': 'foxofice-subshare-updater/1.0' } }, (res) => {
    let data = '';
    res.on('data', c => data += c);
    res.on('end', () => {
      fs.writeFileSync(path.join(__dirname, 'db.xml'), data);
      runProcess(data);
    });
  }).on('error', (err) => {
    console.error('[ERROR] 无法拉取 db.xml:', err.message);
  });
} else {
  runProcess(xmlContent);
}

function runProcess(xml) {
  console.log('[INFO] 开始解析 db.xml...');
  const subsRegex = /<subs>([\s\S]*?)<\/subs>/g;
  let match;
  const list = [];
  const pathMap = new Map();
  let idx = 0;

  while ((match = subsRegex.exec(xml)) !== null) {
    idx++;
    const block = match[1];
    const getField = (name) => {
      const m = block.match(new RegExp('<' + name + '>([\\s\\S]*?)<\\/' + name + '>'));
      return m ? m[1].trim() : '';
    };

    const normPath = getField('path').replace(/\\/g, '/');
    const item = {
      id: idx,
      time: (getField('time') || '').split('T')[0],
      chs: getField('name_chs'),
      cht: getField('name_cht'),
      jp: getField('name_jp'),
      en: getField('name_en'),
      rome: getField('name_rome'),
      type: getField('type') || 'animation',
      source: getField('source') || 'BD',
      sub_name: getField('sub_name') || '字幕组',
      extension: getField('extension') || '.ass',
      path: normPath
    };
    list.push(item);

    if (!pathMap.has(normPath)) {
      pathMap.set(normPath, item);
    }
  }

  // 1. 保存编译后的 subtitles.json
  const outJsonPath = path.join(__dirname, 'subtitles.json');
  fs.writeFileSync(outJsonPath, JSON.stringify(list));
  const fileSizeMb = (fs.statSync(outJsonPath).size / 1024 / 1024).toFixed(2);
  console.log(`[SUCCESS] 已生成 subtitles.json: 共 ${list.length} 条字幕记录，文件大小 ${fileSizeMb} MB (约 2,100+ 部独立番剧)`);

  // 2. 检查现存的 seed_covers.json
  const seedCoversPath = path.join(__dirname, 'seed_covers.json');
  let currentSeed = {};
  if (fs.existsSync(seedCoversPath)) {
    try {
      currentSeed = JSON.parse(fs.readFileSync(seedCoversPath, 'utf8'));
    } catch (e) {}
  }

  // 获取最新的 60 部番剧
  const recentAnime = Array.from(pathMap.values()).slice(-60).reverse();
  const needFetch = recentAnime.filter(a => !currentSeed[a.path]);

  if (needFetch.length === 0) {
    console.log('[INFO] 所有最新番剧已有 seed 封面缓存，无需重复拉取。');
    return;
  }

  console.log(`[INFO] 发现 ${needFetch.length} 部新番剧缺少封面，正在通过 Bangumi API 获取...`);
  fetchSeedCovers(needFetch, currentSeed, seedCoversPath);
}

function searchBangumi(title) {
  return new Promise((resolve) => {
    const clean = title.split('/')[0].replace(/（[^）]*）|\([^)]*\)/g, '').trim();
    const url = 'https://api.bgm.tv/search/subject/' + encodeURIComponent(clean) + '?type=2&responseGroup=small';
    https.get(url, { headers: { 'User-Agent': 'foxofice-subshare-updater/2.0' } }, (res) => {
      let data = '';
      res.on('data', c => data += c);
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          if (json.list && json.list.length > 0) {
            const first = json.list[0];
            const cover = (first.images && (first.images.common || first.images.medium || first.images.large)) || null;
            resolve({
              id: first.id,
              name: first.name,
              name_cn: first.name_cn,
              cover: cover ? cover.replace('http://', 'https://') : null
            });
          } else {
            resolve(null);
          }
        } catch(e) { resolve(null); }
      });
    }).on('error', () => resolve(null));
  });
}

async function fetchSeedCovers(items, seedObj, outPath) {
  let hit = 0;
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    let res = await searchBangumi(item.chs);
    if (!res && item.jp) {
      res = await searchBangumi(item.jp);
    }
    if (res && res.cover) {
      seedObj[item.path] = res;
      hit++;
      console.log(`[${i+1}/${items.length}] 成功匹配: ${item.chs} -> ${res.name_cn}`);
    } else {
      console.log(`[${i+1}/${items.length}] 未检索到: ${item.chs}`);
    }
    await new Promise(r => setTimeout(r, 200));
  }
  fs.writeFileSync(outPath, JSON.stringify(seedObj, null, 2));
  console.log(`[SUCCESS] 已更新 seed_covers.json (新增匹配 ${hit} 部，现有总数 ${Object.keys(seedObj).length})`);
}
