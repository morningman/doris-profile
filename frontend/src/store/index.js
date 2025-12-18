import { createStore } from "vuex";

export default createStore({
  state: {
    // Current analysis result
    analysisResult: null,
    // Loading state
    isLoading: false,
    // Error message
    error: null,
    // Profile text
    profileText: "",
  },

  getters: {
    hasResult: (state) => state.analysisResult !== null,
    hotspots: (state) => state.analysisResult?.hotspots || [],
    suggestions: (state) => state.analysisResult?.suggestions || [],
    executionTree: (state) => state.analysisResult?.execution_tree || null,
    summary: (state) => state.analysisResult?.summary || null,
    performanceScore: (state) => state.analysisResult?.performance_score || 0,
    conclusion: (state) => state.analysisResult?.conclusion || "",
    profileText: (state) => state.profileText,
  },

  mutations: {
    SET_ANALYSIS_RESULT(state, result) {
      state.analysisResult = result;
    },
    SET_LOADING(state, isLoading) {
      state.isLoading = isLoading;
    },
    SET_ERROR(state, error) {
      state.error = error;
    },
    SET_PROFILE_TEXT(state, text) {
      state.profileText = text;
    },
    CLEAR_RESULT(state) {
      state.analysisResult = null;
      state.error = null;
      state.profileText = "";
    },
    UPDATE_HOTSPOT(state, updatedHotspot) {
      if (state.analysisResult && state.analysisResult.hotspots) {
        const index = state.analysisResult.hotspots.findIndex(
          h => h.node_id === updatedHotspot.node_id
        );
        if (index !== -1) {
          state.analysisResult.hotspots[index] = updatedHotspot;
        }
      }
    },
  },

  actions: {
    async analyzeProfile({ commit }, profileText) {
      commit("SET_LOADING", true);
      commit("SET_ERROR", null);
      commit("SET_PROFILE_TEXT", profileText);

      try {
        const response = await fetch("/api/analyze", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ profile_text: profileText }),
        });

        const data = await response.json();

        if (data.success) {
          commit("SET_ANALYSIS_RESULT", data.data);
        } else {
          commit("SET_ERROR", data.error || "Failed to analyze profile");
        }
      } catch (error) {
        commit("SET_ERROR", error.message || "Network error");
      } finally {
        commit("SET_LOADING", false);
      }
    },

    async analyzeFile({ commit }, file) {
      commit("SET_LOADING", true);
      commit("SET_ERROR", null);

      try {
        // Read file content to store it
        const fileContent = await file.text();
        commit("SET_PROFILE_TEXT", fileContent);
        
        const formData = new FormData();
        formData.append("file", file);

        const response = await fetch("/api/analyze-file", {
          method: "POST",
          body: formData,
        });

        const data = await response.json();

        if (data.success) {
          commit("SET_ANALYSIS_RESULT", data.data);
        } else {
          commit("SET_ERROR", data.error || "Failed to analyze profile");
        }
      } catch (error) {
        commit("SET_ERROR", error.message || "Network error");
      } finally {
        commit("SET_LOADING", false);
      }
    },

    clearResult({ commit }) {
      commit("CLEAR_RESULT");
    },
  },

  modules: {},
});

