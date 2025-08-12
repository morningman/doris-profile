export interface ProfileSummary {
  profileId: string;
  taskType: string;
  startTime: string;
  endTime: string;
  total: string;
  taskState: string;
  user: string;
  defaultCatalog: string;
  defaultDb: string;
  sqlStatement: string;
  distributedPlan: string;
}

export interface ExecutionSummary {
  workloadGroup: string;
  parseSqlTime: string;
  planTime: string;
  garbageCollectDuringPlanTime?: string;
  nereidsLockTableTime?: string;
  nereidsAnalysisTime?: string;
  nereidsRewriteTime?: string;
  nereidsFoldConstByBeTime?: string;
  nereidsCollectTablePartitionTime?: string;
  nereidsOptimizeTime?: string;
  nereidsTranslateTime?: string;
  initScanNodeTime?: string;
  finalizeScanNodeTime?: string;
  getSplitsTime?: string;
  getPartitionsTime?: string;
}

export interface CounterValue {
  raw: string;      // 原始值，如 "avg 170.233us, max 170.233us, min 170.233us"
  sum?: number;     // 归一化后的数值
  avg?: number;     // 归一化后的数值
  min?: number;     // 归一化后的数值
  max?: number;     // 归一化后的数值
  unit?: string;    // 推断的单位类型：'time'(微秒)、'bytes'、'count'、'unknown'
}

export interface Counter {
  name: string;
  value: CounterValue;
}

export interface OperatorCounters {
  commonCounters: Counter[];
  customCounters: Counter[];
}

export interface PlanInfo {
  [key: string]: string | number;
}

export interface Operator {
  id: string;
  nereidsId?: string;
  name: string;
  type: string;
  planInfo?: PlanInfo;
  counters: OperatorCounters;
  children?: Operator[];
  instanceNum?: number;
  destId?: string;
  rawData?: string;
}

export interface Pipeline {
  id: string;
  instanceNum: number;
  waitWorkerTime?: string;
  operators: Operator[];
}

export interface Fragment {
  id: string;
  pipelines: Pipeline[];
}

export interface FragmentConnection {
  from: string;
  to: string;
  exchangeId: string;
  sinkOperatorId: string;
  exchangeOperatorId: string;
}

export interface ExecutionProfile {
  fragments: Fragment[];
  connections?: FragmentConnection[];
}

export interface DorisQueryProfile {
  summary: ProfileSummary;
  executionSummary: ExecutionSummary;
  executionProfile: ExecutionProfile;
  rawText: string;
}

export interface ParsedProfileData {
  profile: DorisQueryProfile;
  parseTime: number;
  hasErrors: boolean;
  errors?: string[];
}

export interface TreeNode {
  id: string;
  name: string;
  type: 'fragment' | 'pipeline' | 'operator';
  data: Fragment | Pipeline | Operator;
  children: TreeNode[];
  parent?: TreeNode;
  level: number;
}