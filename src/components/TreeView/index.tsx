import React, { useState } from 'react';
import { Card, Typography, Button, message, Space } from 'antd';
import { CopyOutlined, CheckOutlined } from '@ant-design/icons';
import type { DorisQueryProfile, TreeNode } from '../../types/profile';

const { Text, Paragraph, Title } = Typography;

interface TreeViewProps {
  profile: DorisQueryProfile;
  completeJsonData?: any;
  onNodeSelect: (node: TreeNode) => void;
}

const TreeView: React.FC<TreeViewProps> = ({ profile, completeJsonData }) => {
  const [copiedSection, setCopiedSection] = useState<string | null>(null);

  const handleCopy = async (content: string, section: string) => {
    try {
      await navigator.clipboard.writeText(content);
      setCopiedSection(section);
      message.success(`${section} 已复制到剪贴板`);
      setTimeout(() => setCopiedSection(null), 2000);
    } catch (error) {
      message.error('复制失败');
    }
  };

  const formatJson = (obj: any) => JSON.stringify(obj, null, 2);

  const fullProfileJson = formatJson(profile);
  const completeJsonStr = completeJsonData ? formatJson(completeJsonData) : null;

  return (
    <Card size="small">
      <Title level={4} style={{ marginBottom: '16px' }}>
        Profile JSON 数据
      </Title>
      
      {/* Only show Complete JSON if available, otherwise show Full Profile */}
      {completeJsonStr ? (
        <div>
          <Space align="center" style={{ marginBottom: '8px' }}>
            <Text strong style={{ fontSize: '16px', color: '#1890ff' }}>
              完整 Profile JSON
            </Text>
            <Button
              type="primary"
              size="small"
              icon={copiedSection === 'complete' ? <CheckOutlined /> : <CopyOutlined />}
              onClick={() => handleCopy(completeJsonStr, 'complete')}
              style={{ marginLeft: '8px' }}
            >
              {copiedSection === 'complete' ? '已复制' : '复制 JSON'}
            </Button>
          </Space>
          
          <Card 
            size="small" 
            style={{ 
              background: '#f6ffed',
              border: '1px solid #b7eb8f'
            }}
          >
            <Paragraph 
              code 
              style={{ 
                background: '#f6ffed', 
                padding: '16px', 
                margin: 0,
                whiteSpace: 'pre-wrap',
                maxHeight: '800px',
                overflow: 'auto',
                fontSize: '12px',
                lineHeight: '1.4',
                fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
                textAlign: 'left',
                direction: 'ltr'
              }}
            >
              {completeJsonStr}
            </Paragraph>
          </Card>
        </div>
      ) : (
        <div>
          <Space align="center" style={{ marginBottom: '8px' }}>
            <Text strong style={{ fontSize: '16px' }}>
              Profile JSON
            </Text>
            <Button
              type="primary"
              size="small"
              icon={copiedSection === 'full' ? <CheckOutlined /> : <CopyOutlined />}
              onClick={() => handleCopy(fullProfileJson, 'full')}
              style={{ marginLeft: '8px' }}
            >
              {copiedSection === 'full' ? '已复制' : '复制 JSON'}
            </Button>
          </Space>
          
          <Card 
            size="small" 
            style={{ 
              background: '#fafafa',
              border: '1px solid #d9d9d9'
            }}
          >
            <Paragraph 
              code 
              style={{ 
                background: '#f0f0f0', 
                padding: '16px', 
                margin: 0,
                whiteSpace: 'pre-wrap',
                maxHeight: '800px',
                overflow: 'auto',
                fontSize: '12px',
                lineHeight: '1.4',
                fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
                textAlign: 'left',
                direction: 'ltr'
              }}
            >
              {fullProfileJson}
            </Paragraph>
          </Card>
        </div>
      )}
    </Card>
  );
};

export default TreeView;