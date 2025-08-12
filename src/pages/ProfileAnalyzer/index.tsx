import React, { useState } from 'react';
import { Layout, Card, Typography, Spin, message } from 'antd';
import FileUploader from '../../components/FileUploader';
import TreeView from '../../components/TreeView';
import NodeDetails from '../../components/NodeDetails';
import { useProfileParser } from '../../hooks/useProfileParser';
import type { ParsedProfileData, TreeNode } from '../../types/profile';

const { Header, Content, Sider } = Layout;
const { Title } = Typography;

const ProfileAnalyzer: React.FC = () => {
  const [profileData, setProfileData] = useState<ParsedProfileData | null>(null);
  const [completeJsonData, setCompleteJsonData] = useState<any | null>(null);
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
  const { parseProfile, generateCompleteJson, isLoading, error } = useProfileParser();

  const handleFileUpload = async (file: File) => {
    try {
      const result = await parseProfile(file);
      if (result) {
        setProfileData(result);
        message.success(`Profile parsed successfully in ${result.parseTime}ms`);
        
        // Also generate complete JSON
        try {
          const completeJson = await generateCompleteJson(file);
          setCompleteJsonData(completeJson);
        } catch (error) {
          console.warn('Failed to generate complete JSON:', error);
        }
      } else {
        message.error(error || 'Failed to parse profile');
      }
    } catch (error) {
      console.error('Error parsing profile:', error);
      message.error('Failed to parse profile file');
    }
  };

  const handleNodeSelect = (node: TreeNode) => {
    setSelectedNode(node);
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#fff', padding: '0 24px' }}>
        <Title level={2} style={{ margin: '16px 0', color: '#1890ff' }}>
          Apache Doris Query Profile Analyzer
        </Title>
      </Header>
      
      <Layout>
        <Content style={{ padding: '24px' }}>
          {!profileData ? (
            <Card>
              <Spin spinning={isLoading}>
                <FileUploader onFileUpload={handleFileUpload} />
              </Spin>
            </Card>
          ) : (
            <div style={{ display: 'flex', gap: '24px', height: '100%' }}>
              <div style={{ flex: 1 }}>
                <TreeView 
                  profile={profileData.profile}
                  completeJsonData={completeJsonData}
                  onNodeSelect={handleNodeSelect}
                />
              </div>
            </div>
          )}
        </Content>
        
        {selectedNode && (
          <Sider width={400} style={{ background: '#fff' }}>
            <NodeDetails node={selectedNode} />
          </Sider>
        )}
      </Layout>
    </Layout>
  );
};

export default ProfileAnalyzer;