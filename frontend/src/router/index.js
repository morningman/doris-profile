import { createRouter, createWebHistory } from "vue-router";
import QueryDashboard from "../views/QueryDashboard.vue";

const routes = [
  {
    path: "/",
    name: "QueryDashboard",
    component: QueryDashboard,
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

export default router;

