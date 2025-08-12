import React from 'react';
import { Card, Descriptions, Typography } from 'antd';
import type { ProfileSummary, ExecutionSummary } from '../../types/profile';

const { Text, Paragraph } = Typography;

interface QuerySummaryProps {
  summary: ProfileSummary;
  executionSummary: ExecutionSummary;
}

const QuerySummary: React.FC<QuerySummaryProps> = ({ summary, executionSummary }) => {
  return (
    <div style={{ display: 'flex', gap: '16px', flexDirection: 'column' }}>
      <Card title="Query Summary" size="small">
        <Descriptions column={2} size="small">
          <Descriptions.Item label="Profile ID">
            <Text code>{summary.profileId}</Text>
          </Descriptions.Item>
          <Descriptions.Item label="Task Type">
            {summary.taskType}
          </Descriptions.Item>
          <Descriptions.Item label="Start Time">
            {summary.startTime}
          </Descriptions.Item>
          <Descriptions.Item label="End Time">
            {summary.endTime}
          </Descriptions.Item>
          <Descriptions.Item label="Total Time">
            <Text strong>{summary.total}</Text>
          </Descriptions.Item>
          <Descriptions.Item label="Task State">
            <Text type={summary.taskState === 'OK' ? 'success' : 'danger'}>
              {summary.taskState}
            </Text>
          </Descriptions.Item>
          <Descriptions.Item label="User">
            {summary.user}
          </Descriptions.Item>
          <Descriptions.Item label="Database">
            {summary.defaultCatalog}.{summary.defaultDb}
          </Descriptions.Item>
        </Descriptions>
        
        {summary.sqlStatement && (
          <div style={{ marginTop: '16px' }}>
            <Text strong>SQL Statement:</Text>
            <Paragraph 
              code 
              style={{ 
                background: '#f5f5f5', 
                padding: '8px', 
                marginTop: '8px',
                whiteSpace: 'pre-wrap'
              }}
            >
              {summary.sqlStatement}
            </Paragraph>
          </div>
        )}
      </Card>

      <Card title="Execution Summary" size="small">
        <Descriptions column={3} size="small">
          <Descriptions.Item label="Workload Group">
            {executionSummary.workloadGroup}
          </Descriptions.Item>
          <Descriptions.Item label="Parse SQL Time">
            {executionSummary.parseSqlTime}
          </Descriptions.Item>
          <Descriptions.Item label="Plan Time">
            {executionSummary.planTime}
          </Descriptions.Item>
        </Descriptions>
      </Card>
    </div>
  );
};

export default QuerySummary;