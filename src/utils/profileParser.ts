import type {
  ProfileSummary,
  ExecutionSummary,
  Fragment,
  Pipeline,
  Operator,
  Counter,
  CounterValue,
  DorisQueryProfile,
  ParsedProfileData,
  FragmentConnection,
} from '../types/profile';

export class ProfileParser {
  parse(profileText: string): ParsedProfileData {
    const startTime = Date.now();
    
    try {
      const sections = this.splitProfileSections(profileText);
      
      const summary = this.parseSummarySection(sections.summary);
      const executionSummary = this.parseExecutionSummarySection(sections.executionSummary);
      const executionProfile = this.parseMergedProfileSection(sections.mergedProfile);

      const profile: DorisQueryProfile = {
        summary,
        executionSummary,
        executionProfile,
        rawText: profileText,
      };

      return {
        profile,
        parseTime: Date.now() - startTime,
        hasErrors: false,
      };
    } catch (error) {
      return {
        profile: {} as DorisQueryProfile,
        parseTime: Date.now() - startTime,
        hasErrors: true,
        errors: [error instanceof Error ? error.message : String(error)],
      };
    }
  }

  private splitProfileSections(profileText: string) {
    const lines = profileText.split('\n');
    let summaryStart = -1;
    let executionSummaryStart = -1;
    let changedSessionVariablesStart = -1;
    let mergedProfileStart = -1;
    let detailProfileStart = -1;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (line === 'Summary:') {
        summaryStart = i;
      } else if (line === 'Execution Summary:' || line === 'Execution  Summary:') {
        executionSummaryStart = i;
      } else if (line === 'ChangedSessionVariables:') {
        changedSessionVariablesStart = i;
      } else if (line === 'MergedProfile:') {
        mergedProfileStart = i;
      } else if (line.startsWith('DetailProfile(')) {
        detailProfileStart = i;
        break;
      }
    }

    const summaryEnd = executionSummaryStart > 0 ? executionSummaryStart : lines.length;
    
    // Execution Summary ends at ChangedSessionVariables or MergedProfile
    let executionSummaryEnd = lines.length;
    if (changedSessionVariablesStart > 0) {
      executionSummaryEnd = changedSessionVariablesStart;
    } else if (mergedProfileStart > 0) {
      executionSummaryEnd = mergedProfileStart;
    }
    
    const mergedProfileEnd = detailProfileStart > 0 ? detailProfileStart : lines.length;

    return {
      summary: summaryStart >= 0 ? lines.slice(summaryStart, summaryEnd).join('\n') : '',
      executionSummary: executionSummaryStart >= 0 ? lines.slice(executionSummaryStart, executionSummaryEnd).join('\n') : '',
      mergedProfile: mergedProfileStart >= 0 ? lines.slice(mergedProfileStart, mergedProfileEnd).join('\n') : ''
    };
  }

  private parseSummarySection(summaryText: string): ProfileSummary {
    const lines = summaryText.split('\n');
    const summary: Partial<ProfileSummary> = {};
    let sqlStatement = '';
    let inSqlStatement = false;

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      if (trimmedLine.startsWith('- Profile ID:')) {
        summary.profileId = this.extractValue(trimmedLine, 'Profile ID:');
      } else if (trimmedLine.startsWith('- Task Type:')) {
        summary.taskType = this.extractValue(trimmedLine, 'Task Type:');
      } else if (trimmedLine.startsWith('- Start Time:')) {
        summary.startTime = this.extractValue(trimmedLine, 'Start Time:');
      } else if (trimmedLine.startsWith('- End Time:')) {
        summary.endTime = this.extractValue(trimmedLine, 'End Time:');
      } else if (trimmedLine.startsWith('- Total:')) {
        summary.total = this.extractValue(trimmedLine, 'Total:');
      } else if (trimmedLine.startsWith('- Task State:')) {
        summary.taskState = this.extractValue(trimmedLine, 'Task State:');
      } else if (trimmedLine.startsWith('- User:')) {
        summary.user = this.extractValue(trimmedLine, 'User:');
      } else if (trimmedLine.startsWith('- Default Catalog:')) {
        summary.defaultCatalog = this.extractValue(trimmedLine, 'Default Catalog:');
      } else if (trimmedLine.startsWith('- Default Db:')) {
        summary.defaultDb = this.extractValue(trimmedLine, 'Default Db:');
      } else if (trimmedLine.startsWith('- Sql Statement:')) {
        inSqlStatement = true;
        sqlStatement = this.extractValue(trimmedLine, 'Sql Statement:');
      } else if (trimmedLine.startsWith('- Distributed Plan:')) {
        inSqlStatement = false;
        summary.distributedPlan = this.extractValue(trimmedLine, 'Distributed Plan:');
      } else if (inSqlStatement) {
        sqlStatement += ' ' + trimmedLine;
      }
    }

    summary.sqlStatement = sqlStatement.trim();
    return summary as ProfileSummary;
  }

  private parseExecutionSummarySection(executionSummaryText: string): ExecutionSummary {
    const lines = executionSummaryText.split('\n');
    const executionSummary: Partial<ExecutionSummary> = {};

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      // 解析所有层级的项目：- 项目名称: 值
      if (trimmedLine.startsWith('- ')) {
        const match = trimmedLine.match(/^-\s+([^:]+):\s*(.*)$/);
        if (match) {
          const key = match[1].trim().replace(/\s+/g, '');
          const value = match[2].trim();
          (executionSummary as any)[key] = value;
        }
      }
    }

    return executionSummary as ExecutionSummary;
  }

  private parseMergedProfileSection(mergedProfileText: string) {
    const lines = mergedProfileText.split('\n');
    const fragments: Fragment[] = [];
    
    let currentFragment: Fragment | null = null;
    let currentPipeline: Pipeline | null = null;
    let currentOperator: Operator | null = null;
    let inCommonCounters = false;
    let inCustomCounters = false;
    let inPlanInfo = false;

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i];
      const trimmedLine = line.trim();

      if (trimmedLine === '') continue;

      if (trimmedLine.startsWith('Fragment ')) {
        // Save previous operator/pipeline/fragment
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
        };
        currentPipeline = null;
        currentOperator = null;
        inCommonCounters = false;
        inCustomCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine.startsWith('Pipeline ')) {
        // Save previous operator/pipeline
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
        };
        currentOperator = null;
        inCommonCounters = false;
        inCustomCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine.startsWith('- WaitWorkerTime:')) {
        if (currentPipeline) {
          currentPipeline.waitWorkerTime = this.extractValue(trimmedLine, 'WaitWorkerTime:');
        }
      } else if (this.isOperatorLine(trimmedLine)) {
        // Save previous operator
        if (currentOperator && currentPipeline) {
          currentPipeline.operators.push(currentOperator);
        }
        
        currentOperator = this.parseOperator(trimmedLine, line);
        inCommonCounters = false;
        inCustomCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine === '- PlanInfo' || trimmedLine.startsWith('- PlanInfo') ||
               line.match(/^\s+- PlanInfo\s*$/)) {
        inPlanInfo = true;
        inCommonCounters = false;
        inCustomCounters = false;
      } else if (trimmedLine === 'CommonCounters:' || line.match(/^\s+CommonCounters:\s*$/)) {
        inCommonCounters = true;
        inCustomCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine === 'CustomCounters:' || line.match(/^\s+CustomCounters:\s*$/)) {
        inCustomCounters = true;
        inCommonCounters = false;
        inPlanInfo = false;
      } else if (trimmedLine.startsWith('- ') && currentOperator) {
        if (inPlanInfo) {
          this.parsePlanInfoItem(currentOperator, trimmedLine);
        } else if (inCommonCounters) {
          this.parseCounterItem(currentOperator, trimmedLine, 'common');
        } else if (inCustomCounters) {
          this.parseCounterItem(currentOperator, trimmedLine, 'custom');
        }
      } else if (line.match(/^\s{10,}-\s+/) && currentOperator) {
        const nestedTrimmedLine = line.trim();
        if (inCommonCounters) {
          this.parseCounterItem(currentOperator, nestedTrimmedLine, 'common');
        } else if (inCustomCounters) {
          this.parseCounterItem(currentOperator, nestedTrimmedLine, 'custom');
        }
      }
    }

    // Save final elements
    if (currentOperator && currentPipeline) {
      currentPipeline.operators.push(currentOperator);
    }
    if (currentPipeline && currentFragment) {
      currentFragment.pipelines.push(currentPipeline);
    }
    if (currentFragment) {
      fragments.push(currentFragment);
    }

    const connections = this.analyzeFragmentConnections(fragments);
    return { fragments, connections };
  }

  private extractValue(line: string, key: string): string {
    const index = line.indexOf(key);
    if (index >= 0) {
      return line.substring(index + key.length).trim();
    }
    return '';
  }

  private extractFragmentId(line: string): string {
    const match = line.match(/Fragment\s+(\d+):/);
    return match ? match[1] : '0';
  }

  private extractPipelineInfo(line: string) {
    const match = line.match(/Pipeline\s+(\d+)\(instance_num=(\d+)\):/);
    if (match) {
      return {
        id: match[1],
        instanceNum: parseInt(match[2], 10)
      };
    }
    return { id: '0', instanceNum: 1 };
  }

  private isOperatorLine(line: string): boolean {
    // Support standard format and FILE_SCAN_OPERATOR special format
    return /^[A-Z_]+(?:\([^)]*\))*(?:\(id=[-\d]+\))?:/.test(line) ||
           /^[A-Z_]+\s+\(id=\d+\.\s+nereids_id=\d+\.\s+table\s+name\s*=\s*[^)]+\):/.test(line) ||
           /^[A-Z_]+_SINK_OPERATOR\(dest_id=\d+\):/.test(line);
  }

  private parseOperator(trimmedLine: string, originalLine: string): Operator {
    // Handle FILE_SCAN_OPERATOR special format
    const fileScanMatch = trimmedLine.match(/^([A-Z_]+)\s+\(id=(\d+)\.\s+nereids_id=(\d+)\.\s+table\s+name\s*=\s*([^)]+)\):/);
    if (fileScanMatch) {
      const operatorName = fileScanMatch[1];
      const id = fileScanMatch[2];
      const nereidsId = fileScanMatch[3];
      const tableName = fileScanMatch[4].trim();

      return {
        id,
        name: operatorName,
        nereidsId,
        type: 'operator',
        planInfo: { 'table_name': tableName },
        counters: {
          commonCounters: [],
          customCounters: [],
        },
        rawData: originalLine.trim(),
      };
    }

    // Handle DATA_STREAM_SINK_OPERATOR with dest_id
    const sinkMatch = trimmedLine.match(/^([A-Z_]+_SINK_OPERATOR)\(dest_id=(\d+)\):/);
    if (sinkMatch) {
      return {
        id: 'sink_' + sinkMatch[2], // Generate unique id for sink operator
        name: sinkMatch[1],
        type: 'operator',
        destId: sinkMatch[2],
        planInfo: {},
        counters: {
          commonCounters: [],
          customCounters: [],
        },
        rawData: originalLine.trim(),
      };
    }

    // Handle EXCHANGE_OPERATOR(id=X): format
    const exchangeMatch = trimmedLine.match(/^([A-Z_]+)\(id=([-\d]+)\):$/);
    if (exchangeMatch) {
      return {
        id: exchangeMatch[2],
        name: exchangeMatch[1],
        type: 'operator',
        planInfo: {},
        counters: {
          commonCounters: [],
          customCounters: [],
        },
        rawData: originalLine.trim(),
      };
    }

    // Standard format parsing
    const match = trimmedLine.match(/^([A-Z_]+(?:\([^)]*\))*)(\(id=([-\d]+)\))?:/);
    if (match) {
      const fullName = match[1];
      const id = match[3] || '0';
      
      const nereidsMatch = fullName.match(/\(nereids_id=(\d+)\)/);
      const nereidsId = nereidsMatch ? nereidsMatch[1] : undefined;
      
      const operatorName = fullName.replace(/\([^)]*\)/g, '');

      return {
        id,
        name: operatorName,
        nereidsId,
        type: 'operator',
        planInfo: {},
        counters: {
          commonCounters: [],
          customCounters: [],
        },
        rawData: originalLine.trim(),
      };
    }

    const specialMatch = trimmedLine.match(/^([A-Z_]+\([^)]*\))(\(id=([-\d]+)\))?:/);
    if (specialMatch) {
      return {
        id: specialMatch[3] || '0',
        name: specialMatch[1],
        type: 'operator',
        planInfo: {},
        counters: {
          commonCounters: [],
          customCounters: [],
        },
        rawData: originalLine.trim(),
      };
    }

    const defaultMatch = trimmedLine.match(/^([^(]+)/);
    return {
      id: 'unknown',
      name: defaultMatch ? defaultMatch[1].replace(':', '') : 'UNKNOWN_OPERATOR',
      type: 'operator',
      planInfo: {},
      counters: {
        commonCounters: [],
        customCounters: [],
      },
      rawData: originalLine.trim(),
    };
  }

  private parsePlanInfoItem(operator: Operator, line: string) {
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

  private parseCounterItem(operator: Operator, line: string, type: 'common' | 'custom') {
    const match = line.match(/^-\s+([^:]+):\s*(.*)$/);
    if (match) {
      const key = match[1].trim();
      const valueText = match[2].trim();
      
      const counter: Counter = {
        name: key,
        value: this.parseAndNormalizeMetricValue(valueText, key),
      };

      if (type === 'common') {
        operator.counters.commonCounters.push(counter);
      } else {
        operator.counters.customCounters.push(counter);
      }
    }
  }

  private parseAndNormalizeMetricValue(valueText: string, fieldName: string): CounterValue {
    const result: CounterValue = {
      raw: valueText,
      unit: this.detectUnit(fieldName)
    };

    // Extract sum, avg, max, min values using regex
    const patterns = [
      { key: 'sum', pattern: /sum\s+([^,]+?)(?:\s*,|\s*$)/i },
      { key: 'avg', pattern: /avg\s+([^,]+?)(?:\s*,|\s*$)/i },
      { key: 'max', pattern: /max\s+([^,]+?)(?:\s*,|\s*$)/i },
      { key: 'min', pattern: /min\s+([^,]+?)(?:\s*,|\s*$)/i }
    ];

    for (const { key, pattern } of patterns) {
      const match = valueText.match(pattern);
      if (match) {
        const rawValue = match[1].trim();
        (result as any)[key] = this.normalizeValue(rawValue, result.unit!);
      }
    }

    // If no specific patterns found, try to parse as a single value
    if (!result.sum && !result.avg && !result.max && !result.min) {
      const normalized = this.normalizeValue(valueText.trim(), result.unit!);
      if (normalized !== null) {
        result.avg = normalized;
      }
    }

    return result;
  }

  private detectUnit(fieldName: string): string {
    const lowerField = fieldName.toLowerCase();
    
    if (lowerField.includes('time') || lowerField.includes('latency') || lowerField.includes('duration')) {
      return 'time';
    }
    
    if (lowerField.includes('bytes') || lowerField.includes('memory') || lowerField.includes('usage') && 
        (lowerField.includes('mb') || lowerField.includes('kb') || lowerField.includes('gb'))) {
      return 'bytes';
    }
    
    if (lowerField.includes('rows') || lowerField.includes('count') || lowerField.includes('produced') || 
        lowerField.includes('blocks') || lowerField.includes('input')) {
      return 'count';
    }
    
    return 'unknown';
  }

  private normalizeValue(valueStr: string, unit: string): number | null {
    if (!valueStr || typeof valueStr !== 'string') return null;
    
    // Clean the string
    let cleanStr = valueStr.replace(/[,\s]+$/, '').trim();
    
    switch (unit) {
      case 'time':
        return this.normalizeTimeToMicroseconds(cleanStr);
      case 'bytes':
        return this.normalizeDataSizeToBytes(cleanStr);
      case 'count':
        return this.normalizeCountToNumber(cleanStr);
      default:
        // Try to extract a number
        const numberMatch = cleanStr.match(/^([0-9.]+)/);
        return numberMatch ? parseFloat(numberMatch[1]) : null;
    }
  }

  private normalizeTimeToMicroseconds(timeStr: string): number {
    if (!timeStr || typeof timeStr !== 'string') return 0;
    
    // Handle composite time formats like "3sec846ms"
    const complexMatch = timeStr.match(/(\d+)sec(\d+)ms/);
    if (complexMatch) {
      const seconds = parseInt(complexMatch[1]);
      const milliseconds = parseInt(complexMatch[2]);
      return (seconds * 1000 + milliseconds) * 1000; // Convert to microseconds
    }
    
    // Handle simple time formats
    const simpleMatch = timeStr.match(/([0-9.]+)(ns|us|ms|sec|s)/);
    if (simpleMatch) {
      const value = parseFloat(simpleMatch[1]);
      const unit = simpleMatch[2];
      
      switch (unit) {
        case 'ns': return Math.round(value / 1000); // nanoseconds to microseconds
        case 'us': return Math.round(value); // already microseconds
        case 'ms': return Math.round(value * 1000); // milliseconds to microseconds
        case 'sec':
        case 's': return Math.round(value * 1000000); // seconds to microseconds
        default: return 0;
      }
    }
    
    return 0;
  }

  private normalizeDataSizeToBytes(sizeStr: string): number {
    if (!sizeStr || typeof sizeStr !== 'string') return 0;
    
    // Handle parentheses with exact numbers like "59.859K (59859)"
    const bracketMatch = sizeStr.match(/\((\d+)\)/);
    if (bracketMatch) {
      return parseInt(bracketMatch[1]);
    }
    
    // Handle size units
    const sizeMatch = sizeStr.match(/([0-9.]+)\s*([KMGTPE]?B|[KMGTPE])/i);
    if (sizeMatch) {
      const value = parseFloat(sizeMatch[1]);
      const unit = sizeMatch[2].toUpperCase();
      
      switch (unit) {
        case 'B': return Math.round(value);
        case 'KB': case 'K': return Math.round(value * 1024);
        case 'MB': case 'M': return Math.round(value * 1024 * 1024);
        case 'GB': case 'G': return Math.round(value * 1024 * 1024 * 1024);
        case 'TB': case 'T': return Math.round(value * 1024 * 1024 * 1024 * 1024);
        case 'PB': case 'P': return Math.round(value * 1024 * 1024 * 1024 * 1024 * 1024);
        case 'EB': case 'E': return Math.round(value * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);
        default: return Math.round(value);
      }
    }
    
    return 0;
  }

  private normalizeCountToNumber(countStr: string): number {
    if (!countStr || typeof countStr !== 'string') return 0;
    
    // Handle parentheses with exact numbers like "1.131041M (1131041)"
    const bracketMatch = countStr.match(/\((\d+)\)/);
    if (bracketMatch) {
      return parseInt(bracketMatch[1]);
    }
    
    // Handle count units like 1.5K, 2.3M, etc.
    const countMatch = countStr.match(/([0-9.]+)\s*([KMGTPE])/i);
    if (countMatch) {
      const value = parseFloat(countMatch[1]);
      const unit = countMatch[2].toUpperCase();
      
      switch (unit) {
        case 'K': return Math.round(value * 1000);
        case 'M': return Math.round(value * 1000000);
        case 'G': return Math.round(value * 1000000000);
        case 'T': return Math.round(value * 1000000000000);
        case 'P': return Math.round(value * 1000000000000000);
        case 'E': return Math.round(value * 1000000000000000000);
        default: return Math.round(value);
      }
    }
    
    // Try to parse as a plain number
    const numberMatch = countStr.match(/^([0-9.]+)/);
    return numberMatch ? Math.round(parseFloat(numberMatch[1])) : 0;
  }

  private analyzeFragmentConnections(fragments: Fragment[]): FragmentConnection[] {
    const connections: FragmentConnection[] = [];
    
    console.log('🔍 DEBUG: Starting connection analysis with', fragments.length, 'fragments');
    
    // Find all SINK operators with dest_id
    fragments.forEach(sourceFragment => {
      sourceFragment.pipelines.forEach(pipeline => {
        pipeline.operators.forEach(operator => {
          // Debug: log all operators to see what we have
          if (operator.name.includes('SINK_OPERATOR') || operator.name.includes('EXCHANGE_OPERATOR')) {
            console.log(`🔧 DEBUG: Found operator "${operator.name}" with id "${operator.id}" in Fragment ${sourceFragment.id}`);
            console.log(`   rawData: "${operator.rawData}"`);
          }
          
          // Look for DATA_STREAM_SINK_OPERATOR with dest_id using rawData
          if (operator.name === 'DATA_STREAM_SINK_OPERATOR') {
            console.log(`💡 DEBUG: Found SINK operator in Fragment ${sourceFragment.id}:`, {
              name: operator.name,
              id: operator.id,
              rawData: operator.rawData
            });
            
            if (operator.rawData) {
              const destIdMatch = operator.rawData.match(/dest_id=(\d+)/);
              if (destIdMatch) {
                const destId = destIdMatch[1];
                console.log(`🎯 DEBUG: Extracted destId: ${destId}`);
                
                // Find the corresponding EXCHANGE_OPERATOR in any fragment
                let found = false;
                fragments.forEach(targetFragment => {
                  targetFragment.pipelines.forEach(targetPipeline => {
                    targetPipeline.operators.forEach(targetOperator => {
                      if (targetOperator.name === 'EXCHANGE_OPERATOR') {
                        console.log(`🔍 DEBUG: Checking EXCHANGE_OPERATOR with id "${targetOperator.id}" against destId "${destId}"`);
                        
                        if (targetOperator.id === destId) {
                          console.log(`✅ DEBUG: Found matching EXCHANGE_OPERATOR! Creating connection ${sourceFragment.id} -> ${targetFragment.id}`);
                          found = true;
                          
                          connections.push({
                            from: sourceFragment.id,
                            to: targetFragment.id,
                            exchangeId: destId,
                            sinkOperatorId: operator.id,
                            exchangeOperatorId: targetOperator.id,
                          });
                        }
                      }
                    });
                  });
                });
                
                if (!found) {
                  console.log(`❌ DEBUG: No matching EXCHANGE_OPERATOR found for destId: ${destId}`);
                }
              } else {
                console.log(`❌ DEBUG: No destId match found in rawData: "${operator.rawData}"`);
              }
            } else {
              console.log(`❌ DEBUG: No rawData found for SINK operator`);
            }
          }
        });
      });
    });

    console.log(`🎉 DEBUG: Found ${connections.length} connections:`, connections);
    return connections;
  }

  /**
   * Generate a comprehensive JSON structure with all profile information
   * including summary, execution summary, and fragment tree
   */
  public generateCompleteJson(profileText: string) {
    const parseResult = this.parse(profileText);
    if (parseResult.hasErrors) {
      throw new Error('Failed to parse profile: ' + parseResult.errors?.join(', '));
    }

    const profile = parseResult.profile;
    const fragments = profile.executionProfile.fragments;
    const connections = profile.executionProfile.connections || [];

    // Build fragment tree structure
    const fragmentTree = this.buildFragmentTreeStructure(fragments, connections);

    return {
      // Profile basic information
      summary: profile.summary,
      executionSummary: profile.executionSummary,
      
      // Fragment statistics
      fragmentStats: {
        totalFragments: fragments.length,
        totalConnections: connections.length,
        rootFragments: fragmentTree.roots.length
      },
      
      // Connections
      connections: connections.map(conn => ({
        from: conn.from,
        to: conn.to,
        exchangeId: conn.exchangeId
      })),
      
      // Fragment tree with detailed information
      fragmentTree: fragmentTree.roots.map(rootId => 
        this.buildTreeNode(rootId, fragments, connections, fragmentTree)
      )
    };
  }

  private buildFragmentTreeStructure(fragments: Fragment[], connections: FragmentConnection[]) {
    const fragmentMap = new Map<string, Fragment>();
    fragments.forEach(fragment => {
      fragmentMap.set(fragment.id, fragment);
    });

    const children = new Map<string, string[]>();
    const parents = new Map<string, string>();

    // Build parent-child relationships
    connections.forEach(conn => {
      if (!children.has(conn.to)) {
        children.set(conn.to, []);
      }
      children.get(conn.to)!.push(conn.from);
      parents.set(conn.from, conn.to);
    });

    // Find root fragments (no parents)
    const roots: string[] = [];
    for (const fragmentId of fragmentMap.keys()) {
      if (!parents.has(fragmentId)) {
        roots.push(fragmentId);
      }
    }

    return { children, parents, roots };
  }

  private buildTreeNode(
    fragmentId: string, 
    fragments: Fragment[], 
    connections: FragmentConnection[],
    treeStructure: { children: Map<string, string[]>, parents: Map<string, string>, roots: string[] },
    visited = new Set<string>()
  ): any {
    if (visited.has(fragmentId)) {
      return null; // Avoid circular references
    }
    visited.add(fragmentId);

    const fragment = fragments.find(f => f.id === fragmentId);
    if (!fragment) return null;

    // Calculate fragment statistics
    const totalOperators = fragment.pipelines.reduce((sum, p) => sum + p.operators.length, 0);

    // Operator type statistics
    const operatorStats: { [key: string]: number } = {};
    fragment.pipelines.forEach(pipeline => {
      pipeline.operators.forEach(operator => {
        operatorStats[operator.name] = (operatorStats[operator.name] || 0) + 1;
      });
    });

    // Connection information
    const incomingConnections = connections.filter(c => c.to === fragmentId);
    const outgoingConnections = connections.filter(c => c.from === fragmentId);

    // Build current node
    const node = {
      fragmentId,
      pipelines: fragment.pipelines.length,
      totalOperators,
      operatorStats,
      incomingConnections: incomingConnections.map(c => ({
        fromFragment: c.from,
        exchangeId: c.exchangeId
      })),
      outgoingConnections: outgoingConnections.map(c => ({
        toFragment: c.to,
        exchangeId: c.exchangeId
      })),
      pipelineDetails: fragment.pipelines.map(pipeline => ({
        id: pipeline.id,
        instanceNum: pipeline.instanceNum,
        operators: pipeline.operators.map(operator => ({
          id: operator.id,
          name: operator.name,
          planInfo: operator.planInfo || {},
          commonCounters: operator.counters.commonCounters.reduce((acc, counter) => {
            acc[counter.name] = counter.value;
            return acc;
          }, {} as any),
          customCounters: operator.counters.customCounters.reduce((acc, counter) => {
            acc[counter.name] = counter.value;
            return acc;
          }, {} as any),
          rawData: operator.rawData
        }))
      })),
      children: [] as any[]
    };

    // Recursively build child nodes
    const childFragments = treeStructure.children.get(fragmentId) || [];
    childFragments.forEach(childId => {
      const childNode = this.buildTreeNode(childId, fragments, connections, treeStructure, new Set(visited));
      if (childNode) {
        node.children.push(childNode);
      }
    });

    return node;
  }
}