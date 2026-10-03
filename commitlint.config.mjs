import { readFileSync } from "node:fs";

const event = process.env.GITHUB_EVENT_PATH
  ? JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, "utf8"))
  : {};
const isDependabotPullRequest =
  process.env.GITHUB_EVENT_NAME === "pull_request" &&
  event.pull_request?.user?.login === "dependabot[bot]" &&
  event.pull_request?.user?.type === "Bot";

export default {
  extends: ["@commitlint/config-conventional"],
  rules: {
    // Dependabot's generated release prose can contain long lines.
    // Keep every other conventional rule and the human body limit intact.
    "body-max-line-length": isDependabotPullRequest
      ? [0, "always", 100]
      : [2, "always", 100],
  },
};
