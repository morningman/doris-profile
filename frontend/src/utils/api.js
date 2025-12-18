import axios from "axios";

// Create axios instance with default config
const api = axios.create({
  baseURL: "",
  timeout: 60000,
  headers: {
    "Content-Type": "application/json",
  },
});

// API functions
export const analyzeProfile = async (profileText) => {
  const response = await api.post("/api/analyze", {
    profile_text: profileText,
  });
  return response.data;
};

export const analyzeProfileFile = async (file) => {
  const formData = new FormData();
  formData.append("file", file);

  const response = await api.post("/api/analyze-file", formData, {
    headers: {
      "Content-Type": "multipart/form-data",
    },
  });
  return response.data;
};

export const checkHealth = async () => {
  const response = await api.get("/health");
  return response.data;
};

export const diagnoseNode = async (profileText, nodeId, language = 'en') => {
  const response = await api.post("/api/diagnose-node", {
    profile_text: profileText,
    node_id: nodeId,
    language: language,
  }, {
    timeout: 60000,  // 60 seconds timeout for AI requests
  });
  return response.data;
};

export default api;

