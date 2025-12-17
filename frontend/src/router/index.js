import { createRouter, createWebHistory } from "vue-router";
import ProfileAnalyzer from "../views/ProfileAnalyzer.vue";

const routes = [
  {
    path: "/",
    name: "ProfileAnalyzer",
    component: ProfileAnalyzer,
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

export default router;

