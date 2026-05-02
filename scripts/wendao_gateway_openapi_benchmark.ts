import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { request as httpRequest } from "node:http";
import { request as httpsRequest } from "node:https";
import { dirname, resolve } from "node:path";
import { performance } from "node:perf_hooks";

type HttpMethod = "get" | "post";
type CoverageMode = "benchmark" | "smoke" | "skip";
type CoverageStatus = "passed" | "failed" | "skipped";

export type OpenApiDocument = {
  servers?: Array<{ url?: string }>;
  paths?: Record<string, Record<string, unknown>>;
};

export type OpenApiOperation = {
  method: HttpMethod;
  path: string;
  operationId: string;
  parameterNames: string[];
};

export type RequestMetric = {
  suite: string;
  label: string;
  url: string;
  elapsedMs: number;
  ok: boolean;
  status: number;
  attempts?: number;
  error?: string;
  hitCount?: number;
  total?: number;
  ready?: number;
};

export type BenchmarkSummary = {
  suite: string;
  requests: number;
  ok: number;
  failed: number;
  p50Ms: number;
  p95Ms: number;
  maxMs: number;
  avgMs: number;
  nonEmptyHitCount?: number;
};

export type StressSummary = {
  suite: string;
  configuredDurationMs: number;
  actualDurationMs: number;
  concurrency: number;
  maxRequests: number;
  requests: number;
  ok: number;
  failed: number;
  successRate: number;
  throughputRps: number;
  avgMs: number;
  p50Ms: number;
  p95Ms: number;
  p99Ms: number;
  maxMs: number;
  nonEmptyHitCount?: number;
  capped: boolean;
};

export type CoverageSummary = {
  totalOperations: number;
  benchmarkOperations: number;
  smokeOperations: number;
  skippedOperations: number;
  passedOperations: number;
  failedOperations: number;
};

export type RepoIndexSnapshot = {
  total?: number;
  active?: number;
  queued?: number;
  checking?: number;
  syncing?: number;
  indexing?: number;
  ready?: number;
  unsupported?: number;
  failed?: number;
  targetConcurrency?: number;
  maxConcurrency?: number;
  syncConcurrencyLimit?: number;
  currentRepoId?: string;
};

export type SearchIndexSnapshot = {
  total?: number;
  idle?: number;
  indexing?: number;
  ready?: number;
  degraded?: number;
  failed?: number;
  compactionPending?: number;
  statusReasonCode?: string;
  statusReasonSeverity?: string;
  statusReasonAction?: string;
  affectedCorpusCount?: number;
  readableCorpusCount?: number;
  blockingCorpusCount?: number;
};

export type DiscoveryContext = {
  readyRepoId?: string;
  repoQuery?: string;
  definitionQuery?: string;
  repoFilePath?: string;
  markdownFilePath?: string;
  vfsRootPath?: string;
  vfsFilePath?: string;
  pageId?: string;
  nodeId?: string;
  gapId?: string;
  gapKind?: string;
  pageKind?: string;
  familyKind?: string;
  topologyNodeId?: string;
};

export type OperationCoverage = {
  method: HttpMethod;
  path: string;
  operationId: string;
  mode: CoverageMode;
  status: CoverageStatus;
  suite?: string;
  label?: string;
  url?: string;
  elapsedMs?: number;
  httpStatus?: number;
  attempts?: number;
  skipReason?: string;
  error?: string;
};

export type CliOptions = {
  gatewayUrl?: string;
  openapiPath?: string;
  workspaceConfig?: string;
  reportPath?: string;
  repoLimit?: number;
  concurrency: number;
  limit: number;
  timeoutMs: number;
  stressConcurrency: number;
  stressDurationMs: number;
  stressMaxRequests: number;
  dryRun: boolean;
  json: boolean;
  minRepoCount: number;
  help: boolean;
};

export type PlannedSearchCase = {
  repoId: string;
  query: string;
};

export type BenchmarkPlan = {
  projectRoot: string;
  gatewayUrl: string;
  openapiPath: string;
  workspaceConfig: string;
  sampledRepoIds: string[];
  plannedSearchCases: PlannedSearchCase[];
  openApiOperations: OpenApiOperation[];
};

export type BenchmarkReport = {
  gatewayUrl: string;
  openapiPath: string;
  workspaceConfig: string;
  repoCount: number;
  openapiOperationCount: number;
  summaries: BenchmarkSummary[];
  stressSummaries: StressSummary[];
  aggregateRepoIndexStatus: RequestMetric;
  aggregateSearchIndexStatus: RequestMetric;
  repoIndexSnapshot?: RepoIndexSnapshot;
  searchIndexSnapshot?: SearchIndexSnapshot;
  discovery: DiscoveryContext;
  coverageSummary: CoverageSummary;
  operationCoverage: OperationCoverage[];
  failures: RequestMetric[];
};

export type DryRunReport = {
  gatewayUrl: string;
  openapiPath: string;
  workspaceConfig: string;
  repoCount: number;
  openapiOperationCount: number;
  reportDirectory: string;
  repoStatusPath: string;
  searchStatusPath: string;
  codeSearchPath: string;
  benchmarkOperationPaths: string[];
  coverageModes: Record<CoverageMode, number>;
  stressSettings: {
    concurrency: number;
    durationMs: number;
    maxRequests: number;
  };
  stressSuites: string[];
  codeSearchCases: PlannedSearchCase[];
};

export type CliIo = {
  log: (...args: unknown[]) => void;
  error: (...args: unknown[]) => void;
};

type JsonRequestResult = {
  metric: RequestMetric;
  payload: unknown;
};

export type TransportResponse = {
  status: number;
  ok: boolean;
  body: string;
};

type SmokeRequestPlan = {
  operation: OpenApiOperation;
  label: string;
  url: URL;
};

type StressRequestCase = {
  label: string;
  url: URL;
};

const BENCHMARK_OPERATION_PATHS = new Set<string>([
  "/api/repo/index/status",
  "/api/repo/sync",
  "/api/search/index/status",
  "/api/search/intent",
]);

const DEFAULT_RETRY_COUNT = 2;
const FAILURE_SAMPLE_LIMIT = 50;
const SMOKE_CONCURRENCY_LIMIT = 4;
const SMOKE_LIMIT_CAP = 5;
const DEFAULT_STRESS_SUITE_NAMES = [
  "stress_code_search",
  "stress_mixed_user_hotset",
] as const;

export const DEFAULT_GATEWAY_URL = "http://127.0.0.1:9517";
export const DEFAULT_OPENAPI_PATH =
  "packages/rust/crates/xiuxian-wendao/resources/openapi/wendao_gateway.openapi.json";
export const DEFAULT_WORKSPACE_CONFIG = ".data/wendao-frontend/wendao.toml";
export const DEFAULT_REPORT_DIRNAME = ".benchmark";

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function asArray(value: unknown): unknown[] | undefined {
  return Array.isArray(value) ? value : undefined;
}

function asString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function asNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value)
    ? value
    : undefined;
}

function truncateString(value: string, maxLength: number): string {
  return value.length <= maxLength
    ? value
    : `${value.slice(0, maxLength - 1)}...`;
}

function sleep(delayMs: number): Promise<void> {
  return new Promise((resolvePromise) => setTimeout(resolvePromise, delayMs));
}

function stripDocAnchor(path: string): string {
  const hashIndex = path.indexOf("#");
  return hashIndex >= 0 ? path.slice(0, hashIndex) : path;
}

function replacePathParameter(
  pathTemplate: string,
  parameter: string,
  value: string,
): string {
  return pathTemplate.replace(`{${parameter}}`, encodeURIComponent(value));
}

function appendSearchParams(
  url: URL,
  params: Record<string, string | number | undefined>,
): URL {
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) {
      url.searchParams.set(key, String(value));
    }
  }
  return url;
}

function inferHitCount(payload: unknown): number | undefined {
  if (Array.isArray(payload)) {
    return payload.length;
  }
  const record = asRecord(payload);
  if (!record) {
    return undefined;
  }
  if (typeof record.hitCount === "number") {
    return record.hitCount;
  }
  const candidateKeys = ["hits", "suggestions", "pages", "docs", "gaps"];
  for (const key of candidateKeys) {
    const value = record[key];
    if (Array.isArray(value)) {
      return value.length;
    }
  }
  return undefined;
}

function inferErrorMessage(
  payload: unknown,
  rawBody: string,
): string | undefined {
  if (payload === null || payload === undefined) {
    return rawBody.length > 0 ? truncateString(rawBody, 400) : undefined;
  }
  if (typeof payload === "string") {
    return truncateString(payload, 400);
  }
  try {
    return truncateString(JSON.stringify(payload), 400);
  } catch {
    return rawBody.length > 0 ? truncateString(rawBody, 400) : undefined;
  }
}

function isRetryableStatus(status: number): boolean {
  return status === 408 || status === 429 || status >= 500;
}

function isRetryableErrorMessage(message: string): boolean {
  return /aborted|EADDRNOTAVAIL|ECONNRESET|ECONNREFUSED|ETIMEDOUT|fetch failed/i.test(
    message,
  );
}

function operationPathSortKey(operation: OpenApiOperation): string {
  return `${operation.path}#${operation.method}`;
}

export function parseArgs(argv: string[]): CliOptions {
  const options: CliOptions = {
    concurrency: 6,
    limit: 20,
    timeoutMs: 30_000,
    stressConcurrency: 48,
    stressDurationMs: 30_000,
    stressMaxRequests: 5_000,
    dryRun: false,
    json: false,
    minRepoCount: 150,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];
    switch (arg) {
      case "--gateway-url":
        options.gatewayUrl = next;
        index += 1;
        break;
      case "--openapi":
        options.openapiPath = next;
        index += 1;
        break;
      case "--workspace-config":
        options.workspaceConfig = next;
        index += 1;
        break;
      case "--report-path":
        options.reportPath = next;
        index += 1;
        break;
      case "--repo-limit":
        options.repoLimit = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--concurrency":
        options.concurrency = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--limit":
        options.limit = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--timeout-ms":
        options.timeoutMs = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--stress-concurrency":
        options.stressConcurrency = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--stress-duration-ms":
        options.stressDurationMs = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--stress-max-requests":
        options.stressMaxRequests = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--min-repo-count":
        options.minRepoCount = Number.parseInt(next ?? "", 10);
        index += 1;
        break;
      case "--dry-run":
        options.dryRun = true;
        break;
      case "--json":
        options.json = true;
        break;
      case "--help":
        options.help = true;
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (options.help) {
    return options;
  }
  if (!Number.isInteger(options.concurrency) || options.concurrency <= 0) {
    throw new Error("--concurrency must be a positive integer");
  }
  if (!Number.isInteger(options.limit) || options.limit <= 0) {
    throw new Error("--limit must be a positive integer");
  }
  if (!Number.isInteger(options.timeoutMs) || options.timeoutMs <= 0) {
    throw new Error("--timeout-ms must be a positive integer");
  }
  if (
    !Number.isInteger(options.stressConcurrency) ||
    options.stressConcurrency <= 0
  ) {
    throw new Error("--stress-concurrency must be a positive integer");
  }
  if (
    !Number.isInteger(options.stressDurationMs) ||
    options.stressDurationMs <= 0
  ) {
    throw new Error("--stress-duration-ms must be a positive integer");
  }
  if (
    !Number.isInteger(options.stressMaxRequests) ||
    options.stressMaxRequests <= 0
  ) {
    throw new Error("--stress-max-requests must be a positive integer");
  }
  if (
    options.repoLimit !== undefined &&
    (!Number.isInteger(options.repoLimit) || options.repoLimit <= 0)
  ) {
    throw new Error("--repo-limit must be a positive integer when provided");
  }
  if (!Number.isInteger(options.minRepoCount) || options.minRepoCount <= 0) {
    throw new Error("--min-repo-count must be a positive integer");
  }

  return options;
}

export function renderHelpText(): string {
  return `Benchmark the live Wendao gateway against the bundled OpenAPI contract.

Usage:
  node --experimental-strip-types scripts/benchmark_wendao_gateway_openapi.ts [options]

Options:
  --gateway-url <url>        Override the gateway base URL
  --openapi <path>           Override the bundled OpenAPI JSON path
  --workspace-config <path>  Override the .data/wendao-frontend wendao.toml path
  --report-path <path>       Override the TOML report output path
  --repo-limit <n>           Limit the number of repo-derived benchmark cases
  --concurrency <n>          Concurrent request workers (default: 6)
  --limit <n>                Search result limit for code_search (default: 20)
  --timeout-ms <n>           Per-request timeout in milliseconds (default: 30000)
  --stress-concurrency <n>   Concurrent workers for sustained user-load suites (default: 48)
  --stress-duration-ms <n>   Sustained load duration per stress suite (default: 30000)
  --stress-max-requests <n>  Hard cap per stress suite to prevent runaway fast loops (default: 5000)
  --min-repo-count <n>       Fail if aggregate repo-index total is below this floor (default: 150)
  --dry-run                  Print the planned request set without calling the gateway
  --json                     Print the final report as JSON
  --help                     Show this help
`;
}

export function resolveProjectRoot(
  environment: NodeJS.ProcessEnv = process.env,
  cwd: string = process.cwd(),
): string {
  return resolve(environment.PRJ_ROOT ?? cwd);
}

export function loadOpenApiDocument(openapiPath: string): OpenApiDocument {
  return JSON.parse(readFileSync(openapiPath, "utf8")) as OpenApiDocument;
}

export function resolveGatewayUrl(
  document: OpenApiDocument,
  cliUrl?: string,
  environment: NodeJS.ProcessEnv = process.env,
): string {
  return (
    cliUrl ??
    environment.STUDIO_LIVE_GATEWAY_URL ??
    environment.XIUXIAN_WENDAO_GATEWAY_URL ??
    document.servers?.[0]?.url ??
    DEFAULT_GATEWAY_URL
  );
}

export function requireOpenApiOperation(
  document: OpenApiDocument,
  path: string,
  method: HttpMethod,
): void {
  const pathItem = document.paths?.[path];
  if (!pathItem || !(method in pathItem)) {
    throw new Error(
      `OpenAPI contract is missing ${method.toUpperCase()} ${path}`,
    );
  }
}

export function extractOpenApiOperations(
  document: OpenApiDocument,
): OpenApiOperation[] {
  const operations: OpenApiOperation[] = [];
  for (const [path, pathItem] of Object.entries(document.paths ?? {})) {
    for (const [methodKey, operationValue] of Object.entries(pathItem ?? {})) {
      const method = methodKey.toLowerCase();
      if (method !== "get" && method !== "post") {
        continue;
      }
      const operationRecord = asRecord(operationValue) ?? {};
      const parameters = asArray(operationRecord.parameters) ?? [];
      operations.push({
        method: method as HttpMethod,
        path,
        operationId:
          asString(operationRecord.operationId) ??
          `${method.toUpperCase()} ${path}`,
        parameterNames: parameters
          .map((parameter) => asRecord(parameter))
          .flatMap((parameter) => {
            const name = asString(parameter?.name);
            return name ? [name] : [];
          }),
      });
    }
  }
  return operations.sort((left, right) =>
    operationPathSortKey(left).localeCompare(operationPathSortKey(right)),
  );
}

export function determineCoverageMode(
  operation: OpenApiOperation,
): CoverageMode {
  if (operation.method === "post") {
    return "skip";
  }
  if (BENCHMARK_OPERATION_PATHS.has(operation.path)) {
    return "benchmark";
  }
  return "smoke";
}

export function summariseCoverageModes(
  operations: OpenApiOperation[],
): Record<CoverageMode, number> {
  return operations.reduce<Record<CoverageMode, number>>(
    (counts, operation) => {
      counts[determineCoverageMode(operation)] += 1;
      return counts;
    },
    { benchmark: 0, smoke: 0, skip: 0 },
  );
}

export function extractRepoIds(workspaceConfigText: string): string[] {
  const repoIds: string[] = [];
  let currentRepoId: string | undefined;
  let pluginList: string | undefined;

  const flushCurrentRepo = (): void => {
    if (!currentRepoId || pluginList === undefined) {
      return;
    }
    if (pluginList.trim().length > 0) {
      repoIds.push(currentRepoId);
    }
  };

  for (const line of workspaceConfigText.split(/\r?\n/u)) {
    const headerMatch = line.match(
      /^\[link_graph\.projects\.(?:"([^"]+)"|([^\]]+))\]$/u,
    );
    if (headerMatch) {
      flushCurrentRepo();
      currentRepoId = (headerMatch[1] ?? headerMatch[2] ?? "").trim();
      pluginList = undefined;
      continue;
    }
    if (!currentRepoId) {
      continue;
    }
    const pluginsMatch = line.match(/^\s*plugins\s*=\s*\[(.*?)\]\s*$/u);
    if (pluginsMatch) {
      pluginList = pluginsMatch[1] ?? "";
    }
  }

  flushCurrentRepo();
  return repoIds;
}

export function normalizeCodeSearchQuery(repoId: string): string {
  return repoId
    .replace(/\.jl$/i, "")
    .replace(/[_./-]+/g, " ")
    .trim();
}

function percentile(values: number[], fraction: number): number {
  if (values.length === 0) {
    return 0;
  }
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.max(0, Math.round((sorted.length - 1) * fraction));
  return sorted[index];
}

export function summariseSuite(
  suite: string,
  metrics: RequestMetric[],
): BenchmarkSummary {
  const elapsed = metrics.map((metric) => metric.elapsedMs);
  const ok = metrics.filter((metric) => metric.ok).length;
  const nonEmptyHitCount = metrics.filter(
    (metric) => (metric.hitCount ?? 0) > 0,
  ).length;
  return {
    suite,
    requests: metrics.length,
    ok,
    failed: metrics.length - ok,
    p50Ms: percentile(elapsed, 0.5),
    p95Ms: percentile(elapsed, 0.95),
    maxMs: elapsed.length === 0 ? 0 : Math.max(...elapsed),
    avgMs:
      elapsed.length === 0
        ? 0
        : elapsed.reduce((total, value) => total + value, 0) / elapsed.length,
    ...(metrics.some((metric) => metric.hitCount !== undefined)
      ? { nonEmptyHitCount }
      : {}),
  };
}

export function summariseStressSuite(
  suite: string,
  metrics: RequestMetric[],
  configuredDurationMs: number,
  actualDurationMs: number,
  concurrency: number,
  maxRequests: number,
  capped: boolean,
): StressSummary {
  const benchmarkSummary = summariseSuite(suite, metrics);
  const requests = metrics.length;
  const ok = benchmarkSummary.ok;
  const failed = benchmarkSummary.failed;
  return {
    suite,
    configuredDurationMs,
    actualDurationMs,
    concurrency,
    maxRequests,
    requests,
    ok,
    failed,
    successRate: requests === 0 ? 0 : ok / requests,
    throughputRps:
      actualDurationMs <= 0 ? 0 : (requests * 1000) / actualDurationMs,
    avgMs: benchmarkSummary.avgMs,
    p50Ms: benchmarkSummary.p50Ms,
    p95Ms: benchmarkSummary.p95Ms,
    p99Ms: percentile(
      metrics.map((metric) => metric.elapsedMs),
      0.99,
    ),
    maxMs: benchmarkSummary.maxMs,
    ...(benchmarkSummary.nonEmptyHitCount !== undefined
      ? { nonEmptyHitCount: benchmarkSummary.nonEmptyHitCount }
      : {}),
    capped,
  };
}

export function buildBenchmarkPlan(
  options: CliOptions,
  environment: NodeJS.ProcessEnv = process.env,
  cwd: string = process.cwd(),
): BenchmarkPlan {
  const projectRoot = resolveProjectRoot(environment, cwd);
  const openapiPath = resolve(
    projectRoot,
    options.openapiPath ?? DEFAULT_OPENAPI_PATH,
  );
  const workspaceConfig = resolve(
    projectRoot,
    options.workspaceConfig ?? DEFAULT_WORKSPACE_CONFIG,
  );
  const document = loadOpenApiDocument(openapiPath);

  requireOpenApiOperation(document, "/api/repo/index/status", "get");
  requireOpenApiOperation(document, "/api/repo/sync", "get");
  requireOpenApiOperation(document, "/api/search/index/status", "get");
  requireOpenApiOperation(document, "/api/search/intent", "get");

  const gatewayUrl = resolveGatewayUrl(
    document,
    options.gatewayUrl,
    environment,
  );
  const repoIds = extractRepoIds(readFileSync(workspaceConfig, "utf8"));
  const sampledRepoIds =
    options.repoLimit === undefined
      ? repoIds
      : repoIds.slice(0, options.repoLimit);

  return {
    projectRoot,
    gatewayUrl,
    openapiPath,
    workspaceConfig,
    sampledRepoIds,
    plannedSearchCases: sampledRepoIds.map((repoId) => ({
      repoId,
      query: normalizeCodeSearchQuery(repoId),
    })),
    openApiOperations: extractOpenApiOperations(document),
  };
}

export function createDryRunReport(
  plan: BenchmarkPlan,
  options: CliOptions = parseArgs(["--dry-run"]),
  environment: NodeJS.ProcessEnv = process.env,
  cwd: string = process.cwd(),
): DryRunReport {
  const resolvedOptions: CliOptions = {
    ...options,
    workspaceConfig: options.workspaceConfig ?? plan.workspaceConfig,
  };
  return {
    gatewayUrl: plan.gatewayUrl,
    openapiPath: plan.openapiPath,
    workspaceConfig: plan.workspaceConfig,
    repoCount: plan.sampledRepoIds.length,
    openapiOperationCount: plan.openApiOperations.length,
    reportDirectory: dirname(
      resolveBenchmarkReportPath(resolvedOptions, environment, cwd),
    ),
    repoStatusPath: "/api/repo/index/status",
    searchStatusPath: "/api/search/index/status",
    codeSearchPath: "/api/search/intent",
    benchmarkOperationPaths: [...BENCHMARK_OPERATION_PATHS].sort(
      (left, right) => left.localeCompare(right),
    ),
    coverageModes: summariseCoverageModes(plan.openApiOperations),
    stressSettings: {
      concurrency: resolvedOptions.stressConcurrency,
      durationMs: resolvedOptions.stressDurationMs,
      maxRequests: resolvedOptions.stressMaxRequests,
    },
    stressSuites: [...DEFAULT_STRESS_SUITE_NAMES],
    codeSearchCases: plan.plannedSearchCases.slice(0, 10),
  };
}

function escapeTomlString(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n");
}

function formatTomlString(key: string, value: string): string {
  return `${key} = "${escapeTomlString(value)}"`;
}

function formatTomlNumber(key: string, value: number): string {
  return `${key} = ${Number.isInteger(value) ? value : value.toFixed(2)}`;
}

function formatTomlBoolean(key: string, value: boolean): string {
  return `${key} = ${value ? "true" : "false"}`;
}

function formatOptionalTomlString(
  key: string,
  value: string | undefined,
): string | undefined {
  return value === undefined ? undefined : formatTomlString(key, value);
}

function formatOptionalTomlNumber(
  key: string,
  value: number | undefined,
): string | undefined {
  return value === undefined ? undefined : formatTomlNumber(key, value);
}

function pushOptionalTomlLine(lines: string[], line: string | undefined): void {
  if (line !== undefined) {
    lines.push(line);
  }
}

function formatReportTimestamp(now: Date): string {
  const year = now.getUTCFullYear();
  const month = String(now.getUTCMonth() + 1).padStart(2, "0");
  const day = String(now.getUTCDate()).padStart(2, "0");
  const hour = String(now.getUTCHours()).padStart(2, "0");
  const minute = String(now.getUTCMinutes()).padStart(2, "0");
  const second = String(now.getUTCSeconds()).padStart(2, "0");
  const millisecond = String(now.getUTCMilliseconds()).padStart(3, "0");
  return `${year}_${month}_${day}T${hour}_${minute}_${second}_${millisecond}Z`;
}

export function resolveBenchmarkReportPath(
  options: CliOptions,
  environment: NodeJS.ProcessEnv = process.env,
  cwd: string = process.cwd(),
  now: Date = new Date(),
): string {
  const projectRoot = resolveProjectRoot(environment, cwd);
  if (options.reportPath) {
    return resolve(projectRoot, options.reportPath);
  }
  const workspaceConfigPath = resolve(
    projectRoot,
    options.workspaceConfig ?? DEFAULT_WORKSPACE_CONFIG,
  );
  return resolve(
    dirname(workspaceConfigPath),
    DEFAULT_REPORT_DIRNAME,
    `wendao_gateway_openapi_${formatReportTimestamp(now)}.toml`,
  );
}

function renderSummaryToml(summary: BenchmarkSummary): string[] {
  const lines = [
    "[[summaries]]",
    formatTomlString("suite", summary.suite),
    formatTomlNumber("requests", summary.requests),
    formatTomlNumber("ok", summary.ok),
    formatTomlNumber("failed", summary.failed),
    formatTomlNumber("avg_ms", summary.avgMs),
    formatTomlNumber("p50_ms", summary.p50Ms),
    formatTomlNumber("p95_ms", summary.p95Ms),
    formatTomlNumber("max_ms", summary.maxMs),
  ];
  if (summary.nonEmptyHitCount !== undefined) {
    lines.push(
      formatTomlNumber("non_empty_hit_count", summary.nonEmptyHitCount),
    );
  }
  return lines;
}

function renderStressSummaryToml(summary: StressSummary): string[] {
  const lines = [
    "[[stress_summaries]]",
    formatTomlString("suite", summary.suite),
    formatTomlNumber("configured_duration_ms", summary.configuredDurationMs),
    formatTomlNumber("actual_duration_ms", summary.actualDurationMs),
    formatTomlNumber("concurrency", summary.concurrency),
    formatTomlNumber("max_requests", summary.maxRequests),
    formatTomlNumber("requests", summary.requests),
    formatTomlNumber("ok", summary.ok),
    formatTomlNumber("failed", summary.failed),
    formatTomlNumber("success_rate", summary.successRate),
    formatTomlNumber("throughput_rps", summary.throughputRps),
    formatTomlNumber("avg_ms", summary.avgMs),
    formatTomlNumber("p50_ms", summary.p50Ms),
    formatTomlNumber("p95_ms", summary.p95Ms),
    formatTomlNumber("p99_ms", summary.p99Ms),
    formatTomlNumber("max_ms", summary.maxMs),
    formatTomlBoolean("capped", summary.capped),
  ];
  if (summary.nonEmptyHitCount !== undefined) {
    lines.push(
      formatTomlNumber("non_empty_hit_count", summary.nonEmptyHitCount),
    );
  }
  return lines;
}

function renderMetricToml(tableName: string, metric: RequestMetric): string[] {
  const lines = [
    `[${tableName}]`,
    formatTomlString("suite", metric.suite),
    formatTomlString("label", metric.label),
    formatTomlString("url", metric.url),
    formatTomlNumber("elapsed_ms", metric.elapsedMs),
    formatTomlBoolean("ok", metric.ok),
    formatTomlNumber("status", metric.status),
  ];
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("attempts", metric.attempts),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("hit_count", metric.hitCount),
  );
  pushOptionalTomlLine(lines, formatOptionalTomlNumber("total", metric.total));
  pushOptionalTomlLine(lines, formatOptionalTomlNumber("ready", metric.ready));
  pushOptionalTomlLine(lines, formatOptionalTomlString("error", metric.error));
  return lines;
}

function renderRepoIndexSnapshotToml(snapshot: RepoIndexSnapshot): string[] {
  const lines = ["[repo_index_snapshot]"];
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("total", snapshot.total),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("active", snapshot.active),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("queued", snapshot.queued),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("checking", snapshot.checking),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("syncing", snapshot.syncing),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("indexing", snapshot.indexing),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("ready", snapshot.ready),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("unsupported", snapshot.unsupported),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("failed", snapshot.failed),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("target_concurrency", snapshot.targetConcurrency),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("max_concurrency", snapshot.maxConcurrency),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber(
      "sync_concurrency_limit",
      snapshot.syncConcurrencyLimit,
    ),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("current_repo_id", snapshot.currentRepoId),
  );
  return lines;
}

function renderSearchIndexSnapshotToml(
  snapshot: SearchIndexSnapshot,
): string[] {
  const lines = ["[search_index_snapshot]"];
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("total", snapshot.total),
  );
  pushOptionalTomlLine(lines, formatOptionalTomlNumber("idle", snapshot.idle));
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("indexing", snapshot.indexing),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("ready", snapshot.ready),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("degraded", snapshot.degraded),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("failed", snapshot.failed),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("compaction_pending", snapshot.compactionPending),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("status_reason_code", snapshot.statusReasonCode),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString(
      "status_reason_severity",
      snapshot.statusReasonSeverity,
    ),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString(
      "status_reason_action",
      snapshot.statusReasonAction,
    ),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber(
      "affected_corpus_count",
      snapshot.affectedCorpusCount,
    ),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber(
      "readable_corpus_count",
      snapshot.readableCorpusCount,
    ),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber(
      "blocking_corpus_count",
      snapshot.blockingCorpusCount,
    ),
  );
  return lines;
}

function renderDiscoveryToml(discovery: DiscoveryContext): string[] {
  const lines = ["[discovery]"];
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("ready_repo_id", discovery.readyRepoId),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("repo_query", discovery.repoQuery),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("definition_query", discovery.definitionQuery),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("repo_file_path", discovery.repoFilePath),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("markdown_file_path", discovery.markdownFilePath),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("vfs_root_path", discovery.vfsRootPath),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("vfs_file_path", discovery.vfsFilePath),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("page_id", discovery.pageId),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("node_id", discovery.nodeId),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("gap_id", discovery.gapId),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("gap_kind", discovery.gapKind),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("page_kind", discovery.pageKind),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("family_kind", discovery.familyKind),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("topology_node_id", discovery.topologyNodeId),
  );
  return lines;
}

function renderCoverageSummaryToml(summary: CoverageSummary): string[] {
  return [
    "[coverage_summary]",
    formatTomlNumber("total_operations", summary.totalOperations),
    formatTomlNumber("benchmark_operations", summary.benchmarkOperations),
    formatTomlNumber("smoke_operations", summary.smokeOperations),
    formatTomlNumber("skipped_operations", summary.skippedOperations),
    formatTomlNumber("passed_operations", summary.passedOperations),
    formatTomlNumber("failed_operations", summary.failedOperations),
  ];
}

function renderOperationCoverageToml(entry: OperationCoverage): string[] {
  const lines = [
    "[[operation_coverage]]",
    formatTomlString("method", entry.method.toUpperCase()),
    formatTomlString("path", entry.path),
    formatTomlString("operation_id", entry.operationId),
    formatTomlString("mode", entry.mode),
    formatTomlString("status", entry.status),
  ];
  pushOptionalTomlLine(lines, formatOptionalTomlString("suite", entry.suite));
  pushOptionalTomlLine(lines, formatOptionalTomlString("label", entry.label));
  pushOptionalTomlLine(lines, formatOptionalTomlString("url", entry.url));
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("elapsed_ms", entry.elapsedMs),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("http_status", entry.httpStatus),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("attempts", entry.attempts),
  );
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlString("skip_reason", entry.skipReason),
  );
  pushOptionalTomlLine(lines, formatOptionalTomlString("error", entry.error));
  return lines;
}

function renderFailureToml(metric: RequestMetric): string[] {
  const lines = [
    "[[failures]]",
    formatTomlString("suite", metric.suite),
    formatTomlString("label", metric.label),
    formatTomlString("url", metric.url),
    formatTomlNumber("elapsed_ms", metric.elapsedMs),
    formatTomlBoolean("ok", metric.ok),
    formatTomlNumber("status", metric.status),
  ];
  pushOptionalTomlLine(
    lines,
    formatOptionalTomlNumber("attempts", metric.attempts),
  );
  pushOptionalTomlLine(lines, formatOptionalTomlString("error", metric.error));
  return lines;
}

export function renderBenchmarkReportToml(
  report: BenchmarkReport,
  options: CliOptions,
  generatedAt: string,
): string {
  const lines = [
    'schema_version = "2"',
    formatTomlString("generated_at", generatedAt),
    formatTomlString("gateway_url", report.gatewayUrl),
    formatTomlString("openapi_path", report.openapiPath),
    formatTomlString("workspace_config", report.workspaceConfig),
    formatTomlNumber("repo_count", report.repoCount),
    formatTomlNumber("openapi_operation_count", report.openapiOperationCount),
    "",
    "[settings]",
    formatTomlNumber("concurrency", options.concurrency),
    formatTomlNumber("limit", options.limit),
    formatTomlNumber("timeout_ms", options.timeoutMs),
    formatTomlNumber("min_repo_count", options.minRepoCount),
    formatTomlNumber("stress_concurrency", options.stressConcurrency),
    formatTomlNumber("stress_duration_ms", options.stressDurationMs),
    formatTomlNumber("stress_max_requests", options.stressMaxRequests),
  ];
  if (options.repoLimit !== undefined) {
    lines.push(formatTomlNumber("repo_limit", options.repoLimit));
  }
  lines.push(
    "",
    ...renderCoverageSummaryToml(report.coverageSummary),
    "",
    ...renderDiscoveryToml(report.discovery),
    "",
    ...renderMetricToml(
      "aggregate_repo_index_status",
      report.aggregateRepoIndexStatus,
    ),
  );
  if (report.repoIndexSnapshot) {
    lines.push("", ...renderRepoIndexSnapshotToml(report.repoIndexSnapshot));
  }
  lines.push(
    "",
    ...renderMetricToml(
      "aggregate_search_index_status",
      report.aggregateSearchIndexStatus,
    ),
  );
  if (report.searchIndexSnapshot) {
    lines.push(
      "",
      ...renderSearchIndexSnapshotToml(report.searchIndexSnapshot),
    );
  }
  lines.push("");
  for (const summary of report.summaries) {
    lines.push(...renderSummaryToml(summary), "");
  }
  for (const summary of report.stressSummaries) {
    lines.push(...renderStressSummaryToml(summary), "");
  }
  for (const entry of report.operationCoverage) {
    lines.push(...renderOperationCoverageToml(entry), "");
  }
  for (const failure of report.failures) {
    lines.push(...renderFailureToml(failure), "");
  }
  return `${lines.join("\n").trimEnd()}\n`;
}

export function persistBenchmarkReportToml(
  report: BenchmarkReport,
  options: CliOptions,
  environment: NodeJS.ProcessEnv = process.env,
  cwd: string = process.cwd(),
  now: Date = new Date(),
): { path: string; toml: string; generatedAt: string } {
  const reportPath = resolveBenchmarkReportPath(options, environment, cwd, now);
  const generatedAt = now.toISOString();
  const toml = renderBenchmarkReportToml(report, options, generatedAt);
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, toml, "utf8");
  return { path: reportPath, toml, generatedAt };
}

export function requestJsonTransport(
  url: URL,
  timeoutMs: number,
): Promise<TransportResponse> {
  return new Promise((resolvePromise, rejectPromise) => {
    const requestImpl = url.protocol === "https:" ? httpsRequest : httpRequest;
    const localAddress =
      url.hostname === "127.0.0.1"
        ? "127.0.0.1"
        : url.hostname === "::1"
          ? "::1"
          : undefined;
    const request = requestImpl(
      url,
      {
        method: "GET",
        headers: { accept: "application/json" },
        ...(localAddress ? { localAddress } : {}),
      },
      (response) => {
        let body = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          body += chunk;
        });
        response.on("end", () => {
          const status = response.statusCode ?? 0;
          resolvePromise({
            status,
            ok: status >= 200 && status < 300,
            body,
          });
        });
      },
    );
    request.on("error", rejectPromise);
    request.setTimeout(timeoutMs, () => {
      request.destroy(new Error(`request timed out after ${timeoutMs}ms`));
    });
    request.end();
  });
}

async function timedJsonRequest(
  suite: string,
  label: string,
  url: URL,
  timeoutMs: number,
  retryCount: number = DEFAULT_RETRY_COUNT,
): Promise<JsonRequestResult> {
  let attempt = 0;
  while (true) {
    attempt += 1;
    const started = performance.now();
    try {
      const response = await requestJsonTransport(url, timeoutMs);
      const rawBody = response.body;
      let payload: unknown = null;
      if (rawBody.length > 0) {
        try {
          payload = JSON.parse(rawBody) as unknown;
        } catch {
          payload = rawBody;
        }
      }
      const metric: RequestMetric = {
        suite,
        label,
        url: url.toString(),
        elapsedMs: performance.now() - started,
        ok: response.ok,
        status: response.status,
        attempts: attempt,
        hitCount: inferHitCount(payload),
        total: asNumber(asRecord(payload)?.total),
        ready: asNumber(asRecord(payload)?.ready),
        error: response.ok ? undefined : inferErrorMessage(payload, rawBody),
      };
      if (
        response.ok ||
        attempt > retryCount ||
        !isRetryableStatus(response.status)
      ) {
        return { metric, payload };
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      const metric: RequestMetric = {
        suite,
        label,
        url: url.toString(),
        elapsedMs: performance.now() - started,
        ok: false,
        status: 0,
        attempts: attempt,
        error: message,
      };
      if (attempt > retryCount || !isRetryableErrorMessage(message)) {
        return { metric, payload: null };
      }
    }
    await sleep(Math.min(500, attempt * 100));
  }
}

async function runWithConcurrency<T, R>(
  items: T[],
  concurrency: number,
  worker: (item: T) => Promise<R>,
): Promise<R[]> {
  const results: R[] = Array.from({ length: items.length });
  let cursor = 0;

  async function consume(): Promise<void> {
    while (true) {
      const index = cursor;
      cursor += 1;
      if (index >= items.length) {
        return;
      }
      results[index] = await worker(items[index]);
    }
  }

  await Promise.all(
    Array.from({ length: Math.min(concurrency, items.length) }, () =>
      consume(),
    ),
  );
  return results;
}

async function runSustainedLoad(
  suite: string,
  cases: StressRequestCase[],
  concurrency: number,
  durationMs: number,
  maxRequests: number,
  timeoutMs: number,
): Promise<{ metrics: RequestMetric[]; summary: StressSummary }> {
  if (cases.length === 0) {
    return {
      metrics: [],
      summary: summariseStressSuite(
        suite,
        [],
        durationMs,
        0,
        concurrency,
        maxRequests,
        false,
      ),
    };
  }

  const metrics: RequestMetric[] = [];
  let requestCursor = 0;
  let caseCursor = 0;
  let capped = false;
  const started = performance.now();
  const deadline = started + durationMs;

  async function worker(): Promise<void> {
    while (performance.now() < deadline) {
      if (requestCursor >= maxRequests) {
        capped = true;
        return;
      }
      const requestIndex = requestCursor;
      requestCursor += 1;
      if (requestIndex >= maxRequests) {
        capped = true;
        return;
      }
      const caseIndex = caseCursor % cases.length;
      caseCursor += 1;
      const requestCase = cases[caseIndex];
      const result = await timedJsonRequest(
        suite,
        requestCase.label,
        requestCase.url,
        timeoutMs,
      );
      metrics.push(result.metric);
    }
  }

  await Promise.all(
    Array.from({ length: Math.min(concurrency, maxRequests) }, () => worker()),
  );
  const actualDurationMs = performance.now() - started;
  return {
    metrics,
    summary: summariseStressSuite(
      suite,
      metrics,
      durationMs,
      actualDurationMs,
      concurrency,
      maxRequests,
      capped,
    ),
  };
}

function buildRepoIndexSnapshot(
  payload: unknown,
): RepoIndexSnapshot | undefined {
  const record = asRecord(payload);
  if (!record) {
    return undefined;
  }
  return {
    total: asNumber(record.total),
    active: asNumber(record.active),
    queued: asNumber(record.queued),
    checking: asNumber(record.checking),
    syncing: asNumber(record.syncing),
    indexing: asNumber(record.indexing),
    ready: asNumber(record.ready),
    unsupported: asNumber(record.unsupported),
    failed: asNumber(record.failed),
    targetConcurrency: asNumber(record.targetConcurrency),
    maxConcurrency: asNumber(record.maxConcurrency),
    syncConcurrencyLimit: asNumber(record.syncConcurrencyLimit),
    currentRepoId: asString(record.currentRepoId),
  };
}

function buildSearchIndexSnapshot(
  payload: unknown,
): SearchIndexSnapshot | undefined {
  const record = asRecord(payload);
  if (!record) {
    return undefined;
  }
  const statusReason = asRecord(record.statusReason);
  return {
    total: asNumber(record.total),
    idle: asNumber(record.idle),
    indexing: asNumber(record.indexing),
    ready: asNumber(record.ready),
    degraded: asNumber(record.degraded),
    failed: asNumber(record.failed),
    compactionPending: asNumber(record.compactionPending),
    statusReasonCode: asString(statusReason?.code),
    statusReasonSeverity: asString(statusReason?.severity),
    statusReasonAction: asString(statusReason?.action),
    affectedCorpusCount: asNumber(statusReason?.affectedCorpusCount),
    readableCorpusCount: asNumber(statusReason?.readableCorpusCount),
    blockingCorpusCount: asNumber(statusReason?.blockingCorpusCount),
  };
}

function extractReadyRepoIds(payload: unknown): string[] {
  const repos = asArray(asRecord(payload)?.repos) ?? [];
  const readyIds = repos
    .map((repo) => asRecord(repo))
    .flatMap((repo) => {
      const repoId = asString(repo?.repoId);
      const phase = asString(repo?.phase) ?? asString(repo?.state);
      return repoId && phase === "ready" ? [repoId] : [];
    });
  return readyIds;
}

function selectReadyRepoId(
  payload: unknown,
  sampledRepoIds: string[],
): string | undefined {
  const readyRepoIds = extractReadyRepoIds(payload);
  const readyRepoSet = new Set(readyRepoIds);
  for (const repoId of sampledRepoIds) {
    if (readyRepoSet.has(repoId)) {
      return repoId;
    }
  }
  return readyRepoIds[0];
}

function selectRepoFilePath(payload: unknown): string | undefined {
  const docs = asArray(asRecord(payload)?.docs) ?? [];
  const docPaths = docs
    .map((doc) => asRecord(doc))
    .flatMap((doc) => {
      const path = asString(doc?.path);
      return path ? [stripDocAnchor(path)] : [];
    });
  for (const candidate of docPaths) {
    if (candidate.startsWith("src/") && candidate.endsWith(".jl")) {
      return candidate;
    }
  }
  for (const candidate of docPaths) {
    if (candidate.endsWith(".jl") || candidate.endsWith(".md")) {
      return candidate;
    }
  }
  return docPaths[0];
}

function selectMarkdownFilePath(payload: unknown): string | undefined {
  const docs = asArray(asRecord(payload)?.docs) ?? [];
  const docPaths = docs
    .map((doc) => asRecord(doc))
    .flatMap((doc) => {
      const path = asString(doc?.path);
      return path ? [stripDocAnchor(path)] : [];
    });
  for (const candidate of docPaths) {
    if (candidate.endsWith(".md") || candidate.endsWith(".markdown")) {
      return candidate;
    }
  }
  return undefined;
}

function selectWorkspaceMarkdownFilePath(
  projectRoot: string,
): string | undefined {
  const directCandidates = ["README.md", "CLAUDE.md", "AGENTS.md"];
  for (const candidate of directCandidates) {
    try {
      readFileSync(resolve(projectRoot, candidate), "utf8");
      return candidate;
    } catch {
      continue;
    }
  }

  const ignoredDirectories = new Set([
    ".git",
    ".cache",
    ".run",
    "node_modules",
    "target",
  ]);
  const searchQueue = ["docs", "packages", ".data"];

  while (searchQueue.length > 0) {
    const relativeDirectory = searchQueue.shift();
    if (!relativeDirectory) {
      continue;
    }
    let entries;
    try {
      entries = readdirSync(resolve(projectRoot, relativeDirectory), {
        withFileTypes: true,
      });
    } catch {
      continue;
    }

    for (const entry of entries) {
      if (entry.name.startsWith(".") && entry.name !== ".data") {
        continue;
      }
      const relativePath = `${relativeDirectory}/${entry.name}`;
      if (entry.isDirectory()) {
        if (!ignoredDirectories.has(entry.name)) {
          searchQueue.push(relativePath);
        }
        continue;
      }
      if (
        entry.isFile() &&
        (entry.name.endsWith(".md") || entry.name.endsWith(".markdown"))
      ) {
        return relativePath;
      }
    }
  }

  return undefined;
}

function selectPageInfo(payload: unknown): {
  pageId?: string;
  pageKind?: string;
} {
  const pages = asArray(asRecord(payload)?.pages) ?? [];
  const page = asRecord(pages[0]);
  return {
    pageId: asString(page?.page_id),
    pageKind: asString(page?.kind),
  };
}

function selectNodeId(payload: unknown): string | undefined {
  const tree = asRecord(asRecord(payload)?.tree);
  const roots = asArray(tree?.roots) ?? [];
  const firstRoot = asRecord(roots[0]);
  return asString(firstRoot?.node_id);
}

function selectGapInfo(payload: unknown): {
  gapId?: string;
  gapKind?: string;
  pageKind?: string;
} {
  const gaps = asArray(asRecord(payload)?.gaps) ?? [];
  const gap = asRecord(gaps[0]);
  return {
    gapId: asString(gap?.gap_id),
    gapKind: asString(gap?.kind),
    pageKind: asString(gap?.page_kind),
  };
}

function selectTopologyNodeId(payload: unknown): string | undefined {
  const nodes = asArray(asRecord(payload)?.nodes) ?? [];
  const firstNode = asRecord(nodes[0]);
  return asString(firstNode?.id);
}

function selectDefinitionQuery(payload: unknown): string | undefined {
  const hits = asArray(asRecord(payload)?.hits) ?? [];
  for (const candidate of hits) {
    const name = asString(asRecord(candidate)?.name)?.trim();
    if (name) {
      return name;
    }
  }
  return undefined;
}

async function discoverDefinitionQuery(
  baseUrl: URL,
  timeoutMs: number,
): Promise<string | undefined> {
  const probes = ["search", "config", "build", "index", "router"];
  for (const probe of probes) {
    const result = await timedJsonRequest(
      "discovery",
      `search_ast_definition_seed_${probe}`,
      appendSearchParams(new URL("/api/search/ast", baseUrl), { q: probe }),
      timeoutMs,
    );
    if (!result.metric.ok) {
      continue;
    }
    const definitionQuery = selectDefinitionQuery(result.payload);
    if (definitionQuery) {
      return definitionQuery;
    }
  }
  return undefined;
}

async function discoverGatewayContext(
  baseUrl: URL,
  options: CliOptions,
  plan: BenchmarkPlan,
  repoIndexPayload: unknown,
): Promise<DiscoveryContext> {
  const readyRepoId = selectReadyRepoId(repoIndexPayload, plan.sampledRepoIds);
  const discovery: DiscoveryContext = {
    readyRepoId,
    repoQuery: readyRepoId
      ? normalizeCodeSearchQuery(readyRepoId)
      : plan.plannedSearchCases[0]?.query,
    definitionQuery: await discoverDefinitionQuery(baseUrl, options.timeoutMs),
    markdownFilePath: selectWorkspaceMarkdownFilePath(plan.projectRoot),
  };

  const vfsRootResponse = await timedJsonRequest(
    "discovery",
    "vfs_root",
    new URL("/api/vfs", baseUrl),
    options.timeoutMs,
  );
  const vfsRootEntries = asArray(vfsRootResponse.payload) ?? [];
  const matchingVfsRoot = vfsRootEntries
    .map((entry) => asRecord(entry))
    .find((entry) => {
      const path = asString(entry?.path);
      return path === readyRepoId;
    });
  discovery.vfsRootPath =
    asString(matchingVfsRoot?.path) ??
    asString(asRecord(vfsRootEntries[0])?.path) ??
    readyRepoId;

  if (!readyRepoId) {
    const topologyResponse = await timedJsonRequest(
      "discovery",
      "topology_3d",
      new URL("/api/topology/3d", baseUrl),
      options.timeoutMs,
    );
    discovery.topologyNodeId = selectTopologyNodeId(topologyResponse.payload);
    return discovery;
  }

  const docCoverageUrl = appendSearchParams(
    new URL("/api/repo/doc-coverage", baseUrl),
    {
      repo: readyRepoId,
    },
  );
  const docCoverageResponse = await timedJsonRequest(
    "discovery",
    "repo_doc_coverage",
    docCoverageUrl,
    options.timeoutMs,
  );
  discovery.repoFilePath = selectRepoFilePath(docCoverageResponse.payload);
  discovery.markdownFilePath =
    discovery.markdownFilePath ??
    selectMarkdownFilePath(docCoverageResponse.payload);
  if (discovery.vfsRootPath && discovery.repoFilePath) {
    discovery.vfsFilePath = `${discovery.vfsRootPath}/${discovery.repoFilePath}`;
  }

  const projectedPagesUrl = appendSearchParams(
    new URL("/api/repo/projected-pages", baseUrl),
    {
      repo: readyRepoId,
    },
  );
  const projectedPagesResponse = await timedJsonRequest(
    "discovery",
    "repo_projected_pages",
    projectedPagesUrl,
    options.timeoutMs,
  );
  const pageInfo = selectPageInfo(projectedPagesResponse.payload);
  discovery.pageId = pageInfo.pageId;
  discovery.pageKind = pageInfo.pageKind ?? "explanation";

  if (discovery.pageId) {
    const treeUrl = appendSearchParams(
      new URL("/api/repo/projected-page-index-tree", baseUrl),
      {
        repo: readyRepoId,
        page_id: discovery.pageId,
      },
    );
    const treeResponse = await timedJsonRequest(
      "discovery",
      "repo_projected_page_index_tree",
      treeUrl,
      options.timeoutMs,
    );
    discovery.nodeId = selectNodeId(treeResponse.payload);
  }

  const gapReportUrl = appendSearchParams(
    new URL("/api/repo/projected-gap-report", baseUrl),
    {
      repo: readyRepoId,
    },
  );
  const gapReportResponse = await timedJsonRequest(
    "discovery",
    "repo_projected_gap_report",
    gapReportUrl,
    options.timeoutMs,
  );
  const gapInfo = selectGapInfo(gapReportResponse.payload);
  discovery.gapId = gapInfo.gapId;
  discovery.gapKind =
    gapInfo.gapKind ?? "symbol_reference_without_documentation";
  discovery.familyKind = gapInfo.pageKind ?? discovery.pageKind ?? "reference";

  const topologyResponse = await timedJsonRequest(
    "discovery",
    "topology_3d",
    new URL("/api/topology/3d", baseUrl),
    options.timeoutMs,
  );
  discovery.topologyNodeId = selectTopologyNodeId(topologyResponse.payload);

  return discovery;
}

function buildStressCodeSearchCases(
  baseUrl: URL,
  plannedSearchCases: PlannedSearchCase[],
  limit: number,
): StressRequestCase[] {
  return plannedSearchCases.map(({ repoId, query }) => ({
    label: repoId,
    url: appendSearchParams(new URL("/api/search/intent", baseUrl), {
      intent: "code_search",
      q: query,
      limit,
    }),
  }));
}

function buildStressMixedHotsetCases(
  baseUrl: URL,
  readyRepoIds: string[],
  plannedSearchCases: PlannedSearchCase[],
  limit: number,
): StressRequestCase[] {
  const cases: StressRequestCase[] = [];
  const readyCaseSeed =
    readyRepoIds.length > 0
      ? readyRepoIds
      : plannedSearchCases.slice(0, 16).map((searchCase) => searchCase.repoId);

  for (const { repoId, query } of plannedSearchCases) {
    cases.push({
      label: `code_search:${repoId}`,
      url: appendSearchParams(new URL("/api/search/intent", baseUrl), {
        intent: "code_search",
        q: query,
        limit,
      }),
    });
    cases.push({
      label: `autocomplete:${repoId}`,
      url: appendSearchParams(new URL("/api/search/autocomplete", baseUrl), {
        prefix: query.slice(0, Math.max(3, Math.min(8, query.length))),
      }),
    });
  }

  for (const repoId of readyCaseSeed) {
    const query = normalizeCodeSearchQuery(repoId);
    cases.push({
      label: `repo_symbol:${repoId}`,
      url: appendSearchParams(new URL("/api/repo/symbol-search", baseUrl), {
        repo: repoId,
        query,
        limit,
      }),
    });
    cases.push({
      label: `repo_module:${repoId}`,
      url: appendSearchParams(new URL("/api/repo/module-search", baseUrl), {
        repo: repoId,
        query,
        limit,
      }),
    });
  }

  return cases;
}

function selectBenchmarkCoverageMetric(
  operationPath: string,
  aggregateRepoMetric: RequestMetric,
  aggregateSearchMetric: RequestMetric,
  repoSyncMetrics: RequestMetric[],
  searchMetrics: RequestMetric[],
): RequestMetric | undefined {
  switch (operationPath) {
    case "/api/repo/index/status":
      return aggregateRepoMetric;
    case "/api/search/index/status":
      return aggregateSearchMetric;
    case "/api/repo/sync":
      return repoSyncMetrics[0];
    case "/api/search/intent":
      return searchMetrics[0];
    default:
      return undefined;
  }
}

function benchmarkMetricToCoverage(
  operation: OpenApiOperation,
  metric: RequestMetric | undefined,
): OperationCoverage {
  if (!metric) {
    return {
      method: operation.method,
      path: operation.path,
      operationId: operation.operationId,
      mode: "benchmark",
      status: "failed",
      error: "benchmark metric unavailable",
    };
  }
  return {
    method: operation.method,
    path: operation.path,
    operationId: operation.operationId,
    mode: "benchmark",
    status: metric.ok ? "passed" : "failed",
    suite: metric.suite,
    label: metric.label,
    url: metric.url,
    elapsedMs: metric.elapsedMs,
    httpStatus: metric.status,
    attempts: metric.attempts,
    error: metric.ok ? undefined : metric.error,
  };
}

function missingSeeds(
  entries: Array<[string, string | undefined]>,
): string | undefined {
  const missing = entries.filter(([, value]) => !value).map(([label]) => label);
  return missing.length === 0
    ? undefined
    : `missing discovery seed: ${missing.join(", ")}`;
}

export function buildSmokeRequestPlan(
  operation: OpenApiOperation,
  context: DiscoveryContext,
  baseUrl: URL,
  limit: number,
): SmokeRequestPlan | { skipReason: string } {
  const repo = context.readyRepoId;
  const query = context.repoQuery ?? "ADTypes";
  const definitionQuery = context.definitionQuery;
  const pageId = context.pageId;
  const nodeId = context.nodeId;
  const gapId = context.gapId;
  const gapKind = context.gapKind ?? "symbol_reference_without_documentation";
  const pageKind = context.pageKind ?? "explanation";
  const familyKind = context.familyKind ?? pageKind;
  const repoFilePath = context.repoFilePath;
  const markdownFilePath = context.markdownFilePath;
  const vfsFilePath = context.vfsFilePath;
  const topologyNodeId = context.topologyNodeId;

  switch (operation.path) {
    case "/api/health":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/health", baseUrl),
      };
    case "/api/stats":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/stats", baseUrl),
      };
    case "/api/notify":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/notify", baseUrl),
      };
    case "/api/vfs":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/vfs", baseUrl),
      };
    case "/api/vfs/scan":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/vfs/scan", baseUrl),
      };
    case "/api/vfs/cat": {
      const skipReason = missingSeeds([["vfs_file_path", vfsFilePath]]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/vfs/cat", baseUrl), {
          path: vfsFilePath,
        }),
      };
    }
    case "/api/vfs/{path}": {
      const skipReason = missingSeeds([["vfs_file_path", vfsFilePath]]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: new URL(
          replacePathParameter("/api/vfs/{path}", "path", vfsFilePath!),
          baseUrl,
        ),
      };
    }
    case "/api/neighbors/{id}":
    case "/api/graph/neighbors/{id}": {
      const skipReason = missingSeeds([["topology_node_id", topologyNodeId]]);
      if (skipReason) {
        return { skipReason };
      }
      const path = replacePathParameter(operation.path, "id", topologyNodeId!);
      return {
        operation,
        label: operation.operationId,
        url: new URL(path, baseUrl),
      };
    }
    case "/api/topology/3d":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/topology/3d", baseUrl),
      };
    case "/api/search":
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search", baseUrl), {
          q: query,
          intent: "code_search",
          limit,
        }),
      };
    case "/api/search/attachments":
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search/attachments", baseUrl), {
          q: query,
          limit,
        }),
      };
    case "/api/search/ast":
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search/ast", baseUrl), {
          q: query,
        }),
      };
    case "/api/search/definition":
      if (!definitionQuery) {
        return {
          skipReason: missingSeeds([["definition_query", definitionQuery]])!,
        };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search/definition", baseUrl), {
          q: definitionQuery,
        }),
      };
    case "/api/search/references":
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search/references", baseUrl), {
          q: query,
        }),
      };
    case "/api/search/symbols":
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search/symbols", baseUrl), {
          q: query,
        }),
      };
    case "/api/search/autocomplete":
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/search/autocomplete", baseUrl), {
          prefix: query.slice(0, Math.max(3, Math.min(8, query.length))),
        }),
      };
    case "/api/analysis/markdown":
      if (!markdownFilePath) {
        return { skipReason: "missing discovery seed: markdown_file_path" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/analysis/markdown", baseUrl), {
          path: markdownFilePath,
        }),
      };
    case "/api/analysis/code-ast": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["repo_file_path", repoFilePath],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/analysis/code-ast", baseUrl), {
          repo,
          path: repoFilePath,
          line: 1,
        }),
      };
    }
    case "/api/ui/config":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/ui/config", baseUrl),
      };
    case "/api/ui/capabilities":
      return {
        operation,
        label: operation.operationId,
        url: new URL("/api/ui/capabilities", baseUrl),
      };
    case "/api/repo/overview":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/overview", baseUrl), {
          repo,
        }),
      };
    case "/api/repo/module-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/module-search", baseUrl), {
          repo,
          query,
          limit,
        }),
      };
    case "/api/repo/symbol-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/symbol-search", baseUrl), {
          repo,
          query,
          limit,
        }),
      };
    case "/api/repo/example-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/example-search", baseUrl), {
          repo,
          query,
          limit,
        }),
      };
    case "/api/repo/doc-coverage":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/doc-coverage", baseUrl), {
          repo,
        }),
      };
    case "/api/repo/projected-pages":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/projected-pages", baseUrl), {
          repo,
        }),
      };
    case "/api/docs/projected-gap-report":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/docs/projected-gap-report", baseUrl),
          { repo },
        ),
      };
    case "/api/docs/planner-item": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["gap_id", gapId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/planner-item", baseUrl), {
          repo,
          gap_id: gapId,
          family_kind: familyKind,
          related_limit: limit,
          family_limit: 3,
        }),
      };
    }
    case "/api/docs/planner-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/planner-search", baseUrl), {
          repo,
          query,
          gap_kind: gapKind,
          page_kind: pageKind,
          limit,
        }),
      };
    case "/api/docs/planner-queue":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/planner-queue", baseUrl), {
          repo,
          gap_kind: gapKind,
          page_kind: pageKind,
          per_kind_limit: 3,
        }),
      };
    case "/api/docs/planner-rank":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/planner-rank", baseUrl), {
          repo,
          gap_kind: gapKind,
          page_kind: pageKind,
          limit,
        }),
      };
    case "/api/docs/planner-workset":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/planner-workset", baseUrl), {
          repo,
          gap_kind: gapKind,
          page_kind: pageKind,
          per_kind_limit: 3,
          limit,
          family_kind: familyKind,
          related_limit: limit,
          family_limit: 3,
        }),
      };
    case "/api/docs/search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/search", baseUrl), {
          repo,
          query,
          kind: pageKind,
          limit,
        }),
      };
    case "/api/docs/retrieval":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/retrieval", baseUrl), {
          repo,
          query,
          kind: pageKind,
          limit,
        }),
      };
    case "/api/docs/retrieval-context": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
        ["node_id", nodeId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/docs/retrieval-context", baseUrl),
          {
            repo,
            page_id: pageId,
            node_id: nodeId,
            related_limit: limit,
          },
        ),
      };
    }
    case "/api/docs/retrieval-hit": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
        ["node_id", nodeId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/retrieval-hit", baseUrl), {
          repo,
          page_id: pageId,
          node_id: nodeId,
        }),
      };
    }
    case "/api/docs/page": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/page", baseUrl), {
          repo,
          page_id: pageId,
        }),
      };
    }
    case "/api/docs/family-context": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/family-context", baseUrl), {
          repo,
          page_id: pageId,
          per_kind_limit: 3,
        }),
      };
    }
    case "/api/docs/family-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/family-search", baseUrl), {
          repo,
          query,
          kind: pageKind,
          limit,
          per_kind_limit: 3,
        }),
      };
    case "/api/docs/family-cluster": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/family-cluster", baseUrl), {
          repo,
          page_id: pageId,
          kind: pageKind,
          limit,
        }),
      };
    }
    case "/api/docs/navigation": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/docs/navigation", baseUrl), {
          repo,
          page_id: pageId,
          node_id: nodeId,
          family_kind: familyKind,
          related_limit: limit,
          family_limit: 3,
        }),
      };
    }
    case "/api/docs/navigation-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/docs/navigation-search", baseUrl),
          {
            repo,
            query,
            kind: pageKind,
            family_kind: familyKind,
            limit,
            related_limit: limit,
            family_limit: 3,
          },
        ),
      };
    case "/api/repo/projected-gap-report":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-gap-report", baseUrl),
          {
            repo,
          },
        ),
      };
    case "/api/repo/projected-page": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(new URL("/api/repo/projected-page", baseUrl), {
          repo,
          page_id: pageId,
        }),
      };
    }
    case "/api/repo/projected-page-index-node": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
        ["node_id", nodeId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-index-node", baseUrl),
          {
            repo,
            page_id: pageId,
            node_id: nodeId,
          },
        ),
      };
    }
    case "/api/repo/projected-retrieval-hit": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
        ["node_id", nodeId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-retrieval-hit", baseUrl),
          {
            repo,
            page_id: pageId,
            node_id: nodeId,
          },
        ),
      };
    }
    case "/api/repo/projected-retrieval-context": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
        ["node_id", nodeId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-retrieval-context", baseUrl),
          {
            repo,
            page_id: pageId,
            node_id: nodeId,
            related_limit: limit,
          },
        ),
      };
    }
    case "/api/repo/projected-page-family-context": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-family-context", baseUrl),
          {
            repo,
            page_id: pageId,
            per_kind_limit: 3,
          },
        ),
      };
    }
    case "/api/repo/projected-page-family-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-family-search", baseUrl),
          {
            repo,
            query,
            kind: pageKind,
            limit,
            per_kind_limit: 3,
          },
        ),
      };
    case "/api/repo/projected-page-family-cluster": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-family-cluster", baseUrl),
          {
            repo,
            page_id: pageId,
            kind: pageKind,
            limit,
          },
        ),
      };
    }
    case "/api/repo/projected-page-navigation": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-navigation", baseUrl),
          {
            repo,
            page_id: pageId,
            node_id: nodeId,
            family_kind: familyKind,
            related_limit: limit,
            family_limit: 3,
          },
        ),
      };
    }
    case "/api/repo/projected-page-navigation-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-navigation-search", baseUrl),
          {
            repo,
            query,
            kind: pageKind,
            family_kind: familyKind,
            limit,
            related_limit: limit,
            family_limit: 3,
          },
        ),
      };
    case "/api/repo/projected-page-index-tree": {
      const skipReason = missingSeeds([
        ["ready_repo_id", repo],
        ["page_id", pageId],
      ]);
      if (skipReason) {
        return { skipReason };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-index-tree", baseUrl),
          {
            repo,
            page_id: pageId,
          },
        ),
      };
    }
    case "/api/repo/projected-page-index-tree-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-index-tree-search", baseUrl),
          {
            repo,
            query,
            kind: pageKind,
            limit,
          },
        ),
      };
    case "/api/repo/projected-page-search":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-search", baseUrl),
          {
            repo,
            query,
            kind: pageKind,
            limit,
          },
        ),
      };
    case "/api/repo/projected-retrieval":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-retrieval", baseUrl),
          {
            repo,
            query,
            kind: pageKind,
            limit,
          },
        ),
      };
    case "/api/repo/projected-page-index-trees":
      if (!repo) {
        return { skipReason: "missing discovery seed: ready_repo_id" };
      }
      return {
        operation,
        label: operation.operationId,
        url: appendSearchParams(
          new URL("/api/repo/projected-page-index-trees", baseUrl),
          {
            repo,
          },
        ),
      };
    default:
      return {
        skipReason: `no smoke plan for ${operation.method.toUpperCase()} ${operation.path}`,
      };
  }
}

function buildCoverageSummary(entries: OperationCoverage[]): CoverageSummary {
  return entries.reduce<CoverageSummary>(
    (summary, entry) => {
      summary.totalOperations += 1;
      if (entry.mode === "benchmark") {
        summary.benchmarkOperations += 1;
      } else if (entry.mode === "smoke") {
        summary.smokeOperations += 1;
      }

      if (entry.status === "passed") {
        summary.passedOperations += 1;
      } else if (entry.status === "failed") {
        summary.failedOperations += 1;
      } else {
        summary.skippedOperations += 1;
      }
      return summary;
    },
    {
      totalOperations: 0,
      benchmarkOperations: 0,
      smokeOperations: 0,
      skippedOperations: 0,
      passedOperations: 0,
      failedOperations: 0,
    },
  );
}

export async function runBenchmark(
  options: CliOptions,
  plan: BenchmarkPlan,
): Promise<{ report: BenchmarkReport; failures: RequestMetric[] }> {
  const baseUrl = new URL(
    plan.gatewayUrl.endsWith("/") ? plan.gatewayUrl : `${plan.gatewayUrl}/`,
  );
  const smokeLimit = Math.min(options.limit, SMOKE_LIMIT_CAP);
  const stressLimit = Math.min(options.limit, 10);

  const repoStatusAggregateResult = await timedJsonRequest(
    "repo_index_status",
    "aggregate",
    new URL("/api/repo/index/status", baseUrl),
    options.timeoutMs,
  );
  const searchStatusAggregateResult = await timedJsonRequest(
    "search_index_status",
    "aggregate",
    new URL("/api/search/index/status", baseUrl),
    options.timeoutMs,
  );

  if (!repoStatusAggregateResult.metric.ok) {
    throw new Error(
      `aggregate repo-index status failed: ${
        repoStatusAggregateResult.metric.error ?? "unknown error"
      }`,
    );
  }
  if ((repoStatusAggregateResult.metric.total ?? 0) < options.minRepoCount) {
    throw new Error(
      `aggregate repo-index total ${
        repoStatusAggregateResult.metric.total ?? 0
      } is below min floor ${options.minRepoCount}`,
    );
  }
  if (!searchStatusAggregateResult.metric.ok) {
    throw new Error(
      `aggregate search-index status failed: ${
        searchStatusAggregateResult.metric.error ?? "unknown error"
      }`,
    );
  }

  const repoMetrics = await runWithConcurrency(
    plan.sampledRepoIds,
    options.concurrency,
    async (repoId) => {
      const url = appendSearchParams(
        new URL("/api/repo/index/status", baseUrl),
        { repo: repoId },
      );
      return (
        await timedJsonRequest(
          "repo_index_status",
          repoId,
          url,
          options.timeoutMs,
        )
      ).metric;
    },
  );

  const repoSyncMetrics = await runWithConcurrency(
    plan.sampledRepoIds,
    options.concurrency,
    async (repoId) => {
      const url = appendSearchParams(new URL("/api/repo/sync", baseUrl), {
        repo: repoId,
        mode: "status",
      });
      return (
        await timedJsonRequest(
          "repo_sync_status",
          repoId,
          url,
          options.timeoutMs,
        )
      ).metric;
    },
  );

  const searchMetrics = await runWithConcurrency(
    plan.plannedSearchCases,
    options.concurrency,
    async ({ repoId, query }) => {
      const url = appendSearchParams(new URL("/api/search/intent", baseUrl), {
        intent: "code_search",
        q: query,
        limit: options.limit,
      });
      return (
        await timedJsonRequest("code_search", repoId, url, options.timeoutMs)
      ).metric;
    },
  );

  const discovery = await discoverGatewayContext(
    baseUrl,
    options,
    plan,
    repoStatusAggregateResult.payload,
  );
  const readyRepoIds = extractReadyRepoIds(
    repoStatusAggregateResult.payload,
  ).filter((repoId) => plan.sampledRepoIds.includes(repoId));

  const smokeCoverageEntries: OperationCoverage[] = [];
  const smokePlans: SmokeRequestPlan[] = [];
  for (const operation of plan.openApiOperations) {
    const mode = determineCoverageMode(operation);
    if (mode === "skip") {
      smokeCoverageEntries.push({
        method: operation.method,
        path: operation.path,
        operationId: operation.operationId,
        mode,
        status: "skipped",
        skipReason:
          "mutating OpenAPI operation is intentionally skipped in live benchmark",
      });
      continue;
    }
    if (mode === "smoke") {
      const smokePlan = buildSmokeRequestPlan(
        operation,
        discovery,
        baseUrl,
        smokeLimit,
      );
      if ("skipReason" in smokePlan) {
        smokeCoverageEntries.push({
          method: operation.method,
          path: operation.path,
          operationId: operation.operationId,
          mode,
          status: "skipped",
          skipReason: smokePlan.skipReason,
        });
      } else {
        smokePlans.push(smokePlan);
      }
    }
  }

  const smokeResults = await runWithConcurrency(
    smokePlans,
    Math.min(options.concurrency, SMOKE_CONCURRENCY_LIMIT),
    async (smokePlan) => {
      const result = await timedJsonRequest(
        "openapi_smoke",
        smokePlan.label,
        smokePlan.url,
        options.timeoutMs,
      );
      return { operation: smokePlan.operation, result };
    },
  );

  const smokeMetrics = smokeResults.map(({ result }) => result.metric);
  const stressCodeSearch = await runSustainedLoad(
    "stress_code_search",
    buildStressCodeSearchCases(baseUrl, plan.plannedSearchCases, stressLimit),
    options.stressConcurrency,
    options.stressDurationMs,
    options.stressMaxRequests,
    options.timeoutMs,
  );
  const stressMixedHotset = await runSustainedLoad(
    "stress_mixed_user_hotset",
    buildStressMixedHotsetCases(
      baseUrl,
      readyRepoIds,
      plan.plannedSearchCases,
      stressLimit,
    ),
    options.stressConcurrency,
    options.stressDurationMs,
    options.stressMaxRequests,
    options.timeoutMs,
  );
  const stressMetrics = [
    ...stressCodeSearch.metrics,
    ...stressMixedHotset.metrics,
  ];
  const stressSummaries = [stressCodeSearch.summary, stressMixedHotset.summary];
  const operationCoverage: OperationCoverage[] = [];
  for (const operation of plan.openApiOperations) {
    const mode = determineCoverageMode(operation);
    if (mode === "benchmark") {
      operationCoverage.push(
        benchmarkMetricToCoverage(
          operation,
          selectBenchmarkCoverageMetric(
            operation.path,
            repoStatusAggregateResult.metric,
            searchStatusAggregateResult.metric,
            repoSyncMetrics,
            searchMetrics,
          ),
        ),
      );
      continue;
    }
    const smokeCoverageEntry = smokeCoverageEntries.find(
      (entry) =>
        entry.path === operation.path && entry.method === operation.method,
    );
    if (smokeCoverageEntry) {
      operationCoverage.push(smokeCoverageEntry);
      continue;
    }
    const smokeResult = smokeResults.find(
      ({ operation: smokeOperation }) =>
        smokeOperation.path === operation.path &&
        smokeOperation.method === operation.method,
    );
    if (!smokeResult) {
      operationCoverage.push({
        method: operation.method,
        path: operation.path,
        operationId: operation.operationId,
        mode,
        status: "failed",
        error: "smoke execution result missing",
      });
      continue;
    }
    operationCoverage.push({
      method: operation.method,
      path: operation.path,
      operationId: operation.operationId,
      mode,
      status: smokeResult.result.metric.ok ? "passed" : "failed",
      suite: smokeResult.result.metric.suite,
      label: smokeResult.result.metric.label,
      url: smokeResult.result.metric.url,
      elapsedMs: smokeResult.result.metric.elapsedMs,
      httpStatus: smokeResult.result.metric.status,
      attempts: smokeResult.result.metric.attempts,
      error: smokeResult.result.metric.ok
        ? undefined
        : smokeResult.result.metric.error,
    });
  }

  const allRepoMetrics = [repoStatusAggregateResult.metric, ...repoMetrics];
  const summaries = [
    summariseSuite("repo_index_status", allRepoMetrics),
    summariseSuite("repo_sync_status", repoSyncMetrics),
    summariseSuite("code_search", searchMetrics),
    summariseSuite("search_index_status", [searchStatusAggregateResult.metric]),
  ];
  if (smokeMetrics.length > 0) {
    summaries.push(summariseSuite("openapi_smoke", smokeMetrics));
  }

  const requestFailures = [
    ...allRepoMetrics,
    searchStatusAggregateResult.metric,
    ...repoSyncMetrics,
    ...searchMetrics,
    ...smokeMetrics,
    ...stressMetrics,
  ].filter((metric) => !metric.ok);

  const report: BenchmarkReport = {
    gatewayUrl: plan.gatewayUrl,
    openapiPath: plan.openapiPath,
    workspaceConfig: plan.workspaceConfig,
    repoCount: plan.sampledRepoIds.length,
    openapiOperationCount: plan.openApiOperations.length,
    summaries,
    stressSummaries,
    aggregateRepoIndexStatus: repoStatusAggregateResult.metric,
    aggregateSearchIndexStatus: searchStatusAggregateResult.metric,
    repoIndexSnapshot: buildRepoIndexSnapshot(
      repoStatusAggregateResult.payload,
    ),
    searchIndexSnapshot: buildSearchIndexSnapshot(
      searchStatusAggregateResult.payload,
    ),
    discovery,
    coverageSummary: buildCoverageSummary(operationCoverage),
    operationCoverage,
    failures: requestFailures.slice(0, FAILURE_SAMPLE_LIMIT),
  };

  return { report, failures: requestFailures };
}

export function formatHumanSummary(report: BenchmarkReport): string {
  const lines = [
    `Gateway: ${report.gatewayUrl}`,
    `OpenAPI: ${report.openapiPath}`,
    `Workspace config: ${report.workspaceConfig}`,
    `Repos sampled: ${report.repoCount}`,
    `OpenAPI coverage: total=${report.coverageSummary.totalOperations} passed=${report.coverageSummary.passedOperations} failed=${report.coverageSummary.failedOperations} skipped=${report.coverageSummary.skippedOperations} benchmark=${report.coverageSummary.benchmarkOperations} smoke=${report.coverageSummary.smokeOperations}`,
    "",
  ];
  for (const summary of report.summaries) {
    lines.push(
      `${summary.suite}: requests=${summary.requests} ok=${summary.ok} failed=${summary.failed} avg=${summary.avgMs.toFixed(
        2,
      )}ms p50=${summary.p50Ms.toFixed(2)}ms p95=${summary.p95Ms.toFixed(
        2,
      )}ms max=${summary.maxMs.toFixed(2)}ms`,
    );
    if (summary.nonEmptyHitCount !== undefined) {
      lines.push(
        `${summary.suite}: non-empty responses=${summary.nonEmptyHitCount}`,
      );
    }
  }
  if (report.stressSummaries.length > 0) {
    lines.push("");
    for (const summary of report.stressSummaries) {
      lines.push(
        `${summary.suite}: concurrency=${summary.concurrency} duration=${summary.actualDurationMs.toFixed(
          0,
        )}ms requests=${summary.requests} ok=${summary.ok} failed=${summary.failed} success_rate=${(
          summary.successRate * 100
        ).toFixed(
          2,
        )}% throughput=${summary.throughputRps.toFixed(2)}rps p95=${summary.p95Ms.toFixed(
          2,
        )}ms p99=${summary.p99Ms.toFixed(2)}ms max=${summary.maxMs.toFixed(2)}ms capped=${summary.capped}`,
      );
      if (summary.nonEmptyHitCount !== undefined) {
        lines.push(
          `${summary.suite}: non-empty responses=${summary.nonEmptyHitCount}`,
        );
      }
    }
  }
  lines.push("");
  lines.push(
    `Aggregate repo-index status: total=${report.repoIndexSnapshot?.total ?? report.aggregateRepoIndexStatus.total ?? 0} ready=${
      report.repoIndexSnapshot?.ready ??
      report.aggregateRepoIndexStatus.ready ??
      0
    } queued=${report.repoIndexSnapshot?.queued ?? 0} failed=${report.repoIndexSnapshot?.failed ?? 0} unsupported=${report.repoIndexSnapshot?.unsupported ?? 0}`,
  );
  lines.push(
    `Aggregate search-index status: total=${report.searchIndexSnapshot?.total ?? 0} idle=${
      report.searchIndexSnapshot?.idle ?? 0
    } degraded=${report.searchIndexSnapshot?.degraded ?? 0} failed=${
      report.searchIndexSnapshot?.failed ?? 0
    } status_reason=${report.searchIndexSnapshot?.statusReasonCode ?? "none"}`,
  );
  if (report.discovery.readyRepoId) {
    lines.push(`Discovery seed repo: ${report.discovery.readyRepoId}`);
  }
  lines.push(`Recorded failure samples: ${report.failures.length}`);
  return lines.join("\n");
}

export async function runCli(
  argv: string[],
  io: CliIo = console,
  environment: NodeJS.ProcessEnv = process.env,
  cwd: string = process.cwd(),
): Promise<number> {
  try {
    const options = parseArgs(argv);
    if (options.help) {
      io.log(renderHelpText());
      return 0;
    }

    const plan = buildBenchmarkPlan(options, environment, cwd);
    if (options.dryRun) {
      io.log(
        JSON.stringify(
          createDryRunReport(plan, options, environment, cwd),
          null,
          2,
        ),
      );
      return 0;
    }

    const { report, failures } = await runBenchmark(options, plan);
    const persisted = persistBenchmarkReportToml(
      report,
      options,
      environment,
      cwd,
    );
    if (options.json) {
      io.log(
        JSON.stringify(
          {
            report,
            reportPath: persisted.path,
            generatedAt: persisted.generatedAt,
          },
          null,
          2,
        ),
      );
    } else {
      io.log(formatHumanSummary(report));
      io.log(`Report TOML: ${persisted.path}`);
    }
    return failures.length === 0 &&
      report.coverageSummary.failedOperations === 0
      ? 0
      : 1;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    io.error(`ERROR: ${message}`);
    return 2;
  }
}
