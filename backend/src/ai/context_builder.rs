use crate::config::ContextConfig;
use crate::models::*;

pub struct ContextBuilder;

impl ContextBuilder {
    pub fn build_context(
        node: &ExecutionTreeNode,
        profile: &Profile,
        config: &ContextConfig,
    ) -> String {
        let mut context = String::new();
        
        // 1. 节点基本信息
        context.push_str("## 节点信息\n");
        context.push_str(&format!("- 操作符: {}\n", node.operator_name));
        context.push_str(&format!("- 节点类型: {:?}\n", node.node_type));
        if let Some(plan_id) = node.plan_node_id {
            context.push_str(&format!("- 计划节点ID: {}\n", plan_id));
        }
        if let Some(time_pct) = node.time_percentage {
            context.push_str(&format!("- 执行时间占比: {:.2}%\n", time_pct));
        }
        if let Some(frag_id) = &node.fragment_id {
            context.push_str(&format!("- Fragment: {}\n", frag_id));
        }
        if let Some(pipe_id) = &node.pipeline_id {
            context.push_str(&format!("- Pipeline: {}\n", pipe_id));
        }
        
        // 2. 性能指标
        context.push_str("\n## 性能指标\n");
        if let Some(rows) = node.metrics.rows_returned {
            context.push_str(&format!("- 返回行数: {}\n", rows));
        }
        if let Some(input_rows) = node.metrics.input_rows {
            context.push_str(&format!("- 输入行数: {}\n", input_rows));
        }
        if let Some(exec_time) = node.metrics.operator_total_time {
            let exec_time_ms = exec_time as f64 / 1_000_000.0;
            context.push_str(&format!("- 执行时间: {:.2}ms\n", exec_time_ms));
        }
        if let Some(mem) = node.metrics.memory_used {
            let mem_mb = mem as f64 / 1_048_576.0;
            context.push_str(&format!("- 内存使用: {:.2}MB\n", mem_mb));
        }
        
        // 3. PlanInfo
        if !node.plan_info.is_empty() {
            context.push_str("\n## 计划信息 (PlanInfo)\n");
            Self::append_metrics(&mut context, &node.plan_info);
        }
        
        // 4. Common Counters
        if !node.common_counters.is_empty() {
            context.push_str("\n## 通用指标 (Common Counters)\n");
            Self::append_metrics(&mut context, &node.common_counters);
        }
        
        // 5. Custom Counters
        if !node.custom_counters.is_empty() {
            context.push_str("\n## 自定义指标 (Custom Counters)\n");
            Self::append_metrics(&mut context, &node.custom_counters);
        }
        
        // 6. SQL 语句
        if config.sql_statement && !profile.summary.sql_statement.is_empty() {
            context.push_str("\n## SQL 语句\n```sql\n");
            context.push_str(&profile.summary.sql_statement);
            context.push_str("\n```\n");
        }
        
        // 7. Query Summary
        if config.query_summary {
            context.push_str("\n## 查询概要\n");
            context.push_str(&format!("- Query ID: {}\n", profile.summary.query_id));
            context.push_str(&format!("- 总执行时间: {}\n", profile.summary.total_time));
            context.push_str(&format!("- 查询状态: {}\n", profile.summary.query_state));
            if let Some(ref user) = profile.summary.user {
                context.push_str(&format!("- 用户: {}\n", user));
            }
            if let Some(ref db) = profile.summary.default_db {
                context.push_str(&format!("- 数据库: {}\n", db));
            }
        }
        
        // 8. 子节点信息
        if config.child_nodes && !node.children.is_empty() {
            context.push_str("\n## 子节点信息\n");
            
            // 从 execution_tree 中查找子节点详情
            if let Some(ref tree) = profile.execution_tree {
                let child_count = node.children.len().min(config.max_child_nodes);
                context.push_str(&format!("该节点有 {} 个子节点", node.children.len()));
                if child_count < node.children.len() {
                    context.push_str(&format!("，以下显示前 {} 个：\n", child_count));
                } else {
                    context.push_str("：\n");
                }
                
                for (idx, child_id) in node.children.iter().take(child_count).enumerate() {
                    if let Some(child_node) = tree.nodes.iter().find(|n| &n.id == child_id) {
                        context.push_str(&format!("\n### 子节点 {}\n", idx + 1));
                        context.push_str(&format!("- 操作符: {}\n", child_node.operator_name));
                        if let Some(time_pct) = child_node.time_percentage {
                            context.push_str(&format!("- 执行时间占比: {:.2}%\n", time_pct));
                        }
                        if let Some(rows) = child_node.metrics.rows_returned {
                            context.push_str(&format!("- 返回行数: {}\n", rows));
                        }
                    }
                }
            }
        }
        
        context
    }
    
    fn append_metrics(context: &mut String, metrics: &[MetricItem]) {
        for metric in metrics {
            Self::append_metric_recursive(context, metric, 0);
        }
    }
    
    fn append_metric_recursive(context: &mut String, metric: &MetricItem, indent: usize) {
        let prefix = "  ".repeat(indent);
        context.push_str(&format!("{}- {}: {}\n", prefix, metric.key, metric.value));
        
        for child in &metric.children {
            Self::append_metric_recursive(context, child, indent + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_basic_context_build() {
        let node = ExecutionTreeNode {
            id: "test-node".to_string(),
            operator_name: "TEST_OPERATOR".to_string(),
            node_type: NodeType::OlapScan,
            plan_node_id: Some(1),
            parent_plan_node_id: None,
            metrics: OperatorMetrics {
                rows_returned: Some(1000),
                input_rows: None,
                operator_total_time: Some(1000000),
                operator_total_time_raw: Some("1ms".to_string()),
                memory_used: Some(0),
                cpu_time: None,
                wait_time: None,
            },
            children: vec![],
            depth: 0,
            is_hotspot: false,
            hotspot_severity: HotspotSeverity::None,
            fragment_id: Some("Fragment 0".to_string()),
            pipeline_id: Some("Pipeline 0".to_string()),
            time_percentage: Some(50.0),
            is_most_consuming: false,
            is_second_most_consuming: false,
            plan_info: vec![],
            common_counters: vec![],
            custom_counters: vec![],
            unique_metrics: HashMap::new(),
            table_name: None,
        };
        
        let profile = Profile {
            summary: ProfileSummary {
                query_id: "test-query".to_string(),
                total_time: "1s".to_string(),
                query_state: "OK".to_string(),
                ..Default::default()
            },
            fragments: vec![],
            execution_tree: None,
        };
        
        let config = ContextConfig {
            sql_statement: false,
            query_summary: true,
            child_nodes: false,
            max_child_nodes: 3,
        };
        
        let context = ContextBuilder::build_context(&node, &profile, &config);
        
        assert!(context.contains("TEST_OPERATOR"));
        assert!(context.contains("50.00%"));
        assert!(context.contains("test-query"));
    }
}

