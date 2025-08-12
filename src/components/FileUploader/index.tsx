import React from 'react';
import { Upload, Typography, Space } from 'antd';
import { UploadOutlined, FileTextOutlined } from '@ant-design/icons';
import type { UploadProps } from 'antd';

const { Title, Text } = Typography;
const { Dragger } = Upload;

interface FileUploaderProps {
  onFileUpload: (file: File) => void;
}

const FileUploader: React.FC<FileUploaderProps> = ({ onFileUpload }) => {
  const uploadProps: UploadProps = {
    name: 'file',
    multiple: false,
    accept: '.txt,.log,.profile',
    beforeUpload: (file) => {
      onFileUpload(file);
      return false; // Prevent auto upload
    },
    showUploadList: false,
  };

  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        <div>
          <FileTextOutlined style={{ fontSize: '48px', color: '#1890ff' }} />
          <Title level={3} style={{ marginTop: '16px' }}>
            Upload Apache Doris Query Profile
          </Title>
          <Text type="secondary">
            Select or drag a profile file to analyze query execution details
          </Text>
        </div>

        <Dragger
          {...uploadProps}
          style={{
            padding: '40px 20px',
            background: '#fafafa',
            border: '2px dashed #d9d9d9',
            borderRadius: '8px',
          }}
        >
          <p className="ant-upload-drag-icon">
            <UploadOutlined style={{ fontSize: '32px', color: '#1890ff' }} />
          </p>
          <p className="ant-upload-text">
            Click or drag profile file to this area to upload
          </p>
          <p className="ant-upload-hint">
            Supports .txt, .log, .profile files containing Apache Doris query profile data
          </p>
        </Dragger>

        <div style={{ textAlign: 'left', maxWidth: '600px', margin: '0 auto' }}>
          <Title level={5}>Supported File Formats:</Title>
          <ul>
            <li><Text code>.txt</Text> - Plain text profile files</li>
            <li><Text code>.log</Text> - Log files containing profile data</li>
            <li><Text code>.profile</Text> - Dedicated profile files</li>
          </ul>
          
          <Title level={5} style={{ marginTop: '20px' }}>
            What you'll see:
          </Title>
          <ul>
            <li>Query execution summary and timing information</li>
            <li>Interactive tree visualization of execution plan</li>
            <li>Detailed metrics for each operator and fragment</li>
            <li>Performance bottleneck identification</li>
          </ul>
        </div>
      </Space>
    </div>
  );
};

export default FileUploader;