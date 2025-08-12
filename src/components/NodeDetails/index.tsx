import React from 'react';
import { Card } from 'antd';
import type { TreeNode } from '../../types/profile';

interface NodeDetailsProps {
  node: TreeNode;
}

const NodeDetails: React.FC<NodeDetailsProps> = () => {
  return (
    <Card title="Node Details">
      <p>Node details component - To be implemented</p>
    </Card>
  );
};

export default NodeDetails;