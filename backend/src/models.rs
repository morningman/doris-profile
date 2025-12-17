use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root profile structure containing all parsed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub summary: ProfileSummary,
    pub fragments: Vec<Fragment>,
    pub execution_tree: Option<ExecutionTree>,
}

/// Summary information about the query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileSummary {
    pub query_id: String,
    pub start_time: String,
    pub end_time: String,
    pub total_time: String,
    pub query_state: String,
    pub doris_version: String,
    pub sql_statement: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_db: Option<String>,
    
    pub variables: HashMap<String, String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_time_ms: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_peak_memory: Option<u64>,
}

/// Execution fragment containing pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    pub id: String,
    pub backend_addresses: Vec<String>,
    pub instance_ids: Vec<String>,
    pub pipelines: Vec<Pipeline>,
}

/// Pipeline within a fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub metrics: HashMap<String, String>,
    pub operators: Vec<Operator>,
}

/// Operator within a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operator {
    pub id: String,
    pub name: String,
    pub metrics: HashMap<String, String>,
}

/// Execution tree for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTree {
    pub root: ExecutionTreeNode,
    pub nodes: Vec<ExecutionTreeNode>,
}

/// Node in the execution tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTreeNode {
    pub id: String,
    pub operator_name: String,
    pub node_type: NodeType,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_node_id: Option<i32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_plan_node_id: Option<i32>,
    
    pub metrics: OperatorMetrics,
    pub children: Vec<String>,
    pub depth: usize,
    pub is_hotspot: bool,
    pub hotspot_severity: HotspotSeverity,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_percentage: Option<f64>,
    
    #[serde(default)]
    pub is_most_consuming: bool,
    
    #[serde(default)]
    pub is_second_most_consuming: bool,
    
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub unique_metrics: HashMap<String, String>,
}

/// Type of execution node
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    OlapScan,
    Exchange,
    HashJoin,
    Aggregate,
    Sort,
    Limit,
    Project,
    Filter,
    Union,
    ResultSink,
    DataStreamSink,
    Unknown,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Unknown
    }
}

/// Metrics for an operator
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperatorMetrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_total_time: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_total_time_raw: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows_returned: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_used: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_time: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_time: Option<u64>,
}

/// Severity level for hotspots
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HotspotSeverity {
    Critical,
    High,
    Medium,
    Low,
    None,
}

impl Default for HotspotSeverity {
    fn default() -> Self {
        HotspotSeverity::None
    }
}

/// Detected performance hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSpot {
    pub node_id: String,
    pub node_path: String,
    pub operator_name: String,
    pub severity: HotspotSeverity,
    pub description: String,
    pub impact: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_percentage: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub priority: SuggestionPriority,
    pub category: SuggestionCategory,
}

/// Priority of a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Category of a suggestion
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Query,
    Schema,
    Resource,
    Configuration,
}

/// API response for profile analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAnalysisResponse {
    pub hotspots: Vec<HotSpot>,
    pub conclusion: String,
    pub suggestions: Vec<Suggestion>,
    pub performance_score: u32,
    pub execution_tree: Option<ExecutionTree>,
    pub summary: Option<ProfileSummary>,
}

