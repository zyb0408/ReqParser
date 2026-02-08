import dictionaryData from "../data/dictionary.json";

export interface HeaderInfo {
  name: string;
  desc: string;
  category: string;
  mdn: string;
  request: boolean;
  response: boolean;
  security: boolean;
  values?: Record<string, string>;
}

export interface SecurityAdvisory {
  id: string;
  header: string;
  condition: string;
  risk: "high" | "medium" | "low";
  description: string;
  recommendation: string;
}

type DictionaryData = {
  headers: Record<string, HeaderInfo>;
  securityAdvisories: SecurityAdvisory[];
};

const data = dictionaryData as DictionaryData;

// 预构建小写 key -> 原始 key 的映射，用于忽略大小写查找
const lowerKeyMap = new Map<string, string>();
for (const key of Object.keys(data.headers)) {
  lowerKeyMap.set(key.toLowerCase(), key);
}

/**
 * 根据 header key 获取信息（忽略大小写）
 */
export function getHeaderInfo(key: string): HeaderInfo | null {
  const originalKey = lowerKeyMap.get(key.toLowerCase());
  if (!originalKey) return null;
  return data.headers[originalKey] ?? null;
}

/**
 * 根据 header key 和 value 获取值说明
 */
export function getValueInfo(key: string, value: string): string | null {
  const info = getHeaderInfo(key);
  if (!info?.values) return null;

  // 精确匹配
  const trimmed = value.trim();
  if (info.values[trimmed]) return info.values[trimmed];

  // 忽略大小写匹配
  const lowerValue = trimmed.toLowerCase();
  for (const [k, v] of Object.entries(info.values)) {
    if (k.toLowerCase() === lowerValue) return v;
  }

  // 前缀匹配（如 "max-age=3600" 匹配 "max-age"）
  for (const [k, v] of Object.entries(info.values)) {
    if (lowerValue.startsWith(k.toLowerCase())) return v;
  }

  return null;
}

/**
 * 搜索 header（模糊匹配名称和描述）
 */
export function searchHeaders(query: string): HeaderInfo[] {
  if (!query.trim()) return [];
  const lower = query.toLowerCase();
  const results: HeaderInfo[] = [];

  for (const info of Object.values(data.headers)) {
    if (
      info.name.toLowerCase().includes(lower) ||
      info.desc.toLowerCase().includes(lower) ||
      info.category.toLowerCase().includes(lower)
    ) {
      results.push(info);
    }
  }

  // 名称匹配优先排序
  results.sort((a, b) => {
    const aNameMatch = a.name.toLowerCase().includes(lower) ? 0 : 1;
    const bNameMatch = b.name.toLowerCase().includes(lower) ? 0 : 1;
    return aNameMatch - bNameMatch;
  });

  return results;
}

/**
 * 获取所有分类
 */
export function getCategories(): string[] {
  const categories = new Set<string>();
  for (const info of Object.values(data.headers)) {
    categories.add(info.category);
  }
  return Array.from(categories).sort();
}

/**
 * 按分类获取 headers
 */
export function getHeadersByCategory(category: string): HeaderInfo[] {
  const lower = category.toLowerCase();
  return Object.values(data.headers).filter(
    (info) => info.category.toLowerCase() === lower
  );
}

/**
 * 获取所有安全建议
 */
export function getSecurityAdvisories(): SecurityAdvisory[] {
  return data.securityAdvisories;
}

/**
 * 根据 header key 获取相关的安全建议
 */
export function getSecurityAdvisoriesForHeader(
  key: string
): SecurityAdvisory[] {
  const lower = key.toLowerCase();
  return data.securityAdvisories.filter(
    (advisory) => advisory.header.toLowerCase() === lower
  );
}
