#!/usr/bin/env node

/**
 * Doris Profile Parser 运行脚本
 * 使用方法：
 *   node run-parser.js <profile-file> [options]
 *   
 * 示例：
 *   node run-parser.js test-profile3.txt
 *   node run-parser.js test-profile3.txt --output result.json
 *   node run-parser.js test-profile3.txt --analysis
 *   node run-parser.js test-profile3.txt --tree
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 简化的解析器类 (从test-parser.js复制)
class DorisProfileParser {
  constructor(options = {}) {
    this.options = {
      includeRawData: true,
      validateStructure: true,
      parseCustomCounters: true,
      ...options
    };
    this.errors = [];
    this.warnings = [];
  }

  parse(profileText) {
    this.errors = [];
    this.warnings = [];

    try {
      const sections = this.splitProfileSections(profileText);
      
      const summary = this.parseSummarySection(sections.summary);
      const executionSummary = this.parseExecutionSummarySection(sections.executionSummary);
      const mergedProfile = this.parseMergedProfileSection(sections.mergedProfile);

      const parsedProfile = {
        summary,
        executionSummary,
        mergedProfile,
        rawSections: this.options.includeRawData ? sections : {
          summary: '',
          executionSummary: '',
          mergedProfile: ''
        }
      };

      return {
        success: this.errors.length === 0,
        data: parsedProfile,
        errors: this.errors,
        warnings: this.warnings
      };
    } catch (error) {
      this.errors.push({
        section: 'general',
        line: 0,
        message: `Unexpected error: ${error.message}`
      });

      return {
        success: false,
        errors: this.errors,
        warnings: this.warnings
      };
    }
  }

  splitProfileSections(profileText) {
    const lines = profileText.split('\n');
    let summaryStart = -1;
    let executionSummaryStart = -1;
    let mergedProfileStart = -1;
    let detailProfileStart = -1;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (line === 'Summary:') {
        summaryStart = i;
      } else if (line === 'Execution Summary:' || line === 'Execution  Summary:') {
        executionSummaryStart = i;
      } else if (line === 'MergedProfile:') {
        mergedProfileStart = i;
      } else if (line.startsWith('DetailProfile(')) {
        detailProfileStart = i;
        break;
      }
    }

    const summaryEnd = executionSummaryStart > 0 ? executionSummaryStart : lines.length;
    const executionSummaryEnd = mergedProfileStart > 0 ? mergedProfileStart : lines.length;
    const mergedProfileEnd = detailProfileStart > 0 ? detailProfileStart : lines.length;

    return {
      summary: summaryStart >= 0 ? lines.slice(summaryStart, summaryEnd).join('\n') : '',
      executionSummary: executionSummaryStart >= 0 ? lines.slice(executionSummaryStart, executionSummaryEnd).join('\n') : '',
      mergedProfile: mergedProfileStart >= 0 ? lines.slice(mergedProfileStart, mergedProfileEnd).join('\n') : ''
    };
  }

  parseSummarySection(summaryText) {
    const lines = summaryText.split('\n');
    const summary = {};
    let sqlStatement = '';
    let inSqlStatement = false;

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      if (trimmedLine.startsWith('-  Profile  ID:')) {
        summary.profileId = this.extractValue(trimmedLine, 'Profile  ID:');
      } else if (trimmedLine.startsWith('-  Task  Type:')) {
        summary.taskType = this.extractValue(trimmedLine, 'Task  Type:');
      } else if (trimmedLine.startsWith('-  Start  Time:')) {
        summary.startTime = this.extractValue(trimmedLine, 'Start  Time:');
      } else if (trimmedLine.startsWith('-  End  Time:')) {
        summary.endTime = this.extractValue(trimmedLine, 'End  Time:');
      } else if (trimmedLine.startsWith('-  Total:')) {
        summary.total = this.extractValue(trimmedLine, 'Total:');
      } else if (trimmedLine.startsWith('-  Task  State:')) {
        summary.taskState = this.extractValue(trimmedLine, 'Task  State:');
      } else if (trimmedLine.startsWith('-  User:')) {
        summary.user = this.extractValue(trimmedLine, 'User:');
      } else if (trimmedLine.startsWith('-  Default  Catalog:')) {
        summary.defaultCatalog = this.extractValue(trimmedLine, 'Default  Catalog:');
      } else if (trimmedLine.startsWith('-  Default  Db:')) {
        summary.defaultDb = this.extractValue(trimmedLine, 'Default  Db:');
      } else if (trimmedLine.startsWith('-  Sql  Statement:')) {
        inSqlStatement = true;
        sqlStatement = this.extractValue(trimmedLine, 'Sql  Statement:');
      } else if (trimmedLine.startsWith('-  Distributed  Plan:')) {
        inSqlStatement = false;
        summary.distributedPlan = this.extractValue(trimmedLine, 'Distributed  Plan:');
      } else if (inSqlStatement) {
        sqlStatement += ' ' + trimmedLine;
      }
    }

    summary.sqlStatement = sqlStatement.trim();
    return summary;
  }

  parseExecutionSummarySection(executionSummaryText) {
    const lines = executionSummaryText.split('\n');
    const executionSummary = {};

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      // 解析所有层级的项目：-  项目名称: 值
      if (trimmedLine.startsWith('-  ')) {
        const match = trimmedLine.match(/^-\s+([^:]+):\s*(.*)$/);
        if (match) {
          const key = match[1].trim().replace(/\s+/g, '');
          const value = match[2].trim();
          executionSummary[key] = value;
        }
      }
    }

    return executionSummary;
  }

  parseMergedProfileSection(mergedProfileText) {
    const lines = mergedProfileText.split('\n');
    const fragments = [];
    
    let currentFragment = null;
    let currentPipeline = null;
    let currentOperator = null;
    let inCommonCounters = false;
    let inCustomCounters = false;
    let inPlanInfo = false;

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      if (trimmedLine === '') continue;

      if (trimmedLine.startsWith('Fragment ')) {
        if (currentOperator && currentPipeline) {
          currentPipeline.operators.push(currentOperator);
        }
        if (currentPipeline && currentFragment) {
          currentFragment.pipelines.push(currentPipeline);
        }
        if (currentFragment) {
          fragments.push(currentFragment);
        }
        
        const fragmentId = this.extractFragmentId(trimmedLine);
        currentFragment = {
          id: fragmentId,
          pipelines: [],
          rawData: this.options.includeRawData ? line : ''
        };
        currentPipeline = null;
        currentOperator = null;
        inCommonCounters = false;
        inCustomCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine.startsWith('Pipeline ')) {
        if (currentOperator && currentPipeline) {
          currentPipeline.operators.push(currentOperator);
        }
        if (currentPipeline && currentFragment) {
          currentFragment.pipelines.push(currentPipeline);
        }
        
        const pipelineInfo = this.extractPipelineInfo(trimmedLine);
        currentPipeline = {
          id: pipelineInfo.id,
          instanceNum: pipelineInfo.instanceNum,
          operators: [],
          rawData: this.options.includeRawData ? line : ''
        };
        currentOperator = null;
        inCommonCounters = false;
        inCustomCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine.startsWith('-  WaitWorkerTime:')) {
        if (currentPipeline) {
          currentPipeline.waitWorkerTime = this.parseMetricValue(trimmedLine, 'WaitWorkerTime');
        }
      } else if (this.isOperatorLine(trimmedLine)) {
        if (currentOperator && currentPipeline) {
          currentPipeline.operators.push(currentOperator);
        }
        
        currentOperator = this.parseOperator(trimmedLine);
        inCommonCounters = false;
        inCustomCounters = false;
        inPlanInfo = false;
      // PlanInfo解析 - 支持不同缩进级别
      } else if (trimmedLine === '-  PlanInfo' || trimmedLine.startsWith('-  PlanInfo') ||
               line.match(/^\s+- PlanInfo\s*$/)) {
        inPlanInfo = true;
        inCommonCounters = false;
        inCustomCounters = false;
      // CommonCounters解析 - 支持不同缩进级别
      } else if (trimmedLine === 'CommonCounters:' || line.match(/^\s+CommonCounters:\s*$/)) {
        inCommonCounters = true;
        inCustomCounters = false;
        inPlanInfo = false;
      // CustomCounters解析 - 支持不同缩进级别
      } else if (trimmedLine === 'CustomCounters:' || line.match(/^\s+CustomCounters:\s*$/)) {
        inCustomCounters = true;
        inCommonCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine.startsWith('-  ') && currentOperator) {
        if (inPlanInfo) {
          this.parsePlanInfoItem(currentOperator, trimmedLine);
        } else if (inCommonCounters) {
          this.parseCounterItem(currentOperator.commonCounters, trimmedLine);
        } else if (inCustomCounters) {
          this.parseCounterItem(currentOperator.customCounters, trimmedLine);
        }
      } else if (line.match(/^\s{20,}-\s+/) && currentOperator) {
        const nestedTrimmedLine = line.trim();
        if (inCommonCounters) {
          this.parseCounterItem(currentOperator.commonCounters, nestedTrimmedLine);
        } else if (inCustomCounters) {
          this.parseCounterItem(currentOperator.customCounters, nestedTrimmedLine);
        }
      }
    }

    if (currentOperator && currentPipeline) {
      currentPipeline.operators.push(currentOperator);
    }
    if (currentPipeline && currentFragment) {
      currentFragment.pipelines.push(currentPipeline);
    }
    if (currentFragment) {
      fragments.push(currentFragment);
    }

    return { fragments };
  }

  extractValue(line, key) {
    const index = line.indexOf(key);
    if (index >= 0) {
      return line.substring(index + key.length).trim();
    }
    return '';
  }

  extractFragmentId(line) {
    const match = line.match(/Fragment\s+(\d+):/);
    return match ? parseInt(match[1], 10) : 0;
  }

  extractPipelineInfo(line) {
    const match = line.match(/Pipeline\s+(\d+)\(instance_num=(\d+)\):/);
    if (match) {
      return {
        id: parseInt(match[1], 10),
        instanceNum: parseInt(match[2], 10)
      };
    }
    return { id: 0, instanceNum: 1 };
  }

  isOperatorLine(line) {
    // 支持标准格式和FILE_SCAN_OPERATOR特殊格式
    return /^[A-Z_]+(?:\([^)]*\))*(?:\(id=[-\d]+\))?:/.test(line) ||
           /^[A-Z_]+\s+\(id=\d+\.\s+nereids_id=\d+\.\s+table\s+name\s*=\s*[^)]+\):/.test(line);
  }

  parseOperator(line) {
    // 处理FILE_SCAN_OPERATOR的特殊格式
    const fileScanMatch = line.match(/^([A-Z_]+)\s+\(id=(\d+)\.\s+nereids_id=(\d+)\.\s+table\s+name\s*=\s*([^)]+)\):/);
    if (fileScanMatch) {
      const operatorName = fileScanMatch[1];
      const id = fileScanMatch[2];
      const nereuId = fileScanMatch[3];
      const tableName = fileScanMatch[4].trim();

      return {
        id,
        name: operatorName,
        nereu_id: nereuId,
        planInfo: { 'table_name': tableName },
        commonCounters: {},
        customCounters: {},
        rawData: this.options.includeRawData ? line : ''
      };
    }

    // 标准格式解析
    const match = line.match(/^([A-Z_]+(?:\([^)]*\))*)\(id=([-\d]+)\):/);
    if (match) {
      const fullName = match[1];
      const id = match[2];
      
      const nereuMatch = fullName.match(/\(nereids_id=(\d+)\)/);
      const nereuId = nereuMatch ? nereuMatch[1] : undefined;
      
      const operatorName = fullName.replace(/\([^)]*\)/g, '');

      return {
        id,
        name: operatorName,
        nereu_id: nereuId,
        planInfo: {},
        commonCounters: {},
        customCounters: {},
        rawData: this.options.includeRawData ? line : ''
      };
    }

    const specialMatch = line.match(/^([A-Z_]+\([^)]*\))\(id=([-\d]+)\):/);
    if (specialMatch) {
      return {
        id: specialMatch[2],
        name: specialMatch[1],
        planInfo: {},
        commonCounters: {},
        customCounters: {},
        rawData: this.options.includeRawData ? line : ''
      };
    }

    const defaultMatch = line.match(/^([^(]+)/);
    return {
      id: 'unknown',
      name: defaultMatch ? defaultMatch[1].replace(':', '') : 'UNKNOWN_OPERATOR',
      planInfo: {},
      commonCounters: {},
      customCounters: {},
      rawData: this.options.includeRawData ? line : ''
    };
  }

  parsePlanInfoItem(operator, line) {
    const match = line.match(/^-\s+([^:]+):\s*(.*)$/);
    if (match) {
      const key = match[1].trim();
      const value = match[2].trim();
      if (!operator.planInfo) {
        operator.planInfo = {};
      }
      operator.planInfo[key] = value;
    }
  }

  parseCounterItem(counters, line) {
    const match = line.match(/^-\s+([^:]+):\s*(.*)$/);
    if (match) {
      const key = match[1].trim();
      const value = match[2].trim();
      counters[key] = this.parseMetricValue(`-  ${key}: ${value}`, key);
    }
  }

  /**
   * 将时间值归一化为微秒(us)
   */
  normalizeTimeToMicroseconds(timeStr) {
    if (!timeStr || typeof timeStr !== 'string') return 0;
    
    // 清理字符串，移除尾部逗号和空格
    const cleanStr = timeStr.replace(/[,\s]+$/, '');
    
    // 处理复合时间格式，如 "3sec846ms" 或 "1sec279ms"
    const complexMatch = cleanStr.match(/(\d+)sec(\d+)ms/);
    if (complexMatch) {
      const seconds = parseInt(complexMatch[1]);
      const milliseconds = parseInt(complexMatch[2]);
      return (seconds * 1000 + milliseconds) * 1000; // 转换为微秒
    }
    
    // 处理简单时间格式
    const simpleMatch = cleanStr.match(/([\d.]+)(ns|us|ms|sec|s)/);
    if (simpleMatch) {
      const value = parseFloat(simpleMatch[1]);
      const unit = simpleMatch[2];
      
      switch (unit) {
        case 'ns': return value / 1000; // 纳秒转微秒
        case 'us': return value; // 已经是微秒
        case 'ms': return value * 1000; // 毫秒转微秒
        case 'sec':
        case 's': return value * 1000000; // 秒转微秒
        default: return 0;
      }
    }
    
    return 0;
  }

  /**
   * 将数据量值归一化为具体数值
   */
  normalizeDataSize(sizeStr) {
    if (!sizeStr || typeof sizeStr !== 'string') return 0;
    
    // 清理字符串，移除尾部逗号和空格
    let cleanStr = sizeStr.replace(/[,\s]+$/, '');
    
    // 处理括号中的具体数值，如 "59.859K (59859)"
    const bracketMatch = cleanStr.match(/\((\d+)\)/);
    if (bracketMatch) {
      return parseInt(bracketMatch[1]);
    }
    
    // 处理带单位的数据量
    const sizeMatch = cleanStr.match(/([\d.]+)([KMGTB])?/i);
    if (sizeMatch) {
      const value = parseFloat(sizeMatch[1]);
      const unit = sizeMatch[2]?.toUpperCase() || '';
      
      switch (unit) {
        case 'K': return Math.round(value * 1000);
        case 'M': return Math.round(value * 1000000);
        case 'G': return Math.round(value * 1000000000);
        case 'T': return Math.round(value * 1000000000000);
        case 'B': return Math.round(value); // 字节
        default: return Math.round(value); // 无单位，当作具体数值
      }
    }
    
    return 0;
  }

  /**
   * 检测数值类型并进行归一化
   */
  normalizeValue(value, key) {
    const lowerKey = key.toLowerCase();
    
    // 时间相关字段
    if (lowerKey.includes('time') || lowerKey.includes('latency') || lowerKey.includes('duration')) {
      const normalized = this.normalizeTimeToMicroseconds(value);
      return normalized > 0 ? normalized : value;
    }
    
    // 数据量相关字段
    if (lowerKey.includes('rows') || lowerKey.includes('bytes') || lowerKey.includes('size') || 
        lowerKey.includes('memory') || lowerKey.includes('usage') || lowerKey.includes('throughput')) {
      const normalized = this.normalizeDataSize(value);
      return normalized > 0 ? normalized : value;
    }
    
    // 其他情况保持原值
    return value;
  }

  parseMetricValue(line, fieldName = '') {
    const metricValue = {};
    
    // 解析sum, avg, max, min格式
    // 支持复合时间单位如: 3sec846ms, 1.5KB, 2GB等
    const patterns = [
      { key: 'sum', pattern: /sum\s+([\d.,]+(?:[A-Za-z]+[\d.,]*[A-Za-z]*)*(?:\s*\([^)]+\))?)/i },
      { key: 'avg', pattern: /avg\s+([\d.,]+(?:[A-Za-z]+[\d.,]*[A-Za-z]*)*)/i },
      { key: 'max', pattern: /max\s+([\d.,]+(?:[A-Za-z]+[\d.,]*[A-Za-z]*)*)/i },
      { key: 'min', pattern: /min\s+([\d.,]+(?:[A-Za-z]+[\d.,]*[A-Za-z]*)*)/i }
    ];

    for (const { key, pattern } of patterns) {
      const match = line.match(pattern);
      if (match) {
        const rawValue = match[1].trim();
        // 应用归一化
        metricValue[key] = this.normalizeValue(rawValue, fieldName);
      }
    }

    if (Object.keys(metricValue).length === 0) {
      const simpleMatch = line.match(/:\s*(.+)$/);
      if (simpleMatch) {
        const rawValue = simpleMatch[1].trim();
        // 应用归一化
        metricValue.avg = this.normalizeValue(rawValue, fieldName);
      }
    }

    return metricValue;
  }
}

// 分析函数
function performanceAnalysis(data) {
  const operatorTimes = [];
  
  data.mergedProfile.fragments.forEach(fragment => {
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        if (operator.commonCounters.ExecTime?.avg) {
          operatorTimes.push({
            name: operator.name,
            id: operator.id,
            time: operator.commonCounters.ExecTime.avg,
            fragmentId: fragment.id,
            pipelineId: pipeline.id
          });
        }
      });
    });
  });

  operatorTimes.sort((a, b) => {
    const aTime = parseFloat(a.time.replace(/[^0-9.]/g, ''));
    const bTime = parseFloat(b.time.replace(/[^0-9.]/g, ''));
    return bTime - aTime;
  });

  console.log('\n🕒 执行时间最长的算子（Top 10）');
  console.log('═'.repeat(80));
  operatorTimes.slice(0, 10).forEach((op, index) => {
    console.log(`${(index + 1).toString().padStart(2)}. ${op.name.padEnd(35)} ${op.time.padStart(12)} [F${op.fragmentId}, P${op.pipelineId}]`);
  });
}

function memoryAnalysis(data) {
  const memoryOperators = [];
  let totalMemoryUsage = 0;
  let totalMemoryPeak = 0;

  data.mergedProfile.fragments.forEach(fragment => {
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        if (operator.commonCounters.MemoryUsage?.sum) {
          const memUsage = operator.commonCounters.MemoryUsage.sum;
          const memPeak = operator.commonCounters.MemoryUsagePeak?.max || '0';
          
          memoryOperators.push({
            name: operator.name,
            id: operator.id,
            memory: memUsage,
            peak: memPeak
          });

          const memValue = parseFloat(memUsage.replace(/[^0-9.]/g, ''));
          const peakValue = parseFloat(memPeak.replace(/[^0-9.]/g, ''));
          if (!isNaN(memValue)) totalMemoryUsage += memValue;
          if (!isNaN(peakValue)) totalMemoryPeak += peakValue;
        }
      });
    });
  });

  console.log('\n💾 内存使用分析');
  console.log('═'.repeat(80));
  console.log(`总内存使用: ~${totalMemoryUsage.toFixed(2)} (需要解析单位)`);
  console.log(`峰值内存: ~${totalMemoryPeak.toFixed(2)} (需要解析单位)`);
  
  console.log('\n内存使用最多的算子（Top 5）:');
  memoryOperators
    .filter(op => parseFloat(op.memory.replace(/[^0-9.]/g, '')) > 0)
    .sort((a, b) => {
      const aVal = parseFloat(a.memory.replace(/[^0-9.]/g, ''));
      const bVal = parseFloat(b.memory.replace(/[^0-9.]/g, ''));
      return bVal - aVal;
    })
    .slice(0, 5)
    .forEach((op, index) => {
      console.log(`${index + 1}. ${op.name.padEnd(30)} 使用: ${op.memory.padStart(12)}, 峰值: ${op.peak}`);
    });
}

function operatorStatistics(data) {
  const operatorCount = new Map();
  let totalOperators = 0;

  data.mergedProfile.fragments.forEach(fragment => {
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        totalOperators++;
        const count = operatorCount.get(operator.name) || 0;
        operatorCount.set(operator.name, count + 1);
      });
    });
  });

  console.log('\n📊 算子类型统计');
  console.log('═'.repeat(80));
  console.log(`总算子数量: ${totalOperators}`);
  console.log(`算子类型数量: ${operatorCount.size}`);
  
  console.log('\n各类型算子统计:');
  Array.from(operatorCount.entries())
    .sort((a, b) => b[1] - a[1])
    .forEach(([name, count]) => {
      const percentage = ((count / totalOperators) * 100).toFixed(1);
      console.log(`${name.padEnd(40)} ${count.toString().padStart(3)} (${percentage.padStart(5)}%)`);
    });
}

// Fragment关系分析
function analyzeFragmentRelationships(data) {
  const fragmentMap = new Map();
  const edges = [];  // 存储Fragment间的连接关系
  
  // 建立Fragment映射
  data.mergedProfile.fragments.forEach(fragment => {
    fragmentMap.set(fragment.id, fragment);
  });
  
  // 分析连接关系
  data.mergedProfile.fragments.forEach(fragment => {
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        // 找到DATA_STREAM_SINK_OPERATOR的dest_id
        if (operator.name === 'DATA_STREAM_SINK_OPERATOR') {
          const destIdMatch = operator.rawData.match(/dest_id=(\d+)/);
          if (destIdMatch) {
            const destId = parseInt(destIdMatch[1], 10);
            
            // 查找对应的EXCHANGE_OPERATOR
            data.mergedProfile.fragments.forEach(targetFragment => {
              targetFragment.pipelines.forEach(targetPipeline => {
                targetPipeline.operators.forEach(targetOperator => {
                  if (targetOperator.name === 'EXCHANGE_OPERATOR' && 
                      targetOperator.id === destId.toString()) {
                    // 修复：DATA_STREAM_SINK_OPERATOR 所在的 Fragment 是源，
                    // EXCHANGE_OPERATOR 所在的 Fragment 是目标
                    // 数据流向：源 Fragment → 目标 Fragment
                    edges.push({
                      from: fragment.id,        // 源 Fragment（有 DATA_STREAM_SINK_OPERATOR）
                      to: targetFragment.id,    // 目标 Fragment（有 EXCHANGE_OPERATOR）
                      exchangeId: destId,
                      operator: operator.name
                    });
                  }
                });
              });
            });
          }
        }
      });
    });
  });
  
  return { fragmentMap, edges };
}

// 构建Fragment树
function buildFragmentTree(relationships) {
  const { fragmentMap, edges } = relationships;
  const children = new Map(); // fragment_id -> [child_fragment_ids]
  const parents = new Map();  // fragment_id -> parent_fragment_id
  
  // 构建父子关系（反转方向，让数据接收者成为父节点）
  edges.forEach(edge => {
    // 反转：edge.to 成为父节点，edge.from 成为子节点
    if (!children.has(edge.to)) {
      children.set(edge.to, []);
    }
    children.get(edge.to).push(edge.from);
    parents.set(edge.from, edge.to);
  });
  
  // 找到根节点（没有父节点的Fragment，通常是最终结果输出者）
  const roots = [];
  for (const fragmentId of fragmentMap.keys()) {
    if (!parents.has(fragmentId)) {
      roots.push(fragmentId);
    }
  }
  
  return { children, parents, roots, edges };
}

// 生成完整的Fragment树JSON结构
function generateFragmentTreeJSON(data) {
  const relationships = analyzeFragmentRelationships(data);
  const tree = buildFragmentTree(relationships);
  const { fragmentMap } = relationships;
  
  // 构建树节点的递归函数
  function buildTreeNode(fragmentId, visited = new Set()) {
    if (visited.has(fragmentId)) {
      return null; // 避免循环引用
    }
    visited.add(fragmentId);
    
    const fragment = fragmentMap.get(fragmentId);
    if (!fragment) return null;
    
    // 计算Fragment统计信息
    const totalOperators = fragment.pipelines.reduce((sum, p) => sum + p.operators.length, 0);
    
    // 算子类型统计
    const operatorStats = {};
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        operatorStats[operator.name] = (operatorStats[operator.name] || 0) + 1;
      });
    });
    
    // 连接信息
    const incomingEdges = tree.edges.filter(e => e.to === fragmentId);
    const outgoingEdges = tree.edges.filter(e => e.from === fragmentId);
    
    // 构建当前节点
    const node = {
      fragmentId: fragmentId,
      pipelines: fragment.pipelines.length,
      totalOperators: totalOperators,
      operatorStats: operatorStats,
      incomingConnections: incomingEdges.map(e => ({
        fromFragment: e.from,
        exchangeId: e.exchangeId
      })),
      outgoingConnections: outgoingEdges.map(e => ({
        toFragment: e.to,
        exchangeId: e.exchangeId
      })),
      pipelineDetails: fragment.pipelines.map(pipeline => ({
        id: pipeline.id,
        instanceNum: pipeline.instanceNum,
        operators: pipeline.operators.map(operator => ({
          id: operator.id,
          name: operator.name,
          planInfo: operator.planInfo || {},
          commonCounters: operator.commonCounters || {},
          customCounters: operator.customCounters || {},
          rawData: operator.rawData
        }))
      })),
      children: []
    };
    
    // 递归构建子节点
    const childFragments = tree.children.get(fragmentId) || [];
    childFragments.forEach(childId => {
      const childNode = buildTreeNode(childId, new Set(visited));
      if (childNode) {
        node.children.push(childNode);
      }
    });
    
    return node;
  }
  
  // 构建完整的树结构
  const treeStructure = {
    // Profile基本信息
    summary: data.summary,
    executionSummary: data.executionSummary,
    
    // Fragment统计信息
    fragmentStats: {
      totalFragments: data.mergedProfile.fragments.length,
      totalConnections: tree.edges.length,
      rootFragments: tree.roots.length
    },
    connections: tree.edges.map(edge => ({
      from: edge.from,
      to: edge.to,
      exchangeId: edge.exchangeId
    })),
    fragmentTree: tree.roots.map(rootId => buildTreeNode(rootId))
  };
  
  return treeStructure;
}

// 显示Fragment树形关系
function displayFragmentTree(data) {
  console.log('\nFragment 树形关系');
  console.log('='.repeat(80));
  
  const relationships = analyzeFragmentRelationships(data);
  const tree = buildFragmentTree(relationships);
  const { fragmentMap } = relationships;
  
  console.log(`总Fragment数量: ${data.mergedProfile.fragments.length}`);
  console.log(`连接关系数量: ${tree.edges.length}`);
  console.log(`根Fragment数量: ${tree.roots.length}`);
  
  // 递归显示树结构
  function displayNode(fragmentId, level = 0, isLast = true, prefix = '') {
    const fragment = fragmentMap.get(fragmentId);
    if (!fragment) return;
    
    const connector = isLast ? '└─' : '├─';
    const nextPrefix = prefix + (isLast ? '   ' : '│  ');
    
    // 计算Fragment统计信息
    const totalOperators = fragment.pipelines.reduce((sum, p) => sum + p.operators.length, 0);
    const pipelineCount = fragment.pipelines.length;
    
    // 找到输入的连接信息
    const incomingEdges = tree.edges.filter(e => e.to === fragmentId);
    const outgoingEdges = tree.edges.filter(e => e.from === fragmentId);
    
    let connectionInfo = '';
    if (incomingEdges.length > 0) {
      const fromFragments = incomingEdges.map(e => `F${e.from}`).join(', ');
      connectionInfo = ` ← [${fromFragments}]`;
    }
    
    console.log(`${prefix}${connector} Fragment ${fragmentId} (${pipelineCount} pipelines, ${totalOperators} operators)${connectionInfo}`);
    
    // 显示主要算子类型
    const operatorTypes = new Map();
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        const count = operatorTypes.get(operator.name) || 0;
        operatorTypes.set(operator.name, count + 1);
      });
    });
    
    // 显示最重要的几个算子类型
    const importantOps = Array.from(operatorTypes.entries())
      .filter(([name]) => !name.includes('LOCAL_EXCHANGE') && !name.includes('DATA_STREAM'))
      .sort((a, b) => b[1] - a[1])
      .slice(0, 3);
    
    if (importantOps.length > 0) {
      const opsStr = importantOps.map(([name, count]) => `${name}(${count})`).join(', ');
      console.log(`${nextPrefix.slice(0, -1)} 主要算子: ${opsStr}`);
    }
    
    // 显示输出连接
    if (outgoingEdges.length > 0) {
      outgoingEdges.forEach((edge, index) => {
        const isLastEdge = index === outgoingEdges.length - 1;
        const edgeConnector = isLastEdge ? '└─' : '├─';
        console.log(`${nextPrefix.slice(0, -1)} ${edgeConnector} Exchange(${edge.exchangeId}) -> Fragment ${edge.to}`);
      });
    }
    
    // 递归显示子节点
    const childFragments = tree.children.get(fragmentId) || [];
    childFragments.forEach((childId, index) => {
      const isLastChild = index === childFragments.length - 1;
      displayNode(childId, level + 1, isLastChild, nextPrefix);
    });
  }
  
  // 显示所有根节点
  if (tree.roots.length === 0) {
    console.log('警告: 未找到根Fragment（可能存在循环依赖）');
    // 显示所有Fragment作为根节点
    data.mergedProfile.fragments.forEach((fragment, index) => {
      const isLast = index === data.mergedProfile.fragments.length - 1;
      displayNode(fragment.id, 0, isLast);
    });
  } else {
    tree.roots.forEach((rootId, index) => {
      const isLast = index === tree.roots.length - 1;
      displayNode(rootId, 0, isLast);
    });
  }
  
  // 显示连接关系总结
  console.log('\n连接关系总结:');
  tree.edges.forEach(edge => {
    console.log(`   Fragment ${edge.from} --[Exchange ${edge.exchangeId}]--> Fragment ${edge.to}`);
  });
}

function generateTree(data) {
  console.log('\n执行树结构');
  console.log('='.repeat(80));
  
  data.mergedProfile.fragments.forEach(fragment => {
    console.log(`Fragment ${fragment.id} (${fragment.pipelines.length} pipelines)`);
    
    fragment.pipelines.forEach(pipeline => {
      console.log(`  ├─ Pipeline ${pipeline.id} (${pipeline.instanceNum} instances, ${pipeline.operators.length} operators)`);
      
      pipeline.operators.forEach((operator, index) => {
        const isLast = index === pipeline.operators.length - 1;
        const prefix = isLast ? '  │  └─' : '  │  ├─';
        const execTime = operator.commonCounters.ExecTime?.avg || 'N/A';
        const inputRows = operator.commonCounters.InputRows?.sum || 'N/A';
        
        console.log(`${prefix} ${operator.name}(${operator.id}) [${execTime}, ${inputRows} rows]`);
      });
    });
    console.log('');
  });
}

// 命令行参数解析
function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    help: false,
    file: null,
    output: null,
    analysis: false,
    memory: false,
    stats: false,
    tree: false,
    fragments: false,
    treeJson: false,
    all: false,
    json: false
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else if (arg === '--output' || arg === '-o') {
      options.output = args[++i];
    } else if (arg === '--analysis' || arg === '-a') {
      options.analysis = true;
    } else if (arg === '--memory' || arg === '-m') {
      options.memory = true;
    } else if (arg === '--stats' || arg === '-s') {
      options.stats = true;
    } else if (arg === '--tree' || arg === '-t') {
      options.tree = true;
    } else if (arg === '--fragments' || arg === '-f') {
      options.fragments = true;
    } else if (arg === '--tree-json') {
      options.treeJson = true;
    } else if (arg === '--all') {
      options.all = true;
    } else if (arg === '--json' || arg === '-j') {
      options.json = true;
    } else if (!arg.startsWith('-')) {
      options.file = arg;
    }
  }

  return options;
}

// 显示帮助信息
function showHelp() {
  console.log(`
🚀 Doris Profile Parser - 独立解析器

用法: node run-parser.js <profile-file> [options]

参数:
  profile-file              Profile文件路径

选项:
  -h, --help               显示帮助信息
  -o, --output <file>      将解析结果保存到JSON文件
  -a, --analysis           显示性能分析（执行时间Top 10）
  -m, --memory             显示内存使用分析
  -s, --stats              显示算子类型统计
  -t, --tree               显示执行树结构
  -f, --fragments          显示Fragment树形关系
  --tree-json              输出完整的Fragment树形JSON结构
  -j, --json               只输出JSON格式（不显示分析）
  --all                    显示所有分析信息

示例:
  node run-parser.js test-profile3.txt
  node run-parser.js test-profile3.txt --output result.json
  node run-parser.js test-profile3.txt --analysis --memory
  node run-parser.js test-profile3.txt --all
  node run-parser.js test-profile3.txt --tree --stats
  node run-parser.js test-profile3.txt --json > result.json

支持的Profile文件格式:
  - Doris Query Profile文本文件
  - 包含Summary、Execution Summary、MergedProfile部分
`);
}

// 主函数
function main() {
  const options = parseArgs();

  if (options.help || !options.file) {
    showHelp();
    process.exit(options.help ? 0 : 1);
  }

  // 检查文件是否存在
  if (!fs.existsSync(options.file)) {
    console.error(`❌ 错误: 文件不存在 '${options.file}'`);
    process.exit(1);
  }

  // 在树形JSON模式下，静默运行
  if (!options.treeJson) {
    console.log(`🔍 解析Profile文件: ${options.file}`);
    console.log('─'.repeat(80));
  }

  try {
    // 读取文件
    const profileContent = fs.readFileSync(options.file, 'utf-8');
    if (!options.treeJson) {
      console.log(`📄 文件大小: ${profileContent.length.toLocaleString()} 字符`);
    }

    // 解析Profile
    const parser = new DorisProfileParser({ debug: false });
    const result = parser.parse(profileContent);

    // 检查解析结果
    if (!result.success) {
      console.error('\n❌ 解析失败:');
      result.errors.forEach((error, index) => {
        console.error(`${index + 1}. [${error.section}:${error.line}] ${error.message}`);
      });
      process.exit(1);
    }

    const data = result.data;

    // 树形JSON输出（优先级最高）
    if (options.treeJson) {
      const treeData = generateFragmentTreeJSON(data);
      console.log(JSON.stringify(treeData, null, 2));
      return; // 只输出JSON，不执行其他操作
    }

    // 只输出JSON格式
    if (options.json) {
      console.log(JSON.stringify(result, null, 2));
      return;
    }

    // 显示基本信息
    console.log('\n✅ 解析成功!');
    console.log('─'.repeat(80));
    console.log(`📋 Profile ID: ${data.summary.profileId}`);
    console.log(`⏱️  总执行时间: ${data.summary.total}`);
    console.log(`👤 用户: ${data.summary.user}`);
    console.log(`🗄️  数据库: ${data.summary.defaultDb}`);
    console.log(`📊 状态: ${data.summary.taskState}`);
    console.log(`🔢 Fragments: ${data.mergedProfile.fragments.length}`);
    
    let totalPipelines = 0;
    let totalOperators = 0;
    data.mergedProfile.fragments.forEach(fragment => {
      totalPipelines += fragment.pipelines.length;
      fragment.pipelines.forEach(pipeline => {
        totalOperators += pipeline.operators.length;
      });
    });
    console.log(`🔧 Pipelines: ${totalPipelines}`);
    console.log(`⚙️  Operators: ${totalOperators}`);

    if (data.summary.sqlStatement) {
      console.log(`\n📝 SQL语句:`);
      console.log(data.summary.sqlStatement.substring(0, 200) + (data.summary.sqlStatement.length > 200 ? '...' : ''));
    }

    // 显示分析结果
    if (options.all || options.analysis) {
      performanceAnalysis(data);
    }

    if (options.all || options.memory) {
      memoryAnalysis(data);
    }

    if (options.all || options.stats) {
      operatorStatistics(data);
    }

    if (options.all || options.tree) {
      generateTree(data);
    }

    if (options.all || options.fragments) {
      displayFragmentTree(data);
    }

    // 保存到文件
    if (options.output) {
      fs.writeFileSync(options.output, JSON.stringify(result, null, 2), 'utf-8');
      console.log(`\n💾 结果已保存到: ${options.output}`);
    }

    console.log('\n🎉 解析完成!');

  } catch (error) {
    console.error(`\n❌ 运行失败: ${error.message}`);
    process.exit(1);
  }
}

// 运行主函数
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { DorisProfileParser, main };
