import { readFileSync } from "node:fs";

const [baselinePath, currentPath, metric, maxRatio, maxAbsDeltaArg] = process.argv.slice(2);
if (!baselinePath || !currentPath || !metric || !maxRatio) {
  console.error(
    "usage: node compare-metric.mjs <baseline.json> <current.json> <metric> <max_ratio> [max_abs_delta]",
  );
  process.exit(2);
}

const baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
const current = JSON.parse(readFileSync(currentPath, "utf8"));
const b = baseline[metric];
const c = current[metric];

if (typeof b !== "number" || typeof c !== "number") {
  console.error(`Metric ${metric} missing or not numeric.`);
  process.exit(2);
}
if (b <= 0) {
  console.error(`Baseline metric ${metric} must be > 0, got ${b}. Refresh .perf-baselines first.`);
  process.exit(2);
}

const ratio = (c - b) / b;
const delta = c - b;
const maxAbsDelta = maxAbsDeltaArg === undefined ? undefined : Number(maxAbsDeltaArg);
if (maxAbsDeltaArg !== undefined && (!Number.isFinite(maxAbsDelta) || maxAbsDelta < 0)) {
  console.error(`Optional max_abs_delta must be a non-negative number, got ${maxAbsDeltaArg}.`);
  process.exit(2);
}

console.log(JSON.stringify({ metric, baseline: b, current: c, delta, ratio, maxAbsDelta }, null, 2));

if (ratio > Number(maxRatio) && (maxAbsDelta === undefined || delta > maxAbsDelta)) {
  console.error(
    `Regression on ${metric}: ${(ratio * 100).toFixed(2)}% > ${(Number(maxRatio) * 100).toFixed(2)}%`,
  );
  process.exit(1);
}
