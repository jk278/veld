#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Box, render, Text, useInput } from "ink";
import { useEffect, useState } from "react";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 文件路径
const cacheFilePath = path.join(__dirname, "env.local.json");
const settingsFilePath = path.join(
  __dirname,
  "..",
  "..",
  "settings.local.json",
);

// 组件定义
const EnvSwitcher = () => {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [cacheData, setCacheData] = useState<Record<string, unknown> | null>(
    null,
  );
  const [currentEnvKey, setCurrentEnvKey] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showUsage, setShowUsage] = useState(false);

  useEffect(() => {
    // 检查文件是否存在
    if (!fs.existsSync(cacheFilePath)) {
      setError("env.local.json 文件不存在！");
      return;
    }

    try {
      // 读取缓存文件
      const data = JSON.parse(fs.readFileSync(cacheFilePath, "utf8"));
      setCacheData(data);

      // 读取当前设置
      if (fs.existsSync(settingsFilePath)) {
        try {
          const settings = JSON.parse(
            fs.readFileSync(settingsFilePath, "utf8"),
          );
          const envKey = Object.keys(data).find(
            (envKey) =>
              JSON.stringify(data[envKey]) === JSON.stringify(settings.env),
          );
          setCurrentEnvKey(envKey ?? null);
          if (envKey) {
            setSelectedIndex(Object.keys(data).indexOf(envKey));
          }
        } catch {
          console.warn("⚠️  设置文件格式有误，将创建新文件\n");
        }
      }
    } catch (err) {
      setError(
        `读取文件时出错: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  }, []);

  const handleSelect = (env: string) => {
    if (!cacheData) return;

    try {
      // 读取当前设置
      let settings: Record<string, unknown> = {};
      if (fs.existsSync(settingsFilePath)) {
        settings = JSON.parse(fs.readFileSync(settingsFilePath, "utf8"));
      }

      // 更新环境
      settings.env = cacheData[env];

      // 写入设置文件
      fs.writeFileSync(settingsFilePath, JSON.stringify(settings, null, 2));

      setCurrentEnvKey(env);
      // 等待UI更新显示 ✓ current 后退出
      setTimeout(() => process.exit(0), 50);
    } catch (err) {
      setError(
        `切换环境时出错: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  };

  useInput((input, key) => {
    if (!cacheData) return;

    if (key.upArrow) {
      setSelectedIndex((prev) =>
        prev > 0 ? prev - 1 : Object.keys(cacheData).length - 1,
      );
    } else if (key.downArrow) {
      setSelectedIndex((prev) =>
        prev < Object.keys(cacheData).length - 1 ? prev + 1 : 0,
      );
    } else if (key.return) {
      const envNames = Object.keys(cacheData);
      handleSelect(envNames[selectedIndex]);
    } else if (key.escape) {
      process.exit(0);
    } else if (input === "u" || input === "U") {
      setShowUsage((prev) => !prev);
    }
  });

  if (error) {
    return (
      <Box flexDirection="column">
        <Text color="red">❌ 错误: {error}</Text>
        <Box margin={1}>
          <Text>可用环境配置:</Text>
          <Text> • Kimi: env_kimi</Text>
          <Text> • MiniMax: env_minimax</Text>
          <Text> • GLM: env_glm</Text>
        </Box>
        <Box marginTop={1}>
          <Text>💡 使用方式:</Text>
          <Text> 1. 先运行环境切换任务</Text>
          <Text> 2. 或在 tasks.json 中配置环境</Text>
        </Box>
      </Box>
    );
  }

  if (!cacheData) {
    return <Text>加载中...</Text>;
  }

  const envNames = Object.keys(cacheData);

  return (
    <Box
      flexDirection="column"
      borderStyle="round"
      borderColor="cyan"
      padding={1}
    >
      <Text color="cyan">Claude Code Env Switcher</Text>
      <Box marginTop={1} flexDirection="column">
        {envNames.map((env, index) => {
          const isSelected = index === selectedIndex;
          const isCurrent = env === currentEnvKey;
          const prefix = isSelected ? "▶ " : "  ";
          return (
            <Text key={env} color={isSelected ? "cyan" : undefined}>
              {prefix}
              {index + 1}. {env}
              {isCurrent && <Text color="green"> ✓ current</Text>}
            </Text>
          );
        })}
      </Box>
      <Box marginTop={1}>
        <Text color="gray">↑/↓ Select, Enter Confirm, Esc Quit, U Usage</Text>
      </Box>
      {showUsage && (
        <Box marginTop={1} flexDirection="column">
          <Text color="yellow">USAGE:</Text>
          <Box marginLeft={2} flexDirection="column">
            <Text color="blue">• Kimi: https://www.kimi.com/coding/console?from=membership</Text>
            <Text color="blue">• Minimax: https://platform.minimaxi.com/user-center/payment/coding-plan</Text>
          </Box>
        </Box>
      )}
    </Box>
  );
};

// 检查命令行参数
const argIndex = process.argv.indexOf("--env");
if (argIndex !== -1 && process.argv[argIndex + 1]) {
  const selectedEnv = process.argv[argIndex + 1];

  if (!fs.existsSync(cacheFilePath)) {
    console.error("❌ 错误: env.local.json 文件不存在！");
    console.log("┌─ 可用环境配置 ──────────────────────────┐");
    console.log("│  Kimi    : env_kimi                     │");
    console.log("│  MiniMax : env_minimax                  │");
    console.log("│  GLM     : env_glm                      │");
    console.log("└─────────────────────────────────────────┘");
    console.log("\n💡 使用方式:");
    console.log("  1. 先运行环境切换任务");
    console.log("  2. 或在 tasks.json 中配置环境");
    process.exit(1);
  }

  try {
    const cacheData = JSON.parse(fs.readFileSync(cacheFilePath, "utf8"));
    const envNames = Object.keys(cacheData);

    if (!envNames.includes(selectedEnv)) {
      console.error(`❌ 环境 "${selectedEnv}" 不存在！`);
      console.error("\n可用环境:");
      envNames.forEach((env) => {
        console.error(`  • ${env}`);
      });
      process.exit(1);
    }

    // 读取当前设置
    let settings: Record<string, unknown> = {};
    if (fs.existsSync(settingsFilePath)) {
      settings = JSON.parse(fs.readFileSync(settingsFilePath, "utf8"));
    }

    // 更新环境
    (settings as Record<string, unknown>).env = cacheData[selectedEnv];

    // 写入设置文件
    fs.writeFileSync(settingsFilePath, JSON.stringify(settings, null, 2));
    process.exit(0);
  } catch (error) {
    console.error("\n❌ 切换环境时发生错误:");
    console.error(
      `   ${error instanceof Error ? error.message : String(error)}`,
    );
    process.exit(1);
  }
}

// 渲染组件
try {
  render(<EnvSwitcher />, {
    stdout: process.stdout,
    stdin: process.stdin,
    exitOnCtrlC: false,
  });
} catch (error) {
  console.error("\n❌ 渲染界面时发生错误:");
  console.error(`   ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}
