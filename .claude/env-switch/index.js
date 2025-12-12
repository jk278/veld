#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 文件路径
const cacheFilePath = path.join(__dirname, '..', '.cache', 'env.local.json');
const settingsFilePath = path.join(__dirname, '..', 'settings.local.json');

try {
  // 读取缓存文件
  const cacheData = JSON.parse(fs.readFileSync(cacheFilePath, 'utf8'));

  // 获取环境列表
  const envNames = Object.keys(cacheData);

  // 从命令行参数获取选择的环境（如果提供）
  const argIndex = process.argv.indexOf('--env');
  if (argIndex !== -1 && process.argv[argIndex + 1]) {
    const selectedEnv = process.argv[argIndex + 1];
    if (!envNames.includes(selectedEnv)) {
      console.error(`❌ 环境 "${selectedEnv}" 不存在！`);
      console.log(`可用环境: ${envNames.join(', ')}`);
      process.exit(1);
    }

    // 读取当前设置
    let settings = {};
    if (fs.existsSync(settingsFilePath)) {
      settings = JSON.parse(fs.readFileSync(settingsFilePath, 'utf8'));
    }

    // 更新环境
    settings.env = cacheData[selectedEnv];

    // 写入设置文件
    fs.writeFileSync(settingsFilePath, JSON.stringify(settings, null, 2));

    console.log(`✅ 已切换到环境: ${selectedEnv}`);
    console.log(`📁 设置文件: ${settingsFilePath}`);
    process.exit(0);
  }

  // 如果没有命令行参数，显示交互式选择
  console.log('🔧 Claude 环境切换工具\n');
  console.log('可用环境:');
  envNames.forEach((env, index) => {
    console.log(`  ${index + 1}. ${env}`);
  });

  // 读取当前设置
  let currentSettings = {};
  if (fs.existsSync(settingsFilePath)) {
    try {
      currentSettings = JSON.parse(fs.readFileSync(settingsFilePath, 'utf8'));
    } catch {
      console.warn('⚠️  设置文件格式有误，将创建新文件');
    }
  }

  // 检查当前环境
  const currentEnvKey = Object.keys(cacheData).find(envKey =>
    JSON.stringify(cacheData[envKey]) === JSON.stringify(currentSettings.env)
  );

  if (currentEnvKey) {
    console.log(`\n📌 当前环境: ${currentEnvKey}`);
  }

  console.log('\n💡 使用方式:');
  console.log('  1. 在终端中运行此脚本时，传入 --env <环境名> 参数');
  console.log('  2. 例如: node .claude/env-switch/index.js --env env_kimi');
  console.log('  3. 或者: node .claude/env-switch/index.js --env env_minimax');

} catch (error) {
  console.error('❌ 切换环境时发生错误:', error.message);
  process.exit(1);
}
