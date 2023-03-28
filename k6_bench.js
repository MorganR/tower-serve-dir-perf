import http from "k6/http";

export const options = {
  vus: 1,
  duration: "10s",
  summaryTrendStats: [
    "avg",
    "min",
    "med",
    "max",
    "p(50)",
    "p(95)",
    "p(99)",
    "count",
  ],
};

export default function () {
  http.get("http://localhost:8080/serve_dir/scout.webp");
}
